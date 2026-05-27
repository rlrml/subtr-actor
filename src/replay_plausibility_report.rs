use super::replay_plausibility_constants::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ReplayPlausibilityReport {
    pub ball: RigidBodyPlausibilityReport,
    pub players: RigidBodyPlausibilityReport,
}

impl ReplayPlausibilityReport {
    pub fn all_motion_consistent(&self) -> bool {
        self.ball.motion_consistent() && self.players.motion_consistent()
    }

    pub fn all_field_bounds_plausible(&self) -> bool {
        self.ball.field_bounds_plausible() && self.players.field_bounds_plausible()
    }

    pub fn all_location_bounds_plausible(&self) -> bool {
        self.ball.location_bounds_plausible() && self.players.location_bounds_plausible()
    }

    pub fn all_linear_speed_bounds_plausible(&self) -> bool {
        self.ball.linear_speed_bounds_plausible() && self.players.linear_speed_bounds_plausible()
    }

    pub fn all_quaternion_norms_plausible(&self) -> bool {
        self.ball.quaternion_norms_plausible() && self.players.quaternion_norms_plausible()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RigidBodyPlausibilityReport {
    pub sample_count: usize,
    pub motion_pair_count: usize,
    pub rotation_pair_count: usize,
    pub max_abs_location: f32,
    pub max_linear_speed: f32,
    pub max_angular_speed: f32,
    pub max_quaternion_norm_error: f32,
    pub max_orientation_speed: f32,
    pub median_velocity_to_displacement_ratio: Option<f32>,
    pub median_velocity_log10_error: Option<f32>,
    pub velocity_pairs_within_factor_2_fraction: Option<f32>,
    pub median_rotation_angle_delta_radians: Option<f32>,
    pub median_orientation_speed: Option<f32>,
    pub grounded_forward_alignment_sample_count: usize,
    pub median_grounded_forward_alignment: Option<f32>,
    pub grounded_forward_alignment_positive_fraction: Option<f32>,
}

impl RigidBodyPlausibilityReport {
    pub fn field_bounds_plausible(&self) -> bool {
        self.location_bounds_plausible() && self.linear_speed_bounds_plausible()
    }

    pub fn location_bounds_plausible(&self) -> bool {
        self.max_abs_location <= MAX_PLAUSIBLE_LOCATION_ABS
    }

    pub fn linear_speed_bounds_plausible(&self) -> bool {
        self.max_linear_speed <= MAX_PLAUSIBLE_LINEAR_SPEED
    }

    pub fn quaternion_norms_plausible(&self) -> bool {
        self.max_quaternion_norm_error <= MAX_PLAUSIBLE_QUATERNION_NORM_ERROR
    }

    pub fn motion_consistent(&self) -> bool {
        if self.motion_pair_count < MIN_VELOCITY_PAIR_COUNT {
            return true;
        }

        self.median_velocity_to_displacement_ratio
            .is_some_and(|ratio| (VELOCITY_RATIO_MIN..=VELOCITY_RATIO_MAX).contains(&ratio))
            && self
                .velocity_pairs_within_factor_2_fraction
                .is_some_and(|fraction| fraction >= 0.6)
    }
}
