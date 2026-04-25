use super::*;

impl<'a> ReplayProcessor<'a> {
    /// Searches forward or backward for the next update of a specific actor property.
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
        match direction {
            SearchDirection::Forward => {
                for index in (current_index + 1)..frames.frames.len() {
                    if let Some(attribute) = frames.frames[index]
                        .updated_actors
                        .iter()
                        .find(|update| {
                            &update.actor_id == actor_id && &update.object_id == object_id
                        })
                        .map(|update| update.attribute.clone())
                    {
                        return Ok((attribute, index));
                    }
                }
            }
            SearchDirection::Backward => {
                for index in (0..current_index).rev() {
                    if let Some(attribute) = frames.frames[index]
                        .updated_actors
                        .iter()
                        .find(|update| {
                            &update.actor_id == actor_id && &update.object_id == object_id
                        })
                        .map(|update| update.attribute.clone())
                    {
                        return Ok((attribute, index));
                    }
                }
            }
        }

        SubtrActorError::new_result(SubtrActorErrorVariant::NoUpdateAfterFrame {
            actor_id: *actor_id,
            object_id: *object_id,
            frame_index: current_index,
        })
    }

    /// Resolves a car actor id back to the owning player id.
    pub fn get_player_id_from_car_id(
        &self,
        actor_id: &boxcars::ActorId,
    ) -> SubtrActorResult<PlayerId> {
        self.get_player_id_from_actor_id(&self.get_player_actor_id_from_car_actor_id(actor_id)?)
    }

    /// Resolves a player-controller actor id back to the owning player id.
    pub(crate) fn get_player_id_from_actor_id(
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
        self.car_to_player.get(actor_id).copied().ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::NoMatchingPlayerId {
                actor_id: *actor_id,
            })
        })
    }

    /// Returns whether a demolish has already been recorded within the dedupe window.
    pub(crate) fn demolish_is_known(&self, demo: &DemolishAttribute, frame_index: usize) -> bool {
        self.known_demolishes
            .iter()
            .any(|(existing, existing_frame_index)| {
                existing == demo
                    && frame_index
                        .checked_sub(*existing_frame_index)
                        .or_else(|| existing_frame_index.checked_sub(frame_index))
                        .unwrap()
                        < MAX_DEMOLISH_KNOWN_FRAMES_PASSED
            })
    }

    /// Returns the demolish attribute encoding currently used by the replay, if known.
    pub fn get_demolish_format(&self) -> Option<DemolishFormat> {
        self.demolish_format
    }

    /// Returns the boost-pad events detected while processing the current frame.
    pub fn current_frame_boost_pad_events(&self) -> &[BoostPadEvent] {
        &self.current_frame_boost_pad_events
    }

    /// Returns the touch events detected while processing the current frame.
    pub fn current_frame_touch_events(&self) -> &[TouchEvent] {
        &self.current_frame_touch_events
    }

    /// Returns the dodge-refresh events detected while processing the current frame.
    pub fn current_frame_dodge_refreshed_events(&self) -> &[DodgeRefreshedEvent] {
        &self.current_frame_dodge_refreshed_events
    }

    /// Returns the goal events detected while processing the current frame.
    pub fn current_frame_goal_events(&self) -> &[GoalEvent] {
        &self.current_frame_goal_events
    }

    /// Returns the player stat events detected while processing the current frame.
    pub fn current_frame_player_stat_events(&self) -> &[PlayerStatEvent] {
        &self.current_frame_player_stat_events
    }

    /// Inspects current actor state to infer which demolish attribute format is present.
    pub fn detect_demolish_format(&self) -> Option<DemolishFormat> {
        let actors = self.iter_actors_by_type_err(CAR_TYPE).ok()?;
        for (_actor_id, state) in actors {
            if get_attribute_errors_expected!(
                self,
                &state.attributes,
                DEMOLISH_EXTENDED_KEY,
                boxcars::Attribute::DemolishExtended
            )
            .is_ok()
            {
                return Some(DemolishFormat::Extended);
            }
            if get_attribute_errors_expected!(
                self,
                &state.attributes,
                DEMOLISH_GOAL_EXPLOSION_KEY,
                boxcars::Attribute::DemolishFx
            )
            .is_ok()
            {
                return Some(DemolishFormat::Fx);
            }
        }
        None
    }

    /// Returns an iterator over currently active demolish attributes in actor state.
    pub fn get_active_demos(
        &self,
    ) -> SubtrActorResult<impl Iterator<Item = DemolishAttribute> + '_> {
        let format = self.demolish_format;
        let actors: Vec<_> = self.iter_actors_by_type_err(CAR_TYPE)?.collect();
        Ok(actors
            .into_iter()
            .filter_map(move |(_actor_id, state)| match format {
                Some(DemolishFormat::Extended) => get_attribute_errors_expected!(
                    self,
                    &state.attributes,
                    DEMOLISH_EXTENDED_KEY,
                    boxcars::Attribute::DemolishExtended
                )
                .ok()
                .map(|demo| DemolishAttribute::Extended(**demo)),
                Some(DemolishFormat::Fx) => get_attribute_errors_expected!(
                    self,
                    &state.attributes,
                    DEMOLISH_GOAL_EXPLOSION_KEY,
                    boxcars::Attribute::DemolishFx
                )
                .ok()
                .map(|demo| DemolishAttribute::Fx(**demo)),
                None => None,
            }))
    }

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
        let normalized_rigid_body = self.normalize_rigid_body(rigid_body);
        Ok(apply_velocities_to_rigid_body(
            &normalized_rigid_body,
            interpolation_amount,
        ))
    }

    /// Interpolates an arbitrary actor rigid body to the requested replay time.
    pub fn get_interpolated_actor_rigid_body(
        &self,
        actor_id: &boxcars::ActorId,
        time: f32,
        close_enough: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        let (frame_body, frame_index) = self.get_actor_rigid_body(actor_id)?;
        let frame_time = self.get_frame(*frame_index)?.time;
        let time_and_frame_difference = time - frame_time;

        if time_and_frame_difference.abs() <= close_enough.abs() {
            return Ok(self.normalize_rigid_body(frame_body));
        }

        let search_direction = if time_and_frame_difference > 0.0 {
            SearchDirection::Forward
        } else {
            SearchDirection::Backward
        };

        let object_id = self.get_object_id_for_key(RIGID_BODY_STATE_KEY)?;

        let (attribute, found_frame) =
            self.find_update_in_direction(*frame_index, actor_id, object_id, search_direction)?;
        let found_time = self.get_frame(found_frame)?.time;

        let found_body = attribute_match!(attribute, boxcars::Attribute::RigidBody)?;

        if (found_time - time).abs() <= close_enough {
            return Ok(self.normalize_rigid_body(&found_body));
        }

        let (start_body, start_time, end_body, end_time) = match search_direction {
            SearchDirection::Forward => (frame_body, frame_time, &found_body, found_time),
            SearchDirection::Backward => (&found_body, found_time, frame_body, frame_time),
        };
        let start_body = self.normalize_rigid_body(start_body);
        let end_body = self.normalize_rigid_body(end_body);

        get_interpolated_rigid_body(&start_body, start_time, &end_body, end_time, time)
    }

    /// Looks up the object id associated with a replay property name.
    pub fn get_object_id_for_key(
        &self,
        name: &'static str,
    ) -> SubtrActorResult<&boxcars::ObjectId> {
        self.name_to_object_id
            .get(name)
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::ObjectIdNotFound { name }))
    }

    /// Returns the actor ids currently associated with a named object type.
    pub fn get_actor_ids_by_type(
        &self,
        name: &'static str,
    ) -> SubtrActorResult<&[boxcars::ActorId]> {
        self.get_object_id_for_key(name)
            .map(|object_id| self.get_actor_ids_by_object_id(object_id))
    }

    pub(crate) fn get_actor_ids_by_object_id(
        &self,
        object_id: &boxcars::ObjectId,
    ) -> &[boxcars::ActorId] {
        self.actor_state
            .actor_ids_by_type
            .get(object_id)
            .map(|v| &v[..])
            .unwrap_or_else(|| &EMPTY_ACTOR_IDS)
    }

    /// Returns the current modeled state for an actor id.
    pub(crate) fn get_actor_state(
        &self,
        actor_id: &boxcars::ActorId,
    ) -> SubtrActorResult<&ActorState> {
        self.actor_state.actor_states.get(actor_id).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::NoStateForActorId {
                actor_id: *actor_id,
            })
        })
    }

    /// Returns current or recently deleted modeled state for an actor id.
    pub(crate) fn get_actor_state_or_recently_deleted(
        &self,
        actor_id: &boxcars::ActorId,
    ) -> SubtrActorResult<&ActorState> {
        self.actor_state
            .actor_states
            .get(actor_id)
            .or_else(|| self.actor_state.recently_deleted_actor_states.get(actor_id))
            .ok_or_else(|| {
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

    /// Reads a property from an actor or derived-attribute map by property name.
    pub fn get_attribute<'b>(
        &'b self,
        map: &'b HashMap<boxcars::ObjectId, (boxcars::Attribute, usize)>,
        property: &'static str,
    ) -> SubtrActorResult<&'b boxcars::Attribute> {
        self.get_attribute_and_updated(map, property).map(|v| &v.0)
    }

    /// Reads a property and the frame index when it was last updated.
    pub fn get_attribute_and_updated<'b>(
        &'b self,
        map: &'b HashMap<boxcars::ObjectId, (boxcars::Attribute, usize)>,
        property: &'static str,
    ) -> SubtrActorResult<&'b (boxcars::Attribute, usize)> {
        let attribute_object_id = self.get_object_id_for_key(property)?;
        map.get(attribute_object_id).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState { property })
        })
    }

    /// Scans the actor graph for the first actor that matches a known ball type.
    pub(crate) fn find_ball_actor(&self) -> Option<boxcars::ActorId> {
        BALL_TYPES
            .iter()
            .filter_map(|ball_type| self.iter_actors_by_type(ball_type))
            .flatten()
            .map(|(actor_id, _)| *actor_id)
            .next()
    }

    /// Returns the tracked actor id for the replay ball.
    pub fn get_ball_actor_id(&self) -> SubtrActorResult<boxcars::ActorId> {
        self.ball_actor_id.ok_or(SubtrActorError::new(
            SubtrActorErrorVariant::BallActorNotFound,
        ))
    }

    /// Returns the main game metadata actor id.
    pub fn get_metadata_actor_id(&self) -> SubtrActorResult<&boxcars::ActorId> {
        self.get_actor_ids_by_type(GAME_TYPE)?
            .iter()
            .next()
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::NoGameActor))
    }

    /// Returns the actor id associated with a player id.
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

    /// Returns the car actor id currently associated with a player.
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

    /// Resolves a player to a connected component actor through the supplied mapping.
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

    /// Returns the player's boost component actor id.
    pub fn get_boost_actor_id(&self, player_id: &PlayerId) -> SubtrActorResult<boxcars::ActorId> {
        self.get_car_connected_actor_id(player_id, &self.car_to_boost, "Boost")
    }

    /// Returns the player's jump component actor id.
    pub fn get_jump_actor_id(&self, player_id: &PlayerId) -> SubtrActorResult<boxcars::ActorId> {
        self.get_car_connected_actor_id(player_id, &self.car_to_jump, "Jump")
    }

    /// Returns the player's double-jump component actor id.
    pub fn get_double_jump_actor_id(
        &self,
        player_id: &PlayerId,
    ) -> SubtrActorResult<boxcars::ActorId> {
        self.get_car_connected_actor_id(player_id, &self.car_to_double_jump, "Double Jump")
    }

    /// Returns the player's dodge component actor id.
    pub fn get_dodge_actor_id(&self, player_id: &PlayerId) -> SubtrActorResult<boxcars::ActorId> {
        self.get_car_connected_actor_id(player_id, &self.car_to_dodge, "Dodge")
    }

    /// Returns an actor's rigid body together with the frame index of its last update.
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

    /// Like [`Self::get_actor_rigid_body`], but falls back to recently deleted actor state.
    pub fn get_actor_rigid_body_or_recently_deleted(
        &self,
        actor_id: &boxcars::ActorId,
    ) -> SubtrActorResult<(&boxcars::RigidBody, &usize)> {
        get_attribute_and_updated!(
            self,
            &self
                .get_actor_state_or_recently_deleted(actor_id)?
                .attributes,
            RIGID_BODY_STATE_KEY,
            boxcars::Attribute::RigidBody
        )
    }

    /// Iterates over players in the stable team-zero, then team-one ordering.
    pub fn iter_player_ids_in_order(&self) -> impl Iterator<Item = &PlayerId> {
        self.team_zero.iter().chain(self.team_one.iter())
    }

    /// Counts currently in-game players per team from live actor state.
    pub fn current_in_game_team_player_counts(&self) -> [usize; 2] {
        let mut counts = [0, 0];
        let Ok(player_actor_ids) = self.get_actor_ids_by_type(PLAYER_TYPE) else {
            return counts;
        };
        let mut seen_players = std::collections::HashSet::new();

        for actor_id in player_actor_ids {
            let Ok(player_id) = self.get_player_id_from_actor_id(actor_id) else {
                continue;
            };
            if !seen_players.insert(player_id) {
                continue;
            }

            let Some(team_actor_id) = self.player_to_team.get(actor_id) else {
                continue;
            };
            let Ok(team_state) = self.get_actor_state(team_actor_id) else {
                continue;
            };
            let Some(team_name) = self.object_id_to_name.get(&team_state.object_id) else {
                continue;
            };

            match team_name.chars().last() {
                Some('0') => counts[0] += 1,
                Some('1') => counts[1] += 1,
                _ => {}
            }
        }

        counts
    }

    /// Returns the number of players in the stored replay ordering.
    pub fn player_count(&self) -> usize {
        self.iter_player_ids_in_order().count()
    }

    /// Returns a map from player ids to their resolved display names.
    pub fn get_player_names(&self) -> HashMap<PlayerId, String> {
        self.iter_player_ids_in_order()
            .filter_map(|player_id| {
                self.get_player_name(player_id)
                    .ok()
                    .map(|name| (player_id.clone(), name))
            })
            .collect()
    }

    /// Iterates over actors of a named object type, returning an error if the type is unknown.
    pub(crate) fn iter_actors_by_type_err(
        &self,
        name: &'static str,
    ) -> SubtrActorResult<impl Iterator<Item = (&boxcars::ActorId, &ActorState)>> {
        Ok(self.iter_actors_by_object_id(self.get_object_id_for_key(name)?))
    }

    /// Iterates over actors of a named object type, if that type exists in the replay.
    pub fn iter_actors_by_type(
        &self,
        name: &'static str,
    ) -> Option<impl Iterator<Item = (&boxcars::ActorId, &ActorState)>> {
        self.iter_actors_by_type_err(name).ok()
    }

    /// Iterates over actors for a concrete object id.
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
            .map(move |id| (id, self.actor_state.actor_states.get(id).unwrap()))
    }

    /// Returns the replicated match clock in whole seconds.
    pub fn get_seconds_remaining(&self) -> SubtrActorResult<i32> {
        let seconds_remaining_object_id =
            self.cached_object_ids.seconds_remaining.ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::ObjectIdNotFound {
                    name: SECONDS_REMAINING_KEY,
                })
            })?;
        let metadata_actor_id = self.get_metadata_actor_id()?;
        let metadata_state = self.get_actor_state(metadata_actor_id)?;
        metadata_state
            .attributes
            .get(&seconds_remaining_object_id)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
                    property: SECONDS_REMAINING_KEY,
                })
            })
            .and_then(|(attribute, _)| attribute_match!(attribute, boxcars::Attribute::Int))
            .copied()
    }

    /// Returns the replicated game-state enum value from the metadata actor.
    pub fn get_replicated_state_name(&self) -> SubtrActorResult<i32> {
        get_actor_attribute_matching!(
            self,
            self.get_metadata_actor_id()?,
            REPLICATED_STATE_NAME_KEY,
            boxcars::Attribute::Int
        )
        .cloned()
    }

    /// Returns the replicated kickoff countdown / time-remaining field.
    pub fn get_replicated_game_state_time_remaining(&self) -> SubtrActorResult<i32> {
        get_actor_attribute_matching!(
            self,
            self.get_metadata_actor_id()?,
            REPLICATED_GAME_STATE_TIME_REMAINING_KEY,
            boxcars::Attribute::Int
        )
        .cloned()
    }

    /// Returns whether the replay currently reports that the ball has been hit.
    pub fn get_ball_has_been_hit(&self) -> SubtrActorResult<bool> {
        get_actor_attribute_matching!(
            self,
            self.get_metadata_actor_id()?,
            BALL_HAS_BEEN_HIT_KEY,
            boxcars::Attribute::Boolean
        )
        .cloned()
    }

    /// Returns the ball actor's ignore-syncing flag.
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

    /// Returns the current ball rigid body from live actor state.
    pub fn get_ball_rigid_body(&self) -> SubtrActorResult<&boxcars::RigidBody> {
        self.ball_actor_id
            .ok_or(SubtrActorError::new(
                SubtrActorErrorVariant::BallActorNotFound,
            ))
            .and_then(|actor_id| self.get_actor_rigid_body(&actor_id).map(|v| v.0))
    }

    /// Returns the current ball rigid body after spatial normalization.
    pub fn get_normalized_ball_rigid_body(&self) -> SubtrActorResult<boxcars::RigidBody> {
        self.get_ball_rigid_body()
            .map(|rigid_body| self.normalize_rigid_body(rigid_body))
    }

    /// Returns whether a non-sleeping ball rigid body is currently available.
    pub fn ball_rigid_body_exists(&self) -> SubtrActorResult<bool> {
        Ok(self
            .get_ball_rigid_body()
            .map(|rb| !rb.sleeping)
            .unwrap_or(false))
    }

    /// Returns the current ball rigid body and the frame where it was last updated.
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

    /// Applies stored ball velocity forward to the requested time.
    pub fn get_velocity_applied_ball_rigid_body(
        &self,
        target_time: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        let (current_rigid_body, frame_index) = self.get_ball_rigid_body_and_updated()?;
        self.velocities_applied_rigid_body(current_rigid_body, *frame_index, target_time)
    }

    /// Interpolates the ball rigid body to the requested time.
    pub fn get_interpolated_ball_rigid_body(
        &self,
        time: f32,
        close_enough: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        self.get_interpolated_actor_rigid_body(&self.get_ball_actor_id()?, time, close_enough)
    }

    /// Returns the player's replicated display name.
    pub fn get_player_name(&self, player_id: &PlayerId) -> SubtrActorResult<String> {
        get_actor_attribute_matching!(
            self,
            &self.get_player_actor_id(player_id)?,
            PLAYER_NAME_KEY,
            boxcars::Attribute::String
        )
        .cloned()
    }

    fn get_player_int_stat(
        &self,
        player_id: &PlayerId,
        key: &'static str,
    ) -> SubtrActorResult<i32> {
        get_actor_attribute_matching!(
            self,
            &self.get_player_actor_id(player_id)?,
            key,
            boxcars::Attribute::Int
        )
        .cloned()
    }

    /// Returns the replay object-name key for the player's team actor.
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

    /// Returns whether the player belongs to team 0.
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

    /// Returns the team actor id for the requested side.
    pub(crate) fn get_team_actor_id_for_side(
        &self,
        is_team_0: bool,
    ) -> SubtrActorResult<boxcars::ActorId> {
        let player_id = if is_team_0 {
            self.team_zero.first()
        } else {
            self.team_one.first()
        }
        .ok_or(SubtrActorError::new(SubtrActorErrorVariant::NoGameActor))?;

        self.player_to_team
            .get(&self.get_player_actor_id(player_id)?)
            .copied()
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::ActorNotFound {
                    name: "Team",
                    player_id: player_id.clone(),
                })
            })
    }

    /// Returns the score for the requested team side.
    pub fn get_team_score(&self, is_team_0: bool) -> SubtrActorResult<i32> {
        let team_actor_id = self.get_team_actor_id_for_side(is_team_0)?;
        get_actor_attribute_matching!(
            self,
            &team_actor_id,
            TEAM_GAME_SCORE_KEY,
            boxcars::Attribute::Int
        )
        .cloned()
    }

    /// Returns `(team_zero_score, team_one_score)`.
    pub fn get_team_scores(&self) -> SubtrActorResult<(i32, i32)> {
        Ok((self.get_team_score(true)?, self.get_team_score(false)?))
    }

    /// Returns the player's current car rigid body.
    pub fn get_player_rigid_body(
        &self,
        player_id: &PlayerId,
    ) -> SubtrActorResult<&boxcars::RigidBody> {
        self.get_car_actor_id(player_id)
            .and_then(|actor_id| self.get_actor_rigid_body(&actor_id).map(|v| v.0))
    }

    /// Returns the player's current car rigid body after spatial normalization.
    pub fn get_normalized_player_rigid_body(
        &self,
        player_id: &PlayerId,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        self.get_player_rigid_body(player_id)
            .map(|rigid_body| self.normalize_rigid_body(rigid_body))
    }

    /// Returns the player's rigid body and the frame where it was last updated.
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

    /// Like [`Self::get_player_rigid_body_and_updated`], but can use recently deleted state.
    pub fn get_player_rigid_body_and_updated_or_recently_deleted(
        &self,
        player_id: &PlayerId,
    ) -> SubtrActorResult<(&boxcars::RigidBody, &usize)> {
        self.get_car_actor_id(player_id)
            .and_then(|actor_id| self.get_actor_rigid_body_or_recently_deleted(&actor_id))
    }

    /// Applies stored player velocity forward to the requested time.
    pub fn get_velocity_applied_player_rigid_body(
        &self,
        player_id: &PlayerId,
        target_time: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        let (current_rigid_body, frame_index) =
            self.get_player_rigid_body_and_updated(player_id)?;
        self.velocities_applied_rigid_body(current_rigid_body, *frame_index, target_time)
    }

    /// Interpolates the player's car rigid body to the requested time.
    pub fn get_interpolated_player_rigid_body(
        &self,
        player_id: &PlayerId,
        time: f32,
        close_enough: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        self.get_car_actor_id(player_id).and_then(|car_actor_id| {
            self.get_interpolated_actor_rigid_body(&car_actor_id, time, close_enough)
        })
    }

    /// Returns the player's current boost amount in raw replay units.
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

    /// Returns the previous boost amount recorded for the player in raw replay units.
    pub fn get_player_last_boost_level(&self, player_id: &PlayerId) -> SubtrActorResult<f32> {
        self.get_boost_actor_id(player_id).and_then(|actor_id| {
            let boost_state = self.get_actor_state(&actor_id)?;
            get_derived_attribute!(
                boost_state.derived_attributes,
                LAST_BOOST_AMOUNT_KEY,
                boxcars::Attribute::Byte
            )
            .map(|value| *value as f32)
        })
    }

    /// Returns the player's boost level scaled to the conventional 0.0-100.0 range.
    pub fn get_player_boost_percentage(&self, player_id: &PlayerId) -> SubtrActorResult<f32> {
        self.get_player_boost_level(player_id)
            .map(boost_amount_to_percent)
    }

    /// Returns the player's match assists counter.
    pub fn get_player_match_assists(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        self.get_player_int_stat(player_id, MATCH_ASSISTS_KEY)
    }

    /// Returns the player's match goals counter.
    pub fn get_player_match_goals(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        self.get_player_int_stat(player_id, MATCH_GOALS_KEY)
    }

    /// Returns the player's match saves counter.
    pub fn get_player_match_saves(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        self.get_player_int_stat(player_id, MATCH_SAVES_KEY)
    }

    /// Returns the player's match score counter.
    pub fn get_player_match_score(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        self.get_player_int_stat(player_id, MATCH_SCORE_KEY)
    }

    /// Returns the player's match shots counter.
    pub fn get_player_match_shots(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        self.get_player_int_stat(player_id, MATCH_SHOTS_KEY)
    }

    /// Returns the team number recorded as the last ball-touching side.
    pub fn get_ball_hit_team_num(&self) -> SubtrActorResult<u8> {
        let ball_actor_id = self.get_ball_actor_id()?;
        get_actor_attribute_matching!(
            self,
            &ball_actor_id,
            BALL_HIT_TEAM_NUM_KEY,
            boxcars::Attribute::Byte
        )
        .cloned()
    }

    /// Returns the team number currently marked as having been scored on.
    pub fn get_scored_on_team_num(&self) -> SubtrActorResult<u8> {
        get_actor_attribute_matching!(
            self,
            self.get_metadata_actor_id()?,
            REPLICATED_SCORED_ON_TEAM_KEY,
            boxcars::Attribute::Byte
        )
        .cloned()
    }

    /// Returns a component actor's active byte.
    pub fn get_component_active(&self, actor_id: &boxcars::ActorId) -> SubtrActorResult<u8> {
        get_actor_attribute_matching!(
            self,
            &actor_id,
            COMPONENT_ACTIVE_KEY,
            boxcars::Attribute::Byte
        )
        .cloned()
    }

    /// Returns the active byte for the player's boost component.
    pub fn get_boost_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        self.get_boost_actor_id(player_id)
            .and_then(|actor_id| self.get_component_active(&actor_id))
    }

    /// Returns the active byte for the player's jump component.
    pub fn get_jump_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        self.get_jump_actor_id(player_id)
            .and_then(|actor_id| self.get_component_active(&actor_id))
    }

    /// Returns the active byte for the player's double-jump component.
    pub fn get_double_jump_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        self.get_double_jump_actor_id(player_id)
            .and_then(|actor_id| self.get_component_active(&actor_id))
    }

    /// Returns the active byte for the player's dodge component.
    pub fn get_dodge_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        self.get_dodge_actor_id(player_id)
            .and_then(|actor_id| self.get_component_active(&actor_id))
    }

    /// Returns whether the player's handbrake / powerslide flag is active.
    pub fn get_powerslide_active(&self, player_id: &PlayerId) -> SubtrActorResult<bool> {
        get_actor_attribute_matching!(
            self,
            &self.get_car_actor_id(player_id)?,
            HANDBRAKE_KEY,
            boxcars::Attribute::Boolean
        )
        .cloned()
    }
}
