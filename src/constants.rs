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
pub static COMPONENT_ACTIVE_KEY: &str = "TAGame.CarComponent_TA:ReplicatedActive";
pub static DEMOLISH_GOAL_EXPLOSION_KEY: &str = "TAGame.Car_TA:ReplicatedDemolishGoalExplosion";
pub static IGNORE_SYNCING_KEY: &str = "TAGame.RBActor_TA:bIgnoreSyncing";
pub static LAST_BOOST_AMOUNT_KEY: &str = "TAGame.CarComponent_Boost_TA:ReplicatedBoostAmount.Last";
pub static PLAYER_NAME_KEY: &str = "Engine.PlayerReplicationInfo:PlayerName";
pub static RIGID_BODY_STATE_KEY: &str = "TAGame.RBActor_TA:ReplicatedRBState";
pub static SECONDS_REMAINING_KEY: &str = "TAGame.GameEvent_Soccar_TA:SecondsRemaining";
pub static TEAM_KEY: &str = "Engine.PlayerReplicationInfo:Team";
pub static UNIQUE_ID_KEY: &str = "Engine.PlayerReplicationInfo:UniqueId";
pub static VEHICLE_KEY: &str = "TAGame.CarComponent_TA:Vehicle";

pub static EMPTY_ACTOR_IDS: [boxcars::ActorId; 0] = [];

pub static BOOST_USED_PER_SECOND: f32 = 80.0 / 0.93;

pub static MAX_DEMOLISH_KNOWN_FRAMES_PASSED: usize = 100;
