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
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub attacker: PlayerId,
    /// The [`PlayerId`] of the player who was demolished.
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub victim: PlayerId,
    /// The velocity of the attacker at the time of demolition.
    #[ts(as = "crate::interop::ts_bindings::Vector3fTs")]
    pub attacker_velocity: boxcars::Vector3f,
    /// The velocity of the victim at the time of demolition.
    #[ts(as = "crate::interop::ts_bindings::Vector3fTs")]
    pub victim_velocity: boxcars::Vector3f,
    /// The location of the attacker at the time of demolition.
    #[ts(as = "Option<crate::interop::ts_bindings::Vector3fTs>")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attacker_location: Option<boxcars::Vector3f>,
    /// The location of the victim at the time of demolition.
    #[ts(as = "crate::interop::ts_bindings::Vector3fTs")]
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
    #[ts(as = "Option<crate::interop::ts_bindings::RemoteIdTs>")]
    pub player: Option<PlayerId>,
    #[ts(as = "Option<crate::interop::ts_bindings::Vector3fTs>")]
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
    #[ts(as = "crate::interop::ts_bindings::Vector3fTs")]
    pub position: boxcars::Vector3f,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalEvent {
    pub time: f32,
    pub frame: usize,
    pub scoring_team_is_team_0: bool,
    #[ts(as = "Option<crate::interop::ts_bindings::RemoteIdTs>")]
    pub player: Option<PlayerId>,
    #[ts(as = "Option<crate::interop::ts_bindings::Vector3fTs>")]
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
pub struct ShotSaveMetadata {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[ts(as = "Option<crate::interop::ts_bindings::Vector3fTs>")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<boxcars::Vector3f>,
    pub is_team_0: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ShotEventMetadata {
    #[ts(as = "crate::interop::ts_bindings::Vector3fTs")]
    pub shot_touch_position: boxcars::Vector3f,
    #[ts(as = "crate::interop::ts_bindings::Vector3fTs")]
    pub ball_position: boxcars::Vector3f,
    #[ts(as = "Option<crate::interop::ts_bindings::Vector3fTs>")]
    pub ball_velocity: Option<boxcars::Vector3f>,
    pub ball_speed: Option<f32>,
    #[ts(as = "Option<crate::interop::ts_bindings::Vector3fTs>")]
    pub player_position: Option<boxcars::Vector3f>,
    #[ts(as = "Option<crate::interop::ts_bindings::Vector3fTs>")]
    pub player_velocity: Option<boxcars::Vector3f>,
    pub player_speed: Option<f32>,
    pub player_distance_to_ball: Option<f32>,
    #[ts(as = "crate::interop::ts_bindings::Vector3fTs")]
    pub target_goal_position: boxcars::Vector3f,
    pub distance_to_goal_center: f32,
    pub distance_to_goal_line: f32,
    pub ball_goal_alignment: Option<f32>,
    pub ball_speed_toward_goal: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resulting_save: Option<ShotSaveMetadata>,
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
            shot_touch_position: ball_body.location,
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
            resulting_save: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PlayerStatEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[ts(as = "Option<crate::interop::ts_bindings::Vector3fTs>")]
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
    #[ts(as = "Option<crate::interop::ts_bindings::RemoteIdTs>")]
    pub player: Option<PlayerId>,
    #[ts(as = "Option<crate::interop::ts_bindings::Vector3fTs>")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<boxcars::Vector3f>,
    /// Ball-to-car hitbox contact gap in uu for attributed touches, when estimated.
    ///
    /// This field keeps its historical name for wire compatibility. A value of
    /// `0.0` means the ball intersects or touches the oriented car hitbox after
    /// subtracting the Rocket League ball collision radius.
    pub closest_approach_distance: Option<f32>,
    pub dodge_contact: bool,
}

impl TouchEvent {
    pub(crate) fn timestamp_ordering(left: &Self, right: &Self) -> std::cmp::Ordering {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    }
}

pub(crate) const TOUCH_RATE_LIMIT_SECONDS: f32 = 0.25;

/// Normalized high-level match type inferred from replay headers and network data.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, ts_rs::TS)]
#[ts(export)]
pub enum ReplayGameType {
    /// Public ranked matchmaking.
    Ranked,
    /// Public unranked/casual matchmaking.
    Casual,
    /// Private match.
    Private,
    /// Local/offline exhibition match.
    Offline,
    /// LAN match.
    Lan,
    /// Tournament match.
    Tournament,
    /// The replay did not expose enough recognized metadata to classify the game type.
    #[default]
    Unknown,
}

/// Raw and normalized game-type metadata for a replay.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayGameTypeDetails {
    /// Easy-to-use normalized classification.
    pub game_type: ReplayGameType,
    /// Header `MatchType`, when present. Post-EAC online replays often only say `Online`.
    pub header_match_type: Option<String>,
    /// Network `ProjectX.GRI_X:ReplicatedGamePlaylist`, when present.
    pub playlist_id: Option<i32>,
    /// Network `TAGame.GameEvent_TA:MatchTypeClass`, resolved to its actor object name.
    pub match_type_class: Option<String>,
}

impl ReplayGameTypeDetails {
    pub fn from_headers(headers: &[(String, HeaderProp)]) -> Self {
        let header_match_type = headers
            .iter()
            .find(|(key, _)| key == "MatchType")
            .and_then(|(_, value)| value.as_string())
            .map(ToOwned::to_owned);

        Self::from_signals(header_match_type, None, None)
    }

    pub fn from_signals(
        header_match_type: Option<String>,
        playlist_id: Option<i32>,
        match_type_class: Option<String>,
    ) -> Self {
        let game_type = infer_replay_game_type(
            header_match_type.as_deref(),
            playlist_id,
            match_type_class.as_deref(),
        );
        Self {
            game_type,
            header_match_type,
            playlist_id,
            match_type_class,
        }
    }

    pub fn with_network_signals(
        &self,
        playlist_id: Option<i32>,
        match_type_class: Option<String>,
    ) -> Self {
        Self::from_signals(
            self.header_match_type.clone(),
            playlist_id.or(self.playlist_id),
            match_type_class.or_else(|| self.match_type_class.clone()),
        )
    }
}

fn infer_replay_game_type(
    header_match_type: Option<&str>,
    playlist_id: Option<i32>,
    match_type_class: Option<&str>,
) -> ReplayGameType {
    if let Some(game_type) = match_type_class.and_then(replay_game_type_from_match_type_class) {
        return game_type;
    }
    if let Some(game_type) = header_match_type.and_then(replay_game_type_from_header_match_type) {
        return game_type;
    }
    if let Some(game_type) = playlist_id.and_then(replay_game_type_from_playlist_id) {
        return game_type;
    }
    ReplayGameType::Unknown
}

fn replay_game_type_from_match_type_class(class_name: &str) -> Option<ReplayGameType> {
    let normalized = class_name.to_ascii_lowercase();
    if normalized.contains("publicranked") {
        Some(ReplayGameType::Ranked)
    } else if normalized.contains("private") {
        Some(ReplayGameType::Private)
    } else if normalized.contains("offline") {
        Some(ReplayGameType::Offline)
    } else if normalized.contains("lan") {
        Some(ReplayGameType::Lan)
    } else if normalized.contains("tournament") {
        Some(ReplayGameType::Tournament)
    } else if normalized.contains("public") {
        Some(ReplayGameType::Casual)
    } else {
        None
    }
}

fn replay_game_type_from_playlist_id(playlist_id: i32) -> Option<ReplayGameType> {
    match playlist_id {
        // Private and offline fixtures use these playlist ids, but LAN can also
        // report 6, so header/class signals intentionally take precedence.
        6 => Some(ReplayGameType::Private),
        8 => Some(ReplayGameType::Offline),
        // Unranked Duel, Doubles, Standard, and Chaos.
        1..=4 => Some(ReplayGameType::Casual),
        // Ranked Duel, Doubles, and Standard.
        10 | 11 | 13 => Some(ReplayGameType::Ranked),
        // Tournament-style fixtures observed across older and current replays.
        22 | 34 => Some(ReplayGameType::Tournament),
        // Older public playlist observed in the fixture corpus.
        23 => Some(ReplayGameType::Casual),
        // Ranked extra modes.
        27..=30 => Some(ReplayGameType::Ranked),
        _ => None,
    }
}

fn replay_game_type_from_header_match_type(match_type: &str) -> Option<ReplayGameType> {
    match match_type.to_ascii_lowercase().as_str() {
        "ranked" => Some(ReplayGameType::Ranked),
        "unranked" | "casual" => Some(ReplayGameType::Casual),
        "private" => Some(ReplayGameType::Private),
        "offline" => Some(ReplayGameType::Offline),
        "lan" => Some(ReplayGameType::Lan),
        "tournament" => Some(ReplayGameType::Tournament),
        // Header-only `Online` is intentionally ambiguous.
        "online" => None,
        _ => None,
    }
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
    /// Normalized and raw game-type signals inferred from headers and network data.
    pub game_type: ReplayGameTypeDetails,
    /// A vector of tuples containing the names and properties of all the headers in the replay.
    #[ts(as = "Vec<(String, crate::interop::ts_bindings::HeaderPropTs)>")]
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
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub remote_id: RemoteId,
    /// An optional HashMap containing player-specific stats.
    /// The keys of this HashMap are the names of the stats,
    /// and the values are the corresponding `HeaderProp` instances.
    #[ts(
        as = "Option<std::collections::HashMap<String, crate::interop::ts_bindings::HeaderPropTs>>"
    )]
    pub stats: Option<std::collections::HashMap<String, HeaderProp>>,
    /// The name of the player as represented in the replay.
    pub name: String,
    /// The replicated car body product id from the player's loadout, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub car_body_id: Option<u32>,
    /// The car body name from replay header player stats, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub car_body_name: Option<String>,
    /// The resolved standardized hitbox family for the player's car body, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub car_hitbox_family: Option<String>,
}

#[cfg(test)]
#[path = "replay_model_tests.rs"]
mod tests;
