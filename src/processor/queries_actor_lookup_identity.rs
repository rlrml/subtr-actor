use crate::{PlayerId, ReplayProcessor, SubtrActorError, SubtrActorErrorVariant, SubtrActorResult};

impl<'a> ReplayProcessor<'a> {
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

    pub(crate) fn get_player_actor_id_from_car_actor_id(
        &self,
        actor_id: &boxcars::ActorId,
    ) -> SubtrActorResult<boxcars::ActorId> {
        self.car_to_player.get(actor_id).copied().ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::NoMatchingPlayerId {
                actor_id: *actor_id,
            })
        })
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
}
