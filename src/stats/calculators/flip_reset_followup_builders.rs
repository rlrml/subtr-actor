use super::flip_reset_builders::build_event_from_heuristic;
use super::*;

impl FlipResetTracker {
    pub(crate) fn build_flip_reset_followup_touch_candidate(
        &self,
        processor: &dyn ProcessorView,
        touch_event: &TouchEvent,
        frame_index: usize,
    ) -> Option<FlipResetEvent> {
        let player = touch_event.player.as_ref()?;
        let closest_approach_distance = touch_event.closest_approach_distance?;
        let ball_rigid_body = processor.get_normalized_ball_rigid_body().ok()?;
        let player_rigid_body = processor.get_normalized_player_rigid_body(player).ok()?;
        let heuristic = flip_reset_followup_touch_candidate(
            &ball_rigid_body,
            &player_rigid_body,
            closest_approach_distance,
        )?;

        Some(build_event_from_heuristic(
            heuristic,
            touch_event.time,
            frame_index,
            player,
            touch_event.team_is_team_0,
            closest_approach_distance,
        ))
    }
}
