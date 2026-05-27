use super::*;

impl FlipResetTracker {
    pub(crate) fn build_flip_reset_event(
        &self,
        processor: &dyn ProcessorView,
        touch_event: &TouchEvent,
        frame_index: usize,
    ) -> Option<FlipResetEvent> {
        let player = touch_event.player.as_ref()?;
        let closest_approach_distance = touch_event.closest_approach_distance?;
        self.build_flip_reset_event_for_player(
            processor,
            player,
            touch_event.time,
            frame_index,
            touch_event.team_is_team_0,
            closest_approach_distance,
        )
    }

    pub(crate) fn build_flip_reset_event_for_player(
        &self,
        processor: &dyn ProcessorView,
        player: &PlayerId,
        time: f32,
        frame_index: usize,
        is_team_0: bool,
        closest_approach_distance: f32,
    ) -> Option<FlipResetEvent> {
        let ball_rigid_body = processor.get_normalized_ball_rigid_body().ok()?;
        let player_rigid_body = processor.get_normalized_player_rigid_body(player).ok()?;
        let heuristic = flip_reset_candidate(
            &ball_rigid_body,
            &player_rigid_body,
            closest_approach_distance,
        )?;

        Some(build_event_from_heuristic(
            heuristic,
            time,
            frame_index,
            player,
            is_team_0,
            closest_approach_distance,
        ))
    }
}

pub(crate) fn build_event_from_heuristic(
    heuristic: FlipResetHeuristic,
    time: f32,
    frame_index: usize,
    player: &PlayerId,
    is_team_0: bool,
    closest_approach_distance: f32,
) -> FlipResetEvent {
    FlipResetEvent {
        time,
        frame: frame_index,
        player: player.clone(),
        is_team_0,
        confidence: heuristic.confidence,
        local_ball_position: glam_to_vec(&heuristic.local_ball_position),
        closest_approach_distance,
    }
}
