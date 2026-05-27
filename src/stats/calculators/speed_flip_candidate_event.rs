use super::speed_flip_candidate_event_helpers::{
    speed_flip_cancel_score, speed_flip_confidence, speed_flip_event_from_candidate,
};
use super::*;

impl SpeedFlipCalculator {
    pub(super) fn candidate_event(
        player_id: &PlayerId,
        candidate: ActiveSpeedFlipCandidate,
    ) -> Option<SpeedFlipEvent> {
        let time_since_kickoff_start = candidate
            .kickoff_start_time
            .map(|kickoff_start_time| (candidate.start_time - kickoff_start_time).max(0.0))
            .unwrap_or(0.0);
        let timeliness_score = if candidate.is_kickoff {
            1.0 - Self::normalize_score(time_since_kickoff_start, 0.55, 1.1)
        } else {
            1.0
        };
        let cancel_score = speed_flip_cancel_score(&candidate);
        let speed_score = 0.55 * Self::normalize_score(candidate.max_speed, 1450.0, 1900.0)
            + 0.45
                * Self::normalize_score(candidate.max_speed - candidate.start_speed, 180.0, 650.0);
        let alignment_score = Self::normalize_score(candidate.best_alignment, 0.78, 0.98);
        if candidate.boost_alignment_sample_count == 0 {
            return None;
        }
        if candidate.dodge_acceleration_sample_count == 0
            || candidate.best_dodge_forward_delta < SPEED_FLIP_MIN_FORWARD_DODGE_DELTA
            || candidate.best_dodge_delta_alignment < SPEED_FLIP_MIN_FORWARD_DODGE_DELTA_ALIGNMENT
        {
            return None;
        }

        let boost_alignment_score =
            Self::normalize_score(candidate.best_boost_alignment, 0.82, 0.99);
        let confidence = speed_flip_confidence(
            &candidate,
            cancel_score,
            speed_score,
            alignment_score,
            boost_alignment_score,
            timeliness_score,
        );
        if boost_alignment_score < 0.25
            || cancel_score < 0.35
            || confidence < SPEED_FLIP_MIN_CONFIDENCE
        {
            return None;
        }

        Some(speed_flip_event_from_candidate(
            player_id,
            candidate,
            time_since_kickoff_start,
            cancel_score,
            speed_score,
            confidence,
        ))
    }
}
