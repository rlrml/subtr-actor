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

/// A coalesced change in a player's discrete camera/vehicle toggles.
///
/// Ball cam, behind-view, and the driving flag flip only a handful of times per
/// match, so rather than storing a value on every [`PlayerFrame`] these are
/// emitted as one change per player whenever any of them flips. Each change
/// carries the full discrete state from that frame onward, so a consumer
/// resolves "ball cam at frame N" with a last-change-before-N lookup.
///
/// Changes are grouped by player on [`ReplayData`](crate::ReplayData) (so the
/// player id is stored once, not per change) and ordered by frame within each
/// player. A field is `None` when the replay never replicated it for that
/// player; `time` and `is_team_0` are intentionally omitted because both are
/// derivable from `frame` and the player id.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PlayerCameraStateChange {
    pub frame: usize,
    /// Whether ball cam (secondary camera) is active from this frame onward.
    pub ball_cam_active: Option<bool>,
    /// Whether behind-view is active from this frame onward.
    pub behind_view_active: Option<bool>,
    /// Whether the car reports the driving flag from this frame onward.
    pub driving: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct TouchEvent {
    /// Stable identity for an attributed touch, assigned monotonically when the
    /// stats pipeline confirms the touch. `None` for raw replay team markers,
    /// which exist before attribution. Downstream events that reference a touch
    /// carry this id so consumers can join exactly instead of matching on
    /// player + frame.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number")]
    pub touch_id: Option<u64>,
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
    /// Ball center in the car's local hitbox coordinates at the attributed touch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact_local_ball_position: Option<[f32; 3]>,
    /// Closest point on the car hitbox to the ball center, in local hitbox coordinates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact_local_hitbox_point: Option<[f32; 3]>,
    /// Closest point on the car hitbox to the ball center, in field coordinates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact_world_hitbox_point: Option<[f32; 3]>,
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

/// Which competitive-season numbering era a replay belongs to.
///
/// Rocket League restarted its season counter at 1 when it went free-to-play in
/// September 2020, so the era is required to disambiguate (e.g. legacy Season 8
/// vs free-to-play Season 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[ts(export)]
pub enum SeasonEra {
    /// Pre-free-to-play numbered competitive seasons (Season 1–14, 2016–2020).
    Legacy,
    /// Free-to-play seasons (Season 1 onward, from September 2020).
    FreeToPlay,
}

impl SeasonEra {
    /// Single-character code prefix used in the canonical season code.
    fn code_prefix(self) -> char {
        match self {
            SeasonEra::Legacy => 's',
            SeasonEra::FreeToPlay => 'f',
        }
    }
}

/// A resolved competitive season, identified by its numbering era and number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplaySeason {
    /// Which numbering era the season belongs to.
    pub era: SeasonEra,
    /// Season number within its era (1-based).
    pub number: u8,
}

impl ReplaySeason {
    const fn new(era: SeasonEra, number: u8) -> Self {
        Self { era, number }
    }

    /// Canonical short code, e.g. `f21` (free-to-play) or `s14` (legacy). Used as
    /// the stable string key for storage, filtering, and display.
    pub fn code(self) -> String {
        format!("{}{}", self.era.code_prefix(), self.number)
    }

    /// The UTC instant this season went live, from [`SEASON_BOUNDARIES`].
    ///
    /// Always `Some` for a season produced by resolution, since resolved seasons
    /// come from the table; returns `None` only for a hand-constructed
    /// [`ReplaySeason`] that has no boundary entry.
    pub fn start(self) -> Option<SeasonStart> {
        SEASON_BOUNDARIES
            .iter()
            .find(|(_, season)| *season == self)
            .map(|(start, _)| *start)
    }
}

/// The UTC instant a competitive season went live.
///
/// Stored per season so callers can display or reason about a boundary at finer
/// than day precision. Season *resolution* still uses only the date (see
/// [`season_for_date`]): the replay `Date` header is timezone-less local
/// wall-clock, so its time-of-day cannot be meaningfully compared against a UTC
/// instant. The time is therefore informational, and rough for older seasons
/// (see [`SEASON_BOUNDARIES`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct SeasonStart {
    /// UTC calendar year.
    pub year: i32,
    /// UTC month, 1-based.
    pub month: u32,
    /// UTC day of month, 1-based.
    pub day: u32,
    /// UTC hour, 0–23.
    pub hour: u32,
    /// UTC minute, 0–59.
    pub minute: u32,
}

impl SeasonStart {
    const fn new(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
        }
    }

    /// The `(year, month, day)` UTC date portion, used for day-granular
    /// resolution against the replay's local-wall-clock date.
    const fn date(self) -> (i32, u32, u32) {
        (self.year, self.month, self.day)
    }

    /// Full UTC datetime as a comparable tuple `(year, month, day, hour, minute)`.
    const fn as_datetime_tuple(self) -> (i32, u32, u32, u32, u32) {
        (self.year, self.month, self.day, self.hour, self.minute)
    }
}

/// Competitive-season start dates, ascending by date.
///
/// Rocket League replays do not record the competitive season directly, so it is
/// resolved from the recorded match `Date` against this table. Each entry records
/// only the date a season *began*; a season runs until the next entry's start.
/// Seasons are contiguous, so storing only the start (rather than start/end pairs)
/// keeps the table impossible to leave with gaps or overlaps.
///
/// Each entry is the UTC instant the season went live (see [`SeasonStart`]).
/// [`SeasonStart::new`] is `const` and the entries already order correctly, so
/// the table needs no parsing or lazy initialization.
///
/// Resolution itself is day-granular (see [`season_for_date`]): the replay
/// `Date` header is local wall-clock with no timezone, so a replay recorded
/// within a day of a boundary is inherently ambiguous and the stored time-of-day
/// cannot be compared against it. The time is stored per season purely as data
/// for callers that want it (display, reporting).
///
/// Time precision by source:
/// - Free-to-play S3 and S18–S23: exact go-live time from the official
///   Rocket League "Season N Live" patch notes (which the season megathreads on
///   r/RocketLeague cite). These announce a time like "9 AM PT / 4 PM UTC".
/// - All other seasons: the date is the launch date, but the time is the
///   *approximate* standard ~9 AM Pacific launch converted to UTC (16:00 in
///   PDT, 17:00 in PST). Treat the hour for these as rough.
///
/// TODO(season-dates): the legacy (pre-free-to-play) start dates are still
/// best-effort and have not been cross-checked against patch notes.
const SEASON_BOUNDARIES: &[(SeasonStart, ReplaySeason)] = &[
    // Pre-free-to-play numbered competitive seasons. Times approximate (~9 AM PT).
    (
        SeasonStart::new(2016, 2, 10, 17, 0),
        ReplaySeason::new(SeasonEra::Legacy, 1),
    ),
    (
        SeasonStart::new(2016, 6, 20, 16, 0),
        ReplaySeason::new(SeasonEra::Legacy, 2),
    ),
    (
        SeasonStart::new(2016, 9, 8, 16, 0),
        ReplaySeason::new(SeasonEra::Legacy, 3),
    ),
    (
        SeasonStart::new(2017, 3, 22, 16, 0),
        ReplaySeason::new(SeasonEra::Legacy, 4),
    ),
    (
        SeasonStart::new(2017, 9, 13, 16, 0),
        ReplaySeason::new(SeasonEra::Legacy, 5),
    ),
    (
        SeasonStart::new(2018, 3, 7, 17, 0),
        ReplaySeason::new(SeasonEra::Legacy, 6),
    ),
    (
        SeasonStart::new(2018, 9, 25, 16, 0),
        ReplaySeason::new(SeasonEra::Legacy, 7),
    ),
    (
        SeasonStart::new(2019, 3, 27, 16, 0),
        ReplaySeason::new(SeasonEra::Legacy, 8),
    ),
    (
        SeasonStart::new(2019, 8, 22, 16, 0),
        ReplaySeason::new(SeasonEra::Legacy, 9),
    ),
    (
        SeasonStart::new(2019, 12, 4, 17, 0),
        ReplaySeason::new(SeasonEra::Legacy, 10),
    ),
    (
        SeasonStart::new(2020, 4, 8, 16, 0),
        ReplaySeason::new(SeasonEra::Legacy, 11),
    ),
    (
        SeasonStart::new(2020, 7, 8, 16, 0),
        ReplaySeason::new(SeasonEra::Legacy, 12),
    ),
    // Free-to-play era. Times approximate (~9 AM PT) except where noted "verified".
    (
        SeasonStart::new(2020, 9, 23, 16, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 1),
    ),
    (
        SeasonStart::new(2020, 12, 9, 17, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 2),
    ),
    (
        SeasonStart::new(2021, 4, 7, 15, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 3),
    ), // verified: 8 AM PDT
    (
        SeasonStart::new(2021, 8, 11, 16, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 4),
    ),
    (
        SeasonStart::new(2021, 11, 17, 17, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 5),
    ),
    (
        SeasonStart::new(2022, 3, 9, 17, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 6),
    ),
    (
        SeasonStart::new(2022, 6, 15, 16, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 7),
    ),
    (
        SeasonStart::new(2022, 9, 7, 16, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 8),
    ),
    (
        SeasonStart::new(2022, 12, 7, 17, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 9),
    ),
    (
        SeasonStart::new(2023, 3, 8, 17, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 10),
    ),
    (
        SeasonStart::new(2023, 6, 7, 16, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 11),
    ),
    (
        SeasonStart::new(2023, 9, 6, 16, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 12),
    ),
    (
        SeasonStart::new(2023, 12, 6, 17, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 13),
    ),
    (
        SeasonStart::new(2024, 3, 6, 17, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 14),
    ),
    (
        SeasonStart::new(2024, 6, 5, 16, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 15),
    ),
    (
        SeasonStart::new(2024, 9, 4, 16, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 16),
    ),
    (
        SeasonStart::new(2024, 12, 4, 17, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 17),
    ),
    (
        SeasonStart::new(2025, 3, 14, 16, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 18),
    ), // verified: 9 AM PDT
    (
        SeasonStart::new(2025, 6, 18, 15, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 19),
    ), // verified: 8 AM PDT
    (
        SeasonStart::new(2025, 9, 17, 16, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 20),
    ), // verified: 9 AM PDT
    (
        SeasonStart::new(2025, 12, 10, 17, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 21),
    ), // verified: 9 AM PST
    (
        SeasonStart::new(2026, 3, 11, 16, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 22),
    ), // verified: 9 AM PDT
    (
        SeasonStart::new(2026, 6, 10, 16, 0),
        ReplaySeason::new(SeasonEra::FreeToPlay, 23),
    ), // verified: 9 AM PDT
];

/// Resolves the competitive season from replay headers via the recorded match
/// date. Returns `None` when no usable date is present or the replay predates the
/// first known season.
pub fn season_from_headers(headers: &[(String, HeaderProp)]) -> Option<ReplaySeason> {
    headers
        .iter()
        .find(|(key, _)| {
            ["Date", "ReplayDate", "RecordDate"]
                .iter()
                .any(|name| key.eq_ignore_ascii_case(name))
        })
        .and_then(|(_, value)| value.as_string())
        .and_then(|s| {
            parse_header_datetime_utc(s)
                .and_then(season_for_datetime)
                .or_else(|| parse_header_date(s).and_then(season_for_date))
        })
}

/// Returns the most recent season whose start is on or before `dt` (UTC).
fn season_for_datetime(dt: (i32, u32, u32, u32, u32)) -> Option<ReplaySeason> {
    SEASON_BOUNDARIES
        .iter()
        .rev()
        .find(|(start, _)| start.as_datetime_tuple() <= dt)
        .map(|(_, season)| *season)
}

/// Returns the most recent season that began on or before `date`.
fn season_for_date(date: (i32, u32, u32)) -> Option<ReplaySeason> {
    SEASON_BOUNDARIES
        .iter()
        .rev()
        .find(|(start, _)| start.date() <= date)
        .map(|(_, season)| *season)
}

/// Parses the replay `Date` header as a UTC `(year, month, day, hour, minute)` tuple.
///
/// The timezone-less format `"YYYY-MM-DD HH-MM-SS"` is assumed to be US Eastern
/// Standard Time (UTC−5). The RFC3339 format `"YYYY-MM-DDTHH:MM:SS±HH:MM"` uses
/// the provided UTC offset. Returns `None` if the time component is absent or
/// unparseable; callers should fall back to [`parse_header_date`] in that case.
fn parse_header_datetime_utc(value: &str) -> Option<(i32, u32, u32, u32, u32)> {
    let s = value.trim();
    if let Some(t_pos) = s.find('T') {
        // RFC3339: "2026-04-17T15:01:25-07:00"
        let (year, month, day) = parse_header_date(&s[..t_pos])?;
        let rest = s.get(t_pos + 1..)?;
        let hour: u32 = rest.get(..2)?.parse().ok()?;
        let minute: u32 = rest.get(3..5)?.parse().ok()?;
        // Offset starts after "HH:MM:SS" (8 chars)
        let offset = rest.get(8..)?;
        let sign: i32 = if offset.starts_with('-') { -1 } else { 1 };
        let off_h: i32 = offset.get(1..3)?.parse().ok()?;
        let utc_mins = hour as i32 * 60 + minute as i32 - sign * off_h * 60;
        return normalize_utc_datetime(year, month, day, utc_mins);
    }
    // Plain format: "2026-04-28 14-30-00", assume US Eastern Standard Time (UTC-5).
    let (date_part, time_part) = s.split_once(' ')?;
    let (year, month, day) = parse_header_date(date_part)?;
    let mut tp = time_part.split('-');
    let hour: u32 = tp.next()?.parse().ok()?;
    let minute: u32 = tp.next()?.parse().ok()?;
    normalize_utc_datetime(year, month, day, hour as i32 * 60 + minute as i32 + 5 * 60)
}

/// Converts `(year, month, day)` + total UTC minutes into a `(year, month, day,
/// hour, minute)` tuple, carrying over into the next day as needed. Month/year
/// overflow is not handled — no season boundaries fall on the last day of a month.
fn normalize_utc_datetime(
    year: i32,
    month: u32,
    day: u32,
    utc_mins: i32,
) -> Option<(i32, u32, u32, u32, u32)> {
    let extra_days = utc_mins.div_euclid(24 * 60);
    let mins = utc_mins.rem_euclid(24 * 60);
    Some((
        year,
        month,
        (day as i32 + extra_days) as u32,
        (mins / 60) as u32,
        (mins % 60) as u32,
    ))
}

/// Parses the leading calendar date (`YYYY-MM-DD`) from a replay `Date` header.
///
/// Replay dates appear as `"2026-04-28 14-30-00"` or RFC3339
/// `"2026-04-17T15:01:25-07:00"`; both begin with the calendar date.
fn parse_header_date(value: &str) -> Option<(i32, u32, u32)> {
    let date = value.trim().split(['T', ' ']).next()?;
    let mut parts = date.split('-');
    let year: i32 = parts.next()?.parse().ok()?;
    let month: u32 = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;
    if (1..=12).contains(&month) && (1..=31).contains(&day) {
        Some((year, month, day))
    } else {
        None
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
    /// Competitive season (era + number) resolved from the replay date, when known.
    pub season: Option<ReplaySeason>,
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

/// The Rocket League camera preset a player used during the match, replicated
/// through `TAGame.CameraSettingsActor_TA:ProfileSettings`.
///
/// Values use the in-game units shown in Rocket League's camera settings menu
/// (`fov` is the horizontal field of view in degrees, distances/heights are in
/// unreal units, `angle` in degrees, and `stiffness`/`swivel_speed`/
/// `transition_speed` are the menu's dimensionless multipliers).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PlayerCameraSettings {
    /// Horizontal field of view, in degrees.
    pub fov: f32,
    /// Camera height above the car, in unreal units.
    pub height: f32,
    /// Camera pitch angle, in degrees (negative looks down).
    pub angle: f32,
    /// Camera distance behind the car, in unreal units.
    pub distance: f32,
    /// Camera stiffness in `[0, 1]`; higher tracks the car more rigidly.
    pub stiffness: f32,
    /// Swivel speed multiplier.
    pub swivel_speed: f32,
    /// Transition speed multiplier; absent in replays older than its addition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub transition_speed: Option<f32>,
}

impl From<&boxcars::CamSettings> for PlayerCameraSettings {
    fn from(settings: &boxcars::CamSettings) -> Self {
        Self {
            fov: settings.fov,
            height: settings.height,
            angle: settings.angle,
            distance: settings.distance,
            stiffness: settings.stiffness,
            swivel_speed: settings.swivel,
            transition_speed: settings.transition,
        }
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
    /// The player's replicated Rocket League camera preset, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub camera_settings: Option<PlayerCameraSettings>,
}

#[cfg(test)]
#[path = "replay_model_tests.rs"]
mod tests;
