use crate::*;
use std::backtrace::Backtrace;
use thiserror::Error;

/// [`SubtrActorErrorVariant`] is an enumeration of all the specific error
/// variants that can occur while processing game replays in the subtr-actor
/// domain. These include errors related to network frames, frame indexing,
/// player sets, actor states, object ids, team identities, and data types
/// amongst others.
#[derive(Error, Debug, Clone)]
pub enum SubtrActorErrorVariant {
    #[error("Replay has no network frames")]
    NoNetworkFrames,

    #[error("Frame index out of bounds")]
    FrameIndexOutOfBounds,

    #[error("Players found in frames that were not part of original set. Found: {found:?}, Original: {original:?}")]
    InconsistentPlayerSet {
        found: std::collections::HashSet<PlayerId>,
        original: std::collections::HashSet<PlayerId>,
    },

    #[error(
        "No update for ActorId {actor_id:?} of ObjectId {object_id:?} after frame {frame_index}"
    )]
    NoUpdateAfterFrame {
        actor_id: boxcars::ActorId,
        object_id: boxcars::ObjectId,
        frame_index: usize,
    },

    #[error("No boost amount value.")]
    NoBoostAmountValue,

    #[error("The attribute value that was found was not of the expected type {expected_type} {actual_type:?}")]
    UnexpectedAttributeType {
        expected_type: &'static str,
        actual_type: boxcars::AttributeTag,
    },

    #[error("ActorId {actor_id:?} has no matching player id")]
    NoMatchingPlayerId { actor_id: boxcars::ActorId },

    #[error("No game actor")]
    NoGameActor,

    #[error("ActorId {actor_id:} already exists with object_id {object_id:}")]
    ActorIdAlreadyExists {
        actor_id: boxcars::ActorId,
        object_id: boxcars::ObjectId,
    },

    #[error("{name:?} actor for player {player_id:?} not found")]
    ActorNotFound {
        name: &'static str,
        player_id: PlayerId,
    },

    #[error("There was no actor state for actor_id: {actor_id:?}")]
    NoStateForActorId { actor_id: boxcars::ActorId },

    #[error("Couldn't find object id for {name}")]
    ObjectIdNotFound { name: &'static str },

    #[error("No value found for derived key {name:?}")]
    DerivedKeyValueNotFound { name: String },

    #[error("Ball actor not found")]
    BallActorNotFound,

    #[error("Player team unknown, {player_id:?}")]
    UnknownPlayerTeam { player_id: PlayerId },

    #[error("Team object id not known {object_id:?}, for player {player_id:?}")]
    UnknownTeamObjectId {
        object_id: boxcars::ObjectId,
        player_id: PlayerId,
    },

    #[error("Team name was empty for {player_id:?}")]
    EmptyTeamName { player_id: PlayerId },

    #[error("Error returned to deliberately end processing early")]
    FinishProcessingEarly,

    #[error("Player stats header not found")]
    PlayerStatsHeaderNotFound,

    #[error("Interpolation time order was incorrect start_time {start_time:} {time:} {end_time:}")]
    InterpolationTimeOrderError {
        start_time: f32,
        time: f32,
        end_time: f32,
    },

    #[error("The updated actor id does not exist {update:?}")]
    UpdatedActorIdDoesNotExist { update: boxcars::UpdatedAttribute },

    #[error("Could not find {property:} in state")]
    PropertyNotFoundInState { property: &'static str },

    #[error("Could not build replay meta")]
    CouldNotBuildReplayMeta,

    #[error("Error converting float")]
    FloatConversionError,

    #[error(transparent)]
    NDArrayShapeError(#[from] ::ndarray::ShapeError),

    #[error("{0:?} was not a recognized feature adder")]
    UnknownFeatureAdderName(String),

    #[error("Callback error: {0}")]
    CallbackError(String),

    #[error("Unknown builtin stats module '{0}'")]
    UnknownStatsModuleName(String),

    #[error("Stats serialization error: {0}")]
    StatsSerializationError(String),
}

/// [`SubtrActorError`] struct provides an error variant
/// [`SubtrActorErrorVariant`] along with its backtrace.
#[derive(Debug)]
pub struct SubtrActorError {
    pub backtrace: Backtrace,
    pub variant: SubtrActorErrorVariant,
}

impl SubtrActorError {
    pub fn new(variant: SubtrActorErrorVariant) -> Self {
        Self {
            backtrace: Backtrace::capture(),
            variant,
        }
    }

    pub fn new_result<T>(variant: SubtrActorErrorVariant) -> Result<T, Self> {
        Err(Self::new(variant))
    }
}

#[allow(clippy::result_large_err)]
pub type SubtrActorResult<T> = Result<T, SubtrActorError>;
