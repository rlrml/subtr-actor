use super::*;

impl FlipResetTracker {
    pub(crate) fn update_wall_contact_times(
        &mut self,
        processor: &dyn ProcessorView,
        current_time: f32,
        player_ids: &[PlayerId],
    ) {
        for player_id in player_ids {
            let Ok(player_rigid_body) = processor.get_normalized_player_rigid_body(player_id)
            else {
                self.previous_dodge_active.remove(player_id);
                continue;
            };

            if Self::player_is_grounded_for_wall_sequence(&player_rigid_body) {
                self.recent_wall_contact_time.remove(player_id);
            } else if Self::player_is_touching_wall(&player_rigid_body) {
                self.recent_wall_contact_time
                    .insert(player_id.clone(), current_time);
            }
        }
    }
}
