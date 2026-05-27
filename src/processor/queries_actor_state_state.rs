use crate::{
    ActorState, ReplayProcessor, SubtrActorError, SubtrActorErrorVariant, SubtrActorResult,
};

impl<'a> ReplayProcessor<'a> {
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
}
