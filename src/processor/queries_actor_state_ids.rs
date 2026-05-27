use crate::{
    ReplayProcessor, SubtrActorError, SubtrActorErrorVariant, SubtrActorResult, EMPTY_ACTOR_IDS,
};

impl<'a> ReplayProcessor<'a> {
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
}
