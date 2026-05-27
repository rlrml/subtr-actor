use super::*;

impl RigidBodyPlausibilityAccumulator {
    pub(crate) fn finish(mut self) -> RigidBodyPlausibilityReport {
        let within_factor_2 = if self.motion_ratios.is_empty() {
            None
        } else {
            Some(
                self.motion_ratios
                    .iter()
                    .filter(|ratio| (VELOCITY_RATIO_MIN..=VELOCITY_RATIO_MAX).contains(ratio))
                    .count() as f32
                    / self.motion_ratios.len() as f32,
            )
        };

        RigidBodyPlausibilityReport {
            sample_count: self.sample_count,
            motion_pair_count: self.motion_ratios.len(),
            rotation_pair_count: self.rotation_angle_deltas.len(),
            max_abs_location: self.max_abs_location,
            max_linear_speed: self.max_linear_speed,
            max_angular_speed: self.max_angular_speed,
            max_quaternion_norm_error: self.max_quaternion_norm_error,
            max_orientation_speed: self.max_orientation_speed,
            median_velocity_to_displacement_ratio: median(&mut self.motion_ratios),
            median_velocity_log10_error: median(&mut self.motion_log10_errors),
            velocity_pairs_within_factor_2_fraction: within_factor_2,
            median_rotation_angle_delta_radians: median(&mut self.rotation_angle_deltas),
            median_orientation_speed: median(&mut self.orientation_speeds),
            grounded_forward_alignment_sample_count: self.grounded_forward_alignments.len(),
            median_grounded_forward_alignment: median(&mut self.grounded_forward_alignments),
            grounded_forward_alignment_positive_fraction: positive_fraction(
                &self.grounded_forward_alignments,
            ),
        }
    }
}
