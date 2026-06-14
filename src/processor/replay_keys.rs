/// Every ball archetype the game has shipped lives under this prefix
/// (`Ball_Default`, `Ball_Basketball`, `Ball_Puck`, `CubeBall`, `Ball_Breakout`,
/// `Ball_WorldCup`, ...). Matching the prefix rather than an explicit whitelist
/// means limited-time modes that introduce new ball archetypes are recognized
/// automatically instead of silently disabling touch detection match-wide.
pub(crate) static BALL_TYPE_PREFIX: &str = "Archetypes.Ball.";

pub(crate) static BOOST_TYPE: &str = "Archetypes.CarComponents.CarComponent_Boost";
pub(crate) static CAR_TYPE: &str = "Archetypes.Car.Car_Default";
pub(crate) static DODGE_TYPE: &str = "Archetypes.CarComponents.CarComponent_Dodge";
pub(crate) static DOUBLE_JUMP_TYPE: &str = "Archetypes.CarComponents.CarComponent_DoubleJump";
pub(crate) static GAME_TYPE: &str = "Archetypes.GameEvent.GameEvent_Soccar";
pub(crate) static JUMP_TYPE: &str = "Archetypes.CarComponents.CarComponent_Jump";
pub(crate) static PLAYER_REPLICATION_KEY: &str = "Engine.Pawn:PlayerReplicationInfo";
pub(crate) static PLAYER_TYPE: &str = "TAGame.Default__PRI_TA";

pub(crate) static BOOST_AMOUNT_KEY: &str = "TAGame.CarComponent_Boost_TA:ReplicatedBoostAmount";
pub(crate) static BOOST_REPLICATED_KEY: &str = "TAGame.CarComponent_Boost_TA:ReplicatedBoost";
pub(crate) static BALL_HIT_TEAM_NUM_KEY: &str = "TAGame.Ball_TA:HitTeamNum";
pub(crate) static BALL_EXPLOSION_DATA_KEY: &str = "TAGame.Ball_TA:ReplicatedExplosionData";
pub(crate) static BALL_EXPLOSION_DATA_EXTENDED_KEY: &str =
    "TAGame.Ball_TA:ReplicatedExplosionDataExtended";
pub(crate) static BOT_KEY: &str = "Engine.PlayerReplicationInfo:bBot";
pub(crate) static CAMERA_SETTINGS_PRI_KEY: &str = "TAGame.CameraSettingsActor_TA:PRI";
pub(crate) static CAMERA_SETTINGS_PROFILE_KEY: &str =
    "TAGame.CameraSettingsActor_TA:ProfileSettings";
// Dynamic, per-frame camera state replicated on the
// `TAGame.CameraSettingsActor_TA` actor (linked back to a player actor via
// [`CAMERA_SETTINGS_PRI_KEY`]). These let us reconstruct a player's actual
// in-game camera (ball cam toggle + look direction) during playback rather
// than synthesizing one from car/ball geometry.
pub(crate) static CAMERA_BALL_CAM_KEY: &str = "TAGame.CameraSettingsActor_TA:bUsingSecondaryCamera";
pub(crate) static CAMERA_BEHIND_VIEW_KEY: &str = "TAGame.CameraSettingsActor_TA:bUsingBehindView";
pub(crate) static CAMERA_PITCH_KEY: &str = "TAGame.CameraSettingsActor_TA:CameraPitch";
pub(crate) static CAMERA_YAW_KEY: &str = "TAGame.CameraSettingsActor_TA:CameraYaw";
pub(crate) static CLIENT_LOADOUTS_KEY: &str = "TAGame.PRI_TA:ClientLoadouts";
pub(crate) static COMPONENT_ACTIVE_KEY: &str = "TAGame.CarComponent_TA:ReplicatedActive";
pub(crate) static DEMOLISH_EXTENDED_KEY: &str = "TAGame.Car_TA:ReplicatedDemolishExtended";
pub(crate) static DEMOLISH_GOAL_EXPLOSION_KEY: &str =
    "TAGame.Car_TA:ReplicatedDemolishGoalExplosion";
pub(crate) static DODGES_REFRESHED_COUNTER_KEY: &str = "TAGame.Car_TA:DodgesRefreshedCounter";
pub(crate) static IGNORE_SYNCING_KEY: &str = "TAGame.RBActor_TA:bIgnoreSyncing";
pub(crate) static HANDBRAKE_KEY: &str = "TAGame.Vehicle_TA:bReplicatedHandbrake";
// Per-frame vehicle inputs replicated on the car's `TAGame.Vehicle_TA` actor.
// These drive accurate wheel steering/spin and engine state during playback.
pub(crate) static THROTTLE_KEY: &str = "TAGame.Vehicle_TA:ReplicatedThrottle";
pub(crate) static STEER_KEY: &str = "TAGame.Vehicle_TA:ReplicatedSteer";
pub(crate) static DRIVING_KEY: &str = "TAGame.Vehicle_TA:bDriving";
// Dodge impulse/torque replicated on the dodge component the instant a dodge
// fires; combined with `dodge_active` they give the exact flip direction.
pub(crate) static DODGE_IMPULSE_KEY: &str = "TAGame.CarComponent_Dodge_TA:DodgeImpulse";
pub(crate) static DODGE_TORQUE_KEY: &str = "TAGame.CarComponent_Dodge_TA:DodgeTorque";
pub(crate) static LAST_BOOST_AMOUNT_KEY: &str =
    "TAGame.CarComponent_Boost_TA:ReplicatedBoostAmount.Last";
pub(crate) static MATCH_ASSISTS_KEY: &str = "TAGame.PRI_TA:MatchAssists";
pub(crate) static MATCH_GOALS_KEY: &str = "TAGame.PRI_TA:MatchGoals";
pub(crate) static MATCH_SAVES_KEY: &str = "TAGame.PRI_TA:MatchSaves";
pub(crate) static MATCH_SCORE_KEY: &str = "TAGame.PRI_TA:MatchScore";
pub(crate) static MATCH_SHOTS_KEY: &str = "TAGame.PRI_TA:MatchShots";
pub(crate) static PLAYER_NAME_KEY: &str = "Engine.PlayerReplicationInfo:PlayerName";
pub(crate) static REPLICATED_SCORED_ON_TEAM_KEY: &str =
    "TAGame.GameEvent_Soccar_TA:ReplicatedScoredOnTeam";
pub(crate) static RIGID_BODY_STATE_KEY: &str = "TAGame.RBActor_TA:ReplicatedRBState";
pub(crate) static SECONDS_REMAINING_KEY: &str = "TAGame.GameEvent_Soccar_TA:SecondsRemaining";
pub(crate) static TEAM_GAME_SCORE_KEY: &str = "TAGame.Team_Soccar_TA:GameScore";
pub(crate) static TEAM_INFO_SCORE_KEY: &str = "Engine.TeamInfo:Score";
pub(crate) static REPLICATED_STATE_NAME_KEY: &str = "TAGame.GameEvent_TA:ReplicatedStateName";
pub(crate) static REPLICATED_GAME_STATE_TIME_REMAINING_KEY: &str =
    "TAGame.GameEvent_TA:ReplicatedGameStateTimeRemaining";
pub(crate) static MATCH_TYPE_CLASS_KEY: &str = "TAGame.GameEvent_TA:MatchTypeClass";
pub(crate) static BALL_HAS_BEEN_HIT_KEY: &str = "TAGame.GameEvent_Soccar_TA:bBallHasBeenHit";
pub(crate) static REPLICATED_GAME_PLAYLIST_KEY: &str = "ProjectX.GRI_X:ReplicatedGamePlaylist";
pub(crate) static TEAM_KEY: &str = "Engine.PlayerReplicationInfo:Team";
pub(crate) static UNIQUE_ID_KEY: &str = "Engine.PlayerReplicationInfo:UniqueId";
pub(crate) static VEHICLE_KEY: &str = "TAGame.CarComponent_TA:Vehicle";

pub(crate) static EMPTY_ACTOR_IDS: [boxcars::ActorId; 0] = [];

pub(crate) static MAX_DEMOLISH_KNOWN_FRAMES_PASSED: usize = 150;
