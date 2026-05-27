use super::*;

impl RotationCalculator {
    pub(crate) fn emit_invalid_team_events(
        &mut self,
        is_team_0: bool,
        frame: &FrameInfo,
        players: &PlayerFrameState,
    ) {
        self.team_tracker_mut(is_team_0).reset();
        for player in players
            .players
            .iter()
            .filter(|player| player.is_team_0 == is_team_0)
        {
            self.close_first_man_stint(&player.player_id);
            let (current_role_state, current_depth_state) = {
                let stats = self
                    .player_stats
                    .entry(player.player_id.clone())
                    .or_default();
                stats.current_role_state = RoleState::Unknown;
                (stats.current_role_state, stats.current_depth_state)
            };
            self.emit_player_event_if_changed(
                frame,
                &player.player_id,
                player.is_team_0,
                false,
                current_role_state,
                current_depth_state,
                0,
                0,
            );
        }
    }
}
