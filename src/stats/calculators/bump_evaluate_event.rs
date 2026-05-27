use super::bump_evaluate_selection::BumpSelection;
use super::bump_geometry::vec3_to_array;
use super::*;

pub(super) fn bump_event_from_selection(
    frame: &FrameInfo,
    contact_distance: f32,
    selected: BumpSelection<'_>,
) -> Option<BumpEvent> {
    let (initiator, victim, initiator_body, victim_body, candidate, reverse_score) = selected;
    if candidate.score < BUMP_MIN_DIRECTIONAL_SCORE
        || candidate.score - reverse_score < BUMP_MIN_SCORE_MARGIN
        || candidate.closing_speed < BUMP_MIN_CLOSING_SPEED
        || candidate.victim_impulse < BUMP_MIN_VICTIM_IMPULSE
        || candidate.initiator_slowdown < BUMP_MIN_INITIATOR_SLOWDOWN
    {
        return None;
    }

    let distance_factor = (1.0 - (contact_distance / BUMP_MAX_CONTACT_DISTANCE)).clamp(0.0, 1.0);
    let score_factor = ((candidate.score - BUMP_MIN_DIRECTIONAL_SCORE) / 900.0).clamp(0.0, 1.0);
    let margin_factor =
        ((candidate.score - reverse_score - BUMP_MIN_SCORE_MARGIN) / 500.0).clamp(0.0, 1.0);
    let confidence =
        (0.35 + 0.3 * distance_factor + 0.25 * score_factor + 0.1 * margin_factor).clamp(0.0, 1.0);

    Some(BumpEvent {
        time: frame.time,
        frame: frame.frame_number,
        initiator: initiator.player_id.clone(),
        victim: victim.player_id.clone(),
        initiator_is_team_0: initiator.is_team_0,
        victim_is_team_0: victim.is_team_0,
        is_team_bump: initiator.is_team_0 == victim.is_team_0,
        strength: candidate.score,
        confidence,
        contact_distance,
        closing_speed: candidate.closing_speed,
        victim_impulse: candidate.victim_impulse,
        initiator_position: vec3_to_array(vec_to_glam(&initiator_body.location)),
        victim_position: vec3_to_array(vec_to_glam(&victim_body.location)),
    })
}
