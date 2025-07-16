use crate::*;
use boxcars;
use std::collections::HashMap;

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
macro_rules! attribute_match {
    ($value:expr, $type:path $(,)?) => {{
        let attribute = $value;
        if let $type(value) = attribute {
            Ok(value)
        } else {
            SubtrActorError::new_result(SubtrActorErrorVariant::UnexpectedAttributeType {
                expected_type: stringify!(path).to_string(),
                actual_type: attribute_to_tag(&attribute).to_string(),
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
    pub replay: &'a boxcars::Replay,
    pub actor_state: ActorStateModeler,
    pub object_id_to_name: HashMap<boxcars::ObjectId, String>,
    pub name_to_object_id: HashMap<String, boxcars::ObjectId>,
    pub ball_actor_id: Option<boxcars::ActorId>,
    pub team_zero: Vec<PlayerId>,
    pub team_one: Vec<PlayerId>,
    pub player_to_actor_id: HashMap<PlayerId, boxcars::ActorId>,
    pub player_to_car: HashMap<boxcars::ActorId, boxcars::ActorId>,
    pub player_to_team: HashMap<boxcars::ActorId, boxcars::ActorId>,
    pub car_to_boost: HashMap<boxcars::ActorId, boxcars::ActorId>,
    pub car_to_jump: HashMap<boxcars::ActorId, boxcars::ActorId>,
    pub car_to_double_jump: HashMap<boxcars::ActorId, boxcars::ActorId>,
    pub car_to_dodge: HashMap<boxcars::ActorId, boxcars::ActorId>,
    pub demolishes: Vec<DemolishInfo>,
    known_demolishes: Vec<(boxcars::DemolishFx, usize)>,
}

impl<'a> ReplayProcessor<'a> {
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
        for (id, name) in replay.objects.iter().enumerate() {
            let object_id = boxcars::ObjectId(id as i32);
            object_id_to_name.insert(object_id, name.clone());
            name_to_object_id.insert(name.clone(), object_id);
        }
        let mut processor = Self {
            actor_state: ActorStateModeler::new(),
            replay,
            object_id_to_name,
            name_to_object_id,
            team_zero: Vec::new(),
            team_one: Vec::new(),
            ball_actor_id: None,
            player_to_car: HashMap::new(),
            player_to_team: HashMap::new(),
            player_to_actor_id: HashMap::new(),
            car_to_boost: HashMap::new(),
            car_to_jump: HashMap::new(),
            car_to_double_jump: HashMap::new(),
            car_to_dodge: HashMap::new(),
            demolishes: Vec::new(),
            known_demolishes: Vec::new(),
        };
        processor
            .set_player_order_from_headers()
            .or_else(|_| processor.set_player_order_from_frames())?;

        Ok(processor)
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
        Ok(())
    }

    /// Reset the state of the [`ReplayProcessor`].
    pub fn reset(&mut self) {
        self.player_to_car = HashMap::new();
        self.player_to_team = HashMap::new();
        self.player_to_actor_id = HashMap::new();
        self.car_to_boost = HashMap::new();
        self.car_to_jump = HashMap::new();
        self.car_to_double_jump = HashMap::new();
        self.car_to_dodge = HashMap::new();
        self.actor_state = ActorStateModeler::new();
        self.demolishes = Vec::new();
        self.known_demolishes = Vec::new();
    }

    fn set_player_order_from_headers(&mut self) -> SubtrActorResult<()> {
        let _player_stats = self
            .replay
            .properties
            .iter()
            .find(|(key, _)| key == "PlayerStats")
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PlayerStatsHeaderNotFound)
            })?;
        // XXX: implementation incomplete
        SubtrActorError::new_result(SubtrActorErrorVariant::PlayerStatsHeaderNotFound)
    }

    /// Processes the replay until it has gathered enough information to map
    /// players to their actor IDs.
    ///
    /// This function is designed to ensure that each player that participated
    /// in the game is associated with a corresponding actor ID. It runs the
    /// processing operation for approximately the first 10 seconds of the
    /// replay (10 * 30 frames), as this time span is generally sufficient to
    /// identify all players.
    ///
    /// Note that this function is particularly necessary because the headers of
    /// replays sometimes omit some players.
    ///
    /// # Errors
    ///
    /// If any error other than `FinishProcessingEarly` occurs during the
    /// processing operation, it is propagated up by this function.
    pub fn process_long_enough_to_get_actor_ids(&mut self) -> SubtrActorResult<()> {
        let mut handler = |_p: &ReplayProcessor, _f: &boxcars::Frame, n: usize, _current_time| {
            // XXX: 10 seconds should be enough to find everyone, right?
            if n > 10 * 30 {
                SubtrActorError::new_result(SubtrActorErrorVariant::FinishProcessingEarly)
            } else {
                Ok(TimeAdvance::NextFrame)
            }
        };
        let process_result = self.process(&mut handler);
        if let Some(SubtrActorErrorVariant::FinishProcessingEarly) =
            process_result.as_ref().err().map(|e| e.variant.clone())
        {
            Ok(())
        } else {
            process_result
        }
    }

    fn set_player_order_from_frames(&mut self) -> SubtrActorResult<()> {
        self.process_long_enough_to_get_actor_ids()?;
        let player_to_team_0: HashMap<PlayerId, bool> = self
            .player_to_actor_id
            .keys()
            .filter_map(|player_id| {
                self.get_player_is_team_0(player_id)
                    .ok()
                    .map(|is_team_0| (player_id.clone(), is_team_0))
            })
            .collect();

        let (team_zero, team_one): (Vec<_>, Vec<_>) = player_to_team_0
            .keys()
            .cloned()
            // The unwrap here is fine because we know the get will succeed
            .partition(|player_id| *player_to_team_0.get(player_id).unwrap());

        self.team_zero = team_zero;
        self.team_one = team_one;

        self.team_zero
            .sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
        self.team_one
            .sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));

        self.reset();
        Ok(())
    }

    pub fn check_player_id_set(&self) -> SubtrActorResult<()> {
        let known_players =
            std::collections::HashSet::<_>::from_iter(self.player_to_actor_id.keys());
        let original_players =
            std::collections::HashSet::<_>::from_iter(self.iter_player_ids_in_order());

        if original_players != known_players {
            SubtrActorError::new_result(SubtrActorErrorVariant::InconsistentPlayerSet {
                found: known_players.into_iter().cloned().collect(),
                original: original_players.into_iter().cloned().collect(),
            })
        } else {
            Ok(())
        }
    }

    /// Processes the replay enough to get the actor IDs and then retrieves the replay metadata.
    ///
    /// This method is a convenience function that combines the functionalities
    /// of
    /// [`process_long_enough_to_get_actor_ids`](Self::process_long_enough_to_get_actor_ids)
    /// and [`get_replay_meta`](Self::get_replay_meta) into a single operation.
    /// It's meant to be used when you don't necessarily want to process the
    /// whole replay and need only the replay's metadata.
    pub fn process_and_get_replay_meta(&mut self) -> SubtrActorResult<ReplayMeta> {
        if self.player_to_actor_id.is_empty() {
            self.process_long_enough_to_get_actor_ids()?;
        }
        self.get_replay_meta()
    }

    /// Retrieves the replay metadata.
    ///
    /// This function collects information about each player in the replay and
    /// groups them by team. For each player, it gets the player's name and
    /// statistics. All this information is then wrapped into a [`ReplayMeta`]
    /// object along with the properties from the replay.
    pub fn get_replay_meta(&self) -> SubtrActorResult<ReplayMeta> {
        let empty_player_stats = Vec::new();
        let player_stats = if let Some((_, boxcars::HeaderProp::Array(per_player))) = self
            .replay
            .properties
            .iter()
            .find(|(key, _)| key == "PlayerStats")
        {
            per_player
        } else {
            &empty_player_stats
        };
        let known_count = self.iter_player_ids_in_order().count();
        if player_stats.len() != known_count {
            log::warn!(
                "Replay does not have player stats for all players. encountered {:?} {:?}",
                known_count,
                player_stats.len()
            )
        }
        let get_player_info = |player_id| {
            let name = self.get_player_name(player_id)?;
            let stats = find_player_stats(player_id, &name, player_stats).ok();
            Ok(PlayerInfo {
                name,
                stats,
                remote_id: player_id.clone(),
            })
        };
        let team_zero: SubtrActorResult<Vec<PlayerInfo>> =
            self.team_zero.iter().map(get_player_info).collect();
        let team_one: SubtrActorResult<Vec<PlayerInfo>> =
            self.team_one.iter().map(get_player_info).collect();
        Ok(ReplayMeta {
            team_zero: team_zero?,
            team_one: team_one?,
            all_headers: self.replay.properties.clone(),
        })
    }

    /// Searches for the next or previous update for a specified actor and
    /// object in the replay's network frames.
    ///
    /// This method uses the [`find_in_direction`](util::find_in_direction)
    /// function to search through the network frames of the replay to find the
    /// next (or previous, depending on the direction provided) attribute update
    /// for a specified actor and object.
    ///
    /// # Arguments
    ///
    /// * `current_index` - The index of the network frame from where the search should start.
    /// * `actor_id` - The ID of the actor for which the update is being searched.
    /// * `object_id` - The ID of the object associated with the actor for which
    ///   the update is being searched.
    /// * `direction` - The direction of search, specified as either
    ///   [`SearchDirection::Backward`] or [`SearchDirection::Forward`].
    ///
    /// # Returns
    ///
    /// If a matching update is found, this function returns a
    /// [`SubtrActorResult`] tuple containing the found attribute and its index
    /// in the replay's network frames.
    ///
    /// # Errors
    ///
    /// If no matching update is found, or if the replay has no network frames,
    /// this function returns a [`SubtrActorError`]. Specifically, it returns
    /// `NoUpdateAfterFrame` error variant if no update is found after the
    /// specified frame, or `NoNetworkFrames` if the replay lacks network
    /// frames.
    ///
    /// [`SearchDirection::Backward`]: enum.SearchDirection.html#variant.Backward
    /// [`SearchDirection::Forward`]: enum.SearchDirection.html#variant.Forward
    /// [`SubtrActorResult`]: type.SubtrActorResult.html
    /// [`SubtrActorError`]: struct.SubtrActorError.html
    pub fn find_update_in_direction(
        &self,
        current_index: usize,
        actor_id: &boxcars::ActorId,
        object_id: &boxcars::ObjectId,
        direction: SearchDirection,
    ) -> SubtrActorResult<(boxcars::Attribute, usize)> {
        let frames = self
            .replay
            .network_frames
            .as_ref()
            .ok_or(SubtrActorError::new(
                SubtrActorErrorVariant::NoNetworkFrames,
            ))?;

        let predicate = |frame: &boxcars::Frame| {
            frame
                .updated_actors
                .iter()
                .find(|update| &update.actor_id == actor_id && &update.object_id == object_id)
                .map(|update| &update.attribute)
                .cloned()
        };

        match util::find_in_direction(&frames.frames, current_index, direction, predicate) {
            Some((index, attribute)) => Ok((attribute, index)),
            None => SubtrActorError::new_result(SubtrActorErrorVariant::NoUpdateAfterFrame {
                actor_id: *actor_id,
                object_id: *object_id,
                frame_index: current_index,
            }),
        }
    }

    // Update functions

    /// This method is responsible for updating various mappings that are used
    /// to track and link different actors in the replay.
    ///
    /// The replay data is a stream of [`boxcars::Frame`] objects that contain
    /// information about the game at a specific point in time. These frames
    /// contain updates for different actors, and the goal of this method is to
    /// maintain and update the mappings for these actors as the frames are
    /// processed.
    ///
    /// The method loops over each `updated_actors` field in the
    /// [`boxcars::Frame`]. For each updated actor, it checks whether the
    /// actor's object ID matches the object ID of various keys in the actor
    /// state. If a match is found, the corresponding map is updated with a new
    /// entry linking the actor ID to the value of the attribute in the replay
    /// frame.
    ///
    /// The mappings updated are:
    /// - `player_to_actor_id`: maps a player's [`boxcars::UniqueId`] to their actor ID.
    /// - `player_to_team`: maps a player's actor ID to their team actor ID.
    /// - `player_to_car`: maps a player's actor ID to their car actor ID.
    /// - `car_to_boost`: maps a car's actor ID to its associated boost actor ID.
    /// - `car_to_dodge`: maps a car's actor ID to its associated dodge actor ID.
    /// - `car_to_jump`: maps a car's actor ID to its associated jump actor ID.
    /// - `car_to_double_jump`: maps a car's actor ID to its associated double jump actor ID.
    ///
    /// The function also handles the deletion of actors. When an actor is
    /// deleted, the function removes the actor's ID from the `player_to_car`
    /// mapping.
    fn update_mappings(&mut self, frame: &boxcars::Frame) -> SubtrActorResult<()> {
        for update in frame.updated_actors.iter() {
            macro_rules! maintain_link {
                ($map:expr, $actor_type:expr, $attr:expr, $get_key: expr, $get_value: expr, $type:path) => {{
                    if &update.object_id == self.get_object_id_for_key(&$attr)? {
                        if self
                            .get_actor_ids_by_type($actor_type)?
                            .iter()
                            .any(|id| id == &update.actor_id)
                        {
                            let value = get_actor_attribute_matching!(
                                self,
                                &update.actor_id,
                                $attr,
                                $type
                            )?;
                            let _key = $get_key(update.actor_id, value);
                            let _new_value = $get_value(update.actor_id, value);
                            let _old_value = $map.insert(
                                $get_key(update.actor_id, value),
                                $get_value(update.actor_id, value),
                            );
                        }
                    }
                }};
            }
            macro_rules! maintain_actor_link {
                ($map:expr, $actor_type:expr, $attr:expr) => {
                    maintain_link!(
                        $map,
                        $actor_type,
                        $attr,
                        // This is slightly confusing, but in these cases we are
                        // using the attribute as the key to the current actor.
                        get_actor_id_from_active_actor,
                        use_update_actor,
                        boxcars::Attribute::ActiveActor
                    )
                };
            }
            macro_rules! maintain_vehicle_key_link {
                ($map:expr, $actor_type:expr) => {
                    maintain_actor_link!($map, $actor_type, VEHICLE_KEY)
                };
            }
            maintain_link!(
                self.player_to_actor_id,
                PLAYER_TYPE,
                UNIQUE_ID_KEY,
                |_, unique_id: &boxcars::UniqueId| unique_id.remote_id.clone(),
                use_update_actor,
                boxcars::Attribute::UniqueId
            );
            maintain_link!(
                self.player_to_team,
                PLAYER_TYPE,
                TEAM_KEY,
                // In this case we are using the update actor as the key.
                use_update_actor,
                get_actor_id_from_active_actor,
                boxcars::Attribute::ActiveActor
            );
            maintain_actor_link!(self.player_to_car, CAR_TYPE, PLAYER_REPLICATION_KEY);
            maintain_vehicle_key_link!(self.car_to_boost, BOOST_TYPE);
            maintain_vehicle_key_link!(self.car_to_dodge, DODGE_TYPE);
            maintain_vehicle_key_link!(self.car_to_jump, JUMP_TYPE);
            maintain_vehicle_key_link!(self.car_to_double_jump, DOUBLE_JUMP_TYPE);
        }

        for actor_id in frame.deleted_actors.iter() {
            if let Some(car_id) = self.player_to_car.remove(actor_id) {
                log::info!("Player actor {actor_id:?} deleted, car id: {car_id:?}.");
            }
        }

        Ok(())
    }

    fn update_ball_id(&mut self, frame: &boxcars::Frame) -> SubtrActorResult<()> {
        // XXX: This assumes there is only ever one ball, which is safe (I think?)
        if let Some(actor_id) = self.ball_actor_id {
            if frame.deleted_actors.contains(&actor_id) {
                self.ball_actor_id = None;
            }
        } else {
            self.ball_actor_id = self.find_ball_actor();
            if self.ball_actor_id.is_some() {
                return self.update_ball_id(frame);
            }
        }
        Ok(())
    }

    /// Updates the boost amounts for all the actors in a given frame.
    ///
    /// This function works by iterating over all the actors of a particular
    /// boost type. For each actor, it retrieves the current boost value. If the
    /// actor's boost value hasn't been updated, it continues using the derived
    /// boost value from the last frame. If the actor's boost is active, it
    /// subtracts from the current boost value according to the frame delta and
    /// the constant `BOOST_USED_PER_SECOND`.
    ///
    /// The updated boost values are then stored in the actor's derived
    /// attributes.
    ///
    /// # Arguments
    ///
    /// * `frame` - A reference to the [`Frame`] in which the boost amounts are to be updated.
    /// * `frame_index` - The index of the frame in the replay.
    ///   [`Frame`]: boxcars::Frame
    fn update_boost_amounts(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        let updates: Vec<_> = self
            .iter_actors_by_type_err(BOOST_TYPE)?
            .map(|(actor_id, actor_state)| {
                let (actor_amount_value, last_value, _, derived_value, is_active) =
                    self.get_current_boost_values(actor_state);
                let mut current_value = if actor_amount_value == last_value {
                    // If we don't have an update in the actor, just continue
                    // using our derived value
                    derived_value
                } else {
                    // If we do have an update in the actor, use that value.
                    actor_amount_value.into()
                };
                if is_active {
                    current_value -= frame.delta * BOOST_USED_PER_SECOND;
                }
                (*actor_id, current_value.max(0.0), actor_amount_value)
            })
            .collect();

        for (actor_id, current_value, new_last_value) in updates {
            let derived_attributes = &mut self
                .actor_state
                .actor_states
                .get_mut(&actor_id)
                // This actor is known to exist, so unwrap is fine
                .unwrap()
                .derived_attributes;

            derived_attributes.insert(
                LAST_BOOST_AMOUNT_KEY.to_string(),
                (boxcars::Attribute::Byte(new_last_value), frame_index),
            );
            derived_attributes.insert(
                BOOST_AMOUNT_KEY.to_string(),
                (boxcars::Attribute::Float(current_value), frame_index),
            );
        }
        Ok(())
    }

    /// Gets the current boost values for a given actor state.
    ///
    /// This function retrieves the current boost amount, whether the boost is active,
    /// the derived boost amount, and the last known boost amount from the actor's state.
    /// The derived value is retrieved from the actor's derived attributes, while
    /// the other values are retrieved directly from the actor's attributes.
    ///
    /// # Arguments
    ///
    /// * `actor_state` - A reference to the actor's [`ActorState`] from which
    ///   the boost values are to be retrieved.
    ///
    /// # Returns
    ///
    /// This function returns a tuple consisting of the following:
    /// * Current boost amount
    /// * Last known boost amount
    /// * Boost active value (1 if active, 0 otherwise)
    /// * Derived boost amount
    /// * Whether the boost is active (true if active, false otherwise)
    fn get_current_boost_values(&self, actor_state: &ActorState) -> (u8, u8, u8, f32, bool) {
        // Try to get boost amount from ReplicatedBoost attribute first (new format)
        let amount_value = if let Ok(boxcars::Attribute::ReplicatedBoost(replicated_boost)) =
            self.get_attribute(&actor_state.attributes, BOOST_REPLICATED_KEY)
        {
            replicated_boost.boost_amount
        } else {
            // Fall back to ReplicatedBoostAmount (old format)
            get_attribute_errors_expected!(
                self,
                &actor_state.attributes,
                BOOST_AMOUNT_KEY,
                boxcars::Attribute::Byte
            )
            .cloned()
            .unwrap_or(0)
        };
        let active_value = get_attribute_errors_expected!(
            self,
            &actor_state.attributes,
            COMPONENT_ACTIVE_KEY,
            boxcars::Attribute::Byte
        )
        .cloned()
        .unwrap_or(0);
        let is_active = active_value % 2 == 1;
        let derived_value = actor_state
            .derived_attributes
            .get(BOOST_AMOUNT_KEY)
            .cloned()
            .and_then(|v| attribute_match!(v.0, boxcars::Attribute::Float).ok())
            .unwrap_or(0.0);
        let last_boost_amount = attribute_match!(
            actor_state
                .derived_attributes
                .get(LAST_BOOST_AMOUNT_KEY)
                .cloned()
                .map(|v| v.0)
                .unwrap_or_else(|| boxcars::Attribute::Byte(amount_value)),
            boxcars::Attribute::Byte
        )
        .unwrap_or(0);
        (
            amount_value,
            last_boost_amount,
            active_value,
            derived_value,
            is_active,
        )
    }

    fn update_demolishes(&mut self, frame: &boxcars::Frame, index: usize) -> SubtrActorResult<()> {
        let new_demolishes: Vec<_> = self
            .get_active_demolish_fx()?
            .flat_map(|demolish_fx| {
                if !self.demolish_is_known(demolish_fx, index) {
                    Some(*demolish_fx.as_ref())
                } else {
                    None
                }
            })
            .collect();

        for demolish in new_demolishes {
            match self.build_demolish_info(&demolish, frame, index) {
                Ok(demolish_info) => self.demolishes.push(demolish_info),
                Err(_e) => {
                    log::warn!("Error building demolish info");
                }
            }
            self.known_demolishes.push((demolish, index))
        }

        Ok(())
    }

    fn build_demolish_info(
        &self,
        demolish_fx: &boxcars::DemolishFx,
        frame: &boxcars::Frame,
        index: usize,
    ) -> SubtrActorResult<DemolishInfo> {
        let attacker = self.get_player_id_from_car_id(&demolish_fx.attacker)?;
        let victim = self.get_player_id_from_car_id(&demolish_fx.victim)?;
        Ok(DemolishInfo {
            time: frame.time,
            seconds_remaining: self.get_seconds_remaining()?,
            frame: index,
            attacker,
            victim,
            attacker_velocity: demolish_fx.attack_velocity,
            victim_velocity: demolish_fx.victim_velocity,
        })
    }

    // ID Mapping functions

    fn get_player_id_from_car_id(&self, actor_id: &boxcars::ActorId) -> SubtrActorResult<PlayerId> {
        self.get_player_id_from_actor_id(&self.get_player_actor_id_from_car_actor_id(actor_id)?)
    }

    fn get_player_id_from_actor_id(
        &self,
        actor_id: &boxcars::ActorId,
    ) -> SubtrActorResult<PlayerId> {
        for (player_id, player_actor_id) in self.player_to_actor_id.iter() {
            if actor_id == player_actor_id {
                return Ok(player_id.clone());
            }
        }
        SubtrActorError::new_result(SubtrActorErrorVariant::NoMatchingPlayerId {
            actor_id: *actor_id,
        })
    }

    fn get_player_actor_id_from_car_actor_id(
        &self,
        actor_id: &boxcars::ActorId,
    ) -> SubtrActorResult<boxcars::ActorId> {
        for (player_id, car_id) in self.player_to_car.iter() {
            if actor_id == car_id {
                return Ok(*player_id);
            }
        }
        SubtrActorError::new_result(SubtrActorErrorVariant::NoMatchingPlayerId {
            actor_id: *actor_id,
        })
    }

    fn demolish_is_known(&self, demolish_fx: &boxcars::DemolishFx, frame_index: usize) -> bool {
        self.known_demolishes.iter().any(|(existing, index)| {
            existing == demolish_fx
                && frame_index
                    .checked_sub(*index)
                    .or_else(|| index.checked_sub(frame_index))
                    .unwrap()
                    < MAX_DEMOLISH_KNOWN_FRAMES_PASSED
        })
    }

    /// Provides an iterator over the active demolition effects,
    /// [`boxcars::DemolishFx`], in the current frame.
    pub fn get_active_demolish_fx(
        &self,
    ) -> SubtrActorResult<impl Iterator<Item = &Box<boxcars::DemolishFx>>> {
        Ok(self
            .iter_actors_by_type_err(CAR_TYPE)?
            .flat_map(|(_actor_id, state)| {
                get_attribute_errors_expected!(
                    self,
                    &state.attributes,
                    DEMOLISH_GOAL_EXPLOSION_KEY,
                    boxcars::Attribute::DemolishFx
                )
                .ok()
            }))
    }

    // Interpolation Support functions

    fn get_frame(&self, frame_index: usize) -> SubtrActorResult<&boxcars::Frame> {
        self.replay
            .network_frames
            .as_ref()
            .ok_or(SubtrActorError::new(
                SubtrActorErrorVariant::NoNetworkFrames,
            ))?
            .frames
            .get(frame_index)
            .ok_or(SubtrActorError::new(
                SubtrActorErrorVariant::FrameIndexOutOfBounds,
            ))
    }

    fn velocities_applied_rigid_body(
        &self,
        rigid_body: &boxcars::RigidBody,
        rb_frame_index: usize,
        target_time: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        let rb_frame = self.get_frame(rb_frame_index)?;
        let interpolation_amount = target_time - rb_frame.time;
        Ok(apply_velocities_to_rigid_body(
            rigid_body,
            interpolation_amount,
        ))
    }

    /// This function first retrieves the actor's [`RigidBody`] at the current
    /// frame. If the time difference between the current frame and the provided
    /// time is within the `close_enough` threshold, the function returns the
    /// current frame's [`RigidBody`].
    ///
    /// If the [`RigidBody`] at the exact time is not available, the function
    /// searches in the appropriate direction (either forwards or backwards in
    /// time) to find another [`RigidBody`] to interpolate from. If the found
    /// [`RigidBody`]'s time is within the `close_enough` threshold, it is
    /// returned.
    ///
    /// Otherwise, it interpolates between the two [`RigidBody`]s (from the
    /// current frame and the found frame) to produce a [`RigidBody`] for the
    /// specified time. This is done using the [`get_interpolated_rigid_body`]
    /// function from the `util` module.
    ///
    /// # Arguments
    ///
    /// * `actor_id` - The ID of the actor whose [`RigidBody`] is to be retrieved.
    /// * `time` - The time at which the actor's [`RigidBody`] is to be retrieved.
    /// * `close_enough` - The acceptable threshold for time difference when
    ///   determining if a [`RigidBody`] is close enough to the desired time to not
    ///   require interpolation.
    ///
    /// # Returns
    ///
    /// A [`RigidBody`] for the actor at the specified time.
    ///
    /// [`RigidBody`]: boxcars::RigidBody
    /// [`get_interpolated_rigid_body`]: util::get_interpolated_rigid_body
    pub fn get_interpolated_actor_rigid_body(
        &self,
        actor_id: &boxcars::ActorId,
        time: f32,
        close_enough: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        let (frame_body, frame_index) = self.get_actor_rigid_body(actor_id)?;
        let frame_time = self.get_frame(*frame_index)?.time;
        let time_and_frame_difference = time - frame_time;

        if (time_and_frame_difference).abs() <= close_enough.abs() {
            return Ok(*frame_body);
        }

        let search_direction = if time_and_frame_difference > 0.0 {
            util::SearchDirection::Forward
        } else {
            util::SearchDirection::Backward
        };

        let object_id = self.get_object_id_for_key(RIGID_BODY_STATE_KEY)?;

        let (attribute, found_frame) =
            self.find_update_in_direction(*frame_index, actor_id, object_id, search_direction)?;
        let found_time = self.get_frame(found_frame)?.time;

        let found_body = attribute_match!(attribute, boxcars::Attribute::RigidBody)?;

        if (found_time - time).abs() <= close_enough {
            return Ok(found_body);
        }

        let (start_body, start_time, end_body, end_time) = match search_direction {
            util::SearchDirection::Forward => (frame_body, frame_time, &found_body, found_time),
            util::SearchDirection::Backward => (&found_body, found_time, frame_body, frame_time),
        };

        util::get_interpolated_rigid_body(start_body, start_time, end_body, end_time, time)
    }

    // Actor functions

    fn get_object_id_for_key(&self, name: &'static str) -> SubtrActorResult<&boxcars::ObjectId> {
        self.name_to_object_id
            .get(name)
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::ObjectIdNotFound { name }))
    }

    fn get_actor_ids_by_type(&self, name: &'static str) -> SubtrActorResult<&[boxcars::ActorId]> {
        self.get_object_id_for_key(name)
            .map(|object_id| self.get_actor_ids_by_object_id(object_id))
    }

    fn get_actor_ids_by_object_id(&self, object_id: &boxcars::ObjectId) -> &[boxcars::ActorId] {
        self.actor_state
            .actor_ids_by_type
            .get(object_id)
            .map(|v| &v[..])
            .unwrap_or_else(|| &EMPTY_ACTOR_IDS)
    }

    fn get_actor_state(&self, actor_id: &boxcars::ActorId) -> SubtrActorResult<&ActorState> {
        self.actor_state.actor_states.get(actor_id).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::NoStateForActorId {
                actor_id: *actor_id,
            })
        })
    }

    fn get_actor_attribute<'b>(
        &'b self,
        actor_id: &boxcars::ActorId,
        property: &'static str,
    ) -> SubtrActorResult<&'b boxcars::Attribute> {
        self.get_attribute(&self.get_actor_state(actor_id)?.attributes, property)
    }

    fn get_attribute<'b>(
        &'b self,
        map: &'b HashMap<boxcars::ObjectId, (boxcars::Attribute, usize)>,
        property: &'static str,
    ) -> SubtrActorResult<&'b boxcars::Attribute> {
        self.get_attribute_and_updated(map, property).map(|v| &v.0)
    }

    fn get_attribute_and_updated<'b>(
        &'b self,
        map: &'b HashMap<boxcars::ObjectId, (boxcars::Attribute, usize)>,
        property: &'static str,
    ) -> SubtrActorResult<&'b (boxcars::Attribute, usize)> {
        let attribute_object_id = self.get_object_id_for_key(property)?;
        map.get(attribute_object_id).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState { property })
        })
    }

    fn find_ball_actor(&self) -> Option<boxcars::ActorId> {
        BALL_TYPES
            .iter()
            .filter_map(|ball_type| self.iter_actors_by_type(ball_type))
            .flatten()
            .map(|(actor_id, _)| *actor_id)
            .next()
    }

    pub fn get_ball_actor_id(&self) -> SubtrActorResult<boxcars::ActorId> {
        self.ball_actor_id.ok_or(SubtrActorError::new(
            SubtrActorErrorVariant::BallActorNotFound,
        ))
    }

    pub fn get_metadata_actor_id(&self) -> SubtrActorResult<&boxcars::ActorId> {
        self.get_actor_ids_by_type(GAME_TYPE)?
            .iter()
            .next()
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::NoGameActor))
    }

    pub fn get_player_actor_id(&self, player_id: &PlayerId) -> SubtrActorResult<boxcars::ActorId> {
        self.player_to_actor_id
            .get(player_id)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::ActorNotFound {
                    name: "ActorId",
                    player_id: player_id.clone(),
                })
            })
            .cloned()
    }

    pub fn get_car_actor_id(&self, player_id: &PlayerId) -> SubtrActorResult<boxcars::ActorId> {
        self.player_to_car
            .get(&self.get_player_actor_id(player_id)?)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::ActorNotFound {
                    name: "Car",
                    player_id: player_id.clone(),
                })
            })
            .cloned()
    }

    pub fn get_car_connected_actor_id(
        &self,
        player_id: &PlayerId,
        map: &HashMap<boxcars::ActorId, boxcars::ActorId>,
        name: &'static str,
    ) -> SubtrActorResult<boxcars::ActorId> {
        map.get(&self.get_car_actor_id(player_id)?)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::ActorNotFound {
                    name,
                    player_id: player_id.clone(),
                })
            })
            .cloned()
    }

    pub fn get_boost_actor_id(&self, player_id: &PlayerId) -> SubtrActorResult<boxcars::ActorId> {
        self.get_car_connected_actor_id(player_id, &self.car_to_boost, "Boost")
    }

    pub fn get_jump_actor_id(&self, player_id: &PlayerId) -> SubtrActorResult<boxcars::ActorId> {
        self.get_car_connected_actor_id(player_id, &self.car_to_jump, "Jump")
    }

    pub fn get_double_jump_actor_id(
        &self,
        player_id: &PlayerId,
    ) -> SubtrActorResult<boxcars::ActorId> {
        self.get_car_connected_actor_id(player_id, &self.car_to_double_jump, "Double Jump")
    }

    pub fn get_dodge_actor_id(&self, player_id: &PlayerId) -> SubtrActorResult<boxcars::ActorId> {
        self.get_car_connected_actor_id(player_id, &self.car_to_dodge, "Dodge")
    }

    pub fn get_actor_rigid_body(
        &self,
        actor_id: &boxcars::ActorId,
    ) -> SubtrActorResult<(&boxcars::RigidBody, &usize)> {
        get_attribute_and_updated!(
            self,
            &self.get_actor_state(actor_id)?.attributes,
            RIGID_BODY_STATE_KEY,
            boxcars::Attribute::RigidBody
        )
    }

    // Actor iteration functions

    pub fn iter_player_ids_in_order(&self) -> impl Iterator<Item = &PlayerId> {
        self.team_zero.iter().chain(self.team_one.iter())
    }

    pub fn player_count(&self) -> usize {
        self.iter_player_ids_in_order().count()
    }

    fn iter_actors_by_type_err(
        &self,
        name: &'static str,
    ) -> SubtrActorResult<impl Iterator<Item = (&boxcars::ActorId, &ActorState)>> {
        Ok(self.iter_actors_by_object_id(self.get_object_id_for_key(name)?))
    }

    pub fn iter_actors_by_type(
        &self,
        name: &'static str,
    ) -> Option<impl Iterator<Item = (&boxcars::ActorId, &ActorState)>> {
        self.iter_actors_by_type_err(name).ok()
    }

    pub fn iter_actors_by_object_id<'b>(
        &'b self,
        object_id: &'b boxcars::ObjectId,
    ) -> impl Iterator<Item = (&'b boxcars::ActorId, &'b ActorState)> + 'b {
        let actor_ids = self
            .actor_state
            .actor_ids_by_type
            .get(object_id)
            .map(|v| &v[..])
            .unwrap_or_else(|| &EMPTY_ACTOR_IDS);

        actor_ids
            .iter()
            // This unwrap is fine because we know the actor will exist as it is
            // in the actor_ids_by_type
            .map(move |id| (id, self.actor_state.actor_states.get(id).unwrap()))
    }

    // Properties

    /// Returns the remaining time in seconds in the game as an `i32`.
    pub fn get_seconds_remaining(&self) -> SubtrActorResult<i32> {
        get_actor_attribute_matching!(
            self,
            self.get_metadata_actor_id()?,
            SECONDS_REMAINING_KEY,
            boxcars::Attribute::Int
        )
        .cloned()
    }

    /// Returns a boolean indicating whether ball syncing is ignored.
    pub fn get_ignore_ball_syncing(&self) -> SubtrActorResult<bool> {
        let actor_id = self.get_ball_actor_id()?;
        get_actor_attribute_matching!(
            self,
            &actor_id,
            IGNORE_SYNCING_KEY,
            boxcars::Attribute::Boolean
        )
        .cloned()
    }

    /// Returns a reference to the [`RigidBody`](boxcars::RigidBody) of the ball.
    pub fn get_ball_rigid_body(&self) -> SubtrActorResult<&boxcars::RigidBody> {
        self.ball_actor_id
            .ok_or(SubtrActorError::new(
                SubtrActorErrorVariant::BallActorNotFound,
            ))
            .and_then(|actor_id| self.get_actor_rigid_body(&actor_id).map(|v| v.0))
    }

    /// Returns a boolean indicating whether the ball's
    /// [`RigidBody`](boxcars::RigidBody) exists and is not sleeping.
    pub fn ball_rigid_body_exists(&self) -> SubtrActorResult<bool> {
        Ok(self
            .get_ball_rigid_body()
            .map(|rb| !rb.sleeping)
            .unwrap_or(false))
    }

    /// Returns a reference to the ball's [`RigidBody`](boxcars::RigidBody) and
    /// its last updated frame.
    pub fn get_ball_rigid_body_and_updated(
        &self,
    ) -> SubtrActorResult<(&boxcars::RigidBody, &usize)> {
        self.ball_actor_id
            .ok_or(SubtrActorError::new(
                SubtrActorErrorVariant::BallActorNotFound,
            ))
            .and_then(|actor_id| {
                get_attribute_and_updated!(
                    self,
                    &self.get_actor_state(&actor_id)?.attributes,
                    RIGID_BODY_STATE_KEY,
                    boxcars::Attribute::RigidBody
                )
            })
    }

    /// Returns a [`RigidBody`](boxcars::RigidBody) of the ball with applied
    /// velocity at the target time.
    pub fn get_velocity_applied_ball_rigid_body(
        &self,
        target_time: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        let (current_rigid_body, frame_index) = self.get_ball_rigid_body_and_updated()?;
        self.velocities_applied_rigid_body(current_rigid_body, *frame_index, target_time)
    }

    /// Returns an interpolated [`RigidBody`](boxcars::RigidBody) of the ball at
    /// a specified time.
    pub fn get_interpolated_ball_rigid_body(
        &self,
        time: f32,
        close_enough: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        self.get_interpolated_actor_rigid_body(&self.get_ball_actor_id()?, time, close_enough)
    }

    /// Returns the name of the specified player.
    pub fn get_player_name(&self, player_id: &PlayerId) -> SubtrActorResult<String> {
        get_actor_attribute_matching!(
            self,
            &self.get_player_actor_id(player_id)?,
            PLAYER_NAME_KEY,
            boxcars::Attribute::String
        )
        .cloned()
    }

    /// Returns the team key for the specified player.
    pub fn get_player_team_key(&self, player_id: &PlayerId) -> SubtrActorResult<String> {
        let team_actor_id = self
            .player_to_team
            .get(&self.get_player_actor_id(player_id)?)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::UnknownPlayerTeam {
                    player_id: player_id.clone(),
                })
            })?;
        let state = self.get_actor_state(team_actor_id)?;
        self.object_id_to_name
            .get(&state.object_id)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::UnknownPlayerTeam {
                    player_id: player_id.clone(),
                })
            })
            .cloned()
    }

    /// Determines if the player is on team 0.
    pub fn get_player_is_team_0(&self, player_id: &PlayerId) -> SubtrActorResult<bool> {
        Ok(self
            .get_player_team_key(player_id)?
            .chars()
            .last()
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::EmptyTeamName {
                    player_id: player_id.clone(),
                })
            })?
            == '0')
    }

    /// Returns a reference to the [`RigidBody`](boxcars::RigidBody) of the player's car.
    pub fn get_player_rigid_body(
        &self,
        player_id: &PlayerId,
    ) -> SubtrActorResult<&boxcars::RigidBody> {
        self.get_car_actor_id(player_id)
            .and_then(|actor_id| self.get_actor_rigid_body(&actor_id).map(|v| v.0))
    }

    /// Returns the most recent update to the [`RigidBody`](boxcars::RigidBody)
    /// of the player's car along with the index of the frame in which it was
    /// updated.
    pub fn get_player_rigid_body_and_updated(
        &self,
        player_id: &PlayerId,
    ) -> SubtrActorResult<(&boxcars::RigidBody, &usize)> {
        self.get_car_actor_id(player_id).and_then(|actor_id| {
            get_attribute_and_updated!(
                self,
                &self.get_actor_state(&actor_id)?.attributes,
                RIGID_BODY_STATE_KEY,
                boxcars::Attribute::RigidBody
            )
        })
    }

    pub fn get_velocity_applied_player_rigid_body(
        &self,
        player_id: &PlayerId,
        target_time: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        let (current_rigid_body, frame_index) =
            self.get_player_rigid_body_and_updated(player_id)?;
        self.velocities_applied_rigid_body(current_rigid_body, *frame_index, target_time)
    }

    pub fn get_interpolated_player_rigid_body(
        &self,
        player_id: &PlayerId,
        time: f32,
        close_enough: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        self.get_interpolated_actor_rigid_body(
            &self.get_car_actor_id(player_id).unwrap(),
            time,
            close_enough,
        )
    }

    pub fn get_player_boost_level(&self, player_id: &PlayerId) -> SubtrActorResult<f32> {
        self.get_boost_actor_id(player_id).and_then(|actor_id| {
            let boost_state = self.get_actor_state(&actor_id)?;
            get_derived_attribute!(
                boost_state.derived_attributes,
                BOOST_AMOUNT_KEY,
                boxcars::Attribute::Float
            )
            .cloned()
        })
    }

    pub fn get_component_active(&self, actor_id: &boxcars::ActorId) -> SubtrActorResult<u8> {
        get_actor_attribute_matching!(
            self,
            &actor_id,
            COMPONENT_ACTIVE_KEY,
            boxcars::Attribute::Byte
        )
        .cloned()
    }

    pub fn get_boost_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        self.get_boost_actor_id(player_id)
            .and_then(|actor_id| self.get_component_active(&actor_id))
    }

    pub fn get_jump_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        self.get_jump_actor_id(player_id)
            .and_then(|actor_id| self.get_component_active(&actor_id))
    }

    pub fn get_double_jump_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        self.get_double_jump_actor_id(player_id)
            .and_then(|actor_id| self.get_component_active(&actor_id))
    }

    pub fn get_dodge_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        self.get_dodge_actor_id(player_id)
            .and_then(|actor_id| self.get_component_active(&actor_id))
    }

    // Debugging

    pub fn map_attribute_keys(
        &self,
        hash_map: &HashMap<boxcars::ObjectId, (boxcars::Attribute, usize)>,
    ) -> HashMap<String, boxcars::Attribute> {
        hash_map
            .iter()
            .map(|(k, (v, _updated))| {
                self.object_id_to_name
                    .get(k)
                    .map(|name| (name.clone(), v.clone()))
                    .unwrap()
            })
            .collect()
    }

    pub fn all_mappings_string(&self) -> String {
        let pairs = [
            ("player_to_car", &self.player_to_car),
            ("player_to_team", &self.player_to_team),
            ("car_to_boost", &self.car_to_boost),
            ("car_to_jump", &self.car_to_jump),
            ("car_to_double_jump", &self.car_to_double_jump),
            ("car_to_dodge", &self.car_to_dodge),
        ];
        let strings: Vec<_> = pairs
            .iter()
            .map(|(map_name, map)| format!("{map_name:?}: {map:?}"))
            .collect();
        strings.join("\n")
    }

    pub fn actor_state_string(&self, actor_id: &boxcars::ActorId) -> String {
        format!(
            "{:?}",
            self.get_actor_state(actor_id)
                .map(|s| self.map_attribute_keys(&s.attributes))
        )
    }

    pub fn print_actors_by_id<'b>(&self, actor_ids: impl Iterator<Item = &'b boxcars::ActorId>) {
        actor_ids.for_each(|actor_id| {
            let state = self.get_actor_state(actor_id).unwrap();
            println!(
                "{:?}\n\n\n",
                self.object_id_to_name.get(&state.object_id).unwrap()
            );
            println!("{:?}", self.map_attribute_keys(&state.attributes))
        })
    }

    pub fn print_actors_of_type(&self, actor_type: &'static str) {
        self.iter_actors_by_type(actor_type)
            .unwrap()
            .for_each(|(_actor_id, state)| {
                println!("{:?}", self.map_attribute_keys(&state.attributes));
            });
    }

    pub fn print_actor_types(&self) {
        let types: Vec<_> = self
            .actor_state
            .actor_ids_by_type
            .keys()
            .filter_map(|id| self.object_id_to_name.get(id))
            .collect();
        println!("{types:?}");
    }

    pub fn print_all_actors(&self) {
        self.actor_state
            .actor_states
            .iter()
            .for_each(|(actor_id, _actor_state)| {
                println!("{:?}", self.actor_state_string(actor_id))
            })
    }
}
