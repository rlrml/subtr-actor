use super::rotation_scoring::{raw_first_man, role_assignments, scored_players};
use super::*;

impl RotationCalculator {
    pub(crate) fn update_team(
        &mut self,
        is_team_0: bool,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball_position: glam::Vec3,
        players: &PlayerFrameState,
        demoed_players: &HashSet<PlayerId>,
    ) {
        let present_team_count = players
            .players
            .iter()
            .filter(|player| player.is_team_0 == is_team_0)
            .count();
        let team_size = gameplay
            .current_in_game_team_player_count(is_team_0)
            .max(present_team_count);
        let team_players = active_team_players(is_team_0, players, demoed_players);
        if !(2..=3).contains(&team_size) || team_players.len() != team_size {
            self.emit_invalid_team_events(is_team_0, frame, players);
            return;
        }

        let scored_players = scored_players(&team_players, ball_position);
        let raw_first_man = raw_first_man(&scored_players, self.config.first_man_ambiguity_margin);
        let (mut became_counts, mut lost_counts) =
            self.record_first_man_change(is_team_0, frame, raw_first_man);
        let stable_first_man = raw_first_man
            .and_then(|_| self.team_tracker(is_team_0).stable_first_man.as_ref())
            .cloned();
        let role_assignments = role_assignments(stable_first_man.as_ref(), &scored_players);

        for (player, position) in team_players {
            self.update_rotating_player(
                frame,
                player,
                position,
                ball_position,
                is_team_0,
                &role_assignments,
                &mut became_counts,
                &mut lost_counts,
            );
        }
        self.emit_remaining_first_man_counts(frame, is_team_0, became_counts, lost_counts);
    }
}

fn active_team_players<'a>(
    is_team_0: bool,
    players: &'a PlayerFrameState,
    demoed_players: &HashSet<PlayerId>,
) -> Vec<(&'a PlayerSample, glam::Vec3)> {
    players
        .players
        .iter()
        .filter(|player| player.is_team_0 == is_team_0)
        .filter(|player| !demoed_players.contains(&player.player_id))
        .filter_map(|player| player.position().map(|position| (player, position)))
        .collect()
}
