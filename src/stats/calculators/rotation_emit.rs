use super::*;

impl RotationCalculator {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_player_event_if_changed(
        &mut self,
        frame: &FrameInfo,
        player_id: &PlayerId,
        is_team_0: bool,
        active: bool,
        current_role_state: RoleState,
        current_depth_state: PlayDepthState,
        became_first_man_count: u32,
        lost_first_man_count: u32,
    ) {
        let state = RotationPlayerEventState {
            active,
            current_role_state,
            current_depth_state,
        };
        let state_changed = self.last_emitted_player_states.get(player_id) != Some(&state);
        if !state_changed && became_first_man_count == 0 && lost_first_man_count == 0 {
            return;
        }

        let mut event = RotationPlayerEvent::new(
            frame,
            player_id.clone(),
            is_team_0,
            active,
            current_role_state,
            current_depth_state,
        );
        event.became_first_man_count = became_first_man_count;
        event.lost_first_man_count = lost_first_man_count;
        self.player_events.push(event);
        self.last_emitted_player_states
            .insert(player_id.clone(), state);
    }
}
