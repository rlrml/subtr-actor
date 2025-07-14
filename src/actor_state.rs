use crate::*;
use boxcars;
use std::collections::HashMap;

/// A struct representing the state of an actor.
///
/// This includes both attributes and derived attributes, along with the
/// associated object id and name id.
#[derive(PartialEq, Debug, Clone)]
pub struct ActorState {
    /// A map of the actor's attributes with their corresponding object ids and
    /// frame indices.
    pub attributes: HashMap<boxcars::ObjectId, (boxcars::Attribute, usize)>,
    /// A map of the actor's derived attributes with their corresponding object
    /// ids and frame indices.
    pub derived_attributes: HashMap<String, (boxcars::Attribute, usize)>,
    /// The object id associated with the actor.
    pub object_id: boxcars::ObjectId,
    /// Optional name id associated with the actor.
    pub name_id: Option<i32>,
}

impl ActorState {
    /// Creates a new `ActorState` from a given `NewActor`.
    ///
    /// # Arguments
    ///
    /// * `new_actor` - The new actor to initialize the state from.
    ///
    /// # Returns
    ///
    /// A new `ActorState` object.
    fn new(new_actor: &boxcars::NewActor) -> Self {
        Self {
            attributes: HashMap::new(),
            derived_attributes: HashMap::new(),
            object_id: new_actor.object_id,
            name_id: new_actor.name_id,
        }
    }

    /// Updates an attribute in the `ActorState`.
    ///
    /// # Arguments
    ///
    /// * `update` - The updated attribute.
    /// * `frame_index` - The index of the frame at which the update occurs.
    ///
    /// # Returns
    ///
    /// An optional tuple of the updated attribute and its frame index.
    fn update_attribute(
        &mut self,
        update: &boxcars::UpdatedAttribute,
        frame_index: usize,
    ) -> Option<(boxcars::Attribute, usize)> {
        self.attributes
            .insert(update.object_id, (update.attribute.clone(), frame_index))
    }
}

/// A struct modeling the states of multiple actors at a given point in time.
/// Provides methods to update that state with successive frames from a
/// boxcars::Replay.
pub struct ActorStateModeler {
    /// A map of actor states with their corresponding actor ids.
    pub actor_states: HashMap<boxcars::ActorId, ActorState>,
    /// A map of actor ids with their corresponding object ids.
    pub actor_ids_by_type: HashMap<boxcars::ObjectId, Vec<boxcars::ActorId>>,
}

impl Default for ActorStateModeler {
    fn default() -> Self {
        Self::new()
    }
}

impl ActorStateModeler {
    /// Creates a new [`ActorStateModeler`].
    ///
    /// # Returns
    ///
    /// A new [`ActorStateModeler`]. object.
    pub fn new() -> Self {
        Self {
            actor_states: HashMap::new(),
            actor_ids_by_type: HashMap::new(),
        }
    }

    /// Processes a frame, including handling of new, updated, and deleted actors.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to be processed.
    /// * `frame_index` - The index of the frame to be processed.
    ///
    /// # Returns
    ///
    /// An empty result (`Ok(())`) on success, [`SubtrActorError`] on failure.
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

        Ok(state)
    }
}
