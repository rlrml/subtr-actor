use super::*;

impl RotationCalculator {
    pub(crate) fn emit_inactive_player_events(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
    ) {
        for player in &players.players {
            self.close_first_man_stint(&player.player_id);
            let stats = self
                .player_stats
                .entry(player.player_id.clone())
                .or_default();
            let current_role_state = stats.current_role_state;
            let current_depth_state = stats.current_depth_state;
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

    pub(crate) fn reset_trackers(&mut self) {
        self.team_zero_tracker.reset();
        self.team_one_tracker.reset();
    }
}
