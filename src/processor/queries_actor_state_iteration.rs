use crate::{ActorState, ReplayProcessor, SubtrActorResult, EMPTY_ACTOR_IDS};

impl<'a> ReplayProcessor<'a> {
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
