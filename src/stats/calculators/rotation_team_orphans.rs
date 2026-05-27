use super::*;

impl RotationCalculator {
    pub(crate) fn emit_remaining_first_man_counts(
        &mut self,
        frame: &FrameInfo,
        is_team_0: bool,
        became_first_man_counts: HashMap<PlayerId, u32>,
        lost_first_man_counts: HashMap<PlayerId, u32>,
    ) {
        for (player_id, count) in became_first_man_counts {
            self.emit_orphaned_first_man_count(frame, &player_id, is_team_0, count, 0);
        }
        for (player_id, count) in lost_first_man_counts {
            self.emit_orphaned_first_man_count(frame, &player_id, is_team_0, 0, count);
        }
    }

    fn emit_orphaned_first_man_count(
        &mut self,
        frame: &FrameInfo,
        player_id: &PlayerId,
        is_team_0: bool,
        became_first_man_count: u32,
        lost_first_man_count: u32,
    ) {
        let (current_role_state, current_depth_state) = {
            let stats = self.player_stats.entry(player_id.clone()).or_default();
            (stats.current_role_state, stats.current_depth_state)
        };
        self.emit_player_event_if_changed(
            frame,
            player_id,
            is_team_0,
            false,
            current_role_state,
            current_depth_state,
            became_first_man_count,
            lost_first_man_count,
        );
    }
}
