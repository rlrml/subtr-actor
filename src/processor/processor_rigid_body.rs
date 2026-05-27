use super::*;

impl<'a> ReplayProcessor<'a> {
    fn normalize_rigid_body_velocity(&self, vector: boxcars::Vector3f) -> boxcars::Vector3f {
        self.normalize_vector_by_factor(vector, self.rigid_body_velocity_normalization_factor)
    }

    fn normalize_optional_rigid_body_velocity(
        &self,
        vector: Option<boxcars::Vector3f>,
    ) -> Option<boxcars::Vector3f> {
        vector.map(|value| self.normalize_rigid_body_velocity(value))
    }

    fn normalize_rigid_body_rotation(&self, rotation: boxcars::Quaternion) -> boxcars::Quaternion {
        if !self.uses_legacy_rigid_body_rotation {
            return rotation;
        }

        let normalized = glam::Quat::from_euler(
            glam::EulerRot::ZYX,
            rotation.y * std::f32::consts::PI,
            rotation.x * std::f32::consts::PI,
            -rotation.z * std::f32::consts::PI,
        );
        boxcars::Quaternion {
            x: normalized.x,
            y: normalized.y,
            z: normalized.z,
            w: normalized.w,
        }
    }

    pub(crate) fn normalize_rigid_body(
        &self,
        rigid_body: &boxcars::RigidBody,
    ) -> boxcars::RigidBody {
        if (self.spatial_normalization_factor - 1.0).abs() < f32::EPSILON
            && (self.rigid_body_velocity_normalization_factor - 1.0).abs() < f32::EPSILON
            && !self.uses_legacy_rigid_body_rotation
        {
            *rigid_body
        } else {
            boxcars::RigidBody {
                sleeping: rigid_body.sleeping,
                location: self.normalize_vector(rigid_body.location),
                rotation: self.normalize_rigid_body_rotation(rigid_body.rotation),
                linear_velocity: self
                    .normalize_optional_rigid_body_velocity(rigid_body.linear_velocity),
                angular_velocity: self
                    .normalize_optional_rigid_body_velocity(rigid_body.angular_velocity),
            }
        }
    }
}
