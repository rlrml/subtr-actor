use super::*;

impl<'a> ReplayProcessor<'a> {
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

            if let BoostPadEventKind::PickedUp { sequence } = event.kind {
                // The same pad/sequence can be legitimate later in the match
                // after the pad respawns, so this check must stay time-bounded.
                if self.boost_pad_pickup_sequence_is_recent(&event.pad_id, sequence, event.time) {
                    continue;
                }
                self.boost_pad_pickup_sequence_times
                    .insert((event.pad_id.clone(), sequence), event.time);
            }

            self.current_frame_boost_pad_events.push(event.clone());
            self.boost_pad_events.push(event);
        }

        Ok(())
    }
}
