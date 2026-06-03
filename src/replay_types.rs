use boxcars::{HeaderProp, RemoteId};
use serde::Serialize;

use crate::{glam_to_vec, vec_to_glam};

pub type PlayerId = boxcars::RemoteId;

/// Represents which demolition format a replay uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemolishFormat {
    /// Old format (pre-September 2024): uses `ReplicatedDemolishGoalExplosion`
    Fx,
    /// New format (September 2024+): uses `ReplicatedDemolishExtended`
    Extended,
}

/// Wrapper enum for different demolition attribute formats across Rocket League versions.
///
/// Rocket League changed the demolition data structure around September 2024 (v2.43+),
/// moving from `DemolishFx` to `DemolishExtended`. This enum provides a unified interface
/// for both formats.
#[derive(Debug, Clone, PartialEq)]
pub enum DemolishAttribute {
    Fx(boxcars::DemolishFx),
    Extended(boxcars::DemolishExtended),
}

impl DemolishAttribute {
    pub fn attacker_actor_id(&self) -> boxcars::ActorId {
        match self {
            DemolishAttribute::Fx(fx) => fx.attacker,
            DemolishAttribute::Extended(ext) => ext.attacker.actor,
        }
    }

    pub fn victim_actor_id(&self) -> boxcars::ActorId {
        match self {
            DemolishAttribute::Fx(fx) => fx.victim,
            DemolishAttribute::Extended(ext) => ext.victim.actor,
        }
    }

    pub fn attacker_velocity(&self) -> boxcars::Vector3f {
        match self {
            DemolishAttribute::Fx(fx) => fx.attack_velocity,
            DemolishAttribute::Extended(ext) => ext.attacker_velocity,
        }
    }

    pub fn victim_velocity(&self) -> boxcars::Vector3f {
        match self {
            DemolishAttribute::Fx(fx) => fx.victim_velocity,
            DemolishAttribute::Extended(ext) => ext.victim_velocity,
        }
    }
}

/// [`DemolishInfo`] struct represents data related to a demolition event in the game.
///
/// Demolition events occur when one player 'demolishes' or 'destroys' another by
/// hitting them at a sufficiently high speed. This results in the demolished player
/// being temporarily removed from play.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DemolishInfo {
    /// The exact game time (in seconds) at which the demolition event occurred.
    pub time: f32,
    /// The remaining time in the match when the demolition event occurred.
    pub seconds_remaining: i32,
    /// The frame number at which the demolition occurred.
    pub frame: usize,
    /// The [`PlayerId`] of the player who initiated the demolition.
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub attacker: PlayerId,
    /// The [`PlayerId`] of the player who was demolished.
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub victim: PlayerId,
    /// The velocity of the attacker at the time of demolition.
    #[ts(as = "crate::ts_bindings::Vector3fTs")]
    pub attacker_velocity: boxcars::Vector3f,
    /// The velocity of the victim at the time of demolition.
    #[ts(as = "crate::ts_bindings::Vector3fTs")]
    pub victim_velocity: boxcars::Vector3f,
    /// The location of the attacker at the time of demolition.
    #[ts(as = "Option<crate::ts_bindings::Vector3fTs>")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attacker_location: Option<boxcars::Vector3f>,
    /// The location of the victim at the time of demolition.
    #[ts(as = "crate::ts_bindings::Vector3fTs")]
    pub victim_location: boxcars::Vector3f,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, ts_rs::TS)]
#[ts(export)]
pub enum BoostPadEventKind {
    PickedUp { sequence: u8 },
    Available,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, ts_rs::TS)]
#[ts(export)]
pub enum BoostPadSize {
    Big,
    Small,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BoostPadEvent {
    pub time: f32,
    pub frame: usize,
    pub pad_id: String,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub player: Option<PlayerId>,
    #[ts(as = "Option<crate::ts_bindings::Vector3fTs>")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<boxcars::Vector3f>,
    pub kind: BoostPadEventKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ResolvedBoostPad {
    pub index: usize,
    pub pad_id: Option<String>,
    pub size: BoostPadSize,
    #[ts(as = "crate::ts_bindings::Vector3fTs")]
    pub position: boxcars::Vector3f,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalEvent {
    pub time: f32,
    pub frame: usize,
    pub scoring_team_is_team_0: bool,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub player: Option<PlayerId>,
    #[ts(as = "Option<crate::ts_bindings::Vector3fTs>")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<boxcars::Vector3f>,
    pub team_zero_score: Option<i32>,
    pub team_one_score: Option<i32>,
}

/// A replay tick mark stored in the replay file.
///
/// Rocket League/Boxcars use tick marks for replay timeline annotations such as
/// goal markers and other saved replay highlights. The frame is preserved from
/// the replay body; `time` is resolved from collected frame metadata when that
/// frame is present in the processed replay.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayTickMark {
    pub description: String,
    pub frame: i32,
    pub time: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, ts_rs::TS)]
#[ts(export)]
pub enum PlayerStatEventKind {
    Shot,
    Save,
    Assist,
}

const SHOT_TARGET_GOAL_CENTER_Y: f32 = 5120.0;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ShotEventMetadata {
    #[ts(as = "crate::ts_bindings::Vector3fTs")]
    pub ball_position: boxcars::Vector3f,
    #[ts(as = "Option<crate::ts_bindings::Vector3fTs>")]
    pub ball_velocity: Option<boxcars::Vector3f>,
    pub ball_speed: Option<f32>,
    #[ts(as = "Option<crate::ts_bindings::Vector3fTs>")]
    pub player_position: Option<boxcars::Vector3f>,
    #[ts(as = "Option<crate::ts_bindings::Vector3fTs>")]
    pub player_velocity: Option<boxcars::Vector3f>,
    pub player_speed: Option<f32>,
    pub player_distance_to_ball: Option<f32>,
    #[ts(as = "crate::ts_bindings::Vector3fTs")]
    pub target_goal_position: boxcars::Vector3f,
    pub distance_to_goal_center: f32,
    pub distance_to_goal_line: f32,
    pub ball_goal_alignment: Option<f32>,
    pub ball_speed_toward_goal: Option<f32>,
}

impl ShotEventMetadata {
    pub fn from_rigid_bodies(
        is_team_0: bool,
        ball_body: &boxcars::RigidBody,
        player_body: Option<&boxcars::RigidBody>,
    ) -> Self {
        let ball_position = vec_to_glam(&ball_body.location);
        let ball_velocity = ball_body.linear_velocity.as_ref().map(vec_to_glam);
        let player_position = player_body.map(|body| vec_to_glam(&body.location));
        let player_velocity =
            player_body.and_then(|body| body.linear_velocity.as_ref().map(vec_to_glam));
        let target_goal_y = if is_team_0 {
            SHOT_TARGET_GOAL_CENTER_Y
        } else {
            -SHOT_TARGET_GOAL_CENTER_Y
        };
        let target_goal_position = glam::Vec3::new(0.0, target_goal_y, ball_position.z);
        let goal_vector = target_goal_position - ball_position;
        let goal_direction = goal_vector.normalize_or_zero();
        let forward_sign = if is_team_0 { 1.0 } else { -1.0 };
        let distance_to_goal_line = ((target_goal_y - ball_position.y) * forward_sign).max(0.0);
        let ball_goal_alignment = ball_velocity.map(|velocity| {
            if velocity.length_squared() <= f32::EPSILON {
                0.0
            } else {
                goal_direction.dot(velocity.normalize_or_zero())
            }
        });

        Self {
            ball_position: ball_body.location,
            ball_velocity: ball_body.linear_velocity,
            ball_speed: ball_velocity.map(|velocity| velocity.length()),
            player_position: player_body.map(|body| body.location),
            player_velocity: player_body.and_then(|body| body.linear_velocity),
            player_speed: player_velocity.map(|velocity| velocity.length()),
            player_distance_to_ball: player_position
                .map(|position| (position - ball_position).length()),
            target_goal_position: glam_to_vec(&target_goal_position),
            distance_to_goal_center: goal_vector.length(),
            distance_to_goal_line,
            ball_goal_alignment,
            ball_speed_toward_goal: ball_velocity.map(|velocity| goal_direction.dot(velocity)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PlayerStatEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[ts(as = "Option<crate::ts_bindings::Vector3fTs>")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<boxcars::Vector3f>,
    pub is_team_0: bool,
    pub kind: PlayerStatEventKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shot: Option<ShotEventMetadata>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct TouchEvent {
    pub time: f32,
    pub frame: usize,
    pub team_is_team_0: bool,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub player: Option<PlayerId>,
    #[ts(as = "Option<crate::ts_bindings::Vector3fTs>")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<boxcars::Vector3f>,
    pub closest_approach_distance: Option<f32>,
    pub dodge_contact: bool,
}

/// [`ReplayMeta`] struct represents metadata about the replay being processed.
///
/// This includes information about the players in the match and all replay headers.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayMeta {
    /// A vector of [`PlayerInfo`] instances representing the players on team zero.
    pub team_zero: Vec<PlayerInfo>,
    /// A vector of [`PlayerInfo`] instances representing the players on team one.
    pub team_one: Vec<PlayerInfo>,
    /// A vector of tuples containing the names and properties of all the headers in the replay.
    #[ts(as = "Vec<(String, crate::ts_bindings::HeaderPropTs)>")]
    pub all_headers: Vec<(String, HeaderProp)>,
}

impl ReplayMeta {
    /// Returns the total number of players involved in the game.
    pub fn player_count(&self) -> usize {
        self.team_one.len() + self.team_zero.len()
    }

    /// Returns an iterator over the [`PlayerInfo`] instances representing the players,
    /// in the order they are listed in the replay file.
    pub fn player_order(&self) -> impl Iterator<Item = &PlayerInfo> {
        self.team_zero.iter().chain(self.team_one.iter())
    }
}

/// [`PlayerInfo`] struct provides detailed information about a specific player in the replay.
///
/// This includes player's unique remote ID, player stats if available, and their name.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PlayerInfo {
    /// The unique remote ID of the player. This could be their online ID or local ID.
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub remote_id: RemoteId,
    /// An optional HashMap containing player-specific stats.
    /// The keys of this HashMap are the names of the stats,
    /// and the values are the corresponding `HeaderProp` instances.
    #[ts(as = "Option<std::collections::HashMap<String, crate::ts_bindings::HeaderPropTs>>")]
    pub stats: Option<std::collections::HashMap<String, HeaderProp>>,
    /// The name of the player as represented in the replay.
    pub name: String,
}

#[cfg(test)]
#[path = "replay_types_tests.rs"]
mod tests;
