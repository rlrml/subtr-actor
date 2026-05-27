use std::collections::HashSet;

use super::*;

impl RushCalculator {
    pub(super) fn rush_numbers(
        &self,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        attacking_team_is_team_0: bool,
    ) -> Option<(usize, usize)> {
        let ball_position = ball.position()?;
        let normalized_ball_y = normalized_y(attacking_team_is_team_0, ball_position);
        if normalized_ball_y > self.config.max_start_y {
            return None;
        }

        let demoed_players: HashSet<_> = events
            .active_demos
            .iter()
            .map(|demo| demo.victim.clone())
            .collect();

        let attackers = rush_player_count(
            players,
            &demoed_players,
            attacking_team_is_team_0,
            true,
            normalized_ball_y - self.config.attack_support_distance_y,
        );
        let defenders = rush_player_count(
            players,
            &demoed_players,
            attacking_team_is_team_0,
            false,
            normalized_ball_y + self.config.defender_distance_y,
        );

        (attackers >= 2 && defenders > 0).then_some((attackers, defenders))
    }
}

fn rush_player_count(
    players: &PlayerFrameState,
    demoed_players: &HashSet<PlayerId>,
    attacking_team_is_team_0: bool,
    count_attackers: bool,
    min_normalized_y: f32,
) -> usize {
    players
        .players
        .iter()
        .filter(|player| (player.is_team_0 == attacking_team_is_team_0) == count_attackers)
        .filter(|player| !demoed_players.contains(&player.player_id))
        .filter_map(PlayerSample::position)
        .filter(|position| normalized_y(attacking_team_is_team_0, *position) >= min_normalized_y)
        .count()
        .min(3)
}
