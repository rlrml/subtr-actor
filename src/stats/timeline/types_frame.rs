use serde::Serialize;

use crate::*;

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
