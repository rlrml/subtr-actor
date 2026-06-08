use super::*;
use crate::stats::calculators::*;
use crate::*;

#[derive(Debug, Clone, Default)]
pub struct StatsTimelineEventsState {
    pub events: ReplayStatsTimelineEvents,
}

const MECHANIC_AIR_DRIBBLE: &str = "air_dribble";
const MECHANIC_BALL_CARRY: &str = "ball_carry";
const MECHANIC_CEILING_SHOT: &str = "ceiling_shot";
const MECHANIC_CENTER: &str = "center";
const MECHANIC_DOUBLE_TAP: &str = "double_tap";
const MECHANIC_FLICK: &str = "flick";
const MECHANIC_FLIP_RESET: &str = "flip_reset";
const MECHANIC_HALF_FLIP: &str = "half_flip";
const MECHANIC_HALF_VOLLEY: &str = "half_volley";
const MECHANIC_MUSTY_FLICK: &str = "musty_flick";
const MECHANIC_ONE_TIMER: &str = "one_timer";
const MECHANIC_PASS: &str = "pass";
const MECHANIC_SPEED_FLIP: &str = "speed_flip";
const MECHANIC_WALL_AERIAL: &str = "wall_aerial";
const MECHANIC_WALL_AERIAL_SHOT: &str = "wall_aerial_shot";
const MECHANIC_WAVEDASH: &str = "wavedash";

pub const STATS_TIMELINE_MECHANIC_KINDS: &[&str] = &[
    MECHANIC_AIR_DRIBBLE,
    MECHANIC_BALL_CARRY,
    MECHANIC_CEILING_SHOT,
    MECHANIC_CENTER,
    MECHANIC_DOUBLE_TAP,
    MECHANIC_FLICK,
    MECHANIC_FLIP_RESET,
    MECHANIC_HALF_FLIP,
    MECHANIC_HALF_VOLLEY,
    MECHANIC_MUSTY_FLICK,
    MECHANIC_ONE_TIMER,
    MECHANIC_PASS,
    MECHANIC_SPEED_FLIP,
    MECHANIC_WALL_AERIAL,
    MECHANIC_WALL_AERIAL_SHOT,
    MECHANIC_WAVEDASH,
];

pub struct StatsTimelineEventsNode {
    state: StatsTimelineEventsState,
}

impl StatsTimelineEventsNode {
    pub fn new() -> Self {
        Self {
            state: StatsTimelineEventsState::default(),
        }
    }

    fn dependencies() -> NodeDependencies {
        vec![
            frame_info_dependency(),
            gameplay_state_dependency(),
            live_play_dependency(),
            match_stats_dependency(),
            // Keep compact event transfer independent from full partial-sum projection.
            backboard_dependency(),
            ceiling_shot_dependency(),
            wall_aerial_dependency(),
            wall_aerial_shot_dependency(),
            double_tap_dependency(),
            one_timer_dependency(),
            pass_dependency(),
            controlled_play_dependency(),
            fifty_fifty_dependency(),
            kickoff_dependency(),
            possession_dependency(),
            pressure_dependency(),
            territorial_pressure_dependency(),
            rotation_dependency(),
            rush_dependency(),
            touch_dependency(),
            whiff_dependency(),
            wavedash_dependency(),
            flip_impulse_dependency(),
            speed_flip_dependency(),
            half_flip_dependency(),
            flick_dependency(),
            musty_flick_dependency(),
            dodge_reset_dependency(),
            ball_carry_dependency(),
            boost_dependency(),
            bump_dependency(),
            half_volley_dependency(),
            movement_dependency(),
            positioning_dependency(),
            powerslide_dependency(),
            demo_dependency(),
            center_dependency(),
            aerial_goal_dependency(),
            high_aerial_goal_dependency(),
            long_distance_goal_dependency(),
            own_half_goal_dependency(),
            empty_net_goal_dependency(),
            counter_attack_goal_dependency(),
            flick_goal_dependency(),
            double_tap_goal_dependency(),
            one_timer_goal_dependency(),
            passing_goal_dependency(),
            air_dribble_goal_dependency(),
            flip_reset_goal_dependency(),
            bump_goal_dependency(),
            demo_goal_dependency(),
            half_volley_goal_dependency(),
        ]
    }

    fn capture_events(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let match_stats = ctx.get::<MatchStatsCalculator>()?;
        let possession = ctx.get::<PossessionCalculator>()?;
        let pressure = ctx.get::<PressureCalculator>()?;
        let territorial_pressure = ctx.get::<TerritorialPressureCalculator>()?;
        let movement = ctx.get::<MovementCalculator>()?;
        let positioning = ctx.get::<PositioningCalculator>()?;
        let rotation = ctx.get::<RotationCalculator>()?;
        let demo = ctx.get::<DemoCalculator>()?;
        let backboard = ctx.get::<BackboardCalculator>()?;
        let ball_carry = ctx.get::<BallCarryCalculator>()?;
        let ceiling_shot = ctx.get::<CeilingShotCalculator>()?;
        let wall_aerial = ctx.get::<WallAerialCalculator>()?;
        let wall_aerial_shot = ctx.get::<WallAerialShotCalculator>()?;
        let center = ctx.get::<CenterCalculator>()?;
        let dodge_reset = ctx.get::<DodgeResetCalculator>()?;
        let double_tap = ctx.get::<DoubleTapCalculator>()?;
        let one_timer = ctx.get::<OneTimerCalculator>()?;
        let pass = ctx.get::<PassCalculator>()?;
        let controlled_play = ctx.get::<ControlledPlayCalculator>()?;
        let fifty_fifty = ctx.get::<FiftyFiftyCalculator>()?;
        let kickoff = ctx.get::<KickoffCalculator>()?;
        let flick = ctx.get::<FlickCalculator>()?;
        let musty_flick = ctx.get::<MustyFlickCalculator>()?;
        let aerial_goal = ctx.get::<AerialGoalCalculator>()?;
        let high_aerial_goal = ctx.get::<HighAerialGoalCalculator>()?;
        let long_distance_goal = ctx.get::<LongDistanceGoalCalculator>()?;
        let own_half_goal = ctx.get::<OwnHalfGoalCalculator>()?;
        let empty_net_goal = ctx.get::<EmptyNetGoalCalculator>()?;
        let counter_attack_goal = ctx.get::<CounterAttackGoalCalculator>()?;
        let flick_goal = ctx.get::<FlickGoalCalculator>()?;
        let double_tap_goal = ctx.get::<DoubleTapGoalCalculator>()?;
        let one_timer_goal = ctx.get::<OneTimerGoalCalculator>()?;
        let passing_goal = ctx.get::<PassingGoalCalculator>()?;
        let air_dribble_goal = ctx.get::<AirDribbleGoalCalculator>()?;
        let flip_reset_goal = ctx.get::<FlipResetGoalCalculator>()?;
        let bump_goal = ctx.get::<BumpGoalCalculator>()?;
        let demo_goal = ctx.get::<DemoGoalCalculator>()?;
        let half_volley_goal = ctx.get::<HalfVolleyGoalCalculator>()?;
        let rush = ctx.get::<RushCalculator>()?;
        let flip_impulse = ctx.get::<FlipImpulseCalculator>()?;
        let speed_flip = ctx.get::<SpeedFlipCalculator>()?;
        let half_flip = ctx.get::<HalfFlipCalculator>()?;
        let half_volley = ctx.get::<HalfVolleyCalculator>()?;
        let wavedash = ctx.get::<WavedashCalculator>()?;
        let whiff = ctx.get::<WhiffCalculator>()?;
        let powerslide = ctx.get::<PowerslideCalculator>()?;
        let touch = ctx.get::<TouchCalculator>()?;
        let boost = ctx.get::<BoostCalculator>()?;
        let bump = ctx.get::<BumpCalculator>()?;

        let mut timeline = match_stats.timeline().to_vec();
        timeline.extend(demo.timeline().to_vec());
        timeline.sort_by(|left, right| left.time.total_cmp(&right.time));
        let goal_tag_assignments = combined_goal_tag_assignments(&[
            aerial_goal.events(),
            high_aerial_goal.events(),
            long_distance_goal.events(),
            own_half_goal.events(),
            empty_net_goal.events(),
            counter_attack_goal.events(),
            flick_goal.events(),
            double_tap_goal.events(),
            one_timer_goal.events(),
            passing_goal.events(),
            air_dribble_goal.events(),
            flip_reset_goal.events(),
            bump_goal.events(),
            demo_goal.events(),
            half_volley_goal.events(),
        ]);
        let goal_context =
            goal_context_events_with_tags(match_stats.goal_context_events(), &goal_tag_assignments);

        self.state.events = ReplayStatsTimelineEvents {
            timeline,
            core_player: match_stats.core_player_events().to_vec(),
            core_player_goal_context: match_stats.core_player_goal_context_events().to_vec(),
            possession: possession.events().to_vec(),
            pressure: pressure.events().to_vec(),
            territorial_pressure: territorial_pressure.events().to_vec(),
            movement: movement.events().to_vec(),
            positioning_activity: positioning.activity_events(),
            positioning_distance: positioning.distance_events(),
            positioning_field_zone: positioning.field_zone_events(),
            positioning_ball_depth: positioning.ball_depth_events(),
            positioning_teammate_role: positioning.teammate_role_events(),
            positioning_ball_proximity: positioning.ball_proximity_events(),
            positioning_goal_context: positioning.goal_context_events(),
            rotation_player: rotation.player_events().to_vec(),
            rotation_role_span: rotation.role_span_events(),
            rotation_depth_span: rotation.depth_span_events(),
            rotation_first_man_stint: rotation.first_man_stint_events(),
            rotation_team: rotation.team_events().to_vec(),
            mechanics: build_mechanic_events(
                ball_carry,
                ceiling_shot,
                wall_aerial,
                wall_aerial_shot,
                center,
                dodge_reset,
                double_tap,
                flick,
                musty_flick,
                one_timer,
                pass,
                speed_flip,
                half_flip,
                half_volley,
                wavedash,
            ),
            goal_context,
            backboard: backboard.events().to_vec(),
            ceiling_shot: ceiling_shot.events().to_vec(),
            wall_aerial: wall_aerial.events().to_vec(),
            wall_aerial_shot: wall_aerial_shot.events().to_vec(),
            center: center.events().to_vec(),
            flick: flick.events().to_vec(),
            musty_flick: musty_flick.events().to_vec(),
            dodge_reset: dodge_reset.events().to_vec(),
            double_tap: double_tap.events().to_vec(),
            one_timer: one_timer.events().to_vec(),
            pass: pass.events().to_vec(),
            ball_carry: ball_carry.carry_events().to_vec(),
            controlled_play: controlled_play.events().to_vec(),
            fifty_fifty: fifty_fifty.events().to_vec(),
            kickoff: kickoff.events().to_vec(),
            rush: rush.events().to_vec(),
            flip_impulse: flip_impulse.events().to_vec(),
            speed_flip: speed_flip.events().to_vec(),
            half_flip: half_flip.events().to_vec(),
            half_volley: half_volley.events().to_vec(),
            wavedash: wavedash.events().to_vec(),
            whiff: whiff.events().to_vec(),
            powerslide: powerslide.events().to_vec(),
            touch: touch.events().to_vec(),
            boost_pickups: boost.pickup_comparison_events().to_vec(),
            boost_ledger: boost.ledger_events().to_vec(),
            boost_state: boost.state_events().to_vec(),
            bump: bump.events().to_vec(),
        };
        Ok(())
    }
}

impl Default for StatsTimelineEventsNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for StatsTimelineEventsNode {
    type State = StatsTimelineEventsState;

    fn name(&self) -> &'static str {
        "stats_timeline_events"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        Self::dependencies()
    }

    fn evaluate(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        Ok(())
    }

    fn finish(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.capture_events(ctx)
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

fn moment_mechanic_event(
    kind: &str,
    index: usize,
    frame: usize,
    time: f32,
    player_id: PlayerId,
    player_position: Option<[f32; 3]>,
    is_team_0: bool,
) -> StatsTimelineTagEvent {
    StatsTimelineTagEvent {
        id: format!("{kind}:{frame}:{index}"),
        kind: kind.to_owned(),
        player_id,
        player_position,
        is_team_0,
        timing: StatsEventTiming::Moment { frame, time },
        properties: Vec::new(),
    }
}

#[allow(clippy::too_many_arguments)]
fn span_mechanic_event(
    kind: &str,
    index: usize,
    start_frame: usize,
    end_frame: usize,
    start_time: f32,
    end_time: f32,
    player_id: PlayerId,
    player_position: Option<[f32; 3]>,
    is_team_0: bool,
) -> StatsTimelineTagEvent {
    StatsTimelineTagEvent {
        id: format!("{kind}:{start_frame}:{end_frame}:{index}"),
        kind: kind.to_owned(),
        player_id,
        player_position,
        is_team_0,
        timing: StatsEventTiming::Span {
            start_frame,
            end_frame,
            start_time,
            end_time,
        },
        properties: Vec::new(),
    }
}

fn mechanic_event_text_property(key: &str, value: &str) -> StatsEventProperty {
    StatsEventProperty {
        key: key.to_owned(),
        value: StatsEventPropertyValue::Text(value.to_owned()),
    }
}

fn mechanic_event_unsigned_property(key: &str, value: u32) -> StatsEventProperty {
    StatsEventProperty {
        key: key.to_owned(),
        value: StatsEventPropertyValue::Unsigned(value),
    }
}

fn mechanic_event_float_property(key: &str, value: f32) -> StatsEventProperty {
    StatsEventProperty {
        key: key.to_owned(),
        value: StatsEventPropertyValue::Float(value),
    }
}

fn flick_mechanic_event_properties(event: &FlickEvent) -> Vec<StatsEventProperty> {
    vec![
        mechanic_event_text_property("flick_kind", &event.kind),
        mechanic_event_text_property("setup_rotation_direction", &event.setup_rotation_direction),
        mechanic_event_float_property("setup_rotation_degrees", event.setup_rotation_degrees),
    ]
}

fn ball_carry_mechanic_event_properties(event: &BallCarryEvent) -> Vec<StatsEventProperty> {
    let mut properties = Vec::new();
    if let Some(origin) = event.air_dribble_origin {
        properties.push(mechanic_event_text_property(
            "origin",
            origin.as_label_value(),
        ));
    }
    if event.kind == BallCarryKind::AirDribble {
        properties.push(mechanic_event_unsigned_property(
            "touch_count",
            event.touch_count,
        ));
    }
    properties
}

#[allow(clippy::too_many_arguments)]
fn build_mechanic_events(
    ball_carry: &BallCarryCalculator,
    ceiling_shot: &CeilingShotCalculator,
    wall_aerial: &WallAerialCalculator,
    wall_aerial_shot: &WallAerialShotCalculator,
    center: &CenterCalculator,
    dodge_reset: &DodgeResetCalculator,
    double_tap: &DoubleTapCalculator,
    flick: &FlickCalculator,
    musty_flick: &MustyFlickCalculator,
    one_timer: &OneTimerCalculator,
    pass: &PassCalculator,
    speed_flip: &SpeedFlipCalculator,
    half_flip: &HalfFlipCalculator,
    half_volley: &HalfVolleyCalculator,
    wavedash: &WavedashCalculator,
) -> Vec<StatsTimelineTagEvent> {
    let mut events = Vec::new();

    for (index, event) in ball_carry.carry_events().iter().enumerate() {
        let kind = match event.kind {
            BallCarryKind::Carry => MECHANIC_BALL_CARRY,
            BallCarryKind::AirDribble => MECHANIC_AIR_DRIBBLE,
        };
        let mut mechanic_event = span_mechanic_event(
            kind,
            index,
            event.start_frame,
            event.end_frame,
            event.start_time,
            event.end_time,
            event.player_id.clone(),
            Some(event.end_position),
            event.is_team_0,
        );
        mechanic_event.properties = ball_carry_mechanic_event_properties(event);
        events.push(mechanic_event);
    }

    for (index, event) in ceiling_shot.events().iter().enumerate() {
        events.push(span_mechanic_event(
            MECHANIC_CEILING_SHOT,
            index,
            event.ceiling_contact_frame,
            event.frame,
            event.ceiling_contact_time,
            event.time,
            event.player.clone(),
            event.player_position,
            event.is_team_0,
        ));
    }

    for (index, event) in wall_aerial.events().iter().enumerate() {
        let mut mechanic_event = span_mechanic_event(
            MECHANIC_WALL_AERIAL,
            index,
            event.wall_contact_frame,
            event.frame,
            event.wall_contact_time,
            event.time,
            event.player.clone(),
            Some(event.player_position),
            event.is_team_0,
        );
        mechanic_event.properties = vec![mechanic_event_text_property(
            "wall",
            event.wall.as_label_value(),
        )];
        events.push(mechanic_event);
    }

    for (index, event) in wall_aerial_shot.events().iter().enumerate() {
        let mut mechanic_event = span_mechanic_event(
            MECHANIC_WALL_AERIAL_SHOT,
            index,
            event.takeoff_frame,
            event.frame,
            event.takeoff_time,
            event.time,
            event.player.clone(),
            Some(event.player_position),
            event.is_team_0,
        );
        mechanic_event.properties = vec![mechanic_event_text_property(
            "wall",
            event.wall.as_label_value(),
        )];
        events.push(mechanic_event);
    }

    for (index, event) in center.events().iter().enumerate() {
        events.push(span_mechanic_event(
            MECHANIC_CENTER,
            index,
            event.start_frame,
            event.frame,
            event.start_time,
            event.time,
            event.player.clone(),
            event.player_position,
            event.is_team_0,
        ));
    }

    for (index, event) in dodge_reset.confirmed_flip_reset_events().iter().enumerate() {
        events.push(moment_mechanic_event(
            MECHANIC_FLIP_RESET,
            index,
            event.frame,
            event.time,
            event.player.clone(),
            event.player_position,
            event.is_team_0,
        ));
    }

    for (index, event) in double_tap.events().iter().enumerate() {
        events.push(span_mechanic_event(
            MECHANIC_DOUBLE_TAP,
            index,
            event.backboard_frame,
            event.frame,
            event.backboard_time,
            event.time,
            event.player.clone(),
            event.player_position,
            event.is_team_0,
        ));
    }

    for (index, event) in flick.events().iter().enumerate() {
        let mut mechanic_event = span_mechanic_event(
            MECHANIC_FLICK,
            index,
            event.setup_start_frame,
            event.frame,
            event.setup_start_time,
            event.time,
            event.player.clone(),
            event.player_position,
            event.is_team_0,
        );
        mechanic_event.properties = flick_mechanic_event_properties(event);
        events.push(mechanic_event);
    }

    for (index, event) in musty_flick.events().iter().enumerate() {
        events.push(span_mechanic_event(
            MECHANIC_MUSTY_FLICK,
            index,
            event.dodge_frame,
            event.frame,
            event.dodge_time,
            event.time,
            event.player.clone(),
            event.player_position,
            event.is_team_0,
        ));
    }

    for (index, event) in one_timer.events().iter().enumerate() {
        events.push(span_mechanic_event(
            MECHANIC_ONE_TIMER,
            index,
            event.pass_start_frame,
            event.frame,
            event.pass_start_time,
            event.time,
            event.player.clone(),
            event.player_position,
            event.is_team_0,
        ));
    }

    for (index, event) in pass.events().iter().enumerate() {
        events.push(span_mechanic_event(
            MECHANIC_PASS,
            index,
            event.start_frame,
            event.frame,
            event.start_time,
            event.time,
            event.passer.clone(),
            event.passer_position,
            event.is_team_0,
        ));
    }

    for (index, event) in speed_flip.events().iter().enumerate() {
        events.push(moment_mechanic_event(
            MECHANIC_SPEED_FLIP,
            index,
            event.frame,
            event.time,
            event.player.clone(),
            Some(event.end_position),
            event.is_team_0,
        ));
    }

    for (index, event) in half_flip.events().iter().enumerate() {
        events.push(moment_mechanic_event(
            MECHANIC_HALF_FLIP,
            index,
            event.frame,
            event.time,
            event.player.clone(),
            Some(event.end_position),
            event.is_team_0,
        ));
    }

    for (index, event) in half_volley.events().iter().enumerate() {
        events.push(moment_mechanic_event(
            MECHANIC_HALF_VOLLEY,
            index,
            event.frame,
            event.time,
            event.player.clone(),
            event.player_position,
            event.is_team_0,
        ));
    }

    for (index, event) in wavedash.events().iter().enumerate() {
        events.push(span_mechanic_event(
            MECHANIC_WAVEDASH,
            index,
            event.dodge_frame,
            event.frame,
            event.dodge_time,
            event.time,
            event.player.clone(),
            Some(event.landing_position),
            event.is_team_0,
        ));
    }

    events.sort_by(|left, right| {
        let left_time = mechanic_event_start_time(left);
        let right_time = mechanic_event_start_time(right);
        left_time
            .total_cmp(&right_time)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.id.cmp(&right.id))
    });
    events
}

fn mechanic_event_start_time(event: &StatsTimelineTagEvent) -> f32 {
    match event.timing {
        StatsEventTiming::Moment { time, .. } => time,
        StatsEventTiming::Span { start_time, .. } => start_time,
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(StatsTimelineEventsNode::new())
}
