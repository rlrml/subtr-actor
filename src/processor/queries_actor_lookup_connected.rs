use crate::{PlayerId, ReplayProcessor, SubtrActorError, SubtrActorErrorVariant, SubtrActorResult};
use std::collections::HashMap;

impl<'a> ReplayProcessor<'a> {
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
}
