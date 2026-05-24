use crate::*;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct StatsTimelineConfig {
    pub most_back_forward_threshold_y: f32,
    pub level_ball_depth_margin: f32,
    pub pressure_neutral_zone_half_width_y: f32,
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
    pub one_timer_goal_max_event_to_goal_seconds: f32,
    pub air_dribble_goal_max_end_to_goal_seconds: f32,
    pub flip_reset_goal_max_event_to_goal_seconds: f32,
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayStatsTimelineEvents {
    pub timeline: Vec<TimelineEvent>,
    pub mechanics: Vec<MechanicEvent>,
    pub goal_context: Vec<GoalContextEvent>,
    pub backboard: Vec<BackboardBounceEvent>,
    pub ceiling_shot: Vec<CeilingShotEvent>,
    pub double_tap: Vec<DoubleTapEvent>,
    pub fifty_fifty: Vec<FiftyFiftyEvent>,
    pub one_timer: Vec<OneTimerEvent>,
    pub pass: Vec<PassEvent>,
    pub goal_tags: Vec<GoalTagEvent>,
    pub rush: Vec<RushEvent>,
    pub speed_flip: Vec<SpeedFlipEvent>,
    pub half_flip: Vec<HalfFlipEvent>,
    pub wavedash: Vec<WavedashEvent>,
    pub whiff: Vec<WhiffEvent>,
    pub boost_pickups: Vec<BoostPickupComparisonEvent>,
    pub bump: Vec<BumpEvent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum MechanicTiming {
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
#[ts(export)]
pub struct MechanicEvent {
    pub id: String,
    pub kind: String,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    pub is_team_0: bool,
    pub timing: MechanicTiming,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayStatsFrame {
    pub frame_number: usize,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
    pub game_state: Option<i32>,
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
    pub double_tap: DoubleTapPlayerStats,
    pub one_timer: OneTimerPlayerStats,
    pub pass: PassPlayerStats,
    pub fifty_fifty: FiftyFiftyPlayerStats,
    pub speed_flip: SpeedFlipStats,
    pub half_flip: HalfFlipStats,
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
