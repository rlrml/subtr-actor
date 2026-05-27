use crate::{ReplayProcessor, SubtrActorError, SubtrActorErrorVariant, SubtrActorResult};
use std::collections::HashMap;

impl<'a> ReplayProcessor<'a> {
    pub(crate) fn get_actor_attribute<'b>(
        &'b self,
        actor_id: &boxcars::ActorId,
        property: &'static str,
    ) -> SubtrActorResult<&'b boxcars::Attribute> {
        self.get_attribute(&self.get_actor_state(actor_id)?.attributes, property)
    }

    /// Reads a property from an actor or derived-attribute map by property name.
    pub fn get_attribute<'b>(
        &'b self,
        map: &'b HashMap<boxcars::ObjectId, (boxcars::Attribute, usize)>,
        property: &'static str,
    ) -> SubtrActorResult<&'b boxcars::Attribute> {
        self.get_attribute_and_updated(map, property).map(|v| &v.0)
    }

    /// Reads a property and the frame index when it was last updated.
    pub fn get_attribute_and_updated<'b>(
        &'b self,
        map: &'b HashMap<boxcars::ObjectId, (boxcars::Attribute, usize)>,
        property: &'static str,
    ) -> SubtrActorResult<&'b (boxcars::Attribute, usize)> {
        let attribute_object_id = self.get_object_id_for_key(property)?;
        map.get(attribute_object_id).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState { property })
        })
    }
}
