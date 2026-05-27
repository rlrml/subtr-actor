use super::super::*;

impl<'a> ReplayProcessor<'a> {
    /// Searches forward or backward for the next update of a specific actor property.
    pub fn find_update_in_direction(
        &self,
        current_index: usize,
        actor_id: &boxcars::ActorId,
        object_id: &boxcars::ObjectId,
        direction: SearchDirection,
    ) -> SubtrActorResult<(boxcars::Attribute, usize)> {
        let frames = self
            .replay
            .network_frames
            .as_ref()
            .ok_or(SubtrActorError::new(
                SubtrActorErrorVariant::NoNetworkFrames,
            ))?;
        match direction {
            SearchDirection::Forward => {
                for index in (current_index + 1)..frames.frames.len() {
                    if let Some(attribute) = frames.frames[index]
                        .updated_actors
                        .iter()
                        .find(|update| {
                            &update.actor_id == actor_id && &update.object_id == object_id
                        })
                        .map(|update| update.attribute.clone())
                    {
                        return Ok((attribute, index));
                    }
                }
            }
            SearchDirection::Backward => {
                for index in (0..current_index).rev() {
                    if let Some(attribute) = frames.frames[index]
                        .updated_actors
                        .iter()
                        .find(|update| {
                            &update.actor_id == actor_id && &update.object_id == object_id
                        })
                        .map(|update| update.attribute.clone())
                    {
                        return Ok((attribute, index));
                    }
                }
            }
        }

        SubtrActorError::new_result(SubtrActorErrorVariant::NoUpdateAfterFrame {
            actor_id: *actor_id,
            object_id: *object_id,
            frame_index: current_index,
        })
    }
}
