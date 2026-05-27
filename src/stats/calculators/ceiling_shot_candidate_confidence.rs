use super::*;

impl CeilingShotCandidateMetrics {
    pub(super) fn confidence(
        time_since_ceiling_contact: f32,
        separation_from_ceiling: f32,
        max_touch_height: f32,
        forward_alignment: f32,
        forward_approach_speed: f32,
        ball_speed_change: f32,
        roof_alignment: f32,
    ) -> f32 {
        let timing_score = 1.0
            - CeilingShotCalculator::normalize_score(
                time_since_ceiling_contact,
                0.10,
                CEILING_SHOT_MAX_TOUCH_AFTER_CONTACT_SECONDS,
            );
        let separation_score =
            CeilingShotCalculator::normalize_score(separation_from_ceiling, 140.0, 520.0);
        let height_score = CeilingShotCalculator::normalize_score(
            max_touch_height,
            CEILING_SHOT_MIN_BALL_HEIGHT,
            900.0,
        );
        let alignment_score = CeilingShotCalculator::normalize_score(
            forward_alignment,
            CEILING_SHOT_MIN_FORWARD_ALIGNMENT,
            0.92,
        );
        let approach_score = CeilingShotCalculator::normalize_score(
            forward_approach_speed,
            CEILING_SHOT_MIN_FORWARD_APPROACH_SPEED,
            900.0,
        );
        let impulse_score = CeilingShotCalculator::normalize_score(
            ball_speed_change,
            CEILING_SHOT_MIN_BALL_SPEED_CHANGE,
            900.0,
        );
        let contact_score = CeilingShotCalculator::normalize_score(
            roof_alignment,
            CEILING_CONTACT_MIN_ROOF_ALIGNMENT,
            0.98,
        );

        0.20 * timing_score
            + 0.15 * separation_score
            + 0.12 * height_score
            + 0.17 * alignment_score
            + 0.16 * approach_score
            + 0.10 * impulse_score
            + 0.10 * contact_score
    }
}
