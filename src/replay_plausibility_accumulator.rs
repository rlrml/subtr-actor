use super::replay_plausibility_constants::*;
use super::replay_plausibility_report::RigidBodyPlausibilityReport;
use super::replay_plausibility_stats::{median, positive_fraction};
use super::{quat_to_glam, vec_to_glam};

#[path = "replay_plausibility_accumulator_finish.rs"]
mod finish;
#[path = "replay_plausibility_accumulator_pair.rs"]
mod pair;

#[derive(Debug, Default)]
pub(super) struct RigidBodyPlausibilityAccumulator {
    sample_count: usize,
    motion_ratios: Vec<f32>,
    motion_log10_errors: Vec<f32>,
    rotation_angle_deltas: Vec<f32>,
    orientation_speeds: Vec<f32>,
    grounded_forward_alignments: Vec<f32>,
    max_abs_location: f32,
    max_linear_speed: f32,
    max_angular_speed: f32,
    max_quaternion_norm_error: f32,
    max_orientation_speed: f32,
}

impl RigidBodyPlausibilityAccumulator {
    pub(super) fn add_sample(&mut self, rigid_body: &boxcars::RigidBody) {
        self.sample_count += 1;
        let location = vec_to_glam(&rigid_body.location);
        self.max_abs_location = self
            .max_abs_location
            .max(location.x.abs())
            .max(location.y.abs())
            .max(location.z.abs());

        if let Some(linear_velocity) = rigid_body.linear_velocity {
            self.max_linear_speed = self
                .max_linear_speed
                .max(vec_to_glam(&linear_velocity).length());
        }
        if let Some(angular_velocity) = rigid_body.angular_velocity {
            self.max_angular_speed = self
                .max_angular_speed
                .max(vec_to_glam(&angular_velocity).length());
        }

        let rotation = quat_to_glam(&rigid_body.rotation);
        self.max_quaternion_norm_error = self
            .max_quaternion_norm_error
            .max((rotation.length() - 1.0).abs());
        self.add_grounded_alignment(rigid_body, rotation);
    }

    fn add_grounded_alignment(&mut self, rigid_body: &boxcars::RigidBody, rotation: glam::Quat) {
        if let Some(linear_velocity) = rigid_body.linear_velocity {
            let velocity = vec_to_glam(&linear_velocity);
            let planar_speed = velocity.truncate().length();
            let grounded = rigid_body.location.z.abs() <= MAX_GROUNDED_HEIGHT
                && velocity.z.abs() <= MAX_GROUNDED_VERTICAL_SPEED;
            if grounded
                && planar_speed >= MIN_FORWARD_ALIGNMENT_SPEED
                && rotation.length_squared() > f32::EPSILON
            {
                let forward = rotation.normalize() * glam::Vec3::X;
                let forward_xy = forward.truncate().normalize_or_zero();
                let velocity_xy = velocity.truncate().normalize_or_zero();
                let alignment = forward_xy.dot(velocity_xy);
                if alignment.is_finite() {
                    self.grounded_forward_alignments.push(alignment);
                }
            }
        }
    }
}
