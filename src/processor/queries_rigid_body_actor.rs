use crate::{
    attribute_type_name, ReplayProcessor, SubtrActorError, SubtrActorErrorVariant,
    SubtrActorResult, RIGID_BODY_STATE_KEY,
};

impl<'a> ReplayProcessor<'a> {
    pub(crate) fn get_frame(&self, frame_index: usize) -> SubtrActorResult<&boxcars::Frame> {
        self.replay
            .network_frames
            .as_ref()
            .ok_or(SubtrActorError::new(
                SubtrActorErrorVariant::NoNetworkFrames,
            ))?
            .frames
            .get(frame_index)
            .ok_or(SubtrActorError::new(
                SubtrActorErrorVariant::FrameIndexOutOfBounds,
            ))
    }

    /// Returns an actor's rigid body together with the frame index of its last update.
    pub fn get_actor_rigid_body(
        &self,
        actor_id: &boxcars::ActorId,
    ) -> SubtrActorResult<(&boxcars::RigidBody, &usize)> {
        get_attribute_and_updated!(
            self,
            &self.get_actor_state(actor_id)?.attributes,
            RIGID_BODY_STATE_KEY,
            boxcars::Attribute::RigidBody
        )
    }

    /// Like [`Self::get_actor_rigid_body`], but falls back to recently deleted actor state.
    pub fn get_actor_rigid_body_or_recently_deleted(
        &self,
        actor_id: &boxcars::ActorId,
    ) -> SubtrActorResult<(&boxcars::RigidBody, &usize)> {
        get_attribute_and_updated!(
            self,
            &self
                .get_actor_state_or_recently_deleted(actor_id)?
                .attributes,
            RIGID_BODY_STATE_KEY,
            boxcars::Attribute::RigidBody
        )
    }
}
