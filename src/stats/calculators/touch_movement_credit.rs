use super::*;

impl TouchCalculator {
    pub(crate) fn apply_ball_movement_credit(
        &mut self,
        frame: usize,
        time: f32,
        player_id: &PlayerId,
        team_is_team_0: bool,
        delta: glam::Vec3,
        travel_distance: f32,
    ) {
        let (advance_distance, retreat_distance) =
            directional_ball_distances(delta.y, team_is_team_0);
        self.ball_movement_events.push(TouchBallMovementEvent {
            time,
            frame,
            player: player_id.clone(),
            is_team_0: team_is_team_0,
            travel_distance,
            advance_distance,
            retreat_distance,
        });
        self.add_ball_movement_stats(
            player_id,
            travel_distance,
            advance_distance,
            retreat_distance,
        );
    }

    pub(crate) fn add_ball_movement_stats(
        &mut self,
        player_id: &PlayerId,
        travel_distance: f32,
        advance_distance: f32,
        retreat_distance: f32,
    ) {
        let stats = self.player_stats.entry(player_id.clone()).or_default();
        stats.total_ball_travel_distance += travel_distance;
        stats.total_ball_advance_distance += advance_distance;
        stats.total_ball_retreat_distance += retreat_distance;
    }
}

pub(crate) fn directional_ball_distances(y_delta: f32, team_is_team_0: bool) -> (f32, f32) {
    let team_forward_sign = if team_is_team_0 { 1.0 } else { -1.0 };
    let advance_distance = y_delta * team_forward_sign;
    if advance_distance >= 0.0 {
        (advance_distance, 0.0)
    } else {
        (0.0, -advance_distance)
    }
}
