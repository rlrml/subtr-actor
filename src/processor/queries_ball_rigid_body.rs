use crate::{
    attribute_type_name, ReplayProcessor, SubtrActorError, SubtrActorErrorVariant,
    SubtrActorResult, RIGID_BODY_STATE_KEY,
};

impl<'a> ReplayProcessor<'a> {
    /// Returns the current ball rigid body from live actor state.
    pub fn get_ball_rigid_body(&self) -> SubtrActorResult<&boxcars::RigidBody> {
        self.ball_actor_id
            .ok_or(SubtrActorError::new(
                SubtrActorErrorVariant::BallActorNotFound,
            ))
            .and_then(|actor_id| self.get_actor_rigid_body(&actor_id).map(|v| v.0))
    }

    /// Returns the current ball rigid body after spatial normalization.
    pub fn get_normalized_ball_rigid_body(&self) -> SubtrActorResult<boxcars::RigidBody> {
        self.get_ball_rigid_body()
            .map(|rigid_body| self.normalize_rigid_body(rigid_body))
    }

    /// Returns whether a non-sleeping ball rigid body is currently available.
    pub fn ball_rigid_body_exists(&self) -> SubtrActorResult<bool> {
        Ok(self
            .get_ball_rigid_body()
            .map(|rb| !rb.sleeping)
            .unwrap_or(false))
    }

    /// Returns the current ball rigid body and the frame where it was last updated.
    pub fn get_ball_rigid_body_and_updated(
        &self,
    ) -> SubtrActorResult<(&boxcars::RigidBody, &usize)> {
        self.ball_actor_id
            .ok_or(SubtrActorError::new(
                SubtrActorErrorVariant::BallActorNotFound,
            ))
            .and_then(|actor_id| {
                get_attribute_and_updated!(
                    self,
                    &self.get_actor_state(&actor_id)?.attributes,
                    RIGID_BODY_STATE_KEY,
                    boxcars::Attribute::RigidBody
                )
            })
    }

    /// Applies stored ball velocity forward to the requested time.
    pub fn get_velocity_applied_ball_rigid_body(
        &self,
        target_time: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        let (current_rigid_body, frame_index) = self.get_ball_rigid_body_and_updated()?;
        self.velocities_applied_rigid_body(current_rigid_body, *frame_index, target_time)
    }

    /// Interpolates the ball rigid body to the requested time.
    pub fn get_interpolated_ball_rigid_body(
        &self,
        time: f32,
        close_enough: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        self.get_interpolated_actor_rigid_body(&self.get_ball_actor_id()?, time, close_enough)
    }
}
