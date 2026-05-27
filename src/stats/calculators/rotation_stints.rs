use super::*;

impl RotationCalculator {
    pub(crate) fn close_first_man_stint(&mut self, player_id: &PlayerId) {
        if let Some(state) = self.first_man_stints.get_mut(player_id) {
            state.active = false;
            state.current_first_man_time = 0.0;
            state.non_first_man_seconds = 0.0;
        }
    }

    pub(crate) fn update_first_man_stint(
        &mut self,
        player_id: &PlayerId,
        stats: &mut RotationPlayerStats,
        role_state: RoleState,
        dt: f32,
    ) {
        let state = self.first_man_stints.entry(player_id.clone()).or_default();
        if role_state == RoleState::FirstMan {
            if !state.active {
                state.active = true;
                state.current_first_man_time = 0.0;
                stats.first_man_stint_count += 1;
            }
            state.current_first_man_time += dt;
            stats.longest_first_man_stint_time = stats
                .longest_first_man_stint_time
                .max(state.current_first_man_time);
            state.non_first_man_seconds = 0.0;
            return;
        }

        if state.active {
            state.non_first_man_seconds += dt;
            if state.non_first_man_seconds > self.config.first_man_debounce_seconds {
                state.active = false;
                state.current_first_man_time = 0.0;
                state.non_first_man_seconds = 0.0;
            }
        }
    }
}
