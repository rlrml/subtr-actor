use super::*;

#[path = "goal_tags_point_events_impls.rs"]
mod impls;

pub(super) trait GoalMechanicPointEvent {
    fn event_time(&self) -> f32;
    fn event_frame(&self) -> usize;
    fn event_player(&self) -> &PlayerId;
    fn event_team_is_team_0(&self) -> bool;
    fn event_confidence(&self) -> f32;
    fn evidence_kind(&self) -> GoalTagEvidenceKind;
}
