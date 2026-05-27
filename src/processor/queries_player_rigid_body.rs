use super::*;

impl<'a> ReplayProcessor<'a> {
    /// Returns the player's current car rigid body.
    pub fn get_player_rigid_body(
        &self,
        player_id: &PlayerId,
    ) -> SubtrActorResult<&boxcars::RigidBody> {
        self.get_car_actor_id(player_id)
            .and_then(|actor_id| self.get_actor_rigid_body(&actor_id).map(|v| v.0))
    }

    /// Returns the player's current car rigid body after spatial normalization.
    pub fn get_normalized_player_rigid_body(
        &self,
        player_id: &PlayerId,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        self.get_player_rigid_body(player_id)
            .map(|rigid_body| self.normalize_rigid_body(rigid_body))
    }

    /// Returns the player's rigid body and the frame where it was last updated.
    pub fn get_player_rigid_body_and_updated(
        &self,
        player_id: &PlayerId,
    ) -> SubtrActorResult<(&boxcars::RigidBody, &usize)> {
        self.get_car_actor_id(player_id).and_then(|actor_id| {
            get_attribute_and_updated!(
                self,
                &self.get_actor_state(&actor_id)?.attributes,
                RIGID_BODY_STATE_KEY,
                boxcars::Attribute::RigidBody
            )
        })
    }

    /// Like [`Self::get_player_rigid_body_and_updated`], but can use recently deleted state.
    pub fn get_player_rigid_body_and_updated_or_recently_deleted(
        &self,
        player_id: &PlayerId,
    ) -> SubtrActorResult<(&boxcars::RigidBody, &usize)> {
        self.get_car_actor_id(player_id)
            .and_then(|actor_id| self.get_actor_rigid_body_or_recently_deleted(&actor_id))
    }

    /// Applies stored player velocity forward to the requested time.
    pub fn get_velocity_applied_player_rigid_body(
        &self,
        player_id: &PlayerId,
        target_time: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        let (current_rigid_body, frame_index) =
            self.get_player_rigid_body_and_updated(player_id)?;
        self.velocities_applied_rigid_body(current_rigid_body, *frame_index, target_time)
    }

    /// Interpolates the player's car rigid body to the requested time.
    pub fn get_interpolated_player_rigid_body(
        &self,
        player_id: &PlayerId,
        time: f32,
        close_enough: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        self.get_car_actor_id(player_id).and_then(|car_actor_id| {
            self.get_interpolated_actor_rigid_body(&car_actor_id, time, close_enough)
        })
    }
}
