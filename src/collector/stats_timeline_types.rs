use crate::*;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct StatsTimelineConfig {
    pub most_back_forward_threshold_y: f32,
    pub pressure_neutral_zone_half_width_y: f32,
    pub rush_max_start_y: f32,
    pub rush_attack_support_distance_y: f32,
    pub rush_defender_distance_y: f32,
    pub rush_min_possession_retained_seconds: f32,
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
    pub backboard: Vec<BackboardBounceEvent>,
    pub ceiling_shot: Vec<CeilingShotEvent>,
    pub double_tap: Vec<DoubleTapEvent>,
    pub fifty_fifty: Vec<FiftyFiftyEvent>,
    pub rush: Vec<RushEvent>,
    pub speed_flip: Vec<SpeedFlipEvent>,
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
    pub rush: RushTeamStats,
    pub core: CoreTeamStats,
    pub backboard: BackboardTeamStats,
    pub double_tap: DoubleTapTeamStats,
    pub ball_carry: BallCarryStats,
    pub boost: BoostStats,
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
    pub fifty_fifty: FiftyFiftyPlayerStats,
    pub speed_flip: SpeedFlipStats,
    pub touch: TouchStats,
    pub musty_flick: MustyFlickStats,
    pub dodge_reset: DodgeResetStats,
    pub ball_carry: BallCarryStats,
    pub boost: BoostStats,
    pub movement: MovementStats,
    pub positioning: PositioningStats,
    pub powerslide: PowerslideStats,
    pub demo: DemoPlayerStats,
}
