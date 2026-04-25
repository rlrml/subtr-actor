use super::*;

impl<'a> ReplayProcessor<'a> {
    /// Updates derived boost amounts for each boost component actor in the current frame.
    pub(crate) fn update_boost_amounts(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        let kickoff_phase_active = self.kickoff_phase_active();
        let kickoff_phase_started = kickoff_phase_active && !self.kickoff_phase_active_last_frame;
        let cached = self.cached_object_ids;
        let boost_type_object_id = self.required_cached_object_id(cached.boost_type, BOOST_TYPE)?;
        let boost_replicated_object_id = cached.boost_replicated;
        let boost_amount_object_id = cached.boost_amount;
        let component_active_object_id = cached.component_active;
        let boost_actor_ids = self
            .actor_state
            .actor_ids_by_type
            .get(&boost_type_object_id)
            .cloned()
            .unwrap_or_default();
        let updates: Vec<_> = boost_actor_ids
            .into_iter()
            .map(|actor_id| {
                let actor_state = self.actor_state.actor_states.get(&actor_id).unwrap();
                let (
                    actor_amount_value,
                    last_value,
                    _,
                    derived_value,
                    has_derived_value,
                    is_active,
                ) = Self::get_current_boost_values(
                    actor_state,
                    boost_replicated_object_id,
                    boost_amount_object_id,
                    component_active_object_id,
                );
                let mut current_value = if kickoff_phase_started {
                    BOOST_KICKOFF_START_AMOUNT
                } else if actor_amount_value == last_value {
                    if has_derived_value {
                        derived_value
                    } else {
                        actor_amount_value.into()
                    }
                } else {
                    actor_amount_value.into()
                };
                if is_active {
                    current_value -= frame.delta * BOOST_USED_RAW_UNITS_PER_SECOND;
                }
                (actor_id, current_value.max(0.0), actor_amount_value)
            })
            .collect();

        for (actor_id, current_value, new_last_value) in updates {
            let actor_state = self.actor_state.actor_states.get_mut(&actor_id).unwrap();
            actor_state.set_derived_attribute(
                LAST_BOOST_AMOUNT_KEY,
                boxcars::Attribute::Byte(new_last_value),
                frame_index,
            );
            actor_state.set_derived_attribute(
                BOOST_AMOUNT_KEY,
                boxcars::Attribute::Float(current_value),
                frame_index,
            );
        }
        self.kickoff_phase_active_last_frame = kickoff_phase_active;
        Ok(())
    }

    fn kickoff_phase_active(&self) -> bool {
        let Ok(metadata_actor_id) = self.get_metadata_actor_id() else {
            return false;
        };
        let Ok(metadata_state) = self.get_actor_state(&metadata_actor_id) else {
            return false;
        };
        let metadata_attributes = &metadata_state.attributes;

        let replicated_state_name = self
            .cached_object_ids
            .replicated_state_name
            .and_then(|object_id| metadata_attributes.get(&object_id))
            .and_then(|(attribute, _)| match attribute {
                boxcars::Attribute::Int(value) => Some(*value),
                _ => None,
            });
        let replicated_game_state_time_remaining = self
            .cached_object_ids
            .replicated_game_state_time_remaining
            .and_then(|object_id| metadata_attributes.get(&object_id))
            .and_then(|(attribute, _)| match attribute {
                boxcars::Attribute::Int(value) => Some(*value),
                _ => None,
            });
        let ball_has_been_hit = self
            .cached_object_ids
            .ball_has_been_hit
            .and_then(|object_id| metadata_attributes.get(&object_id))
            .and_then(|(attribute, _)| match attribute {
                boxcars::Attribute::Boolean(value) => Some(*value),
                _ => None,
            });

        replicated_state_name == Some(55)
            || replicated_game_state_time_remaining.is_some_and(|countdown| countdown > 0)
            || ball_has_been_hit == Some(false)
    }

    fn get_current_boost_values(
        actor_state: &ActorState,
        boost_replicated_object_id: Option<boxcars::ObjectId>,
        boost_amount_object_id: Option<boxcars::ObjectId>,
        component_active_object_id: Option<boxcars::ObjectId>,
    ) -> (u8, u8, u8, f32, bool, bool) {
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
            });
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
            derived_value.unwrap_or(0.0),
            derived_value.is_some(),
            is_active,
        )
    }

    fn actor_is_boost_pad(&self, actor_id: &boxcars::ActorId) -> bool {
        self.get_actor_state_or_recently_deleted(actor_id)
            .ok()
            .and_then(|state| usize::try_from(state.object_id.0).ok())
            .and_then(|index| self.is_boost_pad_object.get(index))
            .copied()
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

    /// Detects boost-pad pickup and respawn events in the current frame.
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
}
