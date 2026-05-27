use super::*;

impl PositioningCalculator {
    pub(crate) fn record_teammate_distances(
        &mut self,
        frame: &FrameInfo,
        team_players: &[(&PlayerSample, glam::Vec3)],
        event_deltas: &mut HashMap<PlayerId, PositioningEvent>,
    ) {
        for (player, position) in team_players {
            let teammate_distance_sum: f32 = team_players
                .iter()
                .filter(|(other_player, _)| other_player.player_id != player.player_id)
                .map(|(_, other_position)| position.distance(*other_position))
                .sum();
            let teammate_count = team_players.len().saturating_sub(1);
            if teammate_count == 0 {
                continue;
            }

            let distance_time = teammate_distance_sum * frame.dt / teammate_count as f32;
            self.player_stats
                .entry(player.player_id.clone())
                .or_default()
                .sum_distance_to_teammates += distance_time;
            Self::event_delta(event_deltas, frame, &player.player_id, player.is_team_0)
                .sum_distance_to_teammates += distance_time;
        }
    }
}
