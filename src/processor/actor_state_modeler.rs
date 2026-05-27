use super::*;

/// A struct modeling the states of multiple actors at a given point in time.
/// Provides methods to update that state with successive frames from a
/// boxcars::Replay.
pub struct ActorStateModeler {
    /// A map of actor states with their corresponding actor ids.
    pub actor_states: HashMap<boxcars::ActorId, ActorState>,
    /// A map of actor ids with their corresponding object ids.
    pub actor_ids_by_type: HashMap<boxcars::ObjectId, Vec<boxcars::ActorId>>,
    /// Actor states deleted while processing the current frame.
    ///
    /// This preserves last-known attributes long enough for code that runs after
    /// deletion, such as same-frame demolition extraction, to still inspect the
    /// removed actor.
    pub recently_deleted_actor_states: HashMap<boxcars::ActorId, ActorState>,
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
            recently_deleted_actor_states: HashMap::new(),
        }
    }
}
