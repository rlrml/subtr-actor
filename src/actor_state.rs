use crate::*;
use boxcars;
use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone)]
pub struct ActorState {
    pub attributes: HashMap<boxcars::ObjectId, (boxcars::Attribute, usize)>,
    pub derived_attributes: HashMap<String, (boxcars::Attribute, usize)>,
    pub object_id: boxcars::ObjectId,
    pub name_id: Option<i32>,
}

impl ActorState {
    fn new(new_actor: &boxcars::NewActor) -> Self {
        Self {
            attributes: HashMap::new(),
            derived_attributes: HashMap::new(),
            object_id: new_actor.object_id,
            name_id: new_actor.name_id,
        }
    }

    fn update_attribute(
        &mut self,
        update: &boxcars::UpdatedAttribute,
        frame_index: usize,
    ) -> Option<(boxcars::Attribute, usize)> {
        self.attributes
            .insert(update.object_id, (update.attribute.clone(), frame_index))
    }
}

pub struct ActorStateModeler {
    pub actor_states: HashMap<boxcars::ActorId, ActorState>,
    pub actor_ids_by_type: HashMap<boxcars::ObjectId, Vec<boxcars::ActorId>>,
}

impl ActorStateModeler {
    pub fn new() -> Self {
        Self {
            actor_states: HashMap::new(),
            actor_ids_by_type: HashMap::new(),
        }
    }

    pub fn process_frame(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        if let Some(err) = frame
            .deleted_actors
            .iter()
            .map(|n| self.delete_actor(n))
            .find(|r| r.is_err())
        {
            return err.map(|_| ());
        }
        if let Some(err) = frame
            .new_actors
            .iter()
            .map(|n| self.new_actor(n))
            .find(|r| r.is_err())
        {
            return err;
        }
        if let Some(err) = frame
            .updated_actors
            .iter()
            .map(|u| self.update_attribute(u, frame_index))
            .find(|r| r.is_err())
        {
            return err.map(|_| ());
        }
        Ok(())
    }

    pub fn new_actor(&mut self, new_actor: &boxcars::NewActor) -> SubtrActorResult<()> {
        if let Some(state) = self.actor_states.get(&new_actor.actor_id) {
            if state.object_id != new_actor.object_id {
                return SubtrActorError::new_result(SubtrActorErrorVariant::ActorIdAlreadyExists {
                    actor_id: new_actor.actor_id.clone(),
                    object_id: new_actor.object_id.clone(),
                });
            }
        } else {
            self.actor_states
                .insert(new_actor.actor_id, ActorState::new(new_actor));
            self.actor_ids_by_type
                .entry(new_actor.object_id)
                .or_insert_with(|| Vec::new())
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
                actor_id: actor_id.clone(),
            })
        })?;

        self.actor_ids_by_type
            .entry(state.object_id)
            .or_insert_with(|| Vec::new())
            .retain(|x| x != actor_id);

        Ok(state)
    }
}
