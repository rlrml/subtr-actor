use super::*;

impl FlipResetTracker {
    pub(crate) fn update_dodge_rising_edges(
        &mut self,
        processor: &dyn ProcessorView,
    ) -> SubtrActorResult<()> {
        self.current_frame_dodge_rising_edges.clear();
        let player_ids: Vec<_> = processor.iter_player_ids_in_order().cloned().collect();

        for player_id in player_ids {
            let dodge_active = processor.get_dodge_active(&player_id).unwrap_or(0) % 2 == 1;
            let was_dodge_active = self
                .previous_dodge_active
                .insert(player_id.clone(), dodge_active)
                .unwrap_or(false);
            if dodge_active && !was_dodge_active {
                self.current_frame_dodge_rising_edges.push(player_id);
            }
        }

        Ok(())
    }
}
