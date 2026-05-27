use super::*;

impl ActorStateModeler {
    pub fn new_actor(&mut self, new_actor: &boxcars::NewActor) -> SubtrActorResult<()> {
        if let Some(state) = self.actor_states.get(&new_actor.actor_id) {
            if state.object_id != new_actor.object_id {
                return SubtrActorError::new_result(SubtrActorErrorVariant::ActorIdAlreadyExists {
                    actor_id: new_actor.actor_id,
                    object_id: new_actor.object_id,
                });
            }
        } else {
            self.actor_states
                .insert(new_actor.actor_id, ActorState::new(new_actor));
            self.actor_ids_by_type
                .entry(new_actor.object_id)
                .or_default()
                .push(new_actor.actor_id)
        }
        Ok(())
    }

    pub fn update_attribute(
        &mut self,
        update: &boxcars::UpdatedAttribute,
        frame_index: usize,
    ) -> SubtrActorResult<Option<(boxcars::Attribute, usize)>> {
        self.actor_states
            .get_mut(&update.actor_id)
            .map(|state| state.update_attribute(update, frame_index))
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::UpdatedActorIdDoesNotExist {
                    update: update.clone(),
                })
            })
    }

    pub fn delete_actor(&mut self, actor_id: &boxcars::ActorId) -> SubtrActorResult<ActorState> {
        let state = self.actor_states.remove(actor_id).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::NoStateForActorId {
                actor_id: *actor_id,
            })
        })?;

        self.actor_ids_by_type
            .entry(state.object_id)
            .or_default()
            .retain(|x| x != actor_id);

        self.recently_deleted_actor_states
            .insert(*actor_id, state.clone());

        Ok(state)
    }
}
