use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use boxcars;
use boxcars::HeaderProp;
use serde::Serialize;

use super::boost_invariants::{boost_invariant_violations, BoostInvariantKind};
use crate::*;

#[derive(Debug, Clone)]
pub struct BallSample {
    pub rigid_body: boxcars::RigidBody,
}

impl BallSample {
    pub fn position(&self) -> glam::Vec3 {
        vec_to_glam(&self.rigid_body.location)
    }

    pub fn velocity(&self) -> glam::Vec3 {
        self.rigid_body
            .linear_velocity
            .as_ref()
            .map(vec_to_glam)
            .unwrap_or(glam::Vec3::ZERO)
    }
}

fn interval_fraction_in_scalar_range(start: f32, end: f32, min_value: f32, max_value: f32) -> f32 {
    if (end - start).abs() <= f32::EPSILON {
        return ((start >= min_value) && (start < max_value)) as i32 as f32;
    }

    let t_at_min = (min_value - start) / (end - start);
    let t_at_max = (max_value - start) / (end - start);
    let interval_start = t_at_min.min(t_at_max).max(0.0);
    let interval_end = t_at_min.max(t_at_max).min(1.0);
    (interval_end - interval_start).max(0.0)
}

fn interval_fraction_below_threshold(start: f32, end: f32, threshold: f32) -> f32 {
    if (end - start).abs() <= f32::EPSILON {
        return (start < threshold) as i32 as f32;
    }

    let threshold_time = ((threshold - start) / (end - start)).clamp(0.0, 1.0);
    if start < threshold {
        if end < threshold {
            1.0
        } else {
            threshold_time
        }
    } else if end < threshold {
        1.0 - threshold_time
    } else {
        0.0
    }
}

fn interval_fraction_above_threshold(start: f32, end: f32, threshold: f32) -> f32 {
    if (end - start).abs() <= f32::EPSILON {
        return (start > threshold) as i32 as f32;
    }

    let threshold_time = ((threshold - start) / (end - start)).clamp(0.0, 1.0);
    if start > threshold {
        if end > threshold {
            1.0
        } else {
            threshold_time
        }
    } else if end > threshold {
        1.0 - threshold_time
    } else {
        0.0
    }
}

#[derive(Debug, Clone)]
pub struct PlayerSample {
    pub player_id: PlayerId,
    pub is_team_0: bool,
    pub rigid_body: Option<boxcars::RigidBody>,
    pub boost_amount: Option<f32>,
    pub last_boost_amount: Option<f32>,
    pub boost_active: bool,
    pub dodge_active: bool,
    pub powerslide_active: bool,
    pub match_goals: Option<i32>,
    pub match_assists: Option<i32>,
    pub match_saves: Option<i32>,
    pub match_shots: Option<i32>,
    pub match_score: Option<i32>,
}

impl PlayerSample {
    pub fn position(&self) -> Option<glam::Vec3> {
        self.rigid_body.as_ref().map(|rb| vec_to_glam(&rb.location))
    }

    pub fn velocity(&self) -> Option<glam::Vec3> {
        self.rigid_body
            .as_ref()
            .and_then(|rb| rb.linear_velocity.as_ref().map(vec_to_glam))
    }

    pub fn speed(&self) -> Option<f32> {
        self.velocity().map(|velocity| velocity.length())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemoEventSample {
    pub attacker: PlayerId,
    pub victim: PlayerId,
}

#[derive(Debug, Clone)]
pub struct StatsSample {
    pub frame_number: usize,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
    pub game_state: Option<i32>,
    pub ball_has_been_hit: Option<bool>,
    pub kickoff_countdown_time: Option<i32>,
    pub team_zero_score: Option<i32>,
    pub team_one_score: Option<i32>,
    pub possession_team_is_team_0: Option<bool>,
    pub scored_on_team_is_team_0: Option<bool>,
    pub current_in_game_team_player_counts: Option<[usize; 2]>,
    pub ball: Option<BallSample>,
    pub players: Vec<PlayerSample>,
    pub active_demos: Vec<DemoEventSample>,
    pub demo_events: Vec<DemolishInfo>,
    pub boost_pad_events: Vec<BoostPadEvent>,
    pub touch_events: Vec<TouchEvent>,
    pub dodge_refreshed_events: Vec<DodgeRefreshedEvent>,
    pub player_stat_events: Vec<PlayerStatEvent>,
    pub goal_events: Vec<GoalEvent>,
}

const GAME_STATE_KICKOFF_COUNTDOWN: i32 = 55;
const GAME_STATE_GOAL_SCORED_REPLAY: i32 = 86;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LivePlayTracker {
    post_goal_phase_active: bool,
    last_score: Option<(i32, i32)>,
}

impl LivePlayTracker {
    fn current_score(sample: &StatsSample) -> Option<(i32, i32)> {
        Some((sample.team_zero_score?, sample.team_one_score?))
    }

    fn kickoff_phase_active(sample: &StatsSample) -> bool {
        sample.game_state == Some(GAME_STATE_KICKOFF_COUNTDOWN)
            || sample.kickoff_countdown_time.is_some_and(|time| time > 0)
    }

    fn live_play_internal(&mut self, sample: &StatsSample) -> bool {
        let kickoff_phase_active = Self::kickoff_phase_active(sample);
        let score_changed = Self::current_score(sample)
            .zip(self.last_score)
            .is_some_and(
                |((team_zero_score, team_one_score), (last_team_zero, last_team_one))| {
                    team_zero_score > last_team_zero || team_one_score > last_team_one
                },
            );

        if !sample.goal_events.is_empty() || score_changed {
            self.post_goal_phase_active = true;
        }

        let live_play = sample.is_live_play() && !self.post_goal_phase_active;

        if kickoff_phase_active {
            self.post_goal_phase_active = false;
        }

        if let Some(score) = Self::current_score(sample) {
            self.last_score = Some(score);
        }

        live_play
    }

    pub fn is_live_play(&mut self, sample: &StatsSample) -> bool {
        self.live_play_internal(sample)
    }
}

impl StatsSample {
    pub(crate) fn from_processor(
        processor: &ReplayProcessor,
        frame_number: usize,
        current_time: f32,
        dt: f32,
    ) -> SubtrActorResult<Self> {
        let ball = processor
            .get_interpolated_ball_rigid_body(current_time, 0.0)
            .ok()
            .filter(|rigid_body| !rigid_body.sleeping)
            .map(|rigid_body| BallSample { rigid_body });

        let mut players = Vec::new();
        for player_id in processor.iter_player_ids_in_order() {
            // Some replays include metadata/header players before their actor
            // graph is fully available in-frame. Skip those players until their
            // live actor/team links resolve instead of aborting the whole sample.
            let Ok(is_team_0) = processor.get_player_is_team_0(player_id) else {
                continue;
            };
            players.push(PlayerSample {
                player_id: player_id.clone(),
                is_team_0,
                rigid_body: processor
                    .get_interpolated_player_rigid_body(player_id, current_time, 0.0)
                    .ok()
                    .filter(|rigid_body| !rigid_body.sleeping),
                boost_amount: processor.get_player_boost_level(player_id).ok(),
                last_boost_amount: processor.get_player_last_boost_level(player_id).ok(),
                boost_active: processor.get_boost_active(player_id).unwrap_or(0) % 2 == 1,
                dodge_active: processor.get_dodge_active(player_id).unwrap_or(0) % 2 == 1,
                powerslide_active: processor.get_powerslide_active(player_id).unwrap_or(false),
                match_goals: processor.get_player_match_goals(player_id).ok(),
                match_assists: processor.get_player_match_assists(player_id).ok(),
                match_saves: processor.get_player_match_saves(player_id).ok(),
                match_shots: processor.get_player_match_shots(player_id).ok(),
                match_score: processor.get_player_match_score(player_id).ok(),
            });
        }

        let team_scores = processor.get_team_scores().ok();
        let possession_team_is_team_0 =
            processor
                .get_ball_hit_team_num()
                .ok()
                .and_then(|team_num| match team_num {
                    0 => Some(true),
                    1 => Some(false),
                    _ => None,
                });
        let scored_on_team_is_team_0 =
            processor
                .get_scored_on_team_num()
                .ok()
                .and_then(|team_num| match team_num {
                    0 => Some(true),
                    1 => Some(false),
                    _ => None,
                });
        let active_demos = if let Ok(demos) = processor.get_active_demos() {
            demos
                .filter_map(|demo| {
                    let attacker = processor
                        .get_player_id_from_car_id(&demo.attacker_actor_id())
                        .ok()?;
                    let victim = processor
                        .get_player_id_from_car_id(&demo.victim_actor_id())
                        .ok()?;
                    Some(DemoEventSample { attacker, victim })
                })
                .collect()
        } else {
            Vec::new()
        };

        Ok(Self {
            frame_number,
            time: current_time,
            dt,
            seconds_remaining: processor.get_seconds_remaining().ok(),
            game_state: processor.get_replicated_state_name().ok(),
            ball_has_been_hit: processor.get_ball_has_been_hit().ok(),
            kickoff_countdown_time: processor.get_replicated_game_state_time_remaining().ok(),
            team_zero_score: team_scores.map(|scores| scores.0),
            team_one_score: team_scores.map(|scores| scores.1),
            possession_team_is_team_0,
            scored_on_team_is_team_0,
            current_in_game_team_player_counts: Some(
                processor.current_in_game_team_player_counts(),
            ),
            ball,
            players,
            active_demos,
            demo_events: Vec::new(),
            boost_pad_events: processor.current_frame_boost_pad_events().to_vec(),
            touch_events: processor.current_frame_touch_events().to_vec(),
            dodge_refreshed_events: processor.current_frame_dodge_refreshed_events().to_vec(),
            player_stat_events: processor.current_frame_player_stat_events().to_vec(),
            goal_events: processor.current_frame_goal_events().to_vec(),
        })
    }

    /// Returns whether time-based stats should treat this sample as live play.
    ///
    /// We exclude frozen kickoff countdown frames and post-goal replay frames,
    /// but keep the movable pre-touch kickoff approach live.
    ///
    /// Use [`LivePlayTracker`] when you need to exclude the full post-goal
    /// reset segment that can continue after the goal frame itself.
    pub fn is_live_play(&self) -> bool {
        if matches!(
            self.game_state,
            Some(GAME_STATE_KICKOFF_COUNTDOWN | GAME_STATE_GOAL_SCORED_REPLAY)
        ) {
            return false;
        }

        true
    }

    pub fn current_in_game_team_player_count(&self, is_team_0: bool) -> usize {
        self.current_in_game_team_player_counts
            .map(|counts| counts[usize::from(!is_team_0)])
            .unwrap_or_else(|| {
                self.players
                    .iter()
                    .filter(|player| player.is_team_0 == is_team_0)
                    .count()
            })
    }
}

pub trait StatsReducer {
    fn on_replay_meta(&mut self, _meta: &ReplayMeta) -> SubtrActorResult<()> {
        Ok(())
    }

    fn required_derived_signals(&self) -> Vec<DerivedSignalId> {
        Vec::new()
    }

    fn on_sample(&mut self, _sample: &StatsSample) -> SubtrActorResult<()> {
        Ok(())
    }

    fn on_sample_with_context(
        &mut self,
        sample: &StatsSample,
        _ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        self.on_sample(sample)
    }

    fn finish(&mut self) -> SubtrActorResult<()> {
        Ok(())
    }
}

#[derive(Default)]
pub struct CompositeStatsReducer {
    children: Vec<Box<dyn StatsReducer>>,
}

impl CompositeStatsReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push<R: StatsReducer + 'static>(&mut self, reducer: R) {
        self.children.push(Box::new(reducer));
    }

    pub fn with_child<R: StatsReducer + 'static>(mut self, reducer: R) -> Self {
        self.push(reducer);
        self
    }

    pub fn children(&self) -> &[Box<dyn StatsReducer>] {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut [Box<dyn StatsReducer>] {
        &mut self.children
    }
}

impl StatsReducer for CompositeStatsReducer {
    fn on_replay_meta(&mut self, meta: &ReplayMeta) -> SubtrActorResult<()> {
        for child in &mut self.children {
            child.on_replay_meta(meta)?;
        }
        Ok(())
    }

    fn required_derived_signals(&self) -> Vec<DerivedSignalId> {
        let mut signals = HashSet::new();
        for child in &self.children {
            signals.extend(child.required_derived_signals());
        }
        signals.into_iter().collect()
    }

    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        for child in &mut self.children {
            child.on_sample(sample)?;
        }
        Ok(())
    }

    fn on_sample_with_context(
        &mut self,
        sample: &StatsSample,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        for child in &mut self.children {
            child.on_sample_with_context(sample, ctx)?;
        }
        Ok(())
    }

    fn finish(&mut self) -> SubtrActorResult<()> {
        for child in &mut self.children {
            child.finish()?;
        }
        Ok(())
    }
}

pub struct ReducerCollector<R> {
    reducer: R,
    derived_signals: DerivedSignalGraph,
    last_sample_time: Option<f32>,
    replay_meta_initialized: bool,
    last_demolish_count: usize,
    last_boost_pad_event_count: usize,
    last_touch_event_count: usize,
    last_player_stat_event_count: usize,
    last_goal_event_count: usize,
}

impl<R: StatsReducer> ReducerCollector<R> {
    pub fn new(reducer: R) -> Self {
        let derived_signals = derived_signal_graph_for_ids(reducer.required_derived_signals());
        Self {
            reducer,
            derived_signals,
            last_sample_time: None,
            replay_meta_initialized: false,
            last_demolish_count: 0,
            last_boost_pad_event_count: 0,
            last_touch_event_count: 0,
            last_player_stat_event_count: 0,
            last_goal_event_count: 0,
        }
    }
}

impl<R> ReducerCollector<R> {
    pub fn into_inner(self) -> R {
        self.reducer
    }

    pub fn reducer(&self) -> &R {
        &self.reducer
    }

    pub fn reducer_mut(&mut self) -> &mut R {
        &mut self.reducer
    }
}

impl<R: StatsReducer> From<R> for ReducerCollector<R> {
    fn from(reducer: R) -> Self {
        Self::new(reducer)
    }
}

impl<R: StatsReducer> Collector for ReducerCollector<R> {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        _frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        if !self.replay_meta_initialized {
            let replay_meta = processor.get_replay_meta()?;
            self.derived_signals.on_replay_meta(&replay_meta)?;
            self.reducer.on_replay_meta(&replay_meta)?;
            self.replay_meta_initialized = true;
        }

        let dt = self
            .last_sample_time
            .map(|last_time| (current_time - last_time).max(0.0))
            .unwrap_or(0.0);
        let mut sample = StatsSample::from_processor(processor, frame_number, current_time, dt)?;
        sample.active_demos.clear();
        sample.demo_events = processor.demolishes[self.last_demolish_count..].to_vec();
        sample.boost_pad_events =
            processor.boost_pad_events[self.last_boost_pad_event_count..].to_vec();
        sample.touch_events = processor.touch_events[self.last_touch_event_count..].to_vec();
        sample.player_stat_events =
            processor.player_stat_events[self.last_player_stat_event_count..].to_vec();
        sample.goal_events = processor.goal_events[self.last_goal_event_count..].to_vec();
        let analysis_context = self.derived_signals.evaluate(&sample)?;
        self.reducer
            .on_sample_with_context(&sample, analysis_context)?;
        self.last_sample_time = Some(current_time);
        self.last_demolish_count = processor.demolishes.len();
        self.last_boost_pad_event_count = processor.boost_pad_events.len();
        self.last_touch_event_count = processor.touch_events.len();
        self.last_player_stat_event_count = processor.player_stat_events.len();
        self.last_goal_event_count = processor.goal_events.len();

        Ok(TimeAdvance::NextFrame)
    }

    fn finish_replay(&mut self, _processor: &ReplayProcessor) -> SubtrActorResult<()> {
        self.derived_signals.finish()?;
        self.reducer.finish()
    }
}

const CAR_MAX_SPEED: f32 = 2300.0;
const SUPERSONIC_SPEED_THRESHOLD: f32 = 2200.0;
const BOOST_SPEED_THRESHOLD: f32 = 1410.0;
const GROUND_Z_THRESHOLD: f32 = 20.0;
const POWERSLIDE_MAX_Z_THRESHOLD: f32 = 40.0;
const BALL_RADIUS_Z: f32 = 92.75;
const BALL_CARRY_MIN_BALL_Z: f32 = BALL_RADIUS_Z + 5.0;
const BALL_CARRY_MAX_BALL_Z: f32 = 600.0;
const BALL_CARRY_MAX_HORIZONTAL_GAP: f32 = BALL_RADIUS_Z * 1.4;
const BALL_CARRY_MAX_VERTICAL_GAP: f32 = 220.0;
const BALL_CARRY_MIN_DURATION: f32 = 1.0;
// Ballchasing's high-air bucket lines up better with the car center clearing a
// crossbar-height ball than with plain goal height.
const HIGH_AIR_Z_THRESHOLD: f32 = 642.775 + BALL_RADIUS_Z;
// Ballchasing's defensive / neutral / offensive zones track the standard
// soccar lane markings more closely than a literal geometric third of the full
// playable length.
const FIELD_ZONE_BOUNDARY_Y: f32 = BOOST_PAD_SIDE_LANE_Y;
/// Approximate length of two Octane hitboxes in Unreal units.
const DEFAULT_MOST_BACK_FORWARD_THRESHOLD_Y: f32 = 236.0;
const SMALL_PAD_AMOUNT_RAW: f32 = BOOST_MAX_AMOUNT * 12.0 / 100.0;
const BOOST_ZERO_BAND_RAW: f32 = 1.0;
const BOOST_FULL_BAND_MIN_RAW: f32 = BOOST_MAX_AMOUNT - 1.0;
const STANDARD_PAD_MATCH_RADIUS_SMALL: f32 = 450.0;
const STANDARD_PAD_MATCH_RADIUS_BIG: f32 = 1000.0;
const BOOST_PAD_MIDFIELD_TOLERANCE_Y: f32 = 128.0;
const BOOST_PAD_SMALL_Z: f32 = 70.0;
const BOOST_PAD_BIG_Z: f32 = 73.0;
const BOOST_PAD_BACK_CORNER_X: f32 = 3072.0;
const BOOST_PAD_BACK_CORNER_Y: f32 = 4096.0;
const BOOST_PAD_BACK_LANE_X: f32 = 1792.0;
const BOOST_PAD_BACK_LANE_Y: f32 = 4184.0;
const BOOST_PAD_BACK_MID_X: f32 = 940.0;
const BOOST_PAD_BACK_MID_Y: f32 = 3308.0;
const BOOST_PAD_CENTER_BACK_Y: f32 = 2816.0;
const BOOST_PAD_SIDE_WALL_X: f32 = 3584.0;
const BOOST_PAD_SIDE_WALL_Y: f32 = 2484.0;
const BOOST_PAD_SIDE_LANE_X: f32 = 1788.0;
const BOOST_PAD_SIDE_LANE_Y: f32 = 2300.0;
const BOOST_PAD_FRONT_LANE_X: f32 = 2048.0;
const BOOST_PAD_FRONT_LANE_Y: f32 = 1036.0;
const BOOST_PAD_CENTER_X: f32 = 1024.0;
const BOOST_PAD_CENTER_MID_Y: f32 = 1024.0;
const BOOST_PAD_GOAL_LINE_Y: f32 = 4240.0;

fn push_pad(
    pads: &mut Vec<(glam::Vec3, BoostPadSize)>,
    x: f32,
    y: f32,
    z: f32,
    size: BoostPadSize,
) {
    pads.push((glam::Vec3::new(x, y, z), size));
}

fn push_mirror_x(
    pads: &mut Vec<(glam::Vec3, BoostPadSize)>,
    x: f32,
    y: f32,
    z: f32,
    size: BoostPadSize,
) {
    push_pad(pads, -x, y, z, size);
    push_pad(pads, x, y, z, size);
}

fn push_mirror_y(
    pads: &mut Vec<(glam::Vec3, BoostPadSize)>,
    x: f32,
    y: f32,
    z: f32,
    size: BoostPadSize,
) {
    push_pad(pads, x, -y, z, size);
    push_pad(pads, x, y, z, size);
}

fn push_mirror_xy(
    pads: &mut Vec<(glam::Vec3, BoostPadSize)>,
    x: f32,
    y: f32,
    z: f32,
    size: BoostPadSize,
) {
    push_mirror_x(pads, x, -y, z, size);
    push_mirror_x(pads, x, y, z, size);
}

fn build_standard_soccar_boost_pad_layout() -> Vec<(glam::Vec3, BoostPadSize)> {
    let mut pads = Vec::with_capacity(34);

    push_mirror_y(
        &mut pads,
        0.0,
        BOOST_PAD_GOAL_LINE_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_xy(
        &mut pads,
        BOOST_PAD_BACK_LANE_X,
        BOOST_PAD_BACK_LANE_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_xy(
        &mut pads,
        BOOST_PAD_BACK_CORNER_X,
        BOOST_PAD_BACK_CORNER_Y,
        BOOST_PAD_BIG_Z,
        BoostPadSize::Big,
    );
    push_mirror_xy(
        &mut pads,
        BOOST_PAD_BACK_MID_X,
        BOOST_PAD_BACK_MID_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_y(
        &mut pads,
        0.0,
        BOOST_PAD_CENTER_BACK_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_xy(
        &mut pads,
        BOOST_PAD_SIDE_WALL_X,
        BOOST_PAD_SIDE_WALL_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_xy(
        &mut pads,
        BOOST_PAD_SIDE_LANE_X,
        BOOST_PAD_SIDE_LANE_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_xy(
        &mut pads,
        BOOST_PAD_FRONT_LANE_X,
        BOOST_PAD_FRONT_LANE_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_y(
        &mut pads,
        0.0,
        BOOST_PAD_CENTER_MID_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_x(
        &mut pads,
        BOOST_PAD_SIDE_WALL_X,
        0.0,
        BOOST_PAD_BIG_Z,
        BoostPadSize::Big,
    );
    push_mirror_x(
        &mut pads,
        BOOST_PAD_CENTER_X,
        0.0,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );

    pads
}

static STANDARD_SOCCAR_BOOST_PAD_LAYOUT: LazyLock<Vec<(glam::Vec3, BoostPadSize)>> =
    LazyLock::new(build_standard_soccar_boost_pad_layout);

pub fn standard_soccar_boost_pad_layout() -> &'static [(glam::Vec3, BoostPadSize)] {
    STANDARD_SOCCAR_BOOST_PAD_LAYOUT.as_slice()
}

fn normalized_y(is_team_0: bool, position: glam::Vec3) -> f32 {
    if is_team_0 {
        position.y
    } else {
        -position.y
    }
}

fn is_enemy_side(is_team_0: bool, position: glam::Vec3) -> bool {
    normalized_y(is_team_0, position) > BOOST_PAD_MIDFIELD_TOLERANCE_Y
}

fn standard_soccar_boost_pad_position(index: usize) -> glam::Vec3 {
    STANDARD_SOCCAR_BOOST_PAD_LAYOUT[index].0
}

#[derive(Debug, Clone, Default)]
struct PadPositionEstimate {
    observations: Vec<glam::Vec3>,
}

impl PadPositionEstimate {
    fn observe(&mut self, position: glam::Vec3) {
        self.observations.push(position);
    }

    fn observations(&self) -> &[glam::Vec3] {
        self.observations.as_slice()
    }

    fn mean(&self) -> Option<glam::Vec3> {
        if self.observations.is_empty() {
            return None;
        }

        let sum = self
            .observations
            .iter()
            .copied()
            .fold(glam::Vec3::ZERO, |acc, position| acc + position);
        Some(sum / self.observations.len() as f32)
    }
}

fn header_prop_to_f32(prop: &HeaderProp) -> Option<f32> {
    match prop {
        HeaderProp::Float(value) => Some(*value),
        HeaderProp::Int(value) => Some(*value as f32),
        HeaderProp::QWord(value) => Some(*value as f32),
        _ => None,
    }
}

fn get_header_f32(stats: &HashMap<String, HeaderProp>, keys: &[&str]) -> Option<f32> {
    keys.iter()
        .find_map(|key| stats.get(*key).and_then(header_prop_to_f32))
}

pub mod powerslide;
#[allow(unused_imports)]
pub use powerslide::*;
pub mod analysis;
pub use analysis::*;
pub mod pressure;
#[allow(unused_imports)]
pub use pressure::*;
pub mod rush;
#[allow(unused_imports)]
pub use rush::*;
pub mod possession;
#[allow(unused_imports)]
pub use possession::*;
pub mod settings;
pub use settings::*;
pub mod match_stats;
pub use match_stats::*;
pub mod backboard;
pub use backboard::*;
pub mod double_tap;
pub use double_tap::*;
pub mod demo;
pub use demo::*;
pub mod ceiling_shot;
pub use ceiling_shot::*;
pub mod dodge_reset;
pub use dodge_reset::*;
pub mod musty_flick;
pub use musty_flick::*;
pub mod touch;
pub use touch::*;
pub mod fifty_fifty;
pub use fifty_fifty::*;
pub mod speed_flip;
pub use speed_flip::*;
pub mod movement;
pub use movement::*;
pub mod positioning;
pub use positioning::*;
pub mod ball_carry;
pub use ball_carry::*;
pub mod boost;
pub use boost::*;
