use crate::*;
use boxcars::Attribute;
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

    #[error("The attribute value that was found was not of the expected type {expected_type:?} {actual_type:?}")]
    UnexpectedAttributeType {
        expected_type: String,
        actual_type: String,
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

pub fn attribute_to_tag(attribute: &Attribute) -> &str {
    match attribute {
        Attribute::Boolean(_) => "AttributeTag::Boolean",
        Attribute::Byte(_) => "AttributeTag::Byte",
        Attribute::AppliedDamage(_) => "AttributeTag::AppliedDamage",
        Attribute::DamageState(_) => "AttributeTag::DamageState",
        Attribute::CamSettings(_) => "AttributeTag::CamSettings",
        Attribute::ClubColors(_) => "AttributeTag::ClubColors",
        Attribute::Demolish(_) => "AttributeTag::Demolish",
        Attribute::DemolishFx(_) => "AttributeTag::DemolishFx",
        Attribute::Enum(_) => "AttributeTag::Enum",
        Attribute::Explosion(_) => "AttributeTag::Explosion",
        Attribute::ExtendedExplosion(_) => "AttributeTag::ExtendedExplosion",
        Attribute::FlaggedByte(_, _) => "AttributeTag::FlaggedByte",
        Attribute::ActiveActor(_) => "AttributeTag::ActiveActor",
        Attribute::Float(_) => "AttributeTag::Float",
        Attribute::GameMode(_, _) => "AttributeTag::GameMode",
        Attribute::Int(_) => "AttributeTag::Int",
        Attribute::Int64(_) => "AttributeTag::Int64",
        Attribute::Loadout(_) => "AttributeTag::Loadout",
        Attribute::TeamLoadout(_) => "AttributeTag::TeamLoadout",
        Attribute::Location(_) => "AttributeTag::Location",
        Attribute::MusicStinger(_) => "AttributeTag::MusicStinger",
        Attribute::Pickup(_) => "AttributeTag::Pickup",
        Attribute::PickupNew(_) => "AttributeTag::PickupNew",
        Attribute::PlayerHistoryKey(_) => "AttributeTag::PlayerHistoryKey",
        Attribute::Welded(_) => "AttributeTag::Welded",
        Attribute::RigidBody(_) => "AttributeTag::RigidBody",
        Attribute::Title(_, _, _, _, _, _, _, _) => "AttributeTag::Title",
        Attribute::TeamPaint(_) => "AttributeTag::TeamPaint",
        Attribute::String(_) => "AttributeTag::String",
        Attribute::UniqueId(_) => "AttributeTag::UniqueId",
        Attribute::Reservation(_) => "AttributeTag::Reservation",
        Attribute::PartyLeader(_) => "AttributeTag::PartyLeader",
        Attribute::LoadoutOnline(_) => "AttributeTag::LoadoutOnline",
        Attribute::LoadoutsOnline(_) => "AttributeTag::LoadoutsOnline",
        Attribute::StatEvent(_) => "AttributeTag::StatEvent",
        Attribute::RepStatTitle(_) => "AttributeTag::RepStatTitle",
        Attribute::PickupInfo(_) => "AttributeTag::PickupInfo",
        Attribute::Impulse(_) => "AttributeTag::Impulse",
        Attribute::QWord(_) => "AttributeTag::QWordString",
        Attribute::PrivateMatch(_) => "AttributeTag::PrivateMatchSettings",
        Attribute::Rotation(_) => "AttributeTag::RotationTag",
        Attribute::DemolishExtended(_) => "AttributeTag::DemolishExtended",
        Attribute::ReplicatedBoost(_) => "AttributeTag::ReplicatedBoost",
        Attribute::LogoData(_) => "AttributeTag::LogoData",
    }
}
