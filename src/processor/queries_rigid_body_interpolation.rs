use crate::{
    apply_velocities_to_rigid_body, attribute_type_name, get_interpolated_rigid_body,
    ReplayProcessor, SearchDirection, SubtrActorError, SubtrActorErrorVariant, SubtrActorResult,
    RIGID_BODY_STATE_KEY,
};

impl<'a> ReplayProcessor<'a> {
    pub(crate) fn velocities_applied_rigid_body(
        &self,
        rigid_body: &boxcars::RigidBody,
        rb_frame_index: usize,
        target_time: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        let rb_frame = self.get_frame(rb_frame_index)?;
        let interpolation_amount = target_time - rb_frame.time;
        let normalized_rigid_body = self.normalize_rigid_body(rigid_body);
        Ok(apply_velocities_to_rigid_body(
            &normalized_rigid_body,
            interpolation_amount,
        ))
    }

    /// Interpolates an arbitrary actor rigid body to the requested replay time.
    pub fn get_interpolated_actor_rigid_body(
        &self,
        actor_id: &boxcars::ActorId,
        time: f32,
        close_enough: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        let (frame_body, frame_index) = self.get_actor_rigid_body(actor_id)?;
        let frame_time = self.get_frame(*frame_index)?.time;
        let time_and_frame_difference = time - frame_time;

        if time_and_frame_difference.abs() <= close_enough.abs() {
            return Ok(self.normalize_rigid_body(frame_body));
        }

        let search_direction = if time_and_frame_difference > 0.0 {
            SearchDirection::Forward
        } else {
            SearchDirection::Backward
        };

        let object_id = self.get_object_id_for_key(RIGID_BODY_STATE_KEY)?;
        let (attribute, found_frame) =
            self.find_update_in_direction(*frame_index, actor_id, object_id, search_direction)?;
        let found_time = self.get_frame(found_frame)?.time;
        let found_body = attribute_match!(attribute, boxcars::Attribute::RigidBody)?;

        if (found_time - time).abs() <= close_enough {
            return Ok(self.normalize_rigid_body(&found_body));
        }

        let (start_body, start_time, end_body, end_time) = match search_direction {
            SearchDirection::Forward => (frame_body, frame_time, &found_body, found_time),
            SearchDirection::Backward => (&found_body, found_time, frame_body, frame_time),
        };
        let start_body = self.normalize_rigid_body(start_body);
        let end_body = self.normalize_rigid_body(end_body);

        get_interpolated_rigid_body(&start_body, start_time, &end_body, end_time, time)
    }
}
