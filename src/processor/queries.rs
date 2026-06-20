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
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::NoNetworkFrames))?;
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

    /// Returns the standard Soccar boost pad layout annotated with replay pad ids when known.
    ///
    /// This is incremental reconstruction state and should only be materialized by
    /// final replay-data assembly after the processor has completed its replay pass.
    pub(crate) fn resolved_boost_pads(&self) -> Vec<ResolvedBoostPad> {
        self.boost_pad_resolution.resolved_boost_pads()
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
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::NoNetworkFrames))?
            .frames
            .get(frame_index)
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::FrameIndexOutOfBounds))
    }

    pub(crate) fn velocities_applied_rigid_body(
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

    fn get_actor_object_name(&self, actor_id: &boxcars::ActorId) -> Option<String> {
        self.actor_state
            .actor_states
            .get(actor_id)
            .and_then(|state| self.object_id_to_name.get(&state.object_id))
            .cloned()
            .or_else(|| {
                usize::try_from(actor_id.0)
                    .ok()
                    .and_then(|object_index| self.replay.objects.get(object_index))
                    .cloned()
            })
    }

    fn get_first_attribute_by_object_id(
        &self,
        object_id: Option<boxcars::ObjectId>,
    ) -> Option<&boxcars::Attribute> {
        let object_id = object_id?;
        self.actor_state.actor_states.values().find_map(|state| {
            state
                .attributes
                .get(&object_id)
                .map(|(attribute, _)| attribute)
        })
    }

    /// Returns the replicated Rocket League playlist id, when present in network data.
    pub fn get_replicated_game_playlist(&self) -> Option<i32> {
        match self.get_first_attribute_by_object_id(self.cached_object_ids.replicated_game_playlist)
        {
            Some(boxcars::Attribute::Int(playlist_id)) => Some(*playlist_id),
            _ => None,
        }
    }

    /// Returns the resolved match-type class object name, when present in network data.
    pub fn get_match_type_class(&self) -> Option<String> {
        match self.get_first_attribute_by_object_id(self.cached_object_ids.match_type_class) {
            Some(boxcars::Attribute::ActiveActor(active_actor)) => {
                self.get_actor_object_name(&active_actor.actor)
            }
            _ => None,
        }
    }

    /// Returns the best known normalized game-type metadata.
    pub fn get_replay_game_type_details(&self) -> ReplayGameTypeDetails {
        self.game_type_details.clone()
    }

    pub(crate) fn update_game_type_details(&mut self) {
        self.game_type_details = self.game_type_details.with_network_signals(
            self.get_replicated_game_playlist(),
            self.get_match_type_class(),
        );
    }

    pub(crate) fn get_actor_attribute<'b>(
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

    /// Scans the actor graph for the first actor whose archetype lives under
    /// `Archetypes.Ball.`. Matching the prefix rather than an explicit whitelist
    /// means ball archetypes introduced by new or limited-time game modes are
    /// recognized automatically.
    pub(crate) fn find_ball_actor(&self) -> Option<boxcars::ActorId> {
        self.actor_state
            .actor_ids_by_type
            .iter()
            .filter(|(object_id, _)| {
                self.object_id_to_name
                    .get(object_id)
                    .is_some_and(|name| name.starts_with(BALL_TYPE_PREFIX))
            })
            .flat_map(|(_, actor_ids)| actor_ids.iter().copied())
            .next()
    }

    /// Returns the tracked actor id for the replay ball.
    pub fn get_ball_actor_id(&self) -> SubtrActorResult<boxcars::ActorId> {
        self.ball_actor_id
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::BallActorNotFound))
    }

    /// Returns the main game metadata actor id.
    pub fn get_metadata_actor_id(&self) -> SubtrActorResult<boxcars::ActorId> {
        if let Ok(actor_ids) = self.get_actor_ids_by_type(GAME_TYPE) {
            if let Some(actor_id) = actor_ids.first() {
                return Ok(*actor_id);
            }
        }

        let metadata_object_ids = [
            self.cached_object_ids.seconds_remaining,
            self.cached_object_ids.replicated_state_name,
            self.cached_object_ids.replicated_game_state_time_remaining,
            self.cached_object_ids.ball_has_been_hit,
        ];

        self.actor_state
            .actor_states
            .iter()
            .filter_map(|(actor_id, actor_state)| {
                let metadata_attribute_count = metadata_object_ids
                    .iter()
                    .flatten()
                    .filter(|object_id| actor_state.attributes.contains_key(object_id))
                    .count();
                (metadata_attribute_count > 0).then_some((
                    metadata_attribute_count,
                    std::cmp::Reverse(*actor_id),
                    *actor_id,
                ))
            })
            .max()
            .map(|(_, _, actor_id)| actor_id)
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
        let Ok(player_actor_ids) = self.get_player_type_actor_ids() else {
            return counts;
        };
        let mut seen_players = std::collections::HashSet::new();

        for actor_id in &player_actor_ids {
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
}
