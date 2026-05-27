use super::*;

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
    pub(super) fn new(new_actor: &boxcars::NewActor) -> Self {
        Self {
            attributes: HashMap::new(),
            derived_attributes: HashMap::new(),
            object_id: new_actor.object_id,
            name_id: new_actor.name_id,
        }
    }

    /// Updates an attribute in the `ActorState`.
    pub(super) fn update_attribute(
        &mut self,
        update: &boxcars::UpdatedAttribute,
        frame_index: usize,
    ) -> Option<(boxcars::Attribute, usize)> {
        self.attributes
            .insert(update.object_id, (update.attribute.clone(), frame_index))
    }

    pub(crate) fn set_derived_attribute(
        &mut self,
        key: &'static str,
        attribute: boxcars::Attribute,
        frame_index: usize,
    ) {
        if let Some(entry) = self.derived_attributes.get_mut(key) {
            *entry = (attribute, frame_index);
        } else {
            self.derived_attributes
                .insert(key.to_owned(), (attribute, frame_index));
        }
    }
}
