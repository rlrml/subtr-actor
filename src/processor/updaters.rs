use super::*;

impl<'a> ReplayProcessor<'a> {
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
    /// - `car_to_player`: maps a car's actor ID to the player's actor ID (persists after car destruction).
    /// - `car_to_boost`: maps a car's actor ID to its associated boost actor ID.
    /// - `car_to_dodge`: maps a car's actor ID to its associated dodge actor ID.
    /// - `car_to_jump`: maps a car's actor ID to its associated jump actor ID.
    /// - `car_to_double_jump`: maps a car's actor ID to its associated double jump actor ID.
    ///
    /// Some links support an optional *skip value*: when the update's value equals the
    /// skip value, the map is not updated. This is used for `car_to_player` with skip
    /// value [`ActorId(-1)`](boxcars::ActorId). On demolition frames the replay can set
    /// the victim car's `Engine.Pawn:PlayerReplicationInfo` link to `-1`; if we applied
    /// that update we would overwrite the existing car-to-player mapping and lose the
    /// victim's identity when building demolish info. Skipping the `-1` update keeps
    /// the last valid mapping so victim lookup still succeeds.
    ///
    /// Be careful with directionality here: `player_to_car` is `player actor -> car
    /// actor`, while `car_to_player` must remain `car actor -> player actor`. Demolish
    /// payloads resolve through `get_player_id_from_car_id`, so reversing `car_to_player`
    /// breaks demolition extraction even when the replay contains valid demolish events.
    ///
    /// The function also handles the deletion of actors. When an actor is
    /// deleted, the function removes the actor's ID from the `player_to_car`
    /// mapping.
    pub(crate) fn update_mappings(&mut self, frame: &boxcars::Frame) -> SubtrActorResult<()> {
        let player_type_actor_ids = self.get_actor_ids_by_type(PLAYER_TYPE)?.to_vec();
        let car_type_actor_ids = self.get_actor_ids_by_type(CAR_TYPE)?.to_vec();
        let boost_type_actor_ids = self.get_actor_ids_by_type(BOOST_TYPE)?.to_vec();
        let dodge_type_actor_ids = self.get_actor_ids_by_type(DODGE_TYPE)?.to_vec();
        let jump_type_actor_ids = self.get_actor_ids_by_type(JUMP_TYPE)?.to_vec();
        let double_jump_type_actor_ids = self.get_actor_ids_by_type(DOUBLE_JUMP_TYPE)?.to_vec();
        let unique_id_object_id = *self.get_object_id_for_key(UNIQUE_ID_KEY)?;
        let team_object_id = *self.get_object_id_for_key(TEAM_KEY)?;
        let player_replication_object_id = *self.get_object_id_for_key(PLAYER_REPLICATION_KEY)?;
        let vehicle_object_id = *self.get_object_id_for_key(VEHICLE_KEY)?;

        for update in frame.updated_actors.iter() {
            macro_rules! maintain_link {
                ($map:expr, $actor_ids:expr, $object_id:expr, $get_key:expr, $get_value:expr, $type:path $(, skip_value $skip:expr)?) => {{
                    if update.object_id == $object_id && $actor_ids.contains(&update.actor_id) {
                        let value = attribute_match!(&update.attribute, $type)?;
                        let _key = $get_key(update.actor_id, value);
                        let _new_value = $get_value(update.actor_id, value);
                        if true $(&& _new_value != $skip)? {
                            let _ = $map.insert(_key, _new_value);
                        }
                    }
                }};
            }
            macro_rules! maintain_actor_link {
                ($map:expr, $actor_ids:expr, $object_id:expr $(, skip_value $skip:expr)?) => {
                    maintain_link!(
                        $map,
                        $actor_ids,
                        $object_id,
                        // This is slightly confusing, but in these cases we are
                        // using the attribute as the key to the current actor.
                        get_actor_id_from_active_actor,
                        use_update_actor,
                        boxcars::Attribute::ActiveActor
                        $(, skip_value $skip)?
                    )
                };
            }
            macro_rules! maintain_vehicle_key_link {
                ($map:expr, $actor_ids:expr) => {
                    maintain_actor_link!($map, $actor_ids, vehicle_object_id)
                };
            }
            maintain_link!(
                self.player_to_actor_id,
                player_type_actor_ids,
                unique_id_object_id,
                |_, unique_id: &boxcars::UniqueId| unique_id.remote_id.clone(),
                use_update_actor,
                boxcars::Attribute::UniqueId
            );
            maintain_link!(
                self.player_to_team,
                player_type_actor_ids,
                team_object_id,
                // In this case we are using the update actor as the key.
                use_update_actor,
                get_actor_id_from_active_actor,
                boxcars::Attribute::ActiveActor,
                skip_value boxcars::ActorId(-1)
            );
            maintain_actor_link!(
                self.player_to_car,
                car_type_actor_ids,
                player_replication_object_id
            );
            // `car_to_player` is intentionally the reverse of `player_to_car`:
            // key = car actor, value = player actor. We still skip `ActorId(-1)`
            // so same-frame demolition cleanup does not erase the last valid owner.
            maintain_link!(
                self.car_to_player,
                car_type_actor_ids,
                player_replication_object_id,
                use_update_actor,
                get_actor_id_from_active_actor,
                boxcars::Attribute::ActiveActor,
                skip_value boxcars::ActorId(-1)
            );
            maintain_vehicle_key_link!(self.car_to_boost, boost_type_actor_ids);
            maintain_vehicle_key_link!(self.car_to_dodge, dodge_type_actor_ids);
            maintain_vehicle_key_link!(self.car_to_jump, jump_type_actor_ids);
            maintain_vehicle_key_link!(self.car_to_double_jump, double_jump_type_actor_ids);
        }

        for actor_id in frame.deleted_actors.iter() {
            if let Some(car_id) = self.player_to_car.remove(actor_id) {
                log::info!("Player actor {actor_id:?} deleted, car id: {car_id:?}.");
            }
        }

        Ok(())
    }

    pub(crate) fn update_ball_id(&mut self, frame: &boxcars::Frame) -> SubtrActorResult<()> {
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

    pub(crate) fn update_boost_amounts(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        let kickoff_phase_active = self.kickoff_phase_active();
        let kickoff_phase_started = kickoff_phase_active && !self.kickoff_phase_active_last_frame;
        let boost_replicated_object_id = self.name_to_object_id.get(BOOST_REPLICATED_KEY).copied();
        let boost_amount_object_id = self.name_to_object_id.get(BOOST_AMOUNT_KEY).copied();
        let component_active_object_id = self.name_to_object_id.get(COMPONENT_ACTIVE_KEY).copied();
        let updates: Vec<_> = self
            .iter_actors_by_type_err(BOOST_TYPE)?
            .map(|(actor_id, actor_state)| {
                let (actor_amount_value, last_value, _, derived_value, is_active) =
                    Self::get_current_boost_values(
                        actor_state,
                        boost_replicated_object_id,
                        boost_amount_object_id,
                        component_active_object_id,
                    );
                let mut current_value = if kickoff_phase_started {
                    BOOST_KICKOFF_START_AMOUNT
                } else if actor_amount_value == last_value {
                    derived_value
                } else {
                    actor_amount_value.into()
                };
                if is_active {
                    current_value -= frame.delta * BOOST_USED_RAW_UNITS_PER_SECOND;
                }
                (*actor_id, current_value.max(0.0), actor_amount_value)
            })
            .collect();

        for (actor_id, current_value, new_last_value) in updates {
            let derived_attributes = &mut self
                .actor_state
                .actor_states
                .get_mut(&actor_id)
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
        self.kickoff_phase_active_last_frame = kickoff_phase_active;
        Ok(())
    }

    fn kickoff_phase_active(&self) -> bool {
        self.get_replicated_state_name().ok() == Some(55)
            || self
                .get_replicated_game_state_time_remaining()
                .ok()
                .is_some_and(|countdown| countdown > 0)
            || self.get_ball_has_been_hit().ok() == Some(false)
    }

    fn get_current_boost_values(
        actor_state: &ActorState,
        boost_replicated_object_id: Option<boxcars::ObjectId>,
        boost_amount_object_id: Option<boxcars::ObjectId>,
        component_active_object_id: Option<boxcars::ObjectId>,
    ) -> (u8, u8, u8, f32, bool) {
        let amount_value = boost_replicated_object_id
            .and_then(|object_id| actor_state.attributes.get(&object_id))
            .and_then(|(attribute, _)| match attribute {
                boxcars::Attribute::ReplicatedBoost(replicated_boost) => {
                    Some(replicated_boost.boost_amount)
                }
                _ => None,
            })
            .or_else(|| {
                boost_amount_object_id
                    .and_then(|object_id| actor_state.attributes.get(&object_id))
                    .and_then(|(attribute, _)| match attribute {
                        boxcars::Attribute::Byte(value) => Some(*value),
                        _ => None,
                    })
            })
            .unwrap_or(0);
        let active_value = component_active_object_id
            .and_then(|object_id| actor_state.attributes.get(&object_id))
            .and_then(|(attribute, _)| match attribute {
                boxcars::Attribute::Byte(value) => Some(*value),
                _ => None,
            })
            .unwrap_or(0);
        let is_active = active_value % 2 == 1;
        let derived_value = actor_state
            .derived_attributes
            .get(BOOST_AMOUNT_KEY)
            .and_then(|(attribute, _)| match attribute {
                boxcars::Attribute::Float(value) => Some(*value),
                _ => None,
            })
            .unwrap_or(0.0);
        let last_boost_amount = actor_state
            .derived_attributes
            .get(LAST_BOOST_AMOUNT_KEY)
            .and_then(|(attribute, _)| match attribute {
                boxcars::Attribute::Byte(value) => Some(*value),
                _ => None,
            })
            .unwrap_or(amount_value);
        (
            amount_value,
            last_boost_amount,
            active_value,
            derived_value,
            is_active,
        )
    }

    pub(crate) fn update_demolishes(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        if self.demolish_format.is_none() {
            self.demolish_format = self.detect_demolish_format();
        }

        let new_demolishes: Vec<_> = self.get_active_demos()?.collect();

        for demolish in new_demolishes {
            self.try_push_demolish(&demolish, frame, frame_index);
        }

        for update in &frame.updated_actors {
            let demolish = match &update.attribute {
                boxcars::Attribute::DemolishExtended(d) => {
                    self.demolish_format = Some(DemolishFormat::Extended);
                    Some(DemolishAttribute::Extended(**d))
                }
                boxcars::Attribute::DemolishFx(d) => {
                    self.demolish_format = Some(DemolishFormat::Fx);
                    Some(DemolishAttribute::Fx(**d))
                }
                _ => None,
            };
            if let Some(demolish) = demolish {
                self.try_push_demolish(&demolish, frame, frame_index);
            }
        }

        Ok(())
    }

    fn actor_is_boost_pad(&self, actor_id: &boxcars::ActorId) -> bool {
        self.get_actor_state_or_recently_deleted(actor_id)
            .ok()
            .and_then(|state| self.object_id_to_name.get(&state.object_id))
            .map(|name: &String| name.contains("VehiclePickup_Boost_TA"))
            .unwrap_or(false)
    }

    fn get_actor_instance_name(&self, actor_id: &boxcars::ActorId) -> SubtrActorResult<String> {
        let state = self.get_actor_state_or_recently_deleted(actor_id)?;
        if let Some(name_id) = state.name_id {
            if let Some(name) = self.replay.names.get(name_id as usize) {
                return Ok(name.clone());
            }
        }
        self.object_id_to_name
            .get(&state.object_id)
            .cloned()
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::NoStateForActorId {
                    actor_id: *actor_id,
                })
            })
    }

    pub(crate) fn update_boost_pad_events(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        self.current_frame_boost_pad_events.clear();

        for update in &frame.updated_actors {
            if !self.actor_is_boost_pad(&update.actor_id) {
                continue;
            }

            let Some(event) = (match &update.attribute {
                boxcars::Attribute::PickupNew(pickup) => {
                    let pad_id = self.get_actor_instance_name(&update.actor_id)?;
                    if let Some(instigator) = pickup.instigator {
                        if instigator.0 >= 0 && pickup.picked_up != u8::MAX {
                            Some(BoostPadEvent {
                                time: frame.time,
                                frame: frame_index,
                                pad_id,
                                player: self.get_player_id_from_car_id(&instigator).ok(),
                                kind: BoostPadEventKind::PickedUp {
                                    sequence: pickup.picked_up,
                                },
                            })
                        } else {
                            None
                        }
                    } else if pickup.picked_up == u8::MAX {
                        Some(BoostPadEvent {
                            time: frame.time,
                            frame: frame_index,
                            pad_id,
                            player: None,
                            kind: BoostPadEventKind::Available,
                        })
                    } else {
                        None
                    }
                }
                boxcars::Attribute::Pickup(pickup) => {
                    let pad_id = self.get_actor_instance_name(&update.actor_id)?;
                    if let Some(instigator) = pickup.instigator {
                        if instigator.0 >= 0 && pickup.picked_up {
                            Some(BoostPadEvent {
                                time: frame.time,
                                frame: frame_index,
                                pad_id,
                                player: self.get_player_id_from_car_id(&instigator).ok(),
                                kind: BoostPadEventKind::PickedUp { sequence: 1 },
                            })
                        } else {
                            None
                        }
                    } else if !pickup.picked_up {
                        Some(BoostPadEvent {
                            time: frame.time,
                            frame: frame_index,
                            pad_id,
                            player: None,
                            kind: BoostPadEventKind::Available,
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            }) else {
                continue;
            };

            self.current_frame_boost_pad_events.push(event.clone());
            self.boost_pad_events.push(event);
        }

        Ok(())
    }

    fn estimate_touching_player(
        &self,
        touch_team_is_team_0: bool,
        target_time: f32,
    ) -> Option<(PlayerId, f32)> {
        const TOUCH_PLAYER_DISTANCE_THRESHOLD: f32 = 700.0;

        let ball_rigid_body = self
            .get_velocity_applied_ball_rigid_body(target_time)
            .ok()?;
        self.iter_player_ids_in_order()
            .filter(|player_id| {
                self.get_player_is_team_0(player_id).ok() == Some(touch_team_is_team_0)
            })
            .filter_map(|player_id| {
                self.get_velocity_applied_player_rigid_body(player_id, target_time)
                    .ok()
                    .and_then(|rigid_body| {
                        touch_candidate_rank(&ball_rigid_body, &rigid_body)
                            .map(|rank| (player_id.clone(), rank))
                    })
            })
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .and_then(|(player_id, (closest_distance, _current_distance))| {
                (closest_distance <= TOUCH_PLAYER_DISTANCE_THRESHOLD)
                    .then_some((player_id, closest_distance))
            })
    }

    pub(crate) fn update_touch_events(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        self.current_frame_touch_events.clear();
        let hit_team_num_key = *self.get_object_id_for_key(BALL_HIT_TEAM_NUM_KEY)?;

        for update in &frame.updated_actors {
            if update.object_id != hit_team_num_key {
                continue;
            }

            let boxcars::Attribute::Byte(team_num) = update.attribute else {
                continue;
            };
            let team_is_team_0 = match team_num {
                0 => true,
                1 => false,
                _ => continue,
            };
            let estimated_player = self.estimate_touching_player(team_is_team_0, frame.time);
            let event = TouchEvent {
                time: frame.time,
                frame: frame_index,
                team_is_team_0,
                player: estimated_player.as_ref().map(|(player, _)| player.clone()),
                closest_approach_distance: estimated_player.map(|(_, distance)| distance),
            };
            self.current_frame_touch_events.push(event.clone());
            self.touch_events.push(event);
        }

        Ok(())
    }

    pub(crate) fn update_dodge_refreshed_events(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        self.current_frame_dodge_refreshed_events.clear();
        let dodges_refreshed_counter_key = self
            .get_object_id_for_key(DODGES_REFRESHED_COUNTER_KEY)
            .ok()
            .copied();

        let Some(dodges_refreshed_counter_key) = dodges_refreshed_counter_key else {
            return Ok(());
        };

        for update in &frame.updated_actors {
            if update.object_id != dodges_refreshed_counter_key {
                continue;
            }
            let boxcars::Attribute::Int(counter_value) = update.attribute else {
                continue;
            };
            let Some(player_id) = self.get_player_id_from_car_id(&update.actor_id).ok() else {
                continue;
            };
            let previous_value = self
                .dodge_refreshed_counters
                .get(&player_id)
                .copied()
                .unwrap_or(counter_value);
            self.dodge_refreshed_counters
                .insert(player_id.clone(), counter_value);
            let delta = counter_value - previous_value;
            if delta <= 0 {
                continue;
            }

            let is_team_0 = self.get_player_is_team_0(&player_id).unwrap_or(false);
            for offset in 0..delta {
                let event = DodgeRefreshedEvent {
                    time: frame.time,
                    frame: frame_index,
                    player: player_id.clone(),
                    is_team_0,
                    counter_value: previous_value + offset + 1,
                };
                self.current_frame_dodge_refreshed_events
                    .push(event.clone());
                self.dodge_refreshed_events.push(event);
            }
        }

        Ok(())
    }

    pub(crate) fn update_goal_events(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        self.current_frame_goal_events.clear();

        let ball_explosion_data = self
            .get_object_id_for_key(BALL_EXPLOSION_DATA_KEY)
            .ok()
            .copied();
        let ball_explosion_data_extended = self
            .get_object_id_for_key(BALL_EXPLOSION_DATA_EXTENDED_KEY)
            .ok()
            .copied();

        for update in &frame.updated_actors {
            let is_ball_goal_explosion = matches!(
                &update.attribute,
                boxcars::Attribute::Explosion(_) | boxcars::Attribute::ExtendedExplosion(_)
            ) && (ball_explosion_data == Some(update.object_id)
                || ball_explosion_data_extended == Some(update.object_id));

            if !is_ball_goal_explosion {
                continue;
            }

            let score_updates = self.goal_score_updates_from_frame(frame);
            let scoring_team_is_team_0 = self
                .scoring_team_from_score_updates(score_updates)
                .or_else(|| match self.get_scored_on_team_num() {
                    Ok(0) => Some(false),
                    Ok(1) => Some(true),
                    _ => None,
                });
            let observed_scores = self
                .goal_score_tuple_from_frame(frame)
                .or_else(|| self.get_team_scores().ok());
            let scorer = scoring_team_is_team_0.and_then(|team_is_team_0| {
                self.goal_scorer_from_update(update, frame, team_is_team_0)
            });

            if self.goal_event_is_duplicate(
                frame.time,
                scoring_team_is_team_0.unwrap_or(false),
                observed_scores.map(|scores| scores.0),
                observed_scores.map(|scores| scores.1),
            ) {
                continue;
            }
            let Some(scoring_team_is_team_0) = scoring_team_is_team_0 else {
                continue;
            };
            let (team_zero_score, team_one_score) = observed_scores.map_or_else(
                || self.derived_goal_score_tuple(scoring_team_is_team_0),
                |(team_zero, team_one)| (Some(team_zero), Some(team_one)),
            );

            let event = GoalEvent {
                time: frame.time,
                frame: frame_index,
                scoring_team_is_team_0,
                player: scorer,
                team_zero_score,
                team_one_score,
            };
            self.current_frame_goal_events.push(event.clone());
            self.goal_events.push(event);
        }

        Ok(())
    }

    fn goal_event_is_duplicate(
        &self,
        frame_time: f32,
        scoring_team_is_team_0: bool,
        team_zero_score: Option<i32>,
        team_one_score: Option<i32>,
    ) -> bool {
        const GOAL_EVENT_DEDUPE_WINDOW_SECONDS: f32 = 3.0;

        self.goal_events
            .last()
            .map(|event| {
                match (
                    team_zero_score,
                    team_one_score,
                    event.team_zero_score,
                    event.team_one_score,
                ) {
                    (
                        Some(team_zero),
                        Some(team_one),
                        Some(prev_team_zero),
                        Some(prev_team_one),
                    ) => team_zero == prev_team_zero && team_one == prev_team_one,
                    _ => {
                        event.scoring_team_is_team_0 == scoring_team_is_team_0
                            && (frame_time - event.time).abs() <= GOAL_EVENT_DEDUPE_WINDOW_SECONDS
                    }
                }
            })
            .unwrap_or(false)
    }

    fn derived_goal_score_tuple(&self, scoring_team_is_team_0: bool) -> (Option<i32>, Option<i32>) {
        let (mut team_zero_goals, mut team_one_goals) = self.last_known_goal_score_tuple();
        if scoring_team_is_team_0 {
            team_zero_goals += 1;
        } else {
            team_one_goals += 1;
        }
        (Some(team_zero_goals), Some(team_one_goals))
    }

    fn last_known_goal_score_tuple(&self) -> (i32, i32) {
        self.goal_events
            .last()
            .and_then(|event| event.team_zero_score.zip(event.team_one_score))
            .unwrap_or((0, 0))
    }

    fn goal_score_updates_from_frame(
        &self,
        frame: &boxcars::Frame,
    ) -> Option<(Option<i32>, Option<i32>)> {
        let team_zero_actor_id = self.get_team_actor_id_for_side(true).ok()?;
        let team_one_actor_id = self.get_team_actor_id_for_side(false).ok()?;
        let team_game_score_key = self
            .get_object_id_for_key(TEAM_GAME_SCORE_KEY)
            .ok()
            .copied();
        let team_info_score_key = self
            .get_object_id_for_key(TEAM_INFO_SCORE_KEY)
            .ok()
            .copied();
        let mut team_zero_score = None;
        let mut team_one_score = None;

        for update in &frame.updated_actors {
            let is_score_update = Some(update.object_id) == team_game_score_key
                || Some(update.object_id) == team_info_score_key;
            if !is_score_update {
                continue;
            }
            let boxcars::Attribute::Int(score) = update.attribute else {
                continue;
            };
            if update.actor_id == team_zero_actor_id {
                team_zero_score = Some(score);
            } else if update.actor_id == team_one_actor_id {
                team_one_score = Some(score);
            }
        }

        (team_zero_score.is_some() || team_one_score.is_some())
            .then_some((team_zero_score, team_one_score))
    }

    fn scoring_team_from_score_updates(
        &self,
        score_updates: Option<(Option<i32>, Option<i32>)>,
    ) -> Option<bool> {
        let (team_zero_score, team_one_score) = score_updates?;
        let (previous_team_zero, previous_team_one) = self.last_known_goal_score_tuple();

        match (team_zero_score, team_one_score) {
            (Some(team_zero), Some(team_one))
                if team_zero == previous_team_zero + 1 && team_one == previous_team_one =>
            {
                Some(true)
            }
            (Some(team_zero), Some(team_one))
                if team_zero == previous_team_zero && team_one == previous_team_one + 1 =>
            {
                Some(false)
            }
            (Some(team_zero), _) if team_zero == previous_team_zero + 1 => Some(true),
            (_, Some(team_one)) if team_one == previous_team_one + 1 => Some(false),
            _ => None,
        }
    }

    fn goal_score_tuple_from_frame(&self, frame: &boxcars::Frame) -> Option<(i32, i32)> {
        let (previous_team_zero, previous_team_one) = self.last_known_goal_score_tuple();
        let (team_zero_score, team_one_score) = self.goal_score_updates_from_frame(frame)?;

        Some((
            team_zero_score.unwrap_or(previous_team_zero),
            team_one_score.unwrap_or(previous_team_one),
        ))
    }

    fn goal_scorer_from_update(
        &self,
        goal_update: &boxcars::UpdatedAttribute,
        frame: &boxcars::Frame,
        scoring_team_is_team_0: bool,
    ) -> Option<PlayerId> {
        self.goal_scorer_from_explosion_attribute(&goal_update.attribute, scoring_team_is_team_0)
            .or_else(|| self.goal_scorer_from_frame(frame, scoring_team_is_team_0))
    }

    fn goal_scorer_from_explosion_attribute(
        &self,
        attribute: &boxcars::Attribute,
        scoring_team_is_team_0: bool,
    ) -> Option<PlayerId> {
        let actor_candidates = match attribute {
            boxcars::Attribute::Explosion(explosion) => vec![explosion.actor],
            boxcars::Attribute::ExtendedExplosion(explosion) => {
                vec![explosion.explosion.actor, explosion.secondary_actor]
            }
            _ => return None,
        };

        actor_candidates.into_iter().find_map(|actor_id| {
            self.goal_scorer_from_actor_hint(&actor_id, scoring_team_is_team_0)
        })
    }

    fn goal_scorer_from_actor_hint(
        &self,
        actor_id: &boxcars::ActorId,
        scoring_team_is_team_0: bool,
    ) -> Option<PlayerId> {
        let player_id = self
            .get_player_id_from_actor_id(actor_id)
            .ok()
            .or_else(|| self.get_player_id_from_car_id(actor_id).ok())?;
        let is_team_0 = self.get_player_is_team_0(&player_id).ok()?;
        (is_team_0 == scoring_team_is_team_0).then_some(player_id)
    }

    fn goal_scorer_from_frame(
        &self,
        frame: &boxcars::Frame,
        scoring_team_is_team_0: bool,
    ) -> Option<PlayerId> {
        let match_goals_key = *self.get_object_id_for_key(MATCH_GOALS_KEY).ok()?;

        frame
            .updated_actors
            .iter()
            .filter(|update| update.object_id == match_goals_key)
            .filter_map(|update| {
                let boxcars::Attribute::Int(goals) = update.attribute else {
                    return None;
                };
                let player_id = self.get_player_id_from_actor_id(&update.actor_id).ok()?;
                let is_team_0 = self.get_player_is_team_0(&player_id).ok()?;
                (is_team_0 == scoring_team_is_team_0).then_some((player_id, goals))
            })
            .max_by_key(|(_, goals)| *goals)
            .map(|(player_id, _)| player_id)
    }

    pub(crate) fn update_player_stat_events(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        self.current_frame_player_stat_events.clear();
        let match_shots_key = self.get_object_id_for_key(MATCH_SHOTS_KEY).ok().copied();
        let match_saves_key = self.get_object_id_for_key(MATCH_SAVES_KEY).ok().copied();
        let match_assists_key = self.get_object_id_for_key(MATCH_ASSISTS_KEY).ok().copied();

        for update in &frame.updated_actors {
            let (kind, new_value) = match update.attribute {
                boxcars::Attribute::Int(value) if Some(update.object_id) == match_shots_key => {
                    (PlayerStatEventKind::Shot, value)
                }
                boxcars::Attribute::Int(value) if Some(update.object_id) == match_saves_key => {
                    (PlayerStatEventKind::Save, value)
                }
                boxcars::Attribute::Int(value) if Some(update.object_id) == match_assists_key => {
                    (PlayerStatEventKind::Assist, value)
                }
                _ => continue,
            };
            let Some(player_id) = self.get_player_id_from_actor_id(&update.actor_id).ok() else {
                continue;
            };
            let Ok(is_team_0) = self.get_player_is_team_0(&player_id) else {
                continue;
            };
            let previous_value = self
                .player_stat_counters
                .get(&(player_id.clone(), kind))
                .copied()
                .unwrap_or(0);
            let delta = new_value - previous_value;
            self.player_stat_counters
                .insert((player_id.clone(), kind), new_value);
            for _ in 0..delta.max(0) {
                let event = PlayerStatEvent {
                    time: frame.time,
                    frame: frame_index,
                    player: player_id.clone(),
                    is_team_0,
                    kind,
                };
                self.current_frame_player_stat_events.push(event.clone());
                self.player_stat_events.push(event);
            }
        }

        Ok(())
    }

    fn try_push_demolish(
        &mut self,
        demolish: &DemolishAttribute,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) {
        if self.demolish_is_known(demolish, frame_index) {
            return;
        }
        self.known_demolishes.push((demolish.clone(), frame_index));
        if let Ok(info) = self.build_demolish_info(demolish, frame, frame_index) {
            self.demolishes.push(info);
        } else {
            log::warn!(
                "Error building demolish info: attacker_car={:?}, victim_car={:?}",
                demolish.attacker_actor_id(),
                demolish.victim_actor_id(),
            );
        }
    }

    fn build_demolish_info(
        &self,
        demo: &DemolishAttribute,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<DemolishInfo> {
        let attacker = self.get_player_id_from_car_id(&demo.attacker_actor_id())?;
        let victim = self.get_player_id_from_car_id(&demo.victim_actor_id())?;
        let (current_rigid_body, _) =
            self.get_player_rigid_body_and_updated_or_recently_deleted(&victim)?;
        Ok(DemolishInfo {
            time: frame.time,
            seconds_remaining: self.get_seconds_remaining()?,
            frame: frame_index,
            attacker,
            victim,
            attacker_velocity: self.normalize_vector(demo.attacker_velocity()),
            victim_velocity: self.normalize_vector(demo.victim_velocity()),
            victim_location: self.normalize_vector(current_rigid_body.location),
        })
    }
}
