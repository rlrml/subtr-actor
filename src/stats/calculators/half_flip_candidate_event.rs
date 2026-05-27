use super::*;

impl HalfFlipCalculator {
    pub(super) fn candidate_event(
        player_id: &PlayerId,
        candidate: ActiveHalfFlipCandidate,
    ) -> Option<HalfFlipEvent> {
        if !candidate_meets_thresholds(&candidate) {
            return None;
        }

        let confidence = candidate_confidence(&candidate);
        if confidence < HALF_FLIP_MIN_CONFIDENCE {
            return None;
        }

        Some(HalfFlipEvent {
            time: candidate.latest_time,
            frame: candidate.latest_frame,
            player: player_id.clone(),
            is_team_0: candidate.is_team_0,
            start_position: candidate.start_position,
            end_position: candidate.end_position,
            start_speed: candidate.start_speed,
            end_speed: candidate.end_speed,
            start_backward_alignment: candidate.start_backward_alignment,
            best_reorientation_alignment: candidate.best_reorientation_alignment,
            best_forward_reversal: candidate.best_forward_reversal,
            max_forward_vertical: candidate.max_forward_vertical,
            confidence,
        })
    }
}

fn candidate_meets_thresholds(candidate: &ActiveHalfFlipCandidate) -> bool {
    candidate.best_reorientation_alignment >= HALF_FLIP_MIN_REORIENTATION_ALIGNMENT
        && candidate.best_forward_reversal >= HALF_FLIP_MIN_FORWARD_REVERSAL
        && candidate.max_forward_vertical >= HALF_FLIP_MIN_FORWARD_VERTICAL
}

fn candidate_confidence(candidate: &ActiveHalfFlipCandidate) -> f32 {
    let backward = HalfFlipCalculator::normalize_score(
        candidate.start_backward_alignment,
        HALF_FLIP_MIN_START_BACKWARD_ALIGNMENT,
        0.95,
    );
    let reorientation = HalfFlipCalculator::normalize_score(
        candidate.best_reorientation_alignment,
        HALF_FLIP_MIN_REORIENTATION_ALIGNMENT,
        0.98,
    );
    let reversal = HalfFlipCalculator::normalize_score(
        candidate.best_forward_reversal,
        HALF_FLIP_MIN_FORWARD_REVERSAL,
        0.98,
    );
    let flip = HalfFlipCalculator::normalize_score(
        candidate.max_forward_vertical,
        HALF_FLIP_MIN_FORWARD_VERTICAL,
        0.85,
    );
    let speed = HalfFlipCalculator::normalize_score(candidate.end_speed, 900.0, 1800.0).max(
        HalfFlipCalculator::normalize_score(
            candidate.end_speed - candidate.start_speed,
            100.0,
            700.0,
        ) * 0.7,
    );
    0.25 * backward + 0.30 * reorientation + 0.25 * reversal + 0.10 * flip + 0.10 * speed
}
