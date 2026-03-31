use super::*;

impl<'a> ReplayProcessor<'a> {
    /// This method is responsible for updating various mappings that are used
    /// to track and link different actors in the replay.
    ///
    /// The replay data is a stream of [`boxcars::Frame`] objects that contain
    /// information about the game at a specific point in time. These frames
    /// contain updates for different actors, and the goal of this method is to
    /// maintain and update the mappings for these actors as they are
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
        let cached = self.cached_object_ids;
        let player_type_object_id =
            self.required_cached_object_id(cached.player_type, PLAYER_TYPE)?;
        let car_type_object_id = self.required_cached_object_id(cached.car_type, CAR_TYPE)?;
        let boost_type_object_id = self.required_cached_object_id(cached.boost_type, BOOST_TYPE)?;
        let dodge_type_object_id = self.required_cached_object_id(cached.dodge_type, DODGE_TYPE)?;
        let jump_type_object_id = self.required_cached_object_id(cached.jump_type, JUMP_TYPE)?;
        let double_jump_type_object_id =
            self.required_cached_object_id(cached.double_jump_type, DOUBLE_JUMP_TYPE)?;
        let player_type_actor_ids = self
            .get_actor_ids_by_object_id(&player_type_object_id)
            .to_vec();
        let car_type_actor_ids = self
            .get_actor_ids_by_object_id(&car_type_object_id)
            .to_vec();
        let boost_type_actor_ids = self
            .get_actor_ids_by_object_id(&boost_type_object_id)
            .to_vec();
        let dodge_type_actor_ids = self
            .get_actor_ids_by_object_id(&dodge_type_object_id)
            .to_vec();
        let jump_type_actor_ids = self
            .get_actor_ids_by_object_id(&jump_type_object_id)
            .to_vec();
        let double_jump_type_actor_ids = self
            .get_actor_ids_by_object_id(&double_jump_type_object_id)
            .to_vec();
        let unique_id_object_id =
            self.required_cached_object_id(cached.unique_id, UNIQUE_ID_KEY)?;
        let team_object_id = self.required_cached_object_id(cached.team, TEAM_KEY)?;
        let player_replication_object_id =
            self.required_cached_object_id(cached.player_replication, PLAYER_REPLICATION_KEY)?;
        let vehicle_object_id = self.required_cached_object_id(cached.vehicle, VEHICLE_KEY)?;

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

    /// Refreshes the cached ball actor id, clearing it if the current ball was deleted.
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
}
