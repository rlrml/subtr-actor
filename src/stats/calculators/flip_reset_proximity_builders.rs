use super::flip_reset_builders::build_event_from_heuristic;
use super::*;

impl FlipResetTracker {
    pub(crate) fn build_flip_reset_proximity_event(
        &self,
        processor: &dyn ProcessorView,
        player: &PlayerId,
        time: f32,
        frame_index: usize,
    ) -> Option<FlipResetEvent> {
        let ball_rigid_body = processor.get_normalized_ball_rigid_body().ok()?;
        let player_rigid_body = processor.get_normalized_player_rigid_body(player).ok()?;
        let heuristic = flip_reset_proximity_candidate(&ball_rigid_body, &player_rigid_body)?;
        let raw_ball_position = vec_to_glam(&ball_rigid_body.location);
        let raw_player_position = vec_to_glam(&player_rigid_body.location);
        let scale_factor = scale_factor_for_positions(raw_ball_position, raw_player_position);
        let closest_approach_distance =
            (raw_ball_position - raw_player_position).length() * scale_factor;

        Some(build_event_from_heuristic(
            heuristic,
            time,
            frame_index,
            player,
            processor.get_player_is_team_0(player).unwrap_or(false),
            closest_approach_distance,
        ))
    }
}
