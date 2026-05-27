use super::*;

impl FlipResetTracker {
    pub(crate) fn prune_grounded_flip_reset_candidates(
        &mut self,
        processor: &dyn ProcessorView,
        player_ids: &[PlayerId],
    ) {
        for player_id in player_ids {
            let Ok(player_rigid_body) = processor.get_normalized_player_rigid_body(player_id)
            else {
                self.recent_flip_reset_candidates.remove(player_id);
                continue;
            };
            if Self::player_is_grounded_for_wall_sequence(&player_rigid_body) {
                self.recent_flip_reset_candidates.remove(player_id);
            }
        }
    }
}
