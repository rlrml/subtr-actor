use super::*;

impl<'a> ReplayProcessor<'a> {
    /// Updates actor/player/component mappings used by later replay processing.
    pub(crate) fn update_mappings(&mut self, frame: &boxcars::Frame) -> SubtrActorResult<()> {
        let ctx = self.mapping_update_context()?;

        for update in frame.updated_actors.iter() {
            macro_rules! maintain_link {
                ($map:expr, $actor_ids:expr, $get_key:expr, $get_value:expr, $type:path $(, skip_value $skip:expr)?) => {{
                    if $actor_ids.contains(&update.actor_id) {
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
                ($map:expr, $actor_ids:expr $(, skip_value $skip:expr)?) => {
                    maintain_link!(
                        $map,
                        $actor_ids,
                        get_actor_id_from_active_actor,
                        use_update_actor,
                        boxcars::Attribute::ActiveActor
                        $(, skip_value $skip)?
                    )
                };
            }
            macro_rules! maintain_vehicle_key_link {
                ($map:expr, $actor_ids:expr) => {
                    maintain_actor_link!($map, $actor_ids)
                };
            }

            match update.object_id {
                object_id
                    if object_id == ctx.unique_id_object_id
                        && ctx.player_type_actor_ids.contains(&update.actor_id) =>
                {
                    let unique_id =
                        attribute_match!(&update.attribute, boxcars::Attribute::UniqueId)?;
                    self.insert_player_actor_id(unique_id.remote_id.clone(), update.actor_id);
                }
                object_id if object_id == ctx.team_object_id => {
                    maintain_link!(
                        self.player_to_team,
                        ctx.player_type_actor_ids,
                        use_update_actor,
                        get_actor_id_from_active_actor,
                        boxcars::Attribute::ActiveActor,
                        skip_value boxcars::ActorId(-1)
                    );
                }
                object_id
                    if Some(object_id) == ctx.bot_object_id
                        && ctx.player_type_actor_ids.contains(&update.actor_id) =>
                {
                    self.update_synthetic_bot_player_mapping(update)?;
                }
                object_id if object_id == ctx.player_replication_object_id => {
                    maintain_actor_link!(self.player_to_car, ctx.car_type_actor_ids);
                    maintain_link!(
                        self.car_to_player,
                        ctx.car_type_actor_ids,
                        use_update_actor,
                        get_actor_id_from_active_actor,
                        boxcars::Attribute::ActiveActor,
                        skip_value boxcars::ActorId(-1)
                    );
                }
                object_id if object_id == ctx.vehicle_object_id => {
                    maintain_vehicle_key_link!(self.car_to_boost, ctx.boost_type_actor_ids);
                    maintain_vehicle_key_link!(self.car_to_dodge, ctx.dodge_type_actor_ids);
                    maintain_vehicle_key_link!(self.car_to_jump, ctx.jump_type_actor_ids);
                    maintain_vehicle_key_link!(
                        self.car_to_double_jump,
                        ctx.double_jump_type_actor_ids
                    );
                }
                _ => {}
            }
        }

        for actor_id in frame.deleted_actors.iter() {
            if let Some(car_id) = self.player_to_car.remove(actor_id) {
                log::info!("Player actor {actor_id:?} deleted, car id: {car_id:?}.");
            }
        }

        self.sync_player_order_from_known_mappings();

        Ok(())
    }
}
