use super::*;

impl PositioningCalculator {
    pub(crate) fn record_ball_distance_roles(
        &mut self,
        frame: &FrameInfo,
        ball_position: glam::Vec3,
        team_players: &[(&PlayerSample, glam::Vec3)],
        event_deltas: &mut HashMap<PlayerId, PositioningEvent>,
    ) {
        if let Some((closest_player, _)) = team_players.iter().min_by(|(_, a), (_, b)| {
            a.distance(ball_position)
                .partial_cmp(&b.distance(ball_position))
                .unwrap()
        }) {
            self.player_stats
                .entry(closest_player.player_id.clone())
                .or_default()
                .time_closest_to_ball += frame.dt;
            Self::event_delta(
                event_deltas,
                frame,
                &closest_player.player_id,
                closest_player.is_team_0,
            )
            .time_closest_to_ball += frame.dt;
        }

        if let Some((farthest_player, _)) = team_players.iter().max_by(|(_, a), (_, b)| {
            a.distance(ball_position)
                .partial_cmp(&b.distance(ball_position))
                .unwrap()
        }) {
            self.player_stats
                .entry(farthest_player.player_id.clone())
                .or_default()
                .time_farthest_from_ball += frame.dt;
            Self::event_delta(
                event_deltas,
                frame,
                &farthest_player.player_id,
                farthest_player.is_team_0,
            )
            .time_farthest_from_ball += frame.dt;
        }
    }
}
