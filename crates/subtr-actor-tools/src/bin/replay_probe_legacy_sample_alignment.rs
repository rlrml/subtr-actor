use super::legacy_types::LegacyRotationProbe;
use super::rotation_interpret::{
    reinterpret_euler_rotation, reinterpret_quaternion, rotation_alignment,
};

const MIN_FORWARD_ALIGNMENT_SPEED: f32 = 500.0;
const MAX_GROUNDED_HEIGHT: f32 = 60.0;
const MAX_GROUNDED_VERTICAL_SPEED: f32 = 200.0;

impl LegacyRotationProbe {
    pub(super) fn sample_grounded_alignment(
        &mut self,
        rigid_body: boxcars::RigidBody,
        linear_velocity: boxcars::Vector3f,
    ) {
        let planar_speed = glam::Vec2::new(linear_velocity.x, linear_velocity.y).length();
        let grounded = rigid_body.location.z.abs() <= MAX_GROUNDED_HEIGHT
            && linear_velocity.z.abs() <= MAX_GROUNDED_VERTICAL_SPEED;
        if !grounded || planar_speed < MIN_FORWARD_ALIGNMENT_SPEED {
            return;
        }

        for mode in &self.modes {
            if let Some(quaternion) = reinterpret_quaternion(rigid_body.rotation, *mode) {
                if let Some((alignment, up_z)) = rotation_alignment(quaternion, linear_velocity) {
                    let accumulator = self.accumulators.get_mut(mode).unwrap();
                    accumulator.alignments.push(alignment);
                    accumulator.up_zs.push(up_z);
                }
            }
        }
        for mode in &self.euler_modes {
            let quaternion = reinterpret_euler_rotation(rigid_body.rotation, *mode);
            if let Some((alignment, up_z)) = rotation_alignment(quaternion, linear_velocity) {
                let accumulator = self.euler_accumulators.get_mut(mode).unwrap();
                accumulator.alignments.push(alignment);
                accumulator.up_zs.push(up_z);
            }
        }
    }
}
