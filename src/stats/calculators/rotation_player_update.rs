use super::rotation_depth::play_depth_state;
use super::*;

impl RotationCalculator {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn update_rotating_player(
        &mut self,
        frame: &FrameInfo,
        player: &PlayerSample,
        position: glam::Vec3,
        ball_position: glam::Vec3,
        is_team_0: bool,
        role_assignments: &HashMap<PlayerId, RoleState>,
        became_first_man_counts: &mut HashMap<PlayerId, u32>,
        lost_first_man_counts: &mut HashMap<PlayerId, u32>,
    ) {
        let role_state = role_assignments
            .get(&player.player_id)
            .copied()
            .unwrap_or(RoleState::Ambiguous);
        let depth_state = play_depth_state(
            is_team_0,
            position,
            ball_position,
            self.config.role_depth_margin,
        );
        let (current_role_state, current_depth_state) =
            self.update_player_stats(frame, player, role_state, depth_state);
        self.emit_player_event_if_changed(
            frame,
            &player.player_id,
            player.is_team_0,
            true,
            current_role_state,
            current_depth_state,
            became_first_man_counts
                .remove(&player.player_id)
                .unwrap_or_default(),
            lost_first_man_counts
                .remove(&player.player_id)
                .unwrap_or_default(),
        );
    }

    fn update_player_stats(
        &mut self,
        frame: &FrameInfo,
        player: &PlayerSample,
        role_state: RoleState,
        depth_state: PlayDepthState,
    ) -> (RoleState, PlayDepthState) {
        let mut stats = self
            .player_stats
            .remove(&player.player_id)
            .unwrap_or_default();
        stats.active_game_time += frame.dt;
        stats.tracked_time += frame.dt;
        stats.current_role_state = role_state;
        stats.current_depth_state = depth_state;
        self.update_first_man_stint(&player.player_id, &mut stats, role_state, frame.dt);
        add_role_time(&mut stats, role_state, frame.dt);
        add_depth_time(&mut stats, depth_state, frame.dt);

        let current_role_state = stats.current_role_state;
        let current_depth_state = stats.current_depth_state;
        self.player_stats.insert(player.player_id.clone(), stats);
        (current_role_state, current_depth_state)
    }
}
