pub static BALL_TYPES: [&str; 5] = [
    "Archetypes.Ball.Ball_Default",
    "Archetypes.Ball.Ball_Basketball",
    "Archetypes.Ball.Ball_Puck",
    "Archetypes.Ball.CubeBall",
    "Archetypes.Ball.Ball_Breakout",
];

pub static BOOST_TYPE: &str = "Archetypes.CarComponents.CarComponent_Boost";
pub static CAR_TYPE: &str = "Archetypes.Car.Car_Default";
pub static DODGE_TYPE: &str = "Archetypes.CarComponents.CarComponent_Dodge";
pub static DOUBLE_JUMP_TYPE: &str = "Archetypes.CarComponents.CarComponent_DoubleJump";
pub static GAME_TYPE: &str = "Archetypes.GameEvent.GameEvent_Soccar";
pub static JUMP_TYPE: &str = "Archetypes.CarComponents.CarComponent_Jump";
pub static PLAYER_REPLICATION_KEY: &str = "Engine.Pawn:PlayerReplicationInfo";
pub static PLAYER_TYPE: &str = "TAGame.Default__PRI_TA";

pub static BOOST_AMOUNT_KEY: &str = "TAGame.CarComponent_Boost_TA:ReplicatedBoostAmount";
pub static BOOST_REPLICATED_KEY: &str = "TAGame.CarComponent_Boost_TA:ReplicatedBoost";
pub static BALL_HIT_TEAM_NUM_KEY: &str = "TAGame.Ball_TA:HitTeamNum";
pub static BALL_EXPLOSION_DATA_KEY: &str = "TAGame.Ball_TA:ReplicatedExplosionData";
pub static BALL_EXPLOSION_DATA_EXTENDED_KEY: &str =
    "TAGame.Ball_TA:ReplicatedExplosionDataExtended";
pub static COMPONENT_ACTIVE_KEY: &str = "TAGame.CarComponent_TA:ReplicatedActive";
pub static DEMOLISH_EXTENDED_KEY: &str = "TAGame.Car_TA:ReplicatedDemolishExtended";
pub static DEMOLISH_GOAL_EXPLOSION_KEY: &str = "TAGame.Car_TA:ReplicatedDemolishGoalExplosion";
pub static DODGES_REFRESHED_COUNTER_KEY: &str = "TAGame.Car_TA:DodgesRefreshedCounter";
pub static IGNORE_SYNCING_KEY: &str = "TAGame.RBActor_TA:bIgnoreSyncing";
pub static HANDBRAKE_KEY: &str = "TAGame.Vehicle_TA:bReplicatedHandbrake";
pub static LAST_BOOST_AMOUNT_KEY: &str = "TAGame.CarComponent_Boost_TA:ReplicatedBoostAmount.Last";
pub static MATCH_ASSISTS_KEY: &str = "TAGame.PRI_TA:MatchAssists";
pub static MATCH_GOALS_KEY: &str = "TAGame.PRI_TA:MatchGoals";
pub static MATCH_SAVES_KEY: &str = "TAGame.PRI_TA:MatchSaves";
pub static MATCH_SCORE_KEY: &str = "TAGame.PRI_TA:MatchScore";
pub static MATCH_SHOTS_KEY: &str = "TAGame.PRI_TA:MatchShots";
pub static PLAYER_NAME_KEY: &str = "Engine.PlayerReplicationInfo:PlayerName";
pub static REPLICATED_SCORED_ON_TEAM_KEY: &str =
    "TAGame.GameEvent_Soccar_TA:ReplicatedScoredOnTeam";
pub static RIGID_BODY_STATE_KEY: &str = "TAGame.RBActor_TA:ReplicatedRBState";
pub static SECONDS_REMAINING_KEY: &str = "TAGame.GameEvent_Soccar_TA:SecondsRemaining";
pub static TEAM_GAME_SCORE_KEY: &str = "TAGame.Team_Soccar_TA:GameScore";
pub static TEAM_INFO_SCORE_KEY: &str = "Engine.TeamInfo:Score";
pub static REPLICATED_STATE_NAME_KEY: &str = "TAGame.GameEvent_TA:ReplicatedStateName";
pub static REPLICATED_GAME_STATE_TIME_REMAINING_KEY: &str =
    "TAGame.GameEvent_TA:ReplicatedGameStateTimeRemaining";
pub static BALL_HAS_BEEN_HIT_KEY: &str = "TAGame.GameEvent_Soccar_TA:bBallHasBeenHit";
pub static TEAM_KEY: &str = "Engine.PlayerReplicationInfo:Team";
pub static UNIQUE_ID_KEY: &str = "Engine.PlayerReplicationInfo:UniqueId";
pub static VEHICLE_KEY: &str = "TAGame.CarComponent_TA:Vehicle";

pub static EMPTY_ACTOR_IDS: [boxcars::ActorId; 0] = [];

/// The maximum raw boost value stored in replay data.
///
/// Rocket League replays represent boost on a `0..=255` scale rather than a
/// `0..=100` percentage scale.
pub const BOOST_MAX_AMOUNT: f32 = u8::MAX as f32;

/// The raw replay boost amount players spawn with at each standard kickoff.
///
/// Rocket League starts each kickoff with one third of a full tank, which maps
/// cleanly to `85.0` on the replay's `0..=255` boost scale.
pub const BOOST_KICKOFF_START_AMOUNT: f32 = BOOST_MAX_AMOUNT / 3.0;

/// The rate at which boost drains while active, in raw replay units per second.
pub const BOOST_USED_RAW_UNITS_PER_SECOND: f32 = 80.0 / 0.93;

/// The rate at which boost drains while active, in percentage points per second.
pub const BOOST_USED_PERCENT_PER_SECOND: f32 =
    BOOST_USED_RAW_UNITS_PER_SECOND * 100.0 / BOOST_MAX_AMOUNT;

/// Converts a raw replay boost amount (`0..=255`) to a percentage (`0..=100`).
pub fn boost_amount_to_percent(boost_amount: f32) -> f32 {
    boost_amount * 100.0 / BOOST_MAX_AMOUNT
}

/// Converts a boost percentage (`0..=100`) to a raw replay boost amount (`0..=255`).
pub fn boost_percent_to_amount(boost_percent: f32) -> f32 {
    boost_percent * BOOST_MAX_AMOUNT / 100.0
}

#[deprecated(
    note = "BOOST_USED_PER_SECOND is measured in raw replay units. Use BOOST_USED_RAW_UNITS_PER_SECOND or BOOST_USED_PERCENT_PER_SECOND instead."
)]
pub const BOOST_USED_PER_SECOND: f32 = BOOST_USED_RAW_UNITS_PER_SECOND;

pub static MAX_DEMOLISH_KNOWN_FRAMES_PASSED: usize = 150;
