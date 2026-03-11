use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use boxcars;
use boxcars::HeaderProp;
use serde::Serialize;

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
    pub team_zero_score: Option<i32>,
    pub team_one_score: Option<i32>,
    pub possession_team_is_team_0: Option<bool>,
    pub scored_on_team_is_team_0: Option<bool>,
    pub ball: Option<BallSample>,
    pub players: Vec<PlayerSample>,
    pub active_demos: Vec<DemoEventSample>,
    pub demo_events: Vec<DemolishInfo>,
    pub boost_pad_events: Vec<BoostPadEvent>,
    pub touch_events: Vec<TouchEvent>,
    pub player_stat_events: Vec<PlayerStatEvent>,
    pub goal_events: Vec<GoalEvent>,
}

const GAME_STATE_KICKOFF_COUNTDOWN: i32 = 55;
const GAME_STATE_GOAL_SCORED_REPLAY: i32 = 86;

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
            team_zero_score: team_scores.map(|scores| scores.0),
            team_one_score: team_scores.map(|scores| scores.1),
            possession_team_is_team_0,
            scored_on_team_is_team_0,
            ball,
            players,
            active_demos,
            demo_events: Vec::new(),
            boost_pad_events: processor.current_frame_boost_pad_events().to_vec(),
            touch_events: processor.current_frame_touch_events().to_vec(),
            player_stat_events: processor.current_frame_player_stat_events().to_vec(),
            goal_events: processor.current_frame_goal_events().to_vec(),
        })
    }

    /// Returns whether time-based stats should treat this sample as live play.
    ///
    /// We exclude frozen kickoff countdown frames and post-goal replay frames,
    /// but keep unknown states live so we do not accidentally discard stats
    /// from replay variants whose state enum values we have not catalogued yet.
    pub fn is_live_play(&self) -> bool {
        if matches!(
            self.game_state,
            Some(GAME_STATE_KICKOFF_COUNTDOWN | GAME_STATE_GOAL_SCORED_REPLAY)
        ) {
            return false;
        }

        !matches!(self.ball_has_been_hit, Some(false))
    }
}

pub trait StatsReducer {
    fn on_replay_meta(&mut self, _meta: &ReplayMeta) -> SubtrActorResult<()> {
        Ok(())
    }

    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()>;

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

    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        for child in &mut self.children {
            child.on_sample(sample)?;
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
    last_sample_time: Option<f32>,
    replay_meta_initialized: bool,
    last_demolish_count: usize,
    last_boost_pad_event_count: usize,
    last_touch_event_count: usize,
    last_player_stat_event_count: usize,
    last_goal_event_count: usize,
}

impl<R> ReducerCollector<R> {
    pub fn new(reducer: R) -> Self {
        Self {
            reducer,
            last_sample_time: None,
            replay_meta_initialized: false,
            last_demolish_count: 0,
            last_boost_pad_event_count: 0,
            last_touch_event_count: 0,
            last_player_stat_event_count: 0,
            last_goal_event_count: 0,
        }
    }

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

impl<R> From<R> for ReducerCollector<R> {
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
        self.reducer.on_sample(&sample)?;
        self.last_sample_time = Some(current_time);
        self.last_demolish_count = processor.demolishes.len();
        self.last_boost_pad_event_count = processor.boost_pad_events.len();
        self.last_touch_event_count = processor.touch_events.len();
        self.last_player_stat_event_count = processor.player_stat_events.len();
        self.last_goal_event_count = processor.goal_events.len();

        Ok(TimeAdvance::NextFrame)
    }

    fn finish_replay(&mut self, _processor: &ReplayProcessor) -> SubtrActorResult<()> {
        self.reducer.finish()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct PowerslideStats {
    pub total_duration: f32,
    pub press_count: u32,
}

impl PowerslideStats {
    pub fn average_duration(&self) -> f32 {
        if self.press_count == 0 {
            0.0
        } else {
            self.total_duration / self.press_count as f32
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PowerslideReducer {
    player_stats: HashMap<PlayerId, PowerslideStats>,
    team_zero_stats: PowerslideStats,
    team_one_stats: PowerslideStats,
    last_active: HashMap<PlayerId, bool>,
}

impl PowerslideReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, PowerslideStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &PowerslideStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &PowerslideStats {
        &self.team_one_stats
    }

    fn is_effective_powerslide(player: &PlayerSample) -> bool {
        player.powerslide_active
            && player
                .position()
                .map(|position| position.z <= POWERSLIDE_MAX_Z_THRESHOLD)
                .unwrap_or(false)
    }
}

impl StatsReducer for PowerslideReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        let live_play = sample.is_live_play();
        for player in &sample.players {
            let effective_powerslide = Self::is_effective_powerslide(player);
            let previous_active = self
                .last_active
                .get(&player.player_id)
                .copied()
                .unwrap_or(false);
            let stats = self
                .player_stats
                .entry(player.player_id.clone())
                .or_default();
            let team_stats = if player.is_team_0 {
                &mut self.team_zero_stats
            } else {
                &mut self.team_one_stats
            };

            if live_play && effective_powerslide {
                stats.total_duration += sample.dt;
                team_stats.total_duration += sample.dt;
            }

            if live_play && effective_powerslide && !previous_active {
                stats.press_count += 1;
                team_stats.press_count += 1;
            }

            self.last_active
                .insert(player.player_id.clone(), effective_powerslide);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PressureReducer {
    team_zero_side_duration: f32,
    team_one_side_duration: f32,
}

impl PressureReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn team_zero_side_duration(&self) -> f32 {
        self.team_zero_side_duration
    }

    pub fn team_one_side_duration(&self) -> f32 {
        self.team_one_side_duration
    }

    pub fn total_tracked_duration(&self) -> f32 {
        self.team_zero_side_duration + self.team_one_side_duration
    }

    pub fn team_zero_side_pct(&self) -> f32 {
        if self.total_tracked_duration() == 0.0 {
            0.0
        } else {
            self.team_zero_side_duration * 100.0 / self.total_tracked_duration()
        }
    }

    pub fn team_one_side_pct(&self) -> f32 {
        if self.total_tracked_duration() == 0.0 {
            0.0
        } else {
            self.team_one_side_duration * 100.0 / self.total_tracked_duration()
        }
    }
}

impl StatsReducer for PressureReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        if !sample.is_live_play() {
            return Ok(());
        }
        if let Some(ball) = &sample.ball {
            if ball.position().y < 0.0 {
                self.team_zero_side_duration += sample.dt;
            } else {
                self.team_one_side_duration += sample.dt;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct PossessionStats {
    pub tracked_time: f32,
    pub team_zero_time: f32,
    pub team_one_time: f32,
}

impl PossessionStats {
    pub fn team_zero_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.team_zero_time * 100.0 / self.tracked_time
        }
    }

    pub fn team_one_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.team_one_time * 100.0 / self.tracked_time
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PossessionReducer {
    stats: PossessionStats,
    current_team_is_team_0: Option<bool>,
}

impl PossessionReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> &PossessionStats {
        &self.stats
    }
}

impl StatsReducer for PossessionReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        let live_play = sample.is_live_play();
        let active_team_before_sample = if sample.touch_events.is_empty() {
            self.current_team_is_team_0
                .or(sample.possession_team_is_team_0)
        } else {
            self.current_team_is_team_0
        };

        if live_play {
            if let Some(possession_team_is_team_0) = active_team_before_sample {
                self.stats.tracked_time += sample.dt;
                if possession_team_is_team_0 {
                    self.stats.team_zero_time += sample.dt;
                } else {
                    self.stats.team_one_time += sample.dt;
                }
            }
        }

        if let Some(last_touch) = sample.touch_events.last() {
            self.current_team_is_team_0 = Some(last_touch.team_is_team_0);
        } else {
            self.current_team_is_team_0 = sample
                .possession_team_is_team_0
                .or(self.current_team_is_team_0);
        }
        Ok(())
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
const SMALL_PAD_AMOUNT_RAW: f32 = BOOST_MAX_AMOUNT * 12.0 / 100.0;
const BOOST_ZERO_BAND_RAW: f32 = 1.0;
const BOOST_FULL_BAND_MIN_RAW: f32 = BOOST_MAX_AMOUNT - 1.0;
const STANDARD_PAD_MATCH_RADIUS: f32 = 400.0;
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

#[derive(Debug, Clone, Copy, Default)]
struct PadPositionEstimate {
    sum: glam::Vec3,
    count: u32,
}

impl PadPositionEstimate {
    fn observe(&mut self, position: glam::Vec3) {
        self.sum += position;
        self.count += 1;
    }

    fn mean(&self) -> Option<glam::Vec3> {
        if self.count == 0 {
            None
        } else {
            Some(self.sum / self.count as f32)
        }
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

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlayerSettings {
    pub steering_sensitivity: Option<f32>,
    pub camera_fov: Option<f32>,
    pub camera_height: Option<f32>,
    pub camera_pitch: Option<f32>,
    pub camera_distance: Option<f32>,
    pub camera_stiffness: Option<f32>,
    pub camera_swivel_speed: Option<f32>,
    pub camera_transition_speed: Option<f32>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SettingsReducer {
    player_settings: HashMap<PlayerId, PlayerSettings>,
}

impl SettingsReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_settings(&self) -> &HashMap<PlayerId, PlayerSettings> {
        &self.player_settings
    }
}

impl StatsReducer for SettingsReducer {
    fn on_replay_meta(&mut self, meta: &ReplayMeta) -> SubtrActorResult<()> {
        for player in meta.player_order() {
            let Some(stats) = &player.stats else {
                continue;
            };
            self.player_settings.insert(
                player.remote_id.clone(),
                PlayerSettings {
                    steering_sensitivity: get_header_f32(
                        stats,
                        &["SteeringSensitivity", "SteerSensitivity"],
                    ),
                    camera_fov: get_header_f32(stats, &["CameraFOV"]),
                    camera_height: get_header_f32(stats, &["CameraHeight"]),
                    camera_pitch: get_header_f32(stats, &["CameraPitch"]),
                    camera_distance: get_header_f32(stats, &["CameraDistance"]),
                    camera_stiffness: get_header_f32(stats, &["CameraStiffness"]),
                    camera_swivel_speed: get_header_f32(stats, &["CameraSwivelSpeed"]),
                    camera_transition_speed: get_header_f32(stats, &["CameraTransitionSpeed"]),
                },
            );
        }
        Ok(())
    }

    fn on_sample(&mut self, _sample: &StatsSample) -> SubtrActorResult<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct CorePlayerStats {
    pub score: i32,
    pub goals: i32,
    pub assists: i32,
    pub saves: i32,
    pub shots: i32,
    pub goals_conceded_while_last_defender: u32,
}

impl CorePlayerStats {
    pub fn shooting_percentage(&self) -> f32 {
        if self.shots == 0 {
            0.0
        } else {
            self.goals as f32 * 100.0 / self.shots as f32
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct CoreTeamStats {
    pub score: i32,
    pub goals: i32,
    pub assists: i32,
    pub saves: i32,
    pub shots: i32,
}

impl CoreTeamStats {
    pub fn shooting_percentage(&self) -> f32 {
        if self.shots == 0 {
            0.0
        } else {
            self.goals as f32 * 100.0 / self.shots as f32
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum TimelineEventKind {
    Goal,
    Shot,
    Save,
    Assist,
    Kill,
    Death,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TimelineEvent {
    pub time: f32,
    pub kind: TimelineEventKind,
    pub player_id: Option<PlayerId>,
    pub is_team_0: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct MatchStatsReducer {
    player_stats: HashMap<PlayerId, CorePlayerStats>,
    player_teams: HashMap<PlayerId, bool>,
    previous_player_stats: HashMap<PlayerId, CorePlayerStats>,
    timeline: Vec<TimelineEvent>,
    pending_goal_events: Vec<GoalEvent>,
    previous_team_scores: Option<(i32, i32)>,
}

impl MatchStatsReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, CorePlayerStats> {
        &self.player_stats
    }

    pub fn timeline(&self) -> &[TimelineEvent] {
        &self.timeline
    }

    pub fn team_zero_stats(&self) -> CoreTeamStats {
        self.team_stats_for_side(true)
    }

    pub fn team_one_stats(&self) -> CoreTeamStats {
        self.team_stats_for_side(false)
    }

    fn team_stats_for_side(&self, is_team_0: bool) -> CoreTeamStats {
        self.player_stats
            .iter()
            .filter(|(player_id, _)| self.player_teams.get(*player_id) == Some(&is_team_0))
            .fold(CoreTeamStats::default(), |mut stats, (_, player_stats)| {
                stats.score += player_stats.score;
                stats.goals += player_stats.goals;
                stats.assists += player_stats.assists;
                stats.saves += player_stats.saves;
                stats.shots += player_stats.shots;
                stats
            })
    }

    fn emit_timeline_events(
        &mut self,
        time: f32,
        kind: TimelineEventKind,
        player_id: &PlayerId,
        is_team_0: bool,
        delta: i32,
    ) {
        for _ in 0..delta.max(0) {
            self.timeline.push(TimelineEvent {
                time,
                kind,
                player_id: Some(player_id.clone()),
                is_team_0: Some(is_team_0),
            });
        }
    }

    fn take_goal_event_time(&mut self, player_id: &PlayerId, is_team_0: bool) -> Option<f32> {
        if let Some(index) = self.pending_goal_events.iter().position(|event| {
            event.scoring_team_is_team_0 == is_team_0 && event.player.as_ref() == Some(player_id)
        }) {
            return Some(self.pending_goal_events.remove(index).time);
        }

        self.pending_goal_events
            .iter()
            .position(|event| event.scoring_team_is_team_0 == is_team_0)
            .map(|index| self.pending_goal_events.remove(index).time)
    }

    fn last_defender(
        &self,
        sample: &StatsSample,
        defending_team_is_team_0: bool,
    ) -> Option<PlayerId> {
        sample
            .players
            .iter()
            .filter(|player| player.is_team_0 == defending_team_is_team_0)
            .filter_map(|player| {
                player
                    .position()
                    .map(|position| (player.player_id.clone(), position.y))
            })
            .reduce(|current, candidate| {
                if defending_team_is_team_0 {
                    if candidate.1 < current.1 {
                        candidate
                    } else {
                        current
                    }
                } else if candidate.1 > current.1 {
                    candidate
                } else {
                    current
                }
            })
            .map(|(player_id, _)| player_id)
    }
}

impl StatsReducer for MatchStatsReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        self.pending_goal_events
            .extend(sample.goal_events.iter().cloned());
        let mut processor_event_counts: HashMap<(PlayerId, TimelineEventKind), i32> =
            HashMap::new();
        for event in &sample.player_stat_events {
            let kind = match event.kind {
                PlayerStatEventKind::Shot => TimelineEventKind::Shot,
                PlayerStatEventKind::Save => TimelineEventKind::Save,
                PlayerStatEventKind::Assist => TimelineEventKind::Assist,
            };
            self.timeline.push(TimelineEvent {
                time: event.time,
                kind,
                player_id: Some(event.player.clone()),
                is_team_0: Some(event.is_team_0),
            });
            *processor_event_counts
                .entry((event.player.clone(), kind))
                .or_default() += 1;
        }

        for player in &sample.players {
            self.player_teams
                .insert(player.player_id.clone(), player.is_team_0);
            let current_stats = CorePlayerStats {
                score: player.match_score.unwrap_or(0),
                goals: player.match_goals.unwrap_or(0),
                assists: player.match_assists.unwrap_or(0),
                saves: player.match_saves.unwrap_or(0),
                shots: player.match_shots.unwrap_or(0),
                goals_conceded_while_last_defender: self
                    .player_stats
                    .get(&player.player_id)
                    .map(|stats| stats.goals_conceded_while_last_defender)
                    .unwrap_or(0),
            };

            let previous_stats = self
                .previous_player_stats
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default();

            let shot_delta = current_stats.shots - previous_stats.shots;
            let save_delta = current_stats.saves - previous_stats.saves;
            let assist_delta = current_stats.assists - previous_stats.assists;
            let goal_delta = current_stats.goals - previous_stats.goals;
            let shot_fallback_delta = shot_delta
                - processor_event_counts
                    .get(&(player.player_id.clone(), TimelineEventKind::Shot))
                    .copied()
                    .unwrap_or(0);
            let save_fallback_delta = save_delta
                - processor_event_counts
                    .get(&(player.player_id.clone(), TimelineEventKind::Save))
                    .copied()
                    .unwrap_or(0);
            let assist_fallback_delta = assist_delta
                - processor_event_counts
                    .get(&(player.player_id.clone(), TimelineEventKind::Assist))
                    .copied()
                    .unwrap_or(0);

            if shot_fallback_delta > 0 {
                self.emit_timeline_events(
                    sample.time,
                    TimelineEventKind::Shot,
                    &player.player_id,
                    player.is_team_0,
                    shot_fallback_delta,
                );
            }
            if save_fallback_delta > 0 {
                self.emit_timeline_events(
                    sample.time,
                    TimelineEventKind::Save,
                    &player.player_id,
                    player.is_team_0,
                    save_fallback_delta,
                );
            }
            if assist_fallback_delta > 0 {
                self.emit_timeline_events(
                    sample.time,
                    TimelineEventKind::Assist,
                    &player.player_id,
                    player.is_team_0,
                    assist_fallback_delta,
                );
            }
            if goal_delta > 0 {
                for _ in 0..goal_delta.max(0) {
                    let goal_time = self
                        .take_goal_event_time(&player.player_id, player.is_team_0)
                        .unwrap_or(sample.time);
                    self.timeline.push(TimelineEvent {
                        time: goal_time,
                        kind: TimelineEventKind::Goal,
                        player_id: Some(player.player_id.clone()),
                        is_team_0: Some(player.is_team_0),
                    });
                }
            }

            self.previous_player_stats
                .insert(player.player_id.clone(), current_stats.clone());
            self.player_stats
                .insert(player.player_id.clone(), current_stats);
        }

        if let (Some(team_zero_score), Some(team_one_score)) =
            (sample.team_zero_score, sample.team_one_score)
        {
            if let Some((prev_team_zero_score, prev_team_one_score)) = self.previous_team_scores {
                let team_zero_delta = team_zero_score - prev_team_zero_score;
                let team_one_delta = team_one_score - prev_team_one_score;

                if team_zero_delta > 0 {
                    if let Some(last_defender) = self.last_defender(sample, false) {
                        if let Some(stats) = self.player_stats.get_mut(&last_defender) {
                            stats.goals_conceded_while_last_defender += team_zero_delta as u32;
                        }
                    }
                }

                if team_one_delta > 0 {
                    if let Some(last_defender) = self.last_defender(sample, true) {
                        if let Some(stats) = self.player_stats.get_mut(&last_defender) {
                            stats.goals_conceded_while_last_defender += team_one_delta as u32;
                        }
                    }
                }
            }

            self.previous_team_scores = Some((team_zero_score, team_one_score));
        }

        self.timeline.sort_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(())
    }
}

const DEMO_REPEAT_FRAME_WINDOW: usize = 8;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct DemoPlayerStats {
    pub demos_inflicted: u32,
    pub demos_taken: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct DemoTeamStats {
    pub demos_inflicted: u32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DemoReducer {
    player_stats: HashMap<PlayerId, DemoPlayerStats>,
    player_teams: HashMap<PlayerId, bool>,
    team_zero_stats: DemoTeamStats,
    team_one_stats: DemoTeamStats,
    timeline: Vec<TimelineEvent>,
    last_seen_frame: HashMap<(PlayerId, PlayerId), usize>,
}

impl DemoReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, DemoPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &DemoTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &DemoTeamStats {
        &self.team_one_stats
    }

    pub fn timeline(&self) -> &[TimelineEvent] {
        &self.timeline
    }

    fn should_count_demo(
        &mut self,
        attacker: &PlayerId,
        victim: &PlayerId,
        frame_number: usize,
    ) -> bool {
        let key = (attacker.clone(), victim.clone());
        let already_counted = self
            .last_seen_frame
            .get(&key)
            .map(|previous_frame| {
                frame_number.saturating_sub(*previous_frame) <= DEMO_REPEAT_FRAME_WINDOW
            })
            .unwrap_or(false);
        self.last_seen_frame.insert(key, frame_number);
        !already_counted
    }
}

impl StatsReducer for DemoReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        for player in &sample.players {
            self.player_teams
                .insert(player.player_id.clone(), player.is_team_0);
        }

        if !sample.demo_events.is_empty() {
            for demo in &sample.demo_events {
                self.record_demo(&demo.attacker, &demo.victim, demo.time, demo.frame);
            }
            return Ok(());
        }

        for demo in &sample.active_demos {
            self.record_demo(
                &demo.attacker,
                &demo.victim,
                sample.time,
                sample.frame_number,
            );
        }

        Ok(())
    }
}

impl DemoReducer {
    fn record_demo(
        &mut self,
        attacker: &PlayerId,
        victim: &PlayerId,
        time: f32,
        frame_number: usize,
    ) {
        if !self.should_count_demo(attacker, victim, frame_number) {
            return;
        }

        self.player_stats
            .entry(attacker.clone())
            .or_default()
            .demos_inflicted += 1;
        self.player_stats
            .entry(victim.clone())
            .or_default()
            .demos_taken += 1;

        match self.player_teams.get(attacker).copied() {
            Some(true) => self.team_zero_stats.demos_inflicted += 1,
            Some(false) => self.team_one_stats.demos_inflicted += 1,
            None => {}
        }

        self.timeline.push(TimelineEvent {
            time,
            kind: TimelineEventKind::Kill,
            player_id: Some(attacker.clone()),
            is_team_0: self.player_teams.get(attacker).copied(),
        });
        self.timeline.push(TimelineEvent {
            time,
            kind: TimelineEventKind::Death,
            player_id: Some(victim.clone()),
            is_team_0: self.player_teams.get(victim).copied(),
        });
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct MovementStats {
    pub tracked_time: f32,
    pub total_distance: f32,
    pub speed_integral: f32,
    pub time_slow_speed: f32,
    pub time_boost_speed: f32,
    pub time_supersonic_speed: f32,
    pub time_on_ground: f32,
    pub time_low_air: f32,
    pub time_high_air: f32,
}

impl MovementStats {
    pub fn average_speed(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.speed_integral / self.tracked_time
        }
    }

    pub fn average_speed_pct(&self) -> f32 {
        self.average_speed() * 100.0 / CAR_MAX_SPEED
    }

    pub fn slow_speed_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.time_slow_speed * 100.0 / self.tracked_time
        }
    }

    pub fn boost_speed_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.time_boost_speed * 100.0 / self.tracked_time
        }
    }

    pub fn supersonic_speed_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.time_supersonic_speed * 100.0 / self.tracked_time
        }
    }

    pub fn on_ground_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.time_on_ground * 100.0 / self.tracked_time
        }
    }

    pub fn low_air_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.time_low_air * 100.0 / self.tracked_time
        }
    }

    pub fn high_air_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.time_high_air * 100.0 / self.tracked_time
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MovementReducer {
    player_stats: HashMap<PlayerId, MovementStats>,
    player_teams: HashMap<PlayerId, bool>,
    previous_positions: HashMap<PlayerId, glam::Vec3>,
    team_zero_stats: MovementStats,
    team_one_stats: MovementStats,
}

impl MovementReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, MovementStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &MovementStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &MovementStats {
        &self.team_one_stats
    }
}

impl StatsReducer for MovementReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        let live_play = sample.is_live_play();
        if sample.dt == 0.0 {
            for player in &sample.players {
                if let Some(position) = player.position() {
                    self.previous_positions
                        .insert(player.player_id.clone(), position);
                }
            }
            return Ok(());
        }

        for player in &sample.players {
            self.player_teams
                .insert(player.player_id.clone(), player.is_team_0);
            let Some(position) = player.position() else {
                continue;
            };
            let speed = player.speed().unwrap_or(0.0);
            let stats = self
                .player_stats
                .entry(player.player_id.clone())
                .or_default();
            let team_stats = if player.is_team_0 {
                &mut self.team_zero_stats
            } else {
                &mut self.team_one_stats
            };

            if live_play {
                stats.tracked_time += sample.dt;
                stats.speed_integral += speed * sample.dt;
                team_stats.tracked_time += sample.dt;
                team_stats.speed_integral += speed * sample.dt;

                if let Some(previous_position) = self.previous_positions.get(&player.player_id) {
                    let distance = position.distance(*previous_position);
                    stats.total_distance += distance;
                    team_stats.total_distance += distance;
                }

                if speed >= SUPERSONIC_SPEED_THRESHOLD {
                    stats.time_supersonic_speed += sample.dt;
                    team_stats.time_supersonic_speed += sample.dt;
                } else if speed >= BOOST_SPEED_THRESHOLD {
                    stats.time_boost_speed += sample.dt;
                    team_stats.time_boost_speed += sample.dt;
                } else {
                    stats.time_slow_speed += sample.dt;
                    team_stats.time_slow_speed += sample.dt;
                }

                if position.z <= GROUND_Z_THRESHOLD {
                    stats.time_on_ground += sample.dt;
                    team_stats.time_on_ground += sample.dt;
                } else if position.z >= HIGH_AIR_Z_THRESHOLD {
                    stats.time_high_air += sample.dt;
                    team_stats.time_high_air += sample.dt;
                } else {
                    stats.time_low_air += sample.dt;
                    team_stats.time_low_air += sample.dt;
                }
            }

            self.previous_positions
                .insert(player.player_id.clone(), position);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct PositioningStats {
    pub tracked_time: f32,
    pub sum_distance_to_teammates: f32,
    pub sum_distance_to_ball: f32,
    pub sum_distance_to_ball_has_possession: f32,
    pub time_has_possession: f32,
    pub sum_distance_to_ball_no_possession: f32,
    pub time_no_possession: f32,
    pub time_most_back: f32,
    pub time_most_forward: f32,
    pub time_defensive_zone: f32,
    pub time_neutral_zone: f32,
    pub time_offensive_zone: f32,
    pub time_defensive_half: f32,
    pub time_offensive_half: f32,
    pub time_closest_to_ball: f32,
    pub time_farthest_from_ball: f32,
    pub time_behind_ball: f32,
    pub time_in_front_of_ball: f32,
}

impl PositioningStats {
    pub fn average_distance_to_teammates(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.sum_distance_to_teammates / self.tracked_time
        }
    }

    pub fn average_distance_to_ball(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.sum_distance_to_ball / self.tracked_time
        }
    }

    pub fn average_distance_to_ball_has_possession(&self) -> f32 {
        if self.time_has_possession == 0.0 {
            0.0
        } else {
            self.sum_distance_to_ball_has_possession / self.time_has_possession
        }
    }

    pub fn average_distance_to_ball_no_possession(&self) -> f32 {
        if self.time_no_possession == 0.0 {
            0.0
        } else {
            self.sum_distance_to_ball_no_possession / self.time_no_possession
        }
    }

    fn pct(&self, value: f32) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            value * 100.0 / self.tracked_time
        }
    }

    pub fn most_back_pct(&self) -> f32 {
        self.pct(self.time_most_back)
    }

    pub fn most_forward_pct(&self) -> f32 {
        self.pct(self.time_most_forward)
    }

    pub fn defensive_zone_pct(&self) -> f32 {
        self.pct(self.time_defensive_zone)
    }

    pub fn neutral_zone_pct(&self) -> f32 {
        self.pct(self.time_neutral_zone)
    }

    pub fn offensive_zone_pct(&self) -> f32 {
        self.pct(self.time_offensive_zone)
    }

    pub fn defensive_half_pct(&self) -> f32 {
        self.pct(self.time_defensive_half)
    }

    pub fn offensive_half_pct(&self) -> f32 {
        self.pct(self.time_offensive_half)
    }

    pub fn closest_to_ball_pct(&self) -> f32 {
        self.pct(self.time_closest_to_ball)
    }

    pub fn farthest_from_ball_pct(&self) -> f32 {
        self.pct(self.time_farthest_from_ball)
    }

    pub fn behind_ball_pct(&self) -> f32 {
        self.pct(self.time_behind_ball)
    }

    pub fn in_front_of_ball_pct(&self) -> f32 {
        self.pct(self.time_in_front_of_ball)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PositioningReducer {
    player_stats: HashMap<PlayerId, PositioningStats>,
    current_possession_team_is_team_0: Option<bool>,
    previous_ball_position: Option<glam::Vec3>,
    previous_player_positions: HashMap<PlayerId, glam::Vec3>,
}

impl PositioningReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, PositioningStats> {
        &self.player_stats
    }
}

impl StatsReducer for PositioningReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        if sample.dt == 0.0 {
            if let Some(ball) = &sample.ball {
                self.previous_ball_position = Some(ball.position());
            }
            for player in &sample.players {
                if let Some(position) = player.position() {
                    self.previous_player_positions
                        .insert(player.player_id.clone(), position);
                }
            }
            return Ok(());
        }

        let Some(ball) = &sample.ball else {
            return Ok(());
        };
        let ball_position = ball.position();
        let live_play = sample.is_live_play();
        let possession_team_before_sample = if sample.touch_events.is_empty() {
            self.current_possession_team_is_team_0
                .or(sample.possession_team_is_team_0)
        } else {
            self.current_possession_team_is_team_0
        };

        for player in &sample.players {
            let Some(position) = player.position() else {
                continue;
            };
            let previous_position = self
                .previous_player_positions
                .get(&player.player_id)
                .copied()
                .unwrap_or(position);
            let previous_ball_position = self.previous_ball_position.unwrap_or(ball_position);
            let normalized_position_y = normalized_y(player.is_team_0, position);
            let normalized_previous_position_y = normalized_y(player.is_team_0, previous_position);
            let normalized_ball_y = normalized_y(player.is_team_0, ball_position);
            let normalized_previous_ball_y = normalized_y(player.is_team_0, previous_ball_position);
            let stats = self
                .player_stats
                .entry(player.player_id.clone())
                .or_default();

            if live_play {
                stats.tracked_time += sample.dt;
                stats.sum_distance_to_ball += position.distance(ball_position) * sample.dt;

                if possession_team_before_sample == Some(player.is_team_0) {
                    stats.time_has_possession += sample.dt;
                    stats.sum_distance_to_ball_has_possession +=
                        position.distance(ball_position) * sample.dt;
                } else if possession_team_before_sample.is_some() {
                    stats.time_no_possession += sample.dt;
                    stats.sum_distance_to_ball_no_possession +=
                        position.distance(ball_position) * sample.dt;
                }

                let defensive_zone_fraction = interval_fraction_below_threshold(
                    normalized_previous_position_y,
                    normalized_position_y,
                    -FIELD_ZONE_BOUNDARY_Y,
                );
                let offensive_zone_fraction = interval_fraction_above_threshold(
                    normalized_previous_position_y,
                    normalized_position_y,
                    FIELD_ZONE_BOUNDARY_Y,
                );
                let neutral_zone_fraction = interval_fraction_in_scalar_range(
                    normalized_previous_position_y,
                    normalized_position_y,
                    -FIELD_ZONE_BOUNDARY_Y,
                    FIELD_ZONE_BOUNDARY_Y,
                );
                stats.time_defensive_zone += sample.dt * defensive_zone_fraction;
                stats.time_neutral_zone += sample.dt * neutral_zone_fraction;
                stats.time_offensive_zone += sample.dt * offensive_zone_fraction;

                let defensive_half_fraction = interval_fraction_below_threshold(
                    normalized_previous_position_y,
                    normalized_position_y,
                    0.0,
                );
                stats.time_defensive_half += sample.dt * defensive_half_fraction;
                stats.time_offensive_half += sample.dt * (1.0 - defensive_half_fraction);

                let previous_ball_delta =
                    normalized_previous_position_y - normalized_previous_ball_y;
                let current_ball_delta = normalized_position_y - normalized_ball_y;
                let behind_ball_fraction =
                    interval_fraction_below_threshold(previous_ball_delta, current_ball_delta, 0.0);
                stats.time_behind_ball += sample.dt * behind_ball_fraction;
                stats.time_in_front_of_ball += sample.dt * (1.0 - behind_ball_fraction);
            }
        }

        if live_play {
            for is_team_0 in [true, false] {
                let team_players: Vec<_> = sample
                    .players
                    .iter()
                    .filter(|player| player.is_team_0 == is_team_0)
                    .filter_map(|player| player.position().map(|position| (player, position)))
                    .collect();

                if team_players.is_empty() {
                    continue;
                }

                for (player, position) in &team_players {
                    let teammate_distance_sum: f32 = team_players
                        .iter()
                        .filter(|(other_player, _)| other_player.player_id != player.player_id)
                        .map(|(_, other_position)| position.distance(*other_position))
                        .sum();
                    let teammate_count = team_players.len().saturating_sub(1);
                    if teammate_count > 0 {
                        let stats = self
                            .player_stats
                            .entry(player.player_id.clone())
                            .or_default();
                        stats.sum_distance_to_teammates +=
                            teammate_distance_sum * sample.dt / teammate_count as f32;
                    }
                }

                if let Some((most_back_player, _)) = team_players.iter().min_by(|(_, a), (_, b)| {
                    normalized_y(is_team_0, *a)
                        .partial_cmp(&normalized_y(is_team_0, *b))
                        .unwrap()
                }) {
                    self.player_stats
                        .entry(most_back_player.player_id.clone())
                        .or_default()
                        .time_most_back += sample.dt;
                }

                if let Some((most_forward_player, _)) =
                    team_players.iter().max_by(|(_, a), (_, b)| {
                        normalized_y(is_team_0, *a)
                            .partial_cmp(&normalized_y(is_team_0, *b))
                            .unwrap()
                    })
                {
                    self.player_stats
                        .entry(most_forward_player.player_id.clone())
                        .or_default()
                        .time_most_forward += sample.dt;
                }

                if let Some((closest_player, _)) = team_players.iter().min_by(|(_, a), (_, b)| {
                    a.distance(ball_position)
                        .partial_cmp(&b.distance(ball_position))
                        .unwrap()
                }) {
                    self.player_stats
                        .entry(closest_player.player_id.clone())
                        .or_default()
                        .time_closest_to_ball += sample.dt;
                }

                if let Some((farthest_player, _)) = team_players.iter().max_by(|(_, a), (_, b)| {
                    a.distance(ball_position)
                        .partial_cmp(&b.distance(ball_position))
                        .unwrap()
                }) {
                    self.player_stats
                        .entry(farthest_player.player_id.clone())
                        .or_default()
                        .time_farthest_from_ball += sample.dt;
                }
            }
        }

        if let Some(last_touch) = sample.touch_events.last() {
            self.current_possession_team_is_team_0 = Some(last_touch.team_is_team_0);
        } else {
            self.current_possession_team_is_team_0 = sample
                .possession_team_is_team_0
                .or(self.current_possession_team_is_team_0);
        }

        self.previous_ball_position = Some(ball_position);
        for player in &sample.players {
            if let Some(position) = player.position() {
                self.previous_player_positions
                    .insert(player.player_id.clone(), position);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct BallCarryStats {
    pub carry_count: u32,
    pub total_carry_time: f32,
    pub total_straight_line_distance: f32,
    pub total_path_distance: f32,
    pub longest_carry_time: f32,
    pub furthest_carry_distance: f32,
    pub fastest_carry_speed: f32,
    pub carry_speed_sum: f32,
    pub average_horizontal_gap_sum: f32,
    pub average_vertical_gap_sum: f32,
}

impl BallCarryStats {
    fn pct_count_average(&self, value: f32) -> f32 {
        if self.carry_count == 0 {
            0.0
        } else {
            value / self.carry_count as f32
        }
    }

    pub fn average_carry_time(&self) -> f32 {
        self.pct_count_average(self.total_carry_time)
    }

    pub fn average_straight_line_distance(&self) -> f32 {
        self.pct_count_average(self.total_straight_line_distance)
    }

    pub fn average_path_distance(&self) -> f32 {
        self.pct_count_average(self.total_path_distance)
    }

    pub fn average_carry_speed(&self) -> f32 {
        self.pct_count_average(self.carry_speed_sum)
    }

    pub fn average_horizontal_gap(&self) -> f32 {
        self.pct_count_average(self.average_horizontal_gap_sum)
    }

    pub fn average_vertical_gap(&self) -> f32 {
        self.pct_count_average(self.average_vertical_gap_sum)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BallCarryEvent {
    pub player_id: PlayerId,
    pub is_team_0: bool,
    pub start_frame: usize,
    pub end_frame: usize,
    pub start_time: f32,
    pub end_time: f32,
    pub duration: f32,
    pub straight_line_distance: f32,
    pub path_distance: f32,
    pub average_horizontal_gap: f32,
    pub average_vertical_gap: f32,
    pub average_speed: f32,
}

#[derive(Debug, Clone)]
struct ActiveBallCarry {
    player_id: PlayerId,
    is_team_0: bool,
    start_frame: usize,
    last_frame: usize,
    start_time: f32,
    last_time: f32,
    start_position: glam::Vec3,
    last_position: glam::Vec3,
    duration: f32,
    path_distance: f32,
    horizontal_gap_integral: f32,
    vertical_gap_integral: f32,
    speed_integral: f32,
}

#[derive(Debug, Clone, Copy)]
struct BallCarryFrameSample {
    player_position: glam::Vec3,
    horizontal_gap: f32,
    vertical_gap: f32,
    speed: f32,
}

#[derive(Debug, Clone, Default)]
pub struct BallCarryReducer {
    player_stats: HashMap<PlayerId, BallCarryStats>,
    team_zero_stats: BallCarryStats,
    team_one_stats: BallCarryStats,
    carry_events: Vec<BallCarryEvent>,
    active_carry: Option<ActiveBallCarry>,
    last_touch_player: Option<PlayerId>,
}

impl BallCarryReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, BallCarryStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &BallCarryStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &BallCarryStats {
        &self.team_one_stats
    }

    pub fn carry_events(&self) -> &[BallCarryEvent] {
        &self.carry_events
    }

    fn carry_frame_sample(
        player: &PlayerSample,
        ball: &BallSample,
    ) -> Option<BallCarryFrameSample> {
        let player_position = player.position()?;
        let ball_position = ball.position();
        if !(BALL_CARRY_MIN_BALL_Z..=BALL_CARRY_MAX_BALL_Z).contains(&ball_position.z) {
            return None;
        }

        let horizontal_gap = player_position
            .truncate()
            .distance(ball_position.truncate());
        if horizontal_gap > BALL_CARRY_MAX_HORIZONTAL_GAP {
            return None;
        }

        let vertical_gap = ball_position.z - player_position.z;
        if !(0.0..=BALL_CARRY_MAX_VERTICAL_GAP).contains(&vertical_gap) {
            return None;
        }

        Some(BallCarryFrameSample {
            player_position,
            horizontal_gap,
            vertical_gap,
            speed: player.speed().unwrap_or(0.0),
        })
    }

    fn begin_carry(
        &self,
        sample: &StatsSample,
        player: &PlayerSample,
        frame_sample: BallCarryFrameSample,
    ) -> ActiveBallCarry {
        let start_time = (sample.time - sample.dt).max(0.0);
        let start_frame = sample.frame_number.saturating_sub(1);
        ActiveBallCarry {
            player_id: player.player_id.clone(),
            is_team_0: player.is_team_0,
            start_frame,
            last_frame: sample.frame_number,
            start_time,
            last_time: sample.time,
            start_position: frame_sample.player_position,
            last_position: frame_sample.player_position,
            duration: sample.dt,
            path_distance: 0.0,
            horizontal_gap_integral: frame_sample.horizontal_gap * sample.dt,
            vertical_gap_integral: frame_sample.vertical_gap * sample.dt,
            speed_integral: frame_sample.speed * sample.dt,
        }
    }

    fn extend_carry(
        active_carry: &mut ActiveBallCarry,
        sample: &StatsSample,
        frame_sample: BallCarryFrameSample,
    ) {
        active_carry.duration += sample.dt;
        active_carry.path_distance += frame_sample
            .player_position
            .distance(active_carry.last_position);
        active_carry.last_position = frame_sample.player_position;
        active_carry.last_time = sample.time;
        active_carry.last_frame = sample.frame_number;
        active_carry.horizontal_gap_integral += frame_sample.horizontal_gap * sample.dt;
        active_carry.vertical_gap_integral += frame_sample.vertical_gap * sample.dt;
        active_carry.speed_integral += frame_sample.speed * sample.dt;
    }

    fn finalize_active_carry(&mut self) {
        let Some(active_carry) = self.active_carry.take() else {
            return;
        };
        if active_carry.duration < BALL_CARRY_MIN_DURATION {
            return;
        }

        let event = BallCarryEvent {
            player_id: active_carry.player_id.clone(),
            is_team_0: active_carry.is_team_0,
            start_frame: active_carry.start_frame,
            end_frame: active_carry.last_frame,
            start_time: active_carry.start_time,
            end_time: active_carry.last_time,
            duration: active_carry.duration,
            straight_line_distance: active_carry
                .start_position
                .truncate()
                .distance(active_carry.last_position.truncate()),
            path_distance: active_carry.path_distance,
            average_horizontal_gap: active_carry.horizontal_gap_integral / active_carry.duration,
            average_vertical_gap: active_carry.vertical_gap_integral / active_carry.duration,
            average_speed: active_carry.speed_integral / active_carry.duration,
        };
        self.record_carry_event(event);
    }

    fn record_carry_event(&mut self, event: BallCarryEvent) {
        let player_stats = self
            .player_stats
            .entry(event.player_id.clone())
            .or_default();
        Self::apply_carry_event(player_stats, &event);

        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        Self::apply_carry_event(team_stats, &event);
        self.carry_events.push(event);
    }

    fn apply_carry_event(stats: &mut BallCarryStats, event: &BallCarryEvent) {
        stats.carry_count += 1;
        stats.total_carry_time += event.duration;
        stats.total_straight_line_distance += event.straight_line_distance;
        stats.total_path_distance += event.path_distance;
        stats.longest_carry_time = stats.longest_carry_time.max(event.duration);
        stats.furthest_carry_distance = stats
            .furthest_carry_distance
            .max(event.straight_line_distance);
        stats.fastest_carry_speed = stats.fastest_carry_speed.max(event.average_speed);
        stats.carry_speed_sum += event.average_speed;
        stats.average_horizontal_gap_sum += event.average_horizontal_gap;
        stats.average_vertical_gap_sum += event.average_vertical_gap;
    }
}

impl StatsReducer for BallCarryReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        let controlling_player = self.last_touch_player.clone();
        let live_play = sample.is_live_play();
        let carry_candidate = if live_play && sample.dt > 0.0 {
            if let (Some(ball), Some(player_id)) = (&sample.ball, controlling_player.as_ref()) {
                sample
                    .players
                    .iter()
                    .find(|player| &player.player_id == player_id)
                    .and_then(|player| {
                        Self::carry_frame_sample(player, ball)
                            .map(|frame_sample| (player, frame_sample))
                    })
            } else {
                None
            }
        } else {
            None
        };

        match (self.active_carry.as_mut(), carry_candidate) {
            (Some(active_carry), Some((player, frame_sample)))
                if active_carry.player_id == player.player_id =>
            {
                Self::extend_carry(active_carry, sample, frame_sample);
            }
            (Some(_), Some((player, frame_sample))) => {
                self.finalize_active_carry();
                self.active_carry = Some(self.begin_carry(sample, player, frame_sample));
            }
            (Some(_), None) => {
                self.finalize_active_carry();
            }
            (None, Some((player, frame_sample))) => {
                self.active_carry = Some(self.begin_carry(sample, player, frame_sample));
            }
            (None, None) => {}
        }

        if let Some(last_touch) = sample.touch_events.last() {
            self.last_touch_player = last_touch.player.clone();
            if let Some(active_carry) = &self.active_carry {
                if self.last_touch_player.as_ref() != Some(&active_carry.player_id) {
                    self.finalize_active_carry();
                }
            }
        }

        Ok(())
    }

    fn finish(&mut self) -> SubtrActorResult<()> {
        self.finalize_active_carry();
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct BoostStats {
    pub tracked_time: f32,
    pub boost_integral: f32,
    pub time_zero_boost: f32,
    pub time_hundred_boost: f32,
    pub time_boost_0_25: f32,
    pub time_boost_25_50: f32,
    pub time_boost_50_75: f32,
    pub time_boost_75_100: f32,
    pub amount_collected: f32,
    pub amount_stolen: f32,
    pub big_pads_collected: u32,
    pub small_pads_collected: u32,
    pub big_pads_stolen: u32,
    pub small_pads_stolen: u32,
    pub amount_collected_big: f32,
    pub amount_stolen_big: f32,
    pub amount_collected_small: f32,
    pub amount_stolen_small: f32,
    pub overfill_total: f32,
    pub overfill_from_stolen: f32,
    pub amount_used_while_supersonic: f32,
}

impl BoostStats {
    pub fn average_boost_amount(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.boost_integral / self.tracked_time
        }
    }

    pub fn bpm(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.amount_collected * 60.0 / self.tracked_time
        }
    }

    fn pct(&self, value: f32) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            value * 100.0 / self.tracked_time
        }
    }

    pub fn zero_boost_pct(&self) -> f32 {
        self.pct(self.time_zero_boost)
    }

    pub fn hundred_boost_pct(&self) -> f32 {
        self.pct(self.time_hundred_boost)
    }

    pub fn boost_0_25_pct(&self) -> f32 {
        self.pct(self.time_boost_0_25)
    }

    pub fn boost_25_50_pct(&self) -> f32 {
        self.pct(self.time_boost_25_50)
    }

    pub fn boost_50_75_pct(&self) -> f32 {
        self.pct(self.time_boost_50_75)
    }

    pub fn boost_75_100_pct(&self) -> f32 {
        self.pct(self.time_boost_75_100)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BoostReducerConfig {
    pub include_non_live_pickups: bool,
}

#[derive(Debug, Clone, Default)]
pub struct BoostReducer {
    config: BoostReducerConfig,
    player_stats: HashMap<PlayerId, BoostStats>,
    team_zero_stats: BoostStats,
    team_one_stats: BoostStats,
    previous_boost_amounts: HashMap<PlayerId, f32>,
    previous_player_speeds: HashMap<PlayerId, f32>,
    observed_pad_positions: HashMap<String, PadPositionEstimate>,
    known_pad_sizes: HashMap<String, BoostPadSize>,
    known_pad_indices: HashMap<String, usize>,
    pending_pickups: HashMap<String, PendingBoostPickup>,
    unavailable_pads: HashSet<String>,
    seen_pickup_sequences: HashSet<(String, u8)>,
    pickup_frames: HashMap<(String, PlayerId), usize>,
    last_pickup_times: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
struct PendingBoostPickup {
    player_id: PlayerId,
    is_team_0: bool,
    previous_boost_amount: f32,
    time: f32,
    player_position: glam::Vec3,
}

impl BoostReducer {
    pub fn new() -> Self {
        Self::with_config(BoostReducerConfig::default())
    }

    pub fn with_config(config: BoostReducerConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, BoostStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &BoostStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &BoostStats {
        &self.team_one_stats
    }

    fn estimated_pad_position(&self, pad_id: &str) -> Option<glam::Vec3> {
        self.observed_pad_positions
            .get(pad_id)
            .and_then(PadPositionEstimate::mean)
    }

    fn infer_pad_index(
        &self,
        pad_id: &str,
        pad_size: BoostPadSize,
        observed_position: glam::Vec3,
    ) -> Option<usize> {
        if let Some(index) = self.known_pad_indices.get(pad_id).copied() {
            return Some(index);
        }

        let observed_position = self
            .estimated_pad_position(pad_id)
            .unwrap_or(observed_position);
        let layout = &*STANDARD_SOCCAR_BOOST_PAD_LAYOUT;
        let used_indices: HashSet<usize> = self.known_pad_indices.values().copied().collect();
        let best_unused = layout
            .iter()
            .enumerate()
            .filter(|(index, (_, size))| *size == pad_size && !used_indices.contains(index))
            .min_by(|(_, (a, _)), (_, (b, _))| {
                observed_position
                    .distance_squared(*a)
                    .partial_cmp(&observed_position.distance_squared(*b))
                    .unwrap()
            })
            .map(|(index, _)| index);

        best_unused
            .or_else(|| {
                layout
                    .iter()
                    .enumerate()
                    .filter(|(_, (_, size))| *size == pad_size)
                    .min_by(|(_, (a, _)), (_, (b, _))| {
                        observed_position
                            .distance_squared(*a)
                            .partial_cmp(&observed_position.distance_squared(*b))
                            .unwrap()
                    })
                    .map(|(index, _)| index)
            })
            .filter(|index| {
                observed_position.distance(standard_soccar_boost_pad_position(*index))
                    <= STANDARD_PAD_MATCH_RADIUS
            })
    }

    fn resolve_pickup(
        &mut self,
        pad_id: &str,
        pending_pickup: PendingBoostPickup,
        pad_size: BoostPadSize,
    ) {
        let observed_position = self
            .estimated_pad_position(pad_id)
            .unwrap_or(pending_pickup.player_position);
        let pad_position = self
            .infer_pad_index(pad_id, pad_size, observed_position)
            .map(|index| {
                self.known_pad_indices.insert(pad_id.to_string(), index);
                standard_soccar_boost_pad_position(index)
            })
            .unwrap_or(observed_position);
        let stolen = is_enemy_side(pending_pickup.is_team_0, pad_position);
        let stats = self
            .player_stats
            .entry(pending_pickup.player_id.clone())
            .or_default();
        let team_stats = if pending_pickup.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        let nominal_gain = match pad_size {
            BoostPadSize::Big => BOOST_MAX_AMOUNT,
            BoostPadSize::Small => SMALL_PAD_AMOUNT_RAW,
        };
        let collected_amount =
            (BOOST_MAX_AMOUNT - pending_pickup.previous_boost_amount).min(nominal_gain);
        let overfill = (nominal_gain - collected_amount).max(0.0);

        stats.amount_collected += collected_amount;
        team_stats.amount_collected += collected_amount;

        if stolen {
            stats.amount_stolen += collected_amount;
            team_stats.amount_stolen += collected_amount;
        }

        match pad_size {
            BoostPadSize::Big => {
                stats.big_pads_collected += 1;
                team_stats.big_pads_collected += 1;
                stats.amount_collected_big += collected_amount;
                team_stats.amount_collected_big += collected_amount;
                if stolen {
                    stats.big_pads_stolen += 1;
                    team_stats.big_pads_stolen += 1;
                    stats.amount_stolen_big += collected_amount;
                    team_stats.amount_stolen_big += collected_amount;
                }
            }
            BoostPadSize::Small => {
                stats.small_pads_collected += 1;
                team_stats.small_pads_collected += 1;
                stats.amount_collected_small += collected_amount;
                team_stats.amount_collected_small += collected_amount;
                if stolen {
                    stats.small_pads_stolen += 1;
                    team_stats.small_pads_stolen += 1;
                    stats.amount_stolen_small += collected_amount;
                    team_stats.amount_stolen_small += collected_amount;
                }
            }
        }

        stats.overfill_total += overfill;
        team_stats.overfill_total += overfill;
        if stolen {
            stats.overfill_from_stolen += overfill;
            team_stats.overfill_from_stolen += overfill;
        }
    }

    fn interval_fraction_in_boost_range(
        start_boost: f32,
        end_boost: f32,
        min_boost: f32,
        max_boost: f32,
    ) -> f32 {
        if (end_boost - start_boost).abs() <= f32::EPSILON {
            return ((start_boost >= min_boost) && (start_boost < max_boost)) as i32 as f32;
        }

        let t_at_min = (min_boost - start_boost) / (end_boost - start_boost);
        let t_at_max = (max_boost - start_boost) / (end_boost - start_boost);
        let interval_start = t_at_min.min(t_at_max).max(0.0);
        let interval_end = t_at_min.max(t_at_max).min(1.0);
        (interval_end - interval_start).max(0.0)
    }

    fn pad_respawn_time_seconds(pad_size: BoostPadSize) -> f32 {
        match pad_size {
            BoostPadSize::Big => 10.0,
            BoostPadSize::Small => 4.0,
        }
    }
}

impl StatsReducer for BoostReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        let live_play = sample.is_live_play();
        let mut current_boost_amounts = Vec::new();

        for player in &sample.players {
            let Some(boost_amount) = player.boost_amount else {
                continue;
            };
            let previous_boost_amount = player.last_boost_amount.unwrap_or_else(|| {
                self.previous_boost_amounts
                    .get(&player.player_id)
                    .copied()
                    .unwrap_or(boost_amount)
            });
            let speed = player.speed();
            let previous_speed = self
                .previous_player_speeds
                .get(&player.player_id)
                .copied()
                .or(speed);

            let stats = self
                .player_stats
                .entry(player.player_id.clone())
                .or_default();
            let team_stats = if player.is_team_0 {
                &mut self.team_zero_stats
            } else {
                &mut self.team_one_stats
            };

            if live_play {
                let average_boost_amount = (previous_boost_amount + boost_amount) * 0.5;
                stats.tracked_time += sample.dt;
                stats.boost_integral += average_boost_amount * sample.dt;
                team_stats.tracked_time += sample.dt;
                team_stats.boost_integral += average_boost_amount * sample.dt;

                let time_zero_boost = sample.dt
                    * Self::interval_fraction_in_boost_range(
                        previous_boost_amount,
                        boost_amount,
                        0.0,
                        BOOST_ZERO_BAND_RAW,
                    );
                let time_hundred_boost = sample.dt
                    * Self::interval_fraction_in_boost_range(
                        previous_boost_amount,
                        boost_amount,
                        BOOST_FULL_BAND_MIN_RAW,
                        BOOST_MAX_AMOUNT + 1.0,
                    );
                stats.time_zero_boost += time_zero_boost;
                team_stats.time_zero_boost += time_zero_boost;
                stats.time_hundred_boost += time_hundred_boost;
                team_stats.time_hundred_boost += time_hundred_boost;

                let time_boost_0_25 = sample.dt
                    * Self::interval_fraction_in_boost_range(
                        previous_boost_amount,
                        boost_amount,
                        0.0,
                        boost_percent_to_amount(25.0),
                    );
                let time_boost_25_50 = sample.dt
                    * Self::interval_fraction_in_boost_range(
                        previous_boost_amount,
                        boost_amount,
                        boost_percent_to_amount(25.0),
                        boost_percent_to_amount(50.0),
                    );
                let time_boost_50_75 = sample.dt
                    * Self::interval_fraction_in_boost_range(
                        previous_boost_amount,
                        boost_amount,
                        boost_percent_to_amount(50.0),
                        boost_percent_to_amount(75.0),
                    );
                let time_boost_75_100 = sample.dt
                    * Self::interval_fraction_in_boost_range(
                        previous_boost_amount,
                        boost_amount,
                        boost_percent_to_amount(75.0),
                        BOOST_MAX_AMOUNT + 1.0,
                    );
                stats.time_boost_0_25 += time_boost_0_25;
                team_stats.time_boost_0_25 += time_boost_0_25;
                stats.time_boost_25_50 += time_boost_25_50;
                team_stats.time_boost_25_50 += time_boost_25_50;
                stats.time_boost_50_75 += time_boost_50_75;
                team_stats.time_boost_50_75 += time_boost_50_75;
                stats.time_boost_75_100 += time_boost_75_100;
                team_stats.time_boost_75_100 += time_boost_75_100;

                if player.boost_active
                    && speed.unwrap_or(0.0) >= SUPERSONIC_SPEED_THRESHOLD
                    && previous_speed.unwrap_or(0.0) >= SUPERSONIC_SPEED_THRESHOLD
                {
                    let supersonic_usage = (previous_boost_amount - boost_amount)
                        .max(0.0)
                        .min(BOOST_USED_RAW_UNITS_PER_SECOND * sample.dt);
                    stats.amount_used_while_supersonic += supersonic_usage;
                    team_stats.amount_used_while_supersonic += supersonic_usage;
                }
            }
            current_boost_amounts.push((player.player_id.clone(), boost_amount));
        }

        for event in &sample.boost_pad_events {
            match event.kind {
                BoostPadEventKind::PickedUp { sequence } => {
                    if !live_play && !self.config.include_non_live_pickups {
                        continue;
                    }
                    if self.unavailable_pads.contains(&event.pad_id) {
                        continue;
                    }
                    let Some(player_id) = &event.player else {
                        continue;
                    };
                    let pickup_key = (event.pad_id.clone(), player_id.clone());
                    if self.pickup_frames.get(&pickup_key).copied() == Some(event.frame) {
                        continue;
                    }
                    self.pickup_frames.insert(pickup_key, event.frame);
                    if !self
                        .seen_pickup_sequences
                        .insert((event.pad_id.clone(), sequence))
                    {
                        continue;
                    }
                    self.unavailable_pads.insert(event.pad_id.clone());
                    self.last_pickup_times
                        .insert(event.pad_id.clone(), event.time);
                    let Some(player) = sample
                        .players
                        .iter()
                        .find(|player| &player.player_id == player_id)
                    else {
                        continue;
                    };
                    if let Some(position) = player.position() {
                        self.observed_pad_positions
                            .entry(event.pad_id.clone())
                            .or_default()
                            .observe(position);
                    }
                    let previous_boost_amount = player.last_boost_amount.unwrap_or_else(|| {
                        self.previous_boost_amounts
                            .get(player_id)
                            .copied()
                            .unwrap_or_else(|| player.boost_amount.unwrap_or(0.0))
                    });
                    let pending_pickup = PendingBoostPickup {
                        player_id: player_id.clone(),
                        is_team_0: player.is_team_0,
                        previous_boost_amount,
                        time: sample.time,
                        player_position: player.position().unwrap_or(glam::Vec3::ZERO),
                    };

                    if let Some(pad_size) = self.known_pad_sizes.get(&event.pad_id).copied() {
                        self.resolve_pickup(&event.pad_id, pending_pickup, pad_size);
                    } else {
                        self.pending_pickups
                            .insert(event.pad_id.clone(), pending_pickup);
                    }
                }
                BoostPadEventKind::Available => {
                    if let Some(pad_size) = self.known_pad_sizes.get(&event.pad_id).copied() {
                        let Some(last_pickup_time) = self.last_pickup_times.get(&event.pad_id)
                        else {
                            continue;
                        };
                        if event.time - *last_pickup_time < Self::pad_respawn_time_seconds(pad_size)
                        {
                            continue;
                        }
                    }
                    self.unavailable_pads.remove(&event.pad_id);
                    let Some(pending_pickup) = self.pending_pickups.remove(&event.pad_id) else {
                        continue;
                    };
                    let pad_size = if event.time - pending_pickup.time >= 7.0 {
                        BoostPadSize::Big
                    } else {
                        BoostPadSize::Small
                    };
                    self.known_pad_sizes.insert(event.pad_id.clone(), pad_size);
                    self.resolve_pickup(&event.pad_id, pending_pickup, pad_size);
                }
            }
        }

        for (player_id, boost_amount) in current_boost_amounts {
            self.previous_boost_amounts.insert(player_id, boost_amount);
        }
        for player in &sample.players {
            if let Some(speed) = player.speed() {
                self.previous_player_speeds
                    .insert(player.player_id.clone(), speed);
            }
        }

        Ok(())
    }
}
