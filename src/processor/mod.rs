use crate::*;
use boxcars;
use std::collections::HashMap;

pub mod actor_state;
pub use actor_state::*;

pub(crate) fn attribute_type_name(attribute: &boxcars::Attribute) -> &'static str {
    match attribute {
        boxcars::Attribute::Boolean(_) => "Boolean",
        boxcars::Attribute::Byte(_) => "Byte",
        boxcars::Attribute::AppliedDamage(_) => "AppliedDamage",
        boxcars::Attribute::DamageState(_) => "DamageState",
        boxcars::Attribute::CamSettings(_) => "CamSettings",
        boxcars::Attribute::ClubColors(_) => "ClubColors",
        boxcars::Attribute::Demolish(_) => "Demolish",
        boxcars::Attribute::DemolishExtended(_) => "DemolishExtended",
        boxcars::Attribute::DemolishFx(_) => "DemolishFx",
        boxcars::Attribute::Enum(_) => "Enum",
        boxcars::Attribute::Explosion(_) => "Explosion",
        boxcars::Attribute::ExtendedExplosion(_) => "ExtendedExplosion",
        boxcars::Attribute::FlaggedByte(_, _) => "FlaggedByte",
        boxcars::Attribute::ActiveActor(_) => "ActiveActor",
        boxcars::Attribute::Float(_) => "Float",
        boxcars::Attribute::GameMode(_, _) => "GameMode",
        boxcars::Attribute::Int(_) => "Int",
        boxcars::Attribute::Int64(_) => "Int64",
        boxcars::Attribute::Loadout(_) => "Loadout",
        boxcars::Attribute::TeamLoadout(_) => "TeamLoadout",
        boxcars::Attribute::Location(_) => "Location",
        boxcars::Attribute::MusicStinger(_) => "MusicStinger",
        boxcars::Attribute::PlayerHistoryKey(_) => "PlayerHistoryKey",
        boxcars::Attribute::Pickup(_) => "Pickup",
        boxcars::Attribute::PickupNew(_) => "PickupNew",
        boxcars::Attribute::QWord(_) => "QWord",
        boxcars::Attribute::Welded(_) => "Welded",
        boxcars::Attribute::Title(_, _, _, _, _, _, _, _) => "Title",
        boxcars::Attribute::TeamPaint(_) => "TeamPaint",
        boxcars::Attribute::RigidBody(_) => "RigidBody",
        boxcars::Attribute::String(_) => "String",
        boxcars::Attribute::UniqueId(_) => "UniqueId",
        boxcars::Attribute::Reservation(_) => "Reservation",
        boxcars::Attribute::PartyLeader(_) => "PartyLeader",
        boxcars::Attribute::PrivateMatch(_) => "PrivateMatch",
        boxcars::Attribute::LoadoutOnline(_) => "LoadoutOnline",
        boxcars::Attribute::LoadoutsOnline(_) => "LoadoutsOnline",
        boxcars::Attribute::StatEvent(_) => "StatEvent",
        boxcars::Attribute::Rotation(_) => "Rotation",
        boxcars::Attribute::RepStatTitle(_) => "RepStatTitle",
        boxcars::Attribute::PickupInfo(_) => "PickupInfo",
        boxcars::Attribute::Impulse(_) => "Impulse",
        boxcars::Attribute::ReplicatedBoost(_) => "ReplicatedBoost",
        boxcars::Attribute::LogoData(_) => "LogoData",
    }
}

/// Attempts to match an attribute value with the given type.
///
/// # Arguments
///
/// * `$value` - An expression that yields the attribute value.
/// * `$type` - The expected enum path.
///
/// If the attribute matches the specified type, it is returned wrapped in an
/// [`Ok`] variant of a [`Result`]. If the attribute doesn't match, it results in an
/// [`Err`] variant with a [`SubtrActorError`], specifying the expected type and
/// the actual type.
#[macro_export]
macro_rules! attribute_match {
    ($value:expr, $type:path $(,)?) => {{
        let attribute = $value;
        if let $type(value) = attribute {
            Ok(value)
        } else {
            SubtrActorError::new_result(SubtrActorErrorVariant::UnexpectedAttributeType {
                expected_type: stringify!($type),
                actual_type: attribute_type_name(&attribute),
            })
        }
    }};
}

/// Obtains an attribute from a map and ensures it matches the expected type.
///
/// # Arguments
///
/// * `$self` - The struct or instance on which the function is invoked.
/// * `$map` - The data map.
/// * `$prop` - The attribute key.
/// * `$type` - The expected enum path.
#[macro_export]
macro_rules! get_attribute_errors_expected {
    ($self:ident, $map:expr, $prop:expr, $type:path) => {
        $self
            .get_attribute($map, $prop)
            .and_then(|found| attribute_match!(found, $type))
    };
}

/// Obtains an attribute and its updated status from a map and ensures the
/// attribute matches the expected type.
///
/// # Arguments
///
/// * `$self` - The struct or instance on which the function is invoked.
/// * `$map` - The data map.
/// * `$prop` - The attribute key.
/// * `$type` - The expected enum path.
///
/// It returns a [`Result`] with a tuple of the matched attribute and its updated
/// status, after invoking [`attribute_match!`] on the found attribute.
macro_rules! get_attribute_and_updated {
    ($self:ident, $map:expr, $prop:expr, $type:path) => {
        $self
            .get_attribute_and_updated($map, $prop)
            .and_then(|(found, updated)| attribute_match!(found, $type).map(|v| (v, updated)))
    };
}

/// Obtains an actor attribute and ensures it matches the expected type.
///
/// # Arguments
///
/// * `$self` - The struct or instance on which the function is invoked.
/// * `$actor` - The actor identifier.
/// * `$prop` - The attribute key.
/// * `$type` - The expected enum path.
macro_rules! get_actor_attribute_matching {
    ($self:ident, $actor:expr, $prop:expr, $type:path) => {
        $self
            .get_actor_attribute($actor, $prop)
            .and_then(|found| attribute_match!(found, $type))
    };
}

/// Obtains a derived attribute from a map and ensures it matches the expected
/// type.
///
/// # Arguments
///
/// * `$map` - The data map.
/// * `$key` - The attribute key.
/// * `$type` - The expected enum path.
macro_rules! get_derived_attribute {
    ($map:expr, $key:expr, $type:path) => {
        $map.get($key)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::DerivedKeyValueNotFound {
                    name: $key.to_string(),
                })
            })
            .and_then(|found| attribute_match!(&found.0, $type))
    };
}

fn get_actor_id_from_active_actor<T>(
    _: T,
    active_actor: &boxcars::ActiveActor,
) -> boxcars::ActorId {
    active_actor.actor
}

fn use_update_actor<T>(id: boxcars::ActorId, _: T) -> boxcars::ActorId {
    id
}

#[derive(Clone, Copy, Default)]
struct CachedObjectIds {
    player_type: Option<boxcars::ObjectId>,
    car_type: Option<boxcars::ObjectId>,
    boost_type: Option<boxcars::ObjectId>,
    dodge_type: Option<boxcars::ObjectId>,
    jump_type: Option<boxcars::ObjectId>,
    double_jump_type: Option<boxcars::ObjectId>,
    unique_id: Option<boxcars::ObjectId>,
    team: Option<boxcars::ObjectId>,
    player_replication: Option<boxcars::ObjectId>,
    vehicle: Option<boxcars::ObjectId>,
    boost_replicated: Option<boxcars::ObjectId>,
    boost_amount: Option<boxcars::ObjectId>,
    component_active: Option<boxcars::ObjectId>,
    seconds_remaining: Option<boxcars::ObjectId>,
    replicated_state_name: Option<boxcars::ObjectId>,
    replicated_game_state_time_remaining: Option<boxcars::ObjectId>,
    ball_has_been_hit: Option<boxcars::ObjectId>,
    ball_hit_team_num: Option<boxcars::ObjectId>,
    dodges_refreshed_counter: Option<boxcars::ObjectId>,
}

impl CachedObjectIds {
    fn from_name_map(name_to_object_id: &HashMap<String, boxcars::ObjectId>) -> Self {
        let cached = |name| name_to_object_id.get(name).copied();
        Self {
            player_type: cached(PLAYER_TYPE),
            car_type: cached(CAR_TYPE),
            boost_type: cached(BOOST_TYPE),
            dodge_type: cached(DODGE_TYPE),
            jump_type: cached(JUMP_TYPE),
            double_jump_type: cached(DOUBLE_JUMP_TYPE),
            unique_id: cached(UNIQUE_ID_KEY),
            team: cached(TEAM_KEY),
            player_replication: cached(PLAYER_REPLICATION_KEY),
            vehicle: cached(VEHICLE_KEY),
            boost_replicated: cached(BOOST_REPLICATED_KEY),
            boost_amount: cached(BOOST_AMOUNT_KEY),
            component_active: cached(COMPONENT_ACTIVE_KEY),
            seconds_remaining: cached(SECONDS_REMAINING_KEY),
            replicated_state_name: cached(REPLICATED_STATE_NAME_KEY),
            replicated_game_state_time_remaining: cached(REPLICATED_GAME_STATE_TIME_REMAINING_KEY),
            ball_has_been_hit: cached(BALL_HAS_BEEN_HIT_KEY),
            ball_hit_team_num: cached(BALL_HIT_TEAM_NUM_KEY),
            dodges_refreshed_counter: cached(DODGES_REFRESHED_COUNTER_KEY),
        }
    }
}

mod bootstrap;
mod debug;
mod queries;
mod updaters;

/// The [`ReplayProcessor`] struct is a pivotal component in `subtr-actor`'s
/// replay parsing pipeline. It is designed to process and traverse an actor
/// graph of a Rocket League replay, and expose methods for collectors to gather
/// specific data points as it progresses through the replay.
///
/// The processor pushes frames from a replay through an [`ActorStateModeler`],
/// which models the state all actors in the replay at a given point in time.
/// The [`ReplayProcessor`] also maintains various mappings to allow efficient
/// lookup and traversal of the actor graph, thus assisting [`Collector`]
/// instances in their data accumulation tasks.
///
/// The primary method of this struct is [`process`](ReplayProcessor::process),
/// which takes a collector and processes the replay. As it traverses the
/// replay, it calls the [`Collector::process_frame`] method of the passed
/// collector, passing the current frame along with its contextual data. This
/// allows the collector to extract specific data from each frame as needed.
///
/// The [`ReplayProcessor`] also provides a number of helper methods for
/// navigating the actor graph and extracting information, such as
/// [`get_ball_rigid_body`](ReplayProcessor::get_ball_rigid_body),
/// [`get_player_name`](ReplayProcessor::get_player_name),
/// [`get_player_team_key`](ReplayProcessor::get_player_team_key),
/// [`get_player_is_team_0`](ReplayProcessor::get_player_is_team_0), and
/// [`get_player_rigid_body`](ReplayProcessor::get_player_rigid_body).
///
/// # See Also
///
/// * [`ActorStateModeler`]: A struct used to model the states of multiple
///   actors at a given point in time.
/// * [`Collector`]: A trait implemented by objects that wish to collect data as
///   the `ReplayProcessor` processes a replay.
pub struct ReplayProcessor<'a> {
    /// The replay currently being traversed.
    pub replay: &'a boxcars::Replay,
    spatial_normalization_factor: f32,
    rigid_body_velocity_normalization_factor: f32,
    uses_legacy_rigid_body_rotation: bool,
    cached_object_ids: CachedObjectIds,
    is_boost_pad_object: Vec<bool>,
    /// Modeled actor state for the current replay frame.
    pub actor_state: ActorStateModeler,
    /// Mapping from object ids to their replay object names.
    pub object_id_to_name: HashMap<boxcars::ObjectId, String>,
    /// Reverse lookup from replay object names to object ids.
    pub name_to_object_id: HashMap<String, boxcars::ObjectId>,
    /// Cached actor id for the replay ball when known.
    pub ball_actor_id: Option<boxcars::ActorId>,
    /// Stable ordering of team 0 players.
    pub team_zero: Vec<PlayerId>,
    /// Stable ordering of team 1 players.
    pub team_one: Vec<PlayerId>,
    /// Mapping from player ids to their player-controller actor ids.
    pub player_to_actor_id: HashMap<PlayerId, boxcars::ActorId>,
    /// Mapping from player-controller actors to car actors.
    pub player_to_car: HashMap<boxcars::ActorId, boxcars::ActorId>,
    /// Mapping from player-controller actors to team actors.
    pub player_to_team: HashMap<boxcars::ActorId, boxcars::ActorId>,
    /// Reverse mapping from car actors to player-controller actors.
    pub car_to_player: HashMap<boxcars::ActorId, boxcars::ActorId>,
    /// Mapping from car actors to boost component actors.
    pub car_to_boost: HashMap<boxcars::ActorId, boxcars::ActorId>,
    /// Mapping from car actors to jump component actors.
    pub car_to_jump: HashMap<boxcars::ActorId, boxcars::ActorId>,
    /// Mapping from car actors to double-jump component actors.
    pub car_to_double_jump: HashMap<boxcars::ActorId, boxcars::ActorId>,
    /// Mapping from car actors to dodge component actors.
    pub car_to_dodge: HashMap<boxcars::ActorId, boxcars::ActorId>,
    /// All boost-pad events observed so far in the replay.
    pub boost_pad_events: Vec<BoostPadEvent>,
    current_frame_boost_pad_events: Vec<BoostPadEvent>,
    /// All touch events observed so far in the replay.
    pub touch_events: Vec<TouchEvent>,
    current_frame_touch_events: Vec<TouchEvent>,
    /// All dodge-refresh events observed so far in the replay.
    pub dodge_refreshed_events: Vec<DodgeRefreshedEvent>,
    current_frame_dodge_refreshed_events: Vec<DodgeRefreshedEvent>,
    dodge_refreshed_counters: HashMap<PlayerId, i32>,
    /// All goal events observed so far in the replay.
    pub goal_events: Vec<GoalEvent>,
    current_frame_goal_events: Vec<GoalEvent>,
    /// All shot/save/assist-style stat events observed so far in the replay.
    pub player_stat_events: Vec<PlayerStatEvent>,
    current_frame_player_stat_events: Vec<PlayerStatEvent>,
    player_stat_counters: HashMap<(PlayerId, PlayerStatEventKind), i32>,
    /// All demolishes observed so far in the replay.
    pub demolishes: Vec<DemolishInfo>,
    known_demolishes: Vec<(DemolishAttribute, usize)>,
    demolish_format: Option<DemolishFormat>,
    kickoff_phase_active_last_frame: bool,
}

impl<'a> ReplayProcessor<'a> {
    const LEGACY_RIGID_BODY_NET_VERSION_CUTOFF: i32 = 5;
    const LEGACY_RIGID_BODY_ROTATION_NET_VERSION_CUTOFF: i32 = 7;
    const LEGACY_RIGID_BODY_LOCATION_FACTOR: f32 = 100.0;
    const LEGACY_RIGID_BODY_VELOCITY_FACTOR: f32 = 10.0;

    fn uses_legacy_rigid_body_vector_scale(net_version: Option<i32>) -> bool {
        net_version.is_none_or(|version| version < Self::LEGACY_RIGID_BODY_NET_VERSION_CUTOFF)
    }

    fn uses_legacy_rigid_body_rotation_for_net_version(net_version: Option<i32>) -> bool {
        net_version
            .is_none_or(|version| version < Self::LEGACY_RIGID_BODY_ROTATION_NET_VERSION_CUTOFF)
    }

    fn rigid_body_location_normalization_factor_for_net_version(net_version: Option<i32>) -> f32 {
        if Self::uses_legacy_rigid_body_vector_scale(net_version) {
            Self::LEGACY_RIGID_BODY_LOCATION_FACTOR
        } else {
            1.0
        }
    }

    fn rigid_body_velocity_normalization_factor_for_net_version(net_version: Option<i32>) -> f32 {
        if Self::uses_legacy_rigid_body_vector_scale(net_version) {
            Self::LEGACY_RIGID_BODY_VELOCITY_FACTOR
        } else {
            1.0
        }
    }

    /// Constructs a new [`ReplayProcessor`] instance with the provided replay.
    ///
    /// # Arguments
    ///
    /// * `replay` - A reference to the [`boxcars::Replay`] to be processed.
    ///
    /// # Returns
    ///
    /// Returns a [`SubtrActorResult`] of [`ReplayProcessor`]. In the process of
    /// initialization, the [`ReplayProcessor`]: - Maps each object id in the
    /// replay to its corresponding name. - Initializes empty state and
    /// attribute maps. - Sets the player order from either replay headers or
    /// frames, if available.
    pub fn new(replay: &'a boxcars::Replay) -> SubtrActorResult<Self> {
        let mut object_id_to_name = HashMap::new();
        let mut name_to_object_id = HashMap::new();
        let spatial_normalization_factor =
            Self::rigid_body_location_normalization_factor_for_net_version(replay.net_version);
        let rigid_body_velocity_normalization_factor =
            Self::rigid_body_velocity_normalization_factor_for_net_version(replay.net_version);
        let uses_legacy_rigid_body_rotation =
            Self::uses_legacy_rigid_body_rotation_for_net_version(replay.net_version);
        for (id, name) in replay.objects.iter().enumerate() {
            let object_id = boxcars::ObjectId(id as i32);
            object_id_to_name.insert(object_id, name.clone());
            name_to_object_id.insert(name.clone(), object_id);
        }
        let cached_object_ids = CachedObjectIds::from_name_map(&name_to_object_id);
        let mut processor = Self {
            actor_state: ActorStateModeler::new(),
            replay,
            spatial_normalization_factor,
            rigid_body_velocity_normalization_factor,
            uses_legacy_rigid_body_rotation,
            cached_object_ids,
            is_boost_pad_object: replay
                .objects
                .iter()
                .map(|name| name.contains("VehiclePickup_Boost_TA"))
                .collect(),
            object_id_to_name,
            name_to_object_id,
            team_zero: Vec::new(),
            team_one: Vec::new(),
            ball_actor_id: None,
            player_to_car: HashMap::new(),
            player_to_team: HashMap::new(),
            player_to_actor_id: HashMap::new(),
            car_to_player: HashMap::new(),
            car_to_boost: HashMap::new(),
            car_to_jump: HashMap::new(),
            car_to_double_jump: HashMap::new(),
            car_to_dodge: HashMap::new(),
            boost_pad_events: Vec::new(),
            current_frame_boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            current_frame_touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            current_frame_dodge_refreshed_events: Vec::new(),
            dodge_refreshed_counters: HashMap::new(),
            goal_events: Vec::new(),
            current_frame_goal_events: Vec::new(),
            player_stat_events: Vec::new(),
            current_frame_player_stat_events: Vec::new(),
            player_stat_counters: HashMap::new(),
            demolishes: Vec::new(),
            known_demolishes: Vec::new(),
            demolish_format: None,
            kickoff_phase_active_last_frame: false,
        };
        processor
            .set_player_order_from_headers()
            .or_else(|_| processor.set_player_order_from_frames())?;

        Ok(processor)
    }

    /// Returns the scale factor applied when normalizing replay spatial values.
    pub fn spatial_normalization_factor(&self) -> f32 {
        self.spatial_normalization_factor
    }

    /// Returns the scale factor applied when normalizing rigid-body linear and angular velocity.
    pub fn rigid_body_velocity_normalization_factor(&self) -> f32 {
        self.rigid_body_velocity_normalization_factor
    }

    fn normalize_vector_by_factor(
        &self,
        vector: boxcars::Vector3f,
        factor: f32,
    ) -> boxcars::Vector3f {
        if (factor - 1.0).abs() < f32::EPSILON {
            vector
        } else {
            boxcars::Vector3f {
                x: vector.x * factor,
                y: vector.y * factor,
                z: vector.z * factor,
            }
        }
    }

    fn normalize_vector(&self, vector: boxcars::Vector3f) -> boxcars::Vector3f {
        self.normalize_vector_by_factor(vector, self.spatial_normalization_factor)
    }

    fn normalize_rigid_body_velocity(&self, vector: boxcars::Vector3f) -> boxcars::Vector3f {
        self.normalize_vector_by_factor(vector, self.rigid_body_velocity_normalization_factor)
    }

    fn normalize_optional_rigid_body_velocity(
        &self,
        vector: Option<boxcars::Vector3f>,
    ) -> Option<boxcars::Vector3f> {
        vector.map(|value| self.normalize_rigid_body_velocity(value))
    }

    fn normalize_rigid_body_rotation(&self, rotation: boxcars::Quaternion) -> boxcars::Quaternion {
        if !self.uses_legacy_rigid_body_rotation {
            return rotation;
        }

        // Older replays store rigid-body rotation as fixed compressed
        // (pitch, yaw, roll), not as the modern quaternion shape.
        let normalized = glam::Quat::from_euler(
            glam::EulerRot::ZYX,
            rotation.y * std::f32::consts::PI,
            rotation.x * std::f32::consts::PI,
            rotation.z * std::f32::consts::PI,
        );
        boxcars::Quaternion {
            x: normalized.x,
            y: normalized.y,
            z: normalized.z,
            w: normalized.w,
        }
    }

    fn normalize_rigid_body(&self, rigid_body: &boxcars::RigidBody) -> boxcars::RigidBody {
        if (self.spatial_normalization_factor - 1.0).abs() < f32::EPSILON
            && (self.rigid_body_velocity_normalization_factor - 1.0).abs() < f32::EPSILON
            && !self.uses_legacy_rigid_body_rotation
        {
            *rigid_body
        } else {
            boxcars::RigidBody {
                sleeping: rigid_body.sleeping,
                location: self.normalize_vector(rigid_body.location),
                rotation: self.normalize_rigid_body_rotation(rigid_body.rotation),
                linear_velocity: self
                    .normalize_optional_rigid_body_velocity(rigid_body.linear_velocity),
                angular_velocity: self
                    .normalize_optional_rigid_body_velocity(rigid_body.angular_velocity),
            }
        }
    }

    fn required_cached_object_id(
        &self,
        object_id: Option<boxcars::ObjectId>,
        name: &'static str,
    ) -> SubtrActorResult<boxcars::ObjectId> {
        object_id
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::ObjectIdNotFound { name }))
    }

    /// [`Self::process`] takes a [`Collector`] as an argument and iterates over
    /// each frame in the replay, updating the internal state of the processor
    /// and other relevant mappings based on the current frame.
    ///
    /// After each a frame is processed, [`Collector::process_frame`] of the
    /// collector is called. The [`TimeAdvance`] return value of this call into
    /// [`Collector::process_frame`] is used to determine what happens next: in
    /// the case of [`TimeAdvance::Time`], the notion of current time is
    /// advanced by the provided amount, and only the timestamp of the frame is
    /// exceeded, do we process the next frame. This mechanism allows fine
    /// grained control of frame processing, and the frequency of invocations of
    /// the [`Collector`]. If time is advanced by less than the delay between
    /// frames, the collector will be called more than once per frame, and can
    /// use functions like [`Self::get_interpolated_player_rigid_body`] to get
    /// values that are interpolated between frames. Its also possible to skip
    /// over frames by providing time advance values that are sufficiently
    /// large.
    ///
    /// At the end of processing, it checks to make sure that no unknown players
    /// were encountered during the replay. If any unknown players are found, an
    /// error is returned.
    pub fn process<H: Collector>(&mut self, handler: &mut H) -> SubtrActorResult<()> {
        // Initially, we set target_time to NextFrame to ensure the collector
        // will process the first frame.
        let mut target_time = TimeAdvance::NextFrame;
        for (index, frame) in self
            .replay
            .network_frames
            .as_ref()
            .ok_or(SubtrActorError::new(
                SubtrActorErrorVariant::NoNetworkFrames,
            ))?
            .frames
            .iter()
            .enumerate()
        {
            // Update the internal state of the processor based on the current frame
            self.actor_state.process_frame(frame, index)?;
            self.update_mappings(frame)?;
            self.update_ball_id(frame)?;
            self.update_boost_amounts(frame, index)?;
            self.update_boost_pad_events(frame, index)?;
            self.update_touch_events(frame, index)?;
            self.update_dodge_refreshed_events(frame, index)?;
            self.update_goal_events(frame, index)?;
            self.update_player_stat_events(frame, index)?;
            self.update_demolishes(frame, index)?;

            // Get the time to process for this frame. If target_time is set to
            // NextFrame, we use the time of the current frame.
            let mut current_time = match &target_time {
                TimeAdvance::Time(t) => *t,
                TimeAdvance::NextFrame => frame.time,
            };

            while current_time <= frame.time {
                // Call the handler to process the frame and get the time for
                // the next frame the handler wants to process
                target_time = handler.process_frame(self, frame, index, current_time)?;
                // If the handler specified a specific time, update current_time
                // to that time. If the handler specified NextFrame, we break
                // out of the loop to move on to the next frame in the replay.
                // This design allows the handler to have control over the frame
                // rate, including the possibility of skipping frames.
                if let TimeAdvance::Time(new_target) = target_time {
                    current_time = new_target;
                } else {
                    break;
                }
            }
        }
        handler.finish_replay(self)?;
        Ok(())
    }

    /// Process multiple collectors simultaneously over the same replay frames.
    ///
    /// All collectors receive the same frame data for each frame. This is useful
    /// when you have multiple independent collectors that each gather different
    /// aspects of replay data.
    ///
    /// Note: This method always advances frame-by-frame. If collectors return
    /// [`TimeAdvance::Time`] values, those are ignored.
    pub fn process_all(&mut self, collectors: &mut [&mut dyn Collector]) -> SubtrActorResult<()> {
        for (index, frame) in self
            .replay
            .network_frames
            .as_ref()
            .ok_or(SubtrActorError::new(
                SubtrActorErrorVariant::NoNetworkFrames,
            ))?
            .frames
            .iter()
            .enumerate()
        {
            self.actor_state.process_frame(frame, index)?;
            self.update_mappings(frame)?;
            self.update_ball_id(frame)?;
            self.update_boost_amounts(frame, index)?;
            self.update_boost_pad_events(frame, index)?;
            self.update_touch_events(frame, index)?;
            self.update_dodge_refreshed_events(frame, index)?;
            self.update_goal_events(frame, index)?;
            self.update_player_stat_events(frame, index)?;
            self.update_demolishes(frame, index)?;

            for collector in collectors.iter_mut() {
                collector.process_frame(self, frame, index, frame.time)?;
            }
        }
        for collector in collectors.iter_mut() {
            collector.finish_replay(self)?;
        }
        Ok(())
    }

    /// Reset the state of the [`ReplayProcessor`].
    pub fn reset(&mut self) {
        self.ball_actor_id = None;
        self.player_to_car = HashMap::new();
        self.player_to_team = HashMap::new();
        self.player_to_actor_id = HashMap::new();
        self.car_to_player = HashMap::new();
        self.car_to_boost = HashMap::new();
        self.car_to_jump = HashMap::new();
        self.car_to_double_jump = HashMap::new();
        self.car_to_dodge = HashMap::new();
        self.actor_state = ActorStateModeler::new();
        self.boost_pad_events = Vec::new();
        self.current_frame_boost_pad_events = Vec::new();
        self.touch_events = Vec::new();
        self.current_frame_touch_events = Vec::new();
        self.dodge_refreshed_events = Vec::new();
        self.current_frame_dodge_refreshed_events = Vec::new();
        self.dodge_refreshed_counters = HashMap::new();
        self.goal_events = Vec::new();
        self.current_frame_goal_events = Vec::new();
        self.player_stat_events = Vec::new();
        self.current_frame_player_stat_events = Vec::new();
        self.player_stat_counters = HashMap::new();
        self.demolishes = Vec::new();
        self.known_demolishes = Vec::new();
        self.demolish_format = None;
        self.kickoff_phase_active_last_frame = false;
    }
}

#[cfg(test)]
mod tests {
    use super::ReplayProcessor;

    #[test]
    fn rigid_body_normalization_factors_split_at_expected_legacy_boundary() {
        assert_eq!(
            ReplayProcessor::rigid_body_location_normalization_factor_for_net_version(None),
            100.0
        );
        assert_eq!(
            ReplayProcessor::rigid_body_velocity_normalization_factor_for_net_version(None),
            10.0
        );
        assert_eq!(
            ReplayProcessor::rigid_body_location_normalization_factor_for_net_version(Some(2)),
            100.0
        );
        assert_eq!(
            ReplayProcessor::rigid_body_velocity_normalization_factor_for_net_version(Some(2)),
            10.0
        );
        assert!(ReplayProcessor::uses_legacy_rigid_body_rotation_for_net_version(Some(2)));
        assert_eq!(
            ReplayProcessor::rigid_body_location_normalization_factor_for_net_version(Some(5)),
            1.0
        );
        assert_eq!(
            ReplayProcessor::rigid_body_velocity_normalization_factor_for_net_version(Some(5)),
            1.0
        );
        assert!(ReplayProcessor::uses_legacy_rigid_body_rotation_for_net_version(Some(5)));
        assert_eq!(
            ReplayProcessor::rigid_body_location_normalization_factor_for_net_version(Some(10)),
            1.0
        );
        assert_eq!(
            ReplayProcessor::rigid_body_velocity_normalization_factor_for_net_version(Some(10)),
            1.0
        );
        assert!(!ReplayProcessor::uses_legacy_rigid_body_rotation_for_net_version(Some(7)));
        assert!(!ReplayProcessor::uses_legacy_rigid_body_rotation_for_net_version(Some(10)));
    }
}
