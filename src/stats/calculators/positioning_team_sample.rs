use super::*;

impl PositioningCalculator {
    pub(crate) fn process_team_samples(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        players: &PlayerFrameState,
        ball_position: glam::Vec3,
        demoed_players: &HashSet<PlayerId>,
        event_deltas: &mut HashMap<PlayerId, PositioningEvent>,
    ) {
        for is_team_0 in [true, false] {
            let team_present_player_count = players
                .players
                .iter()
                .filter(|player| player.is_team_0 == is_team_0)
                .count();
            let team_roster_count = gameplay
                .current_in_game_team_player_count(is_team_0)
                .max(team_present_player_count);
            let team_players: Vec<_> = players
                .players
                .iter()
                .filter(|player| player.is_team_0 == is_team_0)
                .filter(|player| !demoed_players.contains(&player.player_id))
                .filter_map(|player| player.position().map(|position| (player, position)))
                .collect();

            if team_players.is_empty() {
                continue;
            }

            self.record_teammate_distances(frame, &team_players, event_deltas);
            self.record_team_roles(
                frame,
                is_team_0,
                team_present_player_count,
                team_roster_count,
                &team_players,
                event_deltas,
            );
            self.record_ball_distance_roles(frame, ball_position, &team_players, event_deltas);
        }
    }
}
