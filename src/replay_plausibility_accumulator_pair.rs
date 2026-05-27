use super::*;

impl RigidBodyPlausibilityAccumulator {
    pub(crate) fn add_pair(
        &mut self,
        previous_time: f32,
        previous: &boxcars::RigidBody,
        current_time: f32,
        current: &boxcars::RigidBody,
    ) {
        let dt = current_time - previous_time;
        if !(0.0..=MAX_PAIR_DT_SECONDS).contains(&dt) {
            return;
        }

        self.add_motion_pair(previous, current, dt);
        self.add_rotation_pair(previous, current, dt);
    }

    fn add_motion_pair(
        &mut self,
        previous: &boxcars::RigidBody,
        current: &boxcars::RigidBody,
        dt: f32,
    ) {
        let displacement_speed =
            (vec_to_glam(&current.location) - vec_to_glam(&previous.location)).length() / dt;
        if let Some(reported_speed) = previous
            .linear_velocity
            .or(current.linear_velocity)
            .map(|velocity| vec_to_glam(&velocity).length())
        {
            if displacement_speed >= MIN_DISPLACEMENT_SPEED && reported_speed >= MIN_REPORTED_SPEED
            {
                let ratio = reported_speed / displacement_speed;
                if ratio.is_finite() && ratio > 0.0 {
                    self.motion_ratios.push(ratio);
                    self.motion_log10_errors.push(ratio.log10().abs());
                }
            }
        }
    }

    fn add_rotation_pair(
        &mut self,
        previous: &boxcars::RigidBody,
        current: &boxcars::RigidBody,
        dt: f32,
    ) {
        let previous_rotation = quat_to_glam(&previous.rotation);
        let current_rotation = quat_to_glam(&current.rotation);
        if previous_rotation.length_squared() > f32::EPSILON
            && current_rotation.length_squared() > f32::EPSILON
        {
            let previous_rotation = previous_rotation.normalize();
            let current_rotation = current_rotation.normalize();
            let dot = previous_rotation
                .dot(current_rotation)
                .abs()
                .clamp(0.0, 1.0);
            let angle_delta = 2.0 * dot.acos();
            let orientation_speed = angle_delta / dt;
            if angle_delta.is_finite() && orientation_speed.is_finite() {
                self.rotation_angle_deltas.push(angle_delta);
                self.orientation_speeds.push(orientation_speed);
                self.max_orientation_speed = self.max_orientation_speed.max(orientation_speed);
            }
        }
    }
}
