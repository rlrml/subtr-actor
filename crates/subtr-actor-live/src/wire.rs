//! Wire-safe mirrors of core `subtr-actor` event types.
//!
//! The core event types (`TouchEvent`, `GoalEvent`, `DemolishInfo`, ...) are
//! `Serialize`-only and use `skip_serializing_if` for optional fields, which
//! makes them unusable on a postcard wire: postcard is not self-describing, so
//! a conditionally-skipped field desynchronizes the byte stream. Every type in
//! this module therefore:
//!
//! - derives both `Serialize` and `Deserialize`,
//! - carries **zero** serde attributes (this crate's wire-type rule), and
//! - converts losslessly to/from its core counterpart via `From` in both
//!   directions.
//!
//! Do not add serde attributes here; the protocol round-trip tests exist to
//! catch exactly that mistake.

use boxcars::Vector3f;
use serde::{Deserialize, Serialize};
use subtr_actor::{
    BoostPadEvent, BoostPadEventKind, DemoEventSample, DemolishInfo, DodgeRefreshedEvent,
    FrameEventsState, GoalEvent, PlayerId, PlayerStatEvent, PlayerStatEventKind, ShotEventMetadata,
    ShotGoalLineCrossing, ShotGoalLineCrossingPredictionKind,
    ShotGoalLineCrossingUnavailableReason, ShotGoalTargetHit, ShotGoalTargetHitKind,
    ShotSaveMetadata, TouchEvent,
};

use crate::generator::LiveEventHistory;

/// Wire mirror of [`subtr_actor::DemoEventSample`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireDemoEventSample {
    pub attacker: PlayerId,
    pub victim: PlayerId,
}

impl From<DemoEventSample> for WireDemoEventSample {
    fn from(value: DemoEventSample) -> Self {
        Self {
            attacker: value.attacker,
            victim: value.victim,
        }
    }
}

impl From<WireDemoEventSample> for DemoEventSample {
    fn from(value: WireDemoEventSample) -> Self {
        Self {
            attacker: value.attacker,
            victim: value.victim,
        }
    }
}

/// Wire mirror of [`subtr_actor::DemolishInfo`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireDemolishInfo {
    pub time: f32,
    pub seconds_remaining: i32,
    pub frame: usize,
    pub attacker: PlayerId,
    pub victim: PlayerId,
    pub attacker_velocity: Vector3f,
    pub victim_velocity: Vector3f,
    pub attacker_location: Option<Vector3f>,
    pub victim_location: Vector3f,
}

impl From<DemolishInfo> for WireDemolishInfo {
    fn from(value: DemolishInfo) -> Self {
        Self {
            time: value.time,
            seconds_remaining: value.seconds_remaining,
            frame: value.frame,
            attacker: value.attacker,
            victim: value.victim,
            attacker_velocity: value.attacker_velocity,
            victim_velocity: value.victim_velocity,
            attacker_location: value.attacker_location,
            victim_location: value.victim_location,
        }
    }
}

impl From<WireDemolishInfo> for DemolishInfo {
    fn from(value: WireDemolishInfo) -> Self {
        Self {
            time: value.time,
            seconds_remaining: value.seconds_remaining,
            frame: value.frame,
            attacker: value.attacker,
            victim: value.victim,
            attacker_velocity: value.attacker_velocity,
            victim_velocity: value.victim_velocity,
            attacker_location: value.attacker_location,
            victim_location: value.victim_location,
        }
    }
}

/// Wire mirror of [`subtr_actor::BoostPadEventKind`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WireBoostPadEventKind {
    PickedUp { sequence: u8 },
    Available,
}

impl From<BoostPadEventKind> for WireBoostPadEventKind {
    fn from(value: BoostPadEventKind) -> Self {
        match value {
            BoostPadEventKind::PickedUp { sequence } => Self::PickedUp { sequence },
            BoostPadEventKind::Available => Self::Available,
        }
    }
}

impl From<WireBoostPadEventKind> for BoostPadEventKind {
    fn from(value: WireBoostPadEventKind) -> Self {
        match value {
            WireBoostPadEventKind::PickedUp { sequence } => Self::PickedUp { sequence },
            WireBoostPadEventKind::Available => Self::Available,
        }
    }
}

/// Wire mirror of [`subtr_actor::BoostPadEvent`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireBoostPadEvent {
    pub time: f32,
    pub frame: usize,
    pub pad_id: String,
    pub player: Option<PlayerId>,
    pub player_position: Option<Vector3f>,
    pub kind: WireBoostPadEventKind,
}

impl From<BoostPadEvent> for WireBoostPadEvent {
    fn from(value: BoostPadEvent) -> Self {
        Self {
            time: value.time,
            frame: value.frame,
            pad_id: value.pad_id,
            player: value.player,
            player_position: value.player_position,
            kind: value.kind.into(),
        }
    }
}

impl From<WireBoostPadEvent> for BoostPadEvent {
    fn from(value: WireBoostPadEvent) -> Self {
        Self {
            time: value.time,
            frame: value.frame,
            pad_id: value.pad_id,
            player: value.player,
            player_position: value.player_position,
            kind: value.kind.into(),
        }
    }
}

/// Wire mirror of [`subtr_actor::GoalEvent`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireGoalEvent {
    pub time: f32,
    pub frame: usize,
    pub scoring_team_is_team_0: bool,
    pub player: Option<PlayerId>,
    pub player_position: Option<Vector3f>,
    pub team_zero_score: Option<i32>,
    pub team_one_score: Option<i32>,
}

impl From<GoalEvent> for WireGoalEvent {
    fn from(value: GoalEvent) -> Self {
        Self {
            time: value.time,
            frame: value.frame,
            scoring_team_is_team_0: value.scoring_team_is_team_0,
            player: value.player,
            player_position: value.player_position,
            team_zero_score: value.team_zero_score,
            team_one_score: value.team_one_score,
        }
    }
}

impl From<WireGoalEvent> for GoalEvent {
    fn from(value: WireGoalEvent) -> Self {
        Self {
            time: value.time,
            frame: value.frame,
            scoring_team_is_team_0: value.scoring_team_is_team_0,
            player: value.player,
            player_position: value.player_position,
            team_zero_score: value.team_zero_score,
            team_one_score: value.team_one_score,
        }
    }
}

/// Wire mirror of [`subtr_actor::TouchEvent`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireTouchEvent {
    pub touch_id: Option<u64>,
    pub time: f32,
    pub frame: usize,
    pub team_is_team_0: bool,
    pub player: Option<PlayerId>,
    pub player_position: Option<Vector3f>,
    pub closest_approach_distance: Option<f32>,
    pub contact_local_ball_position: Option<[f32; 3]>,
    pub contact_local_hitbox_point: Option<[f32; 3]>,
    pub contact_world_hitbox_point: Option<[f32; 3]>,
    pub dodge_contact: bool,
}

impl From<TouchEvent> for WireTouchEvent {
    fn from(value: TouchEvent) -> Self {
        Self {
            touch_id: value.touch_id,
            time: value.time,
            frame: value.frame,
            team_is_team_0: value.team_is_team_0,
            player: value.player,
            player_position: value.player_position,
            closest_approach_distance: value.closest_approach_distance,
            contact_local_ball_position: value.contact_local_ball_position,
            contact_local_hitbox_point: value.contact_local_hitbox_point,
            contact_world_hitbox_point: value.contact_world_hitbox_point,
            dodge_contact: value.dodge_contact,
        }
    }
}

impl From<WireTouchEvent> for TouchEvent {
    fn from(value: WireTouchEvent) -> Self {
        Self {
            touch_id: value.touch_id,
            time: value.time,
            frame: value.frame,
            team_is_team_0: value.team_is_team_0,
            player: value.player,
            player_position: value.player_position,
            closest_approach_distance: value.closest_approach_distance,
            contact_local_ball_position: value.contact_local_ball_position,
            contact_local_hitbox_point: value.contact_local_hitbox_point,
            contact_world_hitbox_point: value.contact_world_hitbox_point,
            dodge_contact: value.dodge_contact,
        }
    }
}

/// Wire mirror of [`subtr_actor::DodgeRefreshedEvent`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireDodgeRefreshedEvent {
    pub time: f32,
    pub frame: usize,
    pub player: PlayerId,
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub counter_value: i32,
}

impl From<DodgeRefreshedEvent> for WireDodgeRefreshedEvent {
    fn from(value: DodgeRefreshedEvent) -> Self {
        Self {
            time: value.time,
            frame: value.frame,
            player: value.player,
            player_position: value.player_position,
            is_team_0: value.is_team_0,
            counter_value: value.counter_value,
        }
    }
}

impl From<WireDodgeRefreshedEvent> for DodgeRefreshedEvent {
    fn from(value: WireDodgeRefreshedEvent) -> Self {
        Self {
            time: value.time,
            frame: value.frame,
            player: value.player,
            player_position: value.player_position,
            is_team_0: value.is_team_0,
            counter_value: value.counter_value,
        }
    }
}

/// Wire mirror of [`subtr_actor::PlayerStatEventKind`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WirePlayerStatEventKind {
    Shot,
    Save,
    Assist,
}

impl From<PlayerStatEventKind> for WirePlayerStatEventKind {
    fn from(value: PlayerStatEventKind) -> Self {
        match value {
            PlayerStatEventKind::Shot => Self::Shot,
            PlayerStatEventKind::Save => Self::Save,
            PlayerStatEventKind::Assist => Self::Assist,
        }
    }
}

impl From<WirePlayerStatEventKind> for PlayerStatEventKind {
    fn from(value: WirePlayerStatEventKind) -> Self {
        match value {
            WirePlayerStatEventKind::Shot => Self::Shot,
            WirePlayerStatEventKind::Save => Self::Save,
            WirePlayerStatEventKind::Assist => Self::Assist,
        }
    }
}

/// Wire mirror of [`subtr_actor::ShotSaveMetadata`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireShotSaveMetadata {
    pub time: f32,
    pub frame: usize,
    pub player: PlayerId,
    pub player_position: Option<Vector3f>,
    pub is_team_0: bool,
}

impl From<ShotSaveMetadata> for WireShotSaveMetadata {
    fn from(value: ShotSaveMetadata) -> Self {
        Self {
            time: value.time,
            frame: value.frame,
            player: value.player,
            player_position: value.player_position,
            is_team_0: value.is_team_0,
        }
    }
}

impl From<WireShotSaveMetadata> for ShotSaveMetadata {
    fn from(value: WireShotSaveMetadata) -> Self {
        Self {
            time: value.time,
            frame: value.frame,
            player: value.player,
            player_position: value.player_position,
            is_team_0: value.is_team_0,
        }
    }
}

/// Wire mirror of [`subtr_actor::ShotGoalLineCrossingPredictionKind`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WireShotGoalLineCrossingPredictionKind {
    SurfaceBounces,
    FreeFlight,
    SavedShotPreSaveSurfaceBounces,
    SavedShotPreSaveFreeFlight,
}

impl From<ShotGoalLineCrossingPredictionKind> for WireShotGoalLineCrossingPredictionKind {
    fn from(value: ShotGoalLineCrossingPredictionKind) -> Self {
        match value {
            ShotGoalLineCrossingPredictionKind::SurfaceBounces => Self::SurfaceBounces,
            ShotGoalLineCrossingPredictionKind::FreeFlight => Self::FreeFlight,
            ShotGoalLineCrossingPredictionKind::SavedShotPreSaveSurfaceBounces => {
                Self::SavedShotPreSaveSurfaceBounces
            }
            ShotGoalLineCrossingPredictionKind::SavedShotPreSaveFreeFlight => {
                Self::SavedShotPreSaveFreeFlight
            }
        }
    }
}

impl From<WireShotGoalLineCrossingPredictionKind> for ShotGoalLineCrossingPredictionKind {
    fn from(value: WireShotGoalLineCrossingPredictionKind) -> Self {
        match value {
            WireShotGoalLineCrossingPredictionKind::SurfaceBounces => Self::SurfaceBounces,
            WireShotGoalLineCrossingPredictionKind::FreeFlight => Self::FreeFlight,
            WireShotGoalLineCrossingPredictionKind::SavedShotPreSaveSurfaceBounces => {
                Self::SavedShotPreSaveSurfaceBounces
            }
            WireShotGoalLineCrossingPredictionKind::SavedShotPreSaveFreeFlight => {
                Self::SavedShotPreSaveFreeFlight
            }
        }
    }
}

/// Wire mirror of [`subtr_actor::ShotGoalLineCrossingUnavailableReason`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WireShotGoalLineCrossingUnavailableReason {
    NoBallVelocity,
    NoGoalwardBallBeforeSaveReference,
    NoGoalLineCrossingBeforeSaveReference,
    OnlyUnphysicalFreeFlightCrossings,
    CrossingsBeforePredictionStart,
    CrossingsBeforeSaveTouch,
    CrossingsBeforeSaveStat,
    NoUsableProjection,
}

impl From<ShotGoalLineCrossingUnavailableReason> for WireShotGoalLineCrossingUnavailableReason {
    fn from(value: ShotGoalLineCrossingUnavailableReason) -> Self {
        match value {
            ShotGoalLineCrossingUnavailableReason::NoBallVelocity => Self::NoBallVelocity,
            ShotGoalLineCrossingUnavailableReason::NoGoalwardBallBeforeSaveReference => {
                Self::NoGoalwardBallBeforeSaveReference
            }
            ShotGoalLineCrossingUnavailableReason::NoGoalLineCrossingBeforeSaveReference => {
                Self::NoGoalLineCrossingBeforeSaveReference
            }
            ShotGoalLineCrossingUnavailableReason::OnlyUnphysicalFreeFlightCrossings => {
                Self::OnlyUnphysicalFreeFlightCrossings
            }
            ShotGoalLineCrossingUnavailableReason::CrossingsBeforePredictionStart => {
                Self::CrossingsBeforePredictionStart
            }
            ShotGoalLineCrossingUnavailableReason::CrossingsBeforeSaveTouch => {
                Self::CrossingsBeforeSaveTouch
            }
            ShotGoalLineCrossingUnavailableReason::CrossingsBeforeSaveStat => {
                Self::CrossingsBeforeSaveStat
            }
            ShotGoalLineCrossingUnavailableReason::NoUsableProjection => Self::NoUsableProjection,
        }
    }
}

impl From<WireShotGoalLineCrossingUnavailableReason> for ShotGoalLineCrossingUnavailableReason {
    fn from(value: WireShotGoalLineCrossingUnavailableReason) -> Self {
        match value {
            WireShotGoalLineCrossingUnavailableReason::NoBallVelocity => Self::NoBallVelocity,
            WireShotGoalLineCrossingUnavailableReason::NoGoalwardBallBeforeSaveReference => {
                Self::NoGoalwardBallBeforeSaveReference
            }
            WireShotGoalLineCrossingUnavailableReason::NoGoalLineCrossingBeforeSaveReference => {
                Self::NoGoalLineCrossingBeforeSaveReference
            }
            WireShotGoalLineCrossingUnavailableReason::OnlyUnphysicalFreeFlightCrossings => {
                Self::OnlyUnphysicalFreeFlightCrossings
            }
            WireShotGoalLineCrossingUnavailableReason::CrossingsBeforePredictionStart => {
                Self::CrossingsBeforePredictionStart
            }
            WireShotGoalLineCrossingUnavailableReason::CrossingsBeforeSaveTouch => {
                Self::CrossingsBeforeSaveTouch
            }
            WireShotGoalLineCrossingUnavailableReason::CrossingsBeforeSaveStat => {
                Self::CrossingsBeforeSaveStat
            }
            WireShotGoalLineCrossingUnavailableReason::NoUsableProjection => {
                Self::NoUsableProjection
            }
        }
    }
}

/// Wire mirror of [`subtr_actor::ShotGoalTargetHitKind`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WireShotGoalTargetHitKind {
    GoalLine,
    BackWall,
    GoalFrame,
}

impl From<ShotGoalTargetHitKind> for WireShotGoalTargetHitKind {
    fn from(value: ShotGoalTargetHitKind) -> Self {
        match value {
            ShotGoalTargetHitKind::GoalLine => Self::GoalLine,
            ShotGoalTargetHitKind::BackWall => Self::BackWall,
            ShotGoalTargetHitKind::GoalFrame => Self::GoalFrame,
        }
    }
}

impl From<WireShotGoalTargetHitKind> for ShotGoalTargetHitKind {
    fn from(value: WireShotGoalTargetHitKind) -> Self {
        match value {
            WireShotGoalTargetHitKind::GoalLine => Self::GoalLine,
            WireShotGoalTargetHitKind::BackWall => Self::BackWall,
            WireShotGoalTargetHitKind::GoalFrame => Self::GoalFrame,
        }
    }
}

/// Wire mirror of [`subtr_actor::ShotGoalLineCrossing`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireShotGoalLineCrossing {
    pub time_after_shot: f32,
    pub prediction_start_time: Option<f32>,
    pub prediction_start_frame: Option<usize>,
    pub position: Vector3f,
    pub velocity: Option<Vector3f>,
    pub inside_goal_mouth: bool,
    pub prediction_kind: WireShotGoalLineCrossingPredictionKind,
}

impl From<ShotGoalLineCrossing> for WireShotGoalLineCrossing {
    fn from(value: ShotGoalLineCrossing) -> Self {
        Self {
            time_after_shot: value.time_after_shot,
            prediction_start_time: value.prediction_start_time,
            prediction_start_frame: value.prediction_start_frame,
            position: value.position,
            velocity: value.velocity,
            inside_goal_mouth: value.inside_goal_mouth,
            prediction_kind: value.prediction_kind.into(),
        }
    }
}

impl From<WireShotGoalLineCrossing> for ShotGoalLineCrossing {
    fn from(value: WireShotGoalLineCrossing) -> Self {
        Self {
            time_after_shot: value.time_after_shot,
            prediction_start_time: value.prediction_start_time,
            prediction_start_frame: value.prediction_start_frame,
            position: value.position,
            velocity: value.velocity,
            inside_goal_mouth: value.inside_goal_mouth,
            prediction_kind: value.prediction_kind.into(),
        }
    }
}

/// Wire mirror of [`subtr_actor::ShotGoalTargetHit`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireShotGoalTargetHit {
    pub time_after_shot: f32,
    pub prediction_start_time: Option<f32>,
    pub prediction_start_frame: Option<usize>,
    pub position: Vector3f,
    pub velocity: Option<Vector3f>,
    pub hit_kind: WireShotGoalTargetHitKind,
}

impl From<ShotGoalTargetHit> for WireShotGoalTargetHit {
    fn from(value: ShotGoalTargetHit) -> Self {
        Self {
            time_after_shot: value.time_after_shot,
            prediction_start_time: value.prediction_start_time,
            prediction_start_frame: value.prediction_start_frame,
            position: value.position,
            velocity: value.velocity,
            hit_kind: value.hit_kind.into(),
        }
    }
}

impl From<WireShotGoalTargetHit> for ShotGoalTargetHit {
    fn from(value: WireShotGoalTargetHit) -> Self {
        Self {
            time_after_shot: value.time_after_shot,
            prediction_start_time: value.prediction_start_time,
            prediction_start_frame: value.prediction_start_frame,
            position: value.position,
            velocity: value.velocity,
            hit_kind: value.hit_kind.into(),
        }
    }
}

/// Wire mirror of [`subtr_actor::ShotEventMetadata`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireShotEventMetadata {
    pub shot_touch_position: Vector3f,
    pub ball_position: Vector3f,
    pub ball_velocity: Option<Vector3f>,
    pub ball_speed: Option<f32>,
    pub player_position: Option<Vector3f>,
    pub player_velocity: Option<Vector3f>,
    pub player_speed: Option<f32>,
    pub player_distance_to_ball: Option<f32>,
    pub target_goal_position: Vector3f,
    pub distance_to_goal_center: f32,
    pub distance_to_goal_line: f32,
    pub ball_goal_alignment: Option<f32>,
    pub ball_speed_toward_goal: Option<f32>,
    pub projected_goal_line_crossing: Option<WireShotGoalLineCrossing>,
    pub projected_goal_line_crossing_unavailable_reason:
        Option<WireShotGoalLineCrossingUnavailableReason>,
    pub projected_goal_target_hit: Option<WireShotGoalTargetHit>,
    pub resulting_save: Option<WireShotSaveMetadata>,
}

impl From<ShotEventMetadata> for WireShotEventMetadata {
    fn from(value: ShotEventMetadata) -> Self {
        Self {
            shot_touch_position: value.shot_touch_position,
            ball_position: value.ball_position,
            ball_velocity: value.ball_velocity,
            ball_speed: value.ball_speed,
            player_position: value.player_position,
            player_velocity: value.player_velocity,
            player_speed: value.player_speed,
            player_distance_to_ball: value.player_distance_to_ball,
            target_goal_position: value.target_goal_position,
            distance_to_goal_center: value.distance_to_goal_center,
            distance_to_goal_line: value.distance_to_goal_line,
            ball_goal_alignment: value.ball_goal_alignment,
            ball_speed_toward_goal: value.ball_speed_toward_goal,
            projected_goal_line_crossing: value.projected_goal_line_crossing.map(Into::into),
            projected_goal_line_crossing_unavailable_reason: value
                .projected_goal_line_crossing_unavailable_reason
                .map(Into::into),
            projected_goal_target_hit: value.projected_goal_target_hit.map(Into::into),
            resulting_save: value.resulting_save.map(Into::into),
        }
    }
}

impl From<WireShotEventMetadata> for ShotEventMetadata {
    fn from(value: WireShotEventMetadata) -> Self {
        Self {
            shot_touch_position: value.shot_touch_position,
            ball_position: value.ball_position,
            ball_velocity: value.ball_velocity,
            ball_speed: value.ball_speed,
            player_position: value.player_position,
            player_velocity: value.player_velocity,
            player_speed: value.player_speed,
            player_distance_to_ball: value.player_distance_to_ball,
            target_goal_position: value.target_goal_position,
            distance_to_goal_center: value.distance_to_goal_center,
            distance_to_goal_line: value.distance_to_goal_line,
            ball_goal_alignment: value.ball_goal_alignment,
            ball_speed_toward_goal: value.ball_speed_toward_goal,
            projected_goal_line_crossing: value.projected_goal_line_crossing.map(Into::into),
            projected_goal_line_crossing_unavailable_reason: value
                .projected_goal_line_crossing_unavailable_reason
                .map(Into::into),
            projected_goal_target_hit: value.projected_goal_target_hit.map(Into::into),
            resulting_save: value.resulting_save.map(Into::into),
        }
    }
}

/// Wire mirror of [`subtr_actor::PlayerStatEvent`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WirePlayerStatEvent {
    pub time: f32,
    pub frame: usize,
    pub player: PlayerId,
    pub player_position: Option<Vector3f>,
    pub is_team_0: bool,
    pub kind: WirePlayerStatEventKind,
    pub shot: Option<WireShotEventMetadata>,
}

impl From<PlayerStatEvent> for WirePlayerStatEvent {
    fn from(value: PlayerStatEvent) -> Self {
        Self {
            time: value.time,
            frame: value.frame,
            player: value.player,
            player_position: value.player_position,
            is_team_0: value.is_team_0,
            kind: value.kind.into(),
            shot: value.shot.map(Into::into),
        }
    }
}

impl From<WirePlayerStatEvent> for PlayerStatEvent {
    fn from(value: WirePlayerStatEvent) -> Self {
        Self {
            time: value.time,
            frame: value.frame,
            player: value.player,
            player_position: value.player_position,
            is_team_0: value.is_team_0,
            kind: value.kind.into(),
            shot: value.shot.map(Into::into),
        }
    }
}

fn into_vec<T, U: From<T>>(values: Vec<T>) -> Vec<U> {
    values.into_iter().map(Into::into).collect()
}

/// Wire mirror of [`subtr_actor::FrameEventsState`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WireFrameEventsState {
    pub active_demos: Vec<WireDemoEventSample>,
    pub demo_events: Vec<WireDemolishInfo>,
    pub boost_pad_events: Vec<WireBoostPadEvent>,
    pub touch_events: Vec<WireTouchEvent>,
    pub dodge_refreshed_counter_available: bool,
    pub dodge_refreshed_events: Vec<WireDodgeRefreshedEvent>,
    pub player_stat_events: Vec<WirePlayerStatEvent>,
    pub goal_events: Vec<WireGoalEvent>,
}

impl WireFrameEventsState {
    /// Whether this frame carries any discrete derived events.
    ///
    /// `active_demos` is continuous per-frame state (present on every frame
    /// while a player is demolished), not a discrete event, so it is ignored
    /// here; frame-rate downsampling uses this to decide which frames can be
    /// skipped without losing events.
    pub fn has_discrete_events(&self) -> bool {
        !self.demo_events.is_empty()
            || !self.boost_pad_events.is_empty()
            || !self.touch_events.is_empty()
            || !self.dodge_refreshed_events.is_empty()
            || !self.player_stat_events.is_empty()
            || !self.goal_events.is_empty()
    }
}

impl From<FrameEventsState> for WireFrameEventsState {
    fn from(value: FrameEventsState) -> Self {
        Self {
            active_demos: into_vec(value.active_demos),
            demo_events: into_vec(value.demo_events),
            boost_pad_events: into_vec(value.boost_pad_events),
            touch_events: into_vec(value.touch_events),
            dodge_refreshed_counter_available: value.dodge_refreshed_counter_available,
            dodge_refreshed_events: into_vec(value.dodge_refreshed_events),
            player_stat_events: into_vec(value.player_stat_events),
            goal_events: into_vec(value.goal_events),
        }
    }
}

impl From<WireFrameEventsState> for FrameEventsState {
    fn from(value: WireFrameEventsState) -> Self {
        Self {
            active_demos: into_vec(value.active_demos),
            demo_events: into_vec(value.demo_events),
            boost_pad_events: into_vec(value.boost_pad_events),
            touch_events: into_vec(value.touch_events),
            dodge_refreshed_counter_available: value.dodge_refreshed_counter_available,
            dodge_refreshed_events: into_vec(value.dodge_refreshed_events),
            player_stat_events: into_vec(value.player_stat_events),
            goal_events: into_vec(value.goal_events),
        }
    }
}

/// Wire mirror of [`LiveEventHistory`], the cumulative event history sent to
/// mid-match joiners.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WireEventHistory {
    pub demo_events: Vec<WireDemolishInfo>,
    pub boost_pad_events: Vec<WireBoostPadEvent>,
    pub touch_events: Vec<WireTouchEvent>,
    pub dodge_refreshed_events: Vec<WireDodgeRefreshedEvent>,
    pub player_stat_events: Vec<WirePlayerStatEvent>,
    pub goal_events: Vec<WireGoalEvent>,
}

impl WireEventHistory {
    /// Appends one frame's derived events, mirroring
    /// [`LiveEventHistory::append_frame_events`].
    pub fn append_frame_events(&mut self, events: &WireFrameEventsState) {
        self.demo_events.extend(events.demo_events.iter().cloned());
        self.boost_pad_events
            .extend(events.boost_pad_events.iter().cloned());
        self.touch_events
            .extend(events.touch_events.iter().cloned());
        self.dodge_refreshed_events
            .extend(events.dodge_refreshed_events.iter().cloned());
        self.player_stat_events
            .extend(events.player_stat_events.iter().cloned());
        self.goal_events.extend(events.goal_events.iter().cloned());
    }
}

impl From<LiveEventHistory> for WireEventHistory {
    fn from(value: LiveEventHistory) -> Self {
        Self {
            demo_events: into_vec(value.demo_events),
            boost_pad_events: into_vec(value.boost_pad_events),
            touch_events: into_vec(value.touch_events),
            dodge_refreshed_events: into_vec(value.dodge_refreshed_events),
            player_stat_events: into_vec(value.player_stat_events),
            goal_events: into_vec(value.goal_events),
        }
    }
}

impl From<WireEventHistory> for LiveEventHistory {
    fn from(value: WireEventHistory) -> Self {
        Self {
            demo_events: into_vec(value.demo_events),
            boost_pad_events: into_vec(value.boost_pad_events),
            touch_events: into_vec(value.touch_events),
            dodge_refreshed_events: into_vec(value.dodge_refreshed_events),
            player_stat_events: into_vec(value.player_stat_events),
            goal_events: into_vec(value.goal_events),
        }
    }
}
