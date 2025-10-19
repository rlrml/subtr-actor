use crate::*;
use boxcars;
use std::collections::HashMap;
use crate::constants::BOOST_PAD_CLASS; // ✅ make sure the constant is imported

/// Represents the state of an individual actor (ball, car, pad, etc.)
#[derive(PartialEq, Debug, Clone)]
pub struct ActorState {
    /// A map of the actor's attributes with their corresponding object ids and frame indices.
    pub attributes: HashMap<boxcars::ObjectId, (boxcars::Attribute, usize)>,
    /// A map of the actor's derived attributes with their corresponding object ids and frame indices.
    pub derived_attributes: HashMap<String, (boxcars::Attribute, usize)>,
    /// The object id associated with the actor.
    pub object_id: boxcars::ObjectId,
    /// Optional name id associated with the actor.
    pub name_id: Option<i32>,
}

impl ActorState {
    /// Creates a new `ActorState` from a given `NewActor`.
    fn new(new_actor: &boxcars::NewActor) -> Self {
        Self {
            attributes: HashMap::new(),
            derived_attributes: HashMap::new(),
            object_id: new_actor.object_id,
            name_id: new_actor.name_id,
        }
    }

    /// Updates an attribute in the `ActorState`.
    fn update_attribute(
        &mut self,
        update: &boxcars::UpdatedAttribute,
        frame_index: usize,
    ) -> Option<(boxcars::Attribute, usize)> {
        self.attributes
            .insert(update.object_id, (update.attribute.clone(), frame_index))
    }
}

/// Models all actor states across the entire replay.
/// Handles creation, update, and deletion of actors as frames are processed.
pub struct ActorStateModeler {
    /// A map of actor states keyed by their actor id.
    pub actor_states: HashMap<boxcars::ActorId, ActorState>,
    /// A map of object ids to the actor ids that belong to them.
    pub actor_ids_by_type: HashMap<boxcars::ObjectId, Vec<boxcars::ActorId>>,
    /// Optional mapping from object id to readable name (used for debugging / filtering)
    pub object_id_to_name: HashMap<boxcars::ObjectId, String>,
}

impl Default for ActorStateModeler {
    fn default() -> Self {
        Self::new()
    }
}

impl ActorStateModeler {
    /// Creates a new [`ActorStateModeler`].
    pub fn new() -> Self {
        Self {
            actor_states: HashMap::new(),
            actor_ids_by_type: HashMap::new(),
            object_id_to_name: HashMap::new(),
        }
    }

    /// Retrieves the readable name of an object id if available.
    pub fn get_object_name(&self, object_id: boxcars::ObjectId) -> Option<&str> {
        self.object_id_to_name.get(&object_id).map(|s| s.as_str())
    }

    /// Processes one frame worth of actor updates.
    pub fn process_frame(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        // Handle deleted actors
        if let Some(err) = frame
            .deleted_actors
            .iter()
            .map(|n| self.delete_actor(n))
            .find(|r| r.is_err())
        {
            return err.map(|_| ());
        }

        // Handle new actors
        if let Some(err) = frame
            .new_actors
            .iter()
            .map(|n| self.new_actor(n))
            .find(|r| r.is_err())
        {
            return err;
        }

        // Handle updated actors
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

    /// Registers a newly spawned actor.
    pub fn new_actor(&mut self, new_actor: &boxcars::NewActor) -> SubtrActorResult<()> {
        if let Some(state) = self.actor_states.get(&new_actor.actor_id) {
            if state.object_id != new_actor.object_id {
                return SubtrActorError::new_result(
                    SubtrActorErrorVariant::ActorIdAlreadyExists {
                        actor_id: new_actor.actor_id,
                        object_id: new_actor.object_id,
                    },
                );
            }
        } else {
            // Insert into actor state tracking
            self.actor_states
                .insert(new_actor.actor_id, ActorState::new(new_actor));

            self.actor_ids_by_type
                .entry(new_actor.object_id)
                .or_default()
                .push(new_actor.actor_id);

            // ---- BOOST PAD SPAWN DETECTION ----
            if let Some(name) = self.get_object_name(new_actor.object_id) {
                if name.contains(BOOST_PAD_CLASS) {
                    if let Some(loc) = &new_actor.initial_trajectory.location {
                        let locf = boxcars::Vector3f {
                            x: loc.x as f32,
                            y: loc.y as f32,
                            z: loc.z as f32,
                        };
                        log::debug!(
                            "[BOOST-PAD] Spawned pad {} at ({:.0}, {:.0}, {:.0})",
                            new_actor.actor_id.0,
                            locf.x,
                            locf.y,
                            locf.z
                        );
                        // Optionally: self.boost_pad_positions.insert(new_actor.actor_id.0 as i32, locf);
                    }
                }
            }
        }
        Ok(())
    }

    /// Updates an actor's attributes for the current frame.
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

    /// Removes an actor once it is deleted.
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
