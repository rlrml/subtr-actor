use super::*;

pub(super) fn speed_flip_cancel_score(candidate: &ActiveSpeedFlipCandidate) -> f32 {
    let cancel_recovery = candidate.latest_forward_z - candidate.min_forward_z;
    let level_recovery_score =
        1.0 - SpeedFlipCalculator::normalize_score(candidate.latest_forward_z.abs(), 0.05, 0.55);
    0.25 * SpeedFlipCalculator::normalize_score(-candidate.min_forward_z, 0.05, 0.35)
        + 0.35 * SpeedFlipCalculator::normalize_score(cancel_recovery, 0.08, 0.5)
        + 0.40 * level_recovery_score
}

pub(super) fn speed_flip_confidence(
    candidate: &ActiveSpeedFlipCandidate,
    cancel_score: f32,
    speed_score: f32,
    alignment_score: f32,
    boost_alignment_score: f32,
    timeliness_score: f32,
) -> f32 {
    0.30 * candidate.best_diagonal_score
        + 0.30 * cancel_score
        + 0.15 * speed_score
        + 0.15 * alignment_score
        + 0.05 * boost_alignment_score
        + 0.05 * timeliness_score
}

pub(super) fn speed_flip_event_from_candidate(
    player_id: &PlayerId,
    candidate: ActiveSpeedFlipCandidate,
    time_since_kickoff_start: f32,
    cancel_score: f32,
    speed_score: f32,
    confidence: f32,
) -> SpeedFlipEvent {
    SpeedFlipEvent {
        time: candidate.start_time,
        frame: candidate.start_frame,
        resolved_time: candidate.latest_time,
        resolved_frame: candidate.latest_frame,
        player: player_id.clone(),
        is_team_0: candidate.is_team_0,
        time_since_kickoff_start,
        start_position: candidate.start_position,
        end_position: candidate.end_position,
        start_speed: candidate.start_speed,
        max_speed: candidate.max_speed,
        best_alignment: candidate.best_alignment,
        diagonal_score: candidate.best_diagonal_score,
        cancel_score,
        speed_score,
        confidence,
    }
}
