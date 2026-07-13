use crate::stats::analysis_graph::*;
use crate::stats::calculators::*;
use crate::*;
use boxcars;
use std::marker::PhantomData;
use std::sync::{Arc, OnceLock};

fn threat_feature_headers() -> &'static [&'static str] {
    static HEADERS: OnceLock<Vec<&'static str>> = OnceLock::new();
    HEADERS.get_or_init(|| {
        ["team_zero", "team_one"]
            .into_iter()
            .flat_map(|team| {
                ThreatFeatures::FEATURE_NAMES.iter().map(move |feature| {
                    let header: &'static mut str =
                        Box::leak(format!("{team}_threat_{feature}").into_boxed_str());
                    &*header
                })
            })
            .collect()
    })
}

/// Both teams' attacking-normalized threat inputs as one ndarray row.
pub struct ThreatFeaturesBothTeams<F>(PhantomData<F>);

impl<F> ThreatFeaturesBothTeams<F> {
    pub fn arc_new() -> Arc<dyn AnalysisFeatureAdder<F> + Send + Sync>
    where
        F: TryFrom<f32> + Send + Sync + 'static,
        <F as TryFrom<f32>>::Error: std::fmt::Debug,
    {
        Arc::new(Self(PhantomData))
    }
}

impl<F> AnalysisFeatureAdder<F> for ThreatFeaturesBothTeams<F>
where
    F: TryFrom<f32> + Send + Sync,
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    fn get_column_headers(&self) -> &[&str] {
        threat_feature_headers()
    }

    fn analysis_dependencies(&self) -> Vec<AnalysisDependency> {
        vec![threat_features_dependency()]
    }

    fn add_features(
        &self,
        context: &AnalysisFeatureContext<'_>,
        _processor: &dyn ProcessorView,
        _frame: &boxcars::Frame,
        _frame_count: usize,
        _current_time: f32,
        vector: &mut Vec<F>,
    ) -> SubtrActorResult<()> {
        let features = context
            .state::<ThreatFeaturesState>()?
            .current()
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
                    "threat features requested outside live play".to_owned(),
                ))
            })?;
        for value in features.iter().flat_map(ThreatFeatures::to_array) {
            vector.push(F::try_from(value).map_err(convert_float_conversion_error)?);
        }
        Ok(())
    }
}

/// Current model values for both teams. Including this adder also requests the
/// expected-goals analysis node, allowing dataset callers to inspect its goal
/// and episode state after matrix collection.
pub struct ThreatModelValues<F>(PhantomData<F>);

impl<F> ThreatModelValues<F> {
    pub fn arc_new() -> Arc<dyn AnalysisFeatureAdder<F> + Send + Sync>
    where
        F: TryFrom<f32> + Send + Sync + 'static,
        <F as TryFrom<f32>>::Error: std::fmt::Debug,
    {
        Arc::new(Self(PhantomData))
    }
}

impl<F> AnalysisFeatureAdder<F> for ThreatModelValues<F>
where
    F: TryFrom<f32> + Send + Sync,
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    fn get_column_headers(&self) -> &[&str] {
        &["team_zero_threat_value", "team_one_threat_value"]
    }

    fn analysis_dependencies(&self) -> Vec<AnalysisDependency> {
        vec![expected_goals_dependency()]
    }

    fn add_features(
        &self,
        context: &AnalysisFeatureContext<'_>,
        _processor: &dyn ProcessorView,
        _frame: &boxcars::Frame,
        _frame_count: usize,
        _current_time: f32,
        vector: &mut Vec<F>,
    ) -> SubtrActorResult<()> {
        let values = context
            .state::<ExpectedGoalsCalculator>()?
            .current_values()
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
                    "threat values requested outside live play".to_owned(),
                ))
            })?;
        for value in values {
            vector.push(F::try_from(value).map_err(convert_float_conversion_error)?);
        }
        Ok(())
    }
}

/// Selects live threat frames at a fixed interval while the backing ndarray
/// analysis graph continues evaluating every replay frame.
pub struct LiveThreatSampleFilter {
    sample_interval_seconds: f32,
    last_sample_time: Option<f32>,
}

impl LiveThreatSampleFilter {
    pub fn new(sample_interval_seconds: f32) -> Self {
        Self {
            sample_interval_seconds,
            last_sample_time: None,
        }
    }
}

impl AnalysisFrameFilter for LiveThreatSampleFilter {
    fn analysis_dependencies(&self) -> Vec<AnalysisDependency> {
        vec![threat_features_dependency()]
    }

    fn include_frame(
        &mut self,
        context: &AnalysisFeatureContext<'_>,
        _processor: &dyn ProcessorView,
        _frame: &boxcars::Frame,
        _frame_count: usize,
        current_time: f32,
    ) -> SubtrActorResult<bool> {
        if context.state::<ThreatFeaturesState>()?.current().is_none() {
            return Ok(false);
        }
        let due = self.last_sample_time.is_none_or(|last| {
            current_time - last >= self.sample_interval_seconds || current_time < last
        });
        if due {
            self.last_sample_time = Some(current_time);
        }
        Ok(due)
    }
}

macro_rules! build_analysis_player_event_indicator {
    (
        $struct_name:ident,
        $dependency:ident,
        $calculator:ty,
        $events:ident,
        $player_matches:expr_2021,
        $column_name:expr_2021 $(,)?
    ) => {
        build_analysis_player_feature_adder!(
            $struct_name,
            |_self_: &$struct_name<F>| vec![$dependency()],
            |_self_: &$struct_name<F>,
             context: &AnalysisFeatureContext<'_>,
             player_id: &PlayerId,
             _processor: &dyn ProcessorView,
             _frame: &boxcars::Frame,
             _frame_count: usize,
             _current_time: f32| {
                let player_matches = $player_matches;
                let has_event = context
                    .state::<$calculator>()?
                    .$events()
                    .iter()
                    .any(|event| player_matches(event, player_id));
                convert_all_floats!(f32::from(has_event))
            },
            $column_name,
        );
    };
}

build_analysis_player_event_indicator!(
    AnalysisPlayerTouches,
    touch_dependency,
    TouchCalculator,
    new_events,
    |event: &TouchClassificationEvent, player_id: &PlayerId| &event.player == player_id,
    "analysis touch event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerCenters,
    center_dependency,
    CenterCalculator,
    new_events,
    |event: &CenterEvent, player_id: &PlayerId| &event.player == player_id,
    "analysis center event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerDoubleTaps,
    double_tap_dependency,
    DoubleTapCalculator,
    new_events,
    |event: &DoubleTapEvent, player_id: &PlayerId| &event.player == player_id,
    "analysis double tap event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerOneTimers,
    one_timer_dependency,
    OneTimerCalculator,
    new_events,
    |event: &OneTimerEvent, player_id: &PlayerId| {
        &event.player == player_id || &event.passer == player_id
    },
    "analysis one timer event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerWallAerials,
    wall_aerial_dependency,
    WallAerialCalculator,
    new_events,
    |event: &WallAerialEvent, player_id: &PlayerId| &event.player == player_id,
    "analysis wall aerial event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerWallAerialShots,
    wall_aerial_shot_dependency,
    WallAerialShotCalculator,
    new_events,
    |event: &WallAerialShotEvent, player_id: &PlayerId| &event.player == player_id,
    "analysis wall aerial shot event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerCeilingShots,
    ceiling_shot_dependency,
    CeilingShotCalculator,
    new_events,
    |event: &CeilingShotEvent, player_id: &PlayerId| &event.player == player_id,
    "analysis ceiling shot event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerFlicks,
    flick_dependency,
    FlickCalculator,
    new_events,
    |event: &FlickEvent, player_id: &PlayerId| &event.player == player_id,
    "analysis flick event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerDodgeResets,
    dodge_reset_dependency,
    DodgeResetCalculator,
    new_events,
    |event: &DodgeResetEvent, player_id: &PlayerId| &event.player == player_id,
    "analysis dodge reset event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerFlipResetDodges,
    dodge_reset_dependency,
    DodgeResetCalculator,
    new_confirmed_flip_reset_events,
    |event: &FlipResetEvent, player_id: &PlayerId| &event.player == player_id,
    "analysis flip reset dodge event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerHalfFlips,
    half_flip_dependency,
    HalfFlipCalculator,
    new_events,
    |event: &HalfFlipEvent, player_id: &PlayerId| &event.player == player_id,
    "analysis half flip event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerHalfVolleys,
    half_volley_dependency,
    HalfVolleyCalculator,
    new_events,
    |event: &HalfVolleyEvent, player_id: &PlayerId| &event.player == player_id,
    "analysis half volley event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerWavedashes,
    wavedash_dependency,
    WavedashCalculator,
    new_events,
    |event: &WavedashEvent, player_id: &PlayerId| &event.player == player_id,
    "analysis wavedash event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerWhiffs,
    whiff_dependency,
    WhiffCalculator,
    new_events,
    |event: &WhiffEvent, player_id: &PlayerId| &event.player == player_id,
    "analysis whiff event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerSpeedFlips,
    speed_flip_dependency,
    SpeedFlipCalculator,
    new_events,
    |event: &SpeedFlipEvent, player_id: &PlayerId| &event.player == player_id,
    "analysis speed flip event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerFlipImpulses,
    flip_impulse_dependency,
    FlipImpulseCalculator,
    new_events,
    |event: &DodgeEvent, player_id: &PlayerId| &event.player == player_id,
    "analysis dodge event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerPowerslides,
    powerslide_dependency,
    PowerslideCalculator,
    new_events,
    |event: &PowerslideEvent, player_id: &PlayerId| &event.player == player_id,
    "analysis powerslide event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerBallCarries,
    ball_carry_dependency,
    BallCarryCalculator,
    new_carry_events,
    |event: &BallCarryEvent, player_id: &PlayerId| &event.player_id == player_id,
    "analysis ball carry event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerBoostPickups,
    boost_dependency,
    BoostCalculator,
    new_pickup_events,
    |event: &BoostPickupEvent, player_id: &PlayerId| &event.player_id == player_id,
    "analysis boost pickup event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerBoostRespawns,
    boost_dependency,
    BoostCalculator,
    new_respawn_events,
    |event: &RespawnEvent, player_id: &PlayerId| &event.player_id == player_id,
    "analysis boost respawn event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerBumps,
    bump_dependency,
    BumpCalculator,
    new_events,
    |event: &BumpEvent, player_id: &PlayerId| {
        &event.initiator == player_id || &event.victim == player_id
    },
    "analysis bump event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerPasses,
    pass_dependency,
    PassCalculator,
    new_events,
    |event: &PassEvent, player_id: &PlayerId| {
        &event.passer == player_id || &event.receiver == player_id
    },
    "analysis pass event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerRotationEvents,
    rotation_dependency,
    RotationCalculator,
    new_role_events,
    |event: &RotationRoleEvent, player_id: &PlayerId| &event.player == player_id,
    "analysis rotation event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerMovementEvents,
    movement_dependency,
    MovementCalculator,
    new_events,
    |event: &MovementEvent, player_id: &PlayerId| &event.player == player_id,
    "analysis movement event",
);

build_analysis_player_event_indicator!(
    AnalysisPlayerPositioningEvents,
    positioning_dependency,
    PositioningCalculator,
    new_event_players,
    |event: &PlayerId, player_id: &PlayerId| event == player_id,
    "analysis positioning event",
);

pub(crate) fn analysis_player_event_feature_adder_from_name<F>(
    name: &str,
) -> Option<NDArrayPlayerFeatureAdder<F>>
where
    F: TryFrom<f32> + Send + Sync + 'static,
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    let event_name = name.strip_prefix("PlayerEvent:")?;

    match event_name {
        "touch" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerTouches::<F>::arc_new(),
        )),
        "center" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerCenters::<F>::arc_new(),
        )),
        "double_tap" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerDoubleTaps::<F>::arc_new(),
        )),
        "one_timer" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerOneTimers::<F>::arc_new(),
        )),
        "wall_aerial" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerWallAerials::<F>::arc_new(),
        )),
        "wall_aerial_shot" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerWallAerialShots::<F>::arc_new(),
        )),
        "ceiling_shot" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerCeilingShots::<F>::arc_new(),
        )),
        "flick" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerFlicks::<F>::arc_new(),
        )),
        "dodge_reset" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerDodgeResets::<F>::arc_new(),
        )),
        "flip_reset_dodge" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerFlipResetDodges::<F>::arc_new(),
        )),
        "half_flip" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerHalfFlips::<F>::arc_new(),
        )),
        "half_volley" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerHalfVolleys::<F>::arc_new(),
        )),
        "wavedash" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerWavedashes::<F>::arc_new(),
        )),
        "whiff" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerWhiffs::<F>::arc_new(),
        )),
        "speed_flip" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerSpeedFlips::<F>::arc_new(),
        )),
        "dodge" | "flip_impulse" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerFlipImpulses::<F>::arc_new(),
        )),
        "powerslide" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerPowerslides::<F>::arc_new(),
        )),
        "ball_carry" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerBallCarries::<F>::arc_new(),
        )),
        "boost_pickup" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerBoostPickups::<F>::arc_new(),
        )),
        "boost_respawn" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerBoostRespawns::<F>::arc_new(),
        )),
        "bump" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerBumps::<F>::arc_new(),
        )),
        "pass" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerPasses::<F>::arc_new(),
        )),
        "rotation" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerRotationEvents::<F>::arc_new(),
        )),
        "movement" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerMovementEvents::<F>::arc_new(),
        )),
        "positioning" => Some(NDArrayPlayerFeatureAdder::analysis(
            AnalysisPlayerPositioningEvents::<F>::arc_new(),
        )),
        _ => None,
    }
}
