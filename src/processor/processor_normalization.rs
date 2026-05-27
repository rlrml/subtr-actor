use super::*;

impl<'a> ReplayProcessor<'a> {
    const LEGACY_RIGID_BODY_NET_VERSION_CUTOFF: i32 = 5;
    const LEGACY_RIGID_BODY_ROTATION_NET_VERSION_CUTOFF: i32 = 7;
    const LEGACY_RIGID_BODY_LOCATION_FACTOR: f32 = 100.0;
    const LEGACY_RIGID_BODY_VELOCITY_FACTOR: f32 = 10.0;

    pub(crate) fn uses_legacy_rigid_body_vector_scale(net_version: Option<i32>) -> bool {
        net_version.is_none_or(|version| version < Self::LEGACY_RIGID_BODY_NET_VERSION_CUTOFF)
    }

    pub(crate) fn uses_legacy_rigid_body_rotation_for_net_version(
        net_version: Option<i32>,
    ) -> bool {
        net_version
            .is_none_or(|version| version < Self::LEGACY_RIGID_BODY_ROTATION_NET_VERSION_CUTOFF)
    }

    pub(crate) fn rigid_body_location_normalization_factor_for_net_version(
        net_version: Option<i32>,
    ) -> f32 {
        if Self::uses_legacy_rigid_body_vector_scale(net_version) {
            Self::LEGACY_RIGID_BODY_LOCATION_FACTOR
        } else {
            1.0
        }
    }

    pub(crate) fn rigid_body_velocity_normalization_factor_for_net_version(
        net_version: Option<i32>,
    ) -> f32 {
        if Self::uses_legacy_rigid_body_vector_scale(net_version) {
            Self::LEGACY_RIGID_BODY_VELOCITY_FACTOR
        } else {
            1.0
        }
    }

    pub fn spatial_normalization_factor(&self) -> f32 {
        self.spatial_normalization_factor
    }

    pub fn rigid_body_velocity_normalization_factor(&self) -> f32 {
        self.rigid_body_velocity_normalization_factor
    }

    pub(crate) fn normalize_vector_by_factor(
        &self,
        vector: boxcars::Vector3f,
        factor: f32,
    ) -> boxcars::Vector3f {
        if (factor - 1.0).abs() < f32::EPSILON {
            vector
        } else {
            boxcars::Vector3f {
                x: vector.x * factor,
                y: vector.y * factor,
                z: vector.z * factor,
            }
        }
    }

    pub(crate) fn normalize_vector(&self, vector: boxcars::Vector3f) -> boxcars::Vector3f {
        self.normalize_vector_by_factor(vector, self.spatial_normalization_factor)
    }
}
