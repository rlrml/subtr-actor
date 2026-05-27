use super::*;

impl SpeedFlipCalculator {
    pub(super) fn update_candidate_dodge_acceleration(
        candidate: &mut ActiveSpeedFlipCandidate,
        frame: &FrameInfo,
        player: &PlayerSample,
    ) {
        if frame.time <= candidate.start_time
            || frame.time - candidate.start_time > SPEED_FLIP_DODGE_ACCELERATION_SAMPLE_SECONDS
        {
            return;
        }
        let Some(velocity) = player.velocity() else {
            return;
        };
        let velocity_delta = velocity.truncate() - candidate.start_velocity_xy;
        let delta_length = velocity_delta.length();
        if delta_length <= f32::EPSILON {
            return;
        }
        let forward_delta = velocity_delta.dot(candidate.start_forward_xy);
        candidate.best_dodge_forward_delta = candidate.best_dodge_forward_delta.max(forward_delta);
        candidate.best_dodge_delta_alignment = candidate
            .best_dodge_delta_alignment
            .max(forward_delta / delta_length);
        candidate.dodge_acceleration_sample_count += 1;
    }

    pub(super) fn update_candidate_rotation(
        candidate: &mut ActiveSpeedFlipCandidate,
        frame: &FrameInfo,
        rigid_body: &boxcars::RigidBody,
    ) {
        let rotation = quat_to_glam(&rigid_body.rotation);
        let local_angular_velocity = rigid_body
            .angular_velocity
            .as_ref()
            .map(vec_to_glam)
            .map(|angular_velocity| rotation.inverse() * angular_velocity)
            .unwrap_or(glam::Vec3::ZERO);
        candidate.best_diagonal_score = candidate
            .best_diagonal_score
            .max(Self::diagonal_score(local_angular_velocity));

        let forward_z = (rotation * glam::Vec3::X).z;
        candidate.min_forward_z = candidate.min_forward_z.min(forward_z);
        candidate.latest_forward_z = forward_z;
        candidate.latest_time = frame.time;
        candidate.latest_frame = frame.frame_number;
    }
}
