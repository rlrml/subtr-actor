use crate::*;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct StatsTimelineConfig {
    pub most_back_forward_threshold_y: f32,
    pub level_ball_depth_margin: f32,
    pub pressure_neutral_zone_half_width_y: f32,
    pub territorial_pressure_neutral_zone_half_width_y: f32,
    pub territorial_pressure_min_establish_seconds: f32,
    pub territorial_pressure_min_establish_third_seconds: f32,
    pub territorial_pressure_relief_grace_seconds: f32,
    pub territorial_pressure_confirmed_relief_grace_seconds: f32,
    pub rotation_role_depth_margin: f32,
    pub rotation_first_man_ambiguity_margin: f32,
    pub rotation_first_man_debounce_seconds: f32,
    pub rush_max_start_y: f32,
    pub rush_attack_support_distance_y: f32,
    pub rush_defender_distance_y: f32,
    pub rush_min_possession_retained_seconds: f32,
    pub aerial_goal_min_ball_z: f32,
    pub high_aerial_goal_min_ball_z: f32,
    pub long_distance_goal_max_attacking_y: f32,
    pub own_half_goal_max_attacking_y: f32,
    pub empty_net_min_defender_y_margin: f32,
    pub empty_net_min_defender_distance: f32,
    pub empty_net_max_touch_attacking_y: f32,
    pub flick_goal_max_event_to_goal_seconds: f32,
    pub double_tap_goal_max_event_to_goal_seconds: f32,
    pub one_timer_goal_max_event_to_goal_seconds: f32,
    pub air_dribble_goal_max_end_to_goal_seconds: f32,
    pub flip_reset_goal_max_event_to_goal_seconds: f32,
    pub half_volley_max_bounce_to_touch_seconds: f32,
    pub half_volley_min_ball_speed: f32,
    pub half_volley_goal_max_touch_to_goal_seconds: f32,
    pub half_volley_goal_min_goal_alignment: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayStatsTimeline {
    pub config: StatsTimelineConfig,
    pub replay_meta: ReplayMeta,
    pub events: ReplayStatsTimelineEvents,
    pub frames: Vec<ReplayStatsFrame>,
}

impl ReplayStatsTimeline {
    pub fn frame_by_number(&self, frame_number: usize) -> Option<&ReplayStatsFrame> {
        self.frames
            .iter()
            .find(|frame| frame.frame_number == frame_number)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayStatsTimelineScaffold {
    pub config: StatsTimelineConfig,
    pub replay_meta: ReplayMeta,
    pub events: ReplayStatsTimelineEvents,
    pub frames: Vec<ReplayStatsFrameScaffold>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayStatsFrameScaffold {
    pub frame_number: usize,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
    pub game_state: Option<i32>,
    pub ball_has_been_hit: Option<bool>,
    pub kickoff_countdown_time: Option<i32>,
    pub gameplay_phase: GameplayPhase,
    pub is_live_play: bool,
    #[ts(type = "Record<string, unknown>")]
    pub team_zero: BTreeMap<String, serde_json::Value>,
    #[ts(type = "Record<string, unknown>")]
    pub team_one: BTreeMap<String, serde_json::Value>,
    pub players: Vec<ReplayStatsPlayerIdentity>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayStatsPlayerIdentity {
    #[serde(rename = "player_id")]
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    pub name: String,
    pub is_team_0: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayStatsTimelineEvents {
    pub timeline: Vec<TimelineEvent>,
    pub core_player: Vec<CorePlayerStatsEvent>,
    pub core_team: Vec<CoreTeamStatsEvent>,
    pub possession: Vec<PossessionEvent>,
    pub pressure: Vec<PressureEvent>,
    pub territorial_pressure: Vec<TerritorialPressureEvent>,
    pub movement: Vec<MovementEvent>,
    pub positioning: Vec<PositioningEvent>,
    pub rotation_player: Vec<RotationPlayerEvent>,
    pub rotation_team: Vec<RotationTeamEvent>,
    pub mechanics: Vec<StatsTimelineTagEvent>,
    pub goal_context: Vec<GoalContextEvent>,
    pub backboard: Vec<BackboardBounceEvent>,
    pub ceiling_shot: Vec<CeilingShotEvent>,
    pub wall_aerial: Vec<WallAerialEvent>,
    pub wall_aerial_shot: Vec<WallAerialShotEvent>,
    pub center: Vec<CenterEvent>,
    pub flick: Vec<FlickEvent>,
    pub musty_flick: Vec<MustyFlickEvent>,
    pub dodge_reset: Vec<DodgeResetEvent>,
    pub double_tap: Vec<DoubleTapEvent>,
    pub fifty_fifty: Vec<FiftyFiftyEvent>,
    pub one_timer: Vec<OneTimerEvent>,
    pub pass: Vec<PassEvent>,
    pub pass_last_completed: Vec<PassLastCompletedEvent>,
    pub ball_carry: Vec<BallCarryEvent>,
    pub goal_tags: Vec<GoalTagEvent>,
    pub rush: Vec<RushEvent>,
    pub speed_flip: Vec<SpeedFlipEvent>,
    pub half_flip: Vec<HalfFlipEvent>,
    pub half_volley: Vec<HalfVolleyEvent>,
    pub wavedash: Vec<WavedashEvent>,
    pub whiff: Vec<WhiffEvent>,
    pub powerslide: Vec<PowerslideEvent>,
    pub touch: Vec<TouchStatsEvent>,
    pub touch_ball_movement: Vec<TouchBallMovementEvent>,
    pub touch_last_touch: Vec<TouchLastTouchEvent>,
    pub boost_pickups: Vec<BoostPickupComparisonEvent>,
    pub boost_ledger: Vec<BoostLedgerEvent>,
    pub boost_state: Vec<BoostStateEvent>,
    pub bump: Vec<BumpEvent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum StatsEventTiming {
    Moment {
        frame: usize,
        time: f32,
    },
    Span {
        start_frame: usize,
        end_frame: usize,
        start_time: f32,
        end_time: f32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
#[ts(export)]
pub enum StatsEventPropertyValue {
    Text(String),
    Unsigned(u32),
    Float(f32),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct StatsEventProperty {
    pub key: String,
    pub value: StatsEventPropertyValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct StatsTimelineTagEvent {
    pub id: String,
    pub kind: String,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    pub is_team_0: bool,
    pub timing: StatsEventTiming,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<StatsEventProperty>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayStatsFrame {
    pub frame_number: usize,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
    pub game_state: Option<i32>,
    pub ball_has_been_hit: Option<bool>,
    pub kickoff_countdown_time: Option<i32>,
    pub gameplay_phase: GameplayPhase,
    pub is_live_play: bool,
    pub team_zero: TeamStatsSnapshot,
    pub team_one: TeamStatsSnapshot,
    pub players: Vec<PlayerStatsSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct TeamStatsSnapshot {
    pub fifty_fifty: FiftyFiftyTeamStats,
    pub possession: PossessionTeamStats,
    pub pressure: PressureTeamStats,
    pub territorial_pressure: TerritorialPressureTeamStats,
    pub rotation: RotationTeamStats,
    pub rush: RushTeamStats,
    pub core: CoreTeamStats,
    pub backboard: BackboardTeamStats,
    pub double_tap: DoubleTapTeamStats,
    pub one_timer: OneTimerTeamStats,
    pub pass: PassTeamStats,
    pub ball_carry: BallCarryStats,
    pub air_dribble: AirDribbleStats,
    pub boost: BoostStats,
    pub bump: BumpTeamStats,
    pub half_volley: HalfVolleyTeamStats,
    pub movement: MovementStats,
    pub powerslide: PowerslideStats,
    pub demo: DemoTeamStats,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PlayerStatsSnapshot {
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    pub name: String,
    pub is_team_0: bool,
    pub core: CorePlayerStats,
    pub backboard: BackboardPlayerStats,
    pub ceiling_shot: CeilingShotStats,
    pub wall_aerial: WallAerialStats,
    pub wall_aerial_shot: WallAerialShotStats,
    pub double_tap: DoubleTapPlayerStats,
    pub one_timer: OneTimerPlayerStats,
    pub pass: PassPlayerStats,
    pub fifty_fifty: FiftyFiftyPlayerStats,
    pub speed_flip: SpeedFlipStats,
    pub half_flip: HalfFlipStats,
    pub half_volley: HalfVolleyPlayerStats,
    pub wavedash: WavedashStats,
    pub touch: TouchStats,
    pub whiff: WhiffStats,
    pub flick: FlickStats,
    pub musty_flick: MustyFlickStats,
    pub dodge_reset: DodgeResetStats,
    pub ball_carry: BallCarryStats,
    pub air_dribble: AirDribbleStats,
    pub boost: BoostStats,
    pub bump: BumpPlayerStats,
    pub movement: MovementStats,
    pub positioning: PositioningStats,
    pub rotation: RotationPlayerStats,
    pub powerslide: PowerslideStats,
    pub demo: DemoPlayerStats,
}
