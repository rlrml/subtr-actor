use super::*;

impl FlipResetTracker {
    pub(crate) fn update_flip_reset_followup_dodge_events(
        &mut self,
        processor: &dyn ProcessorView,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        self.current_frame_flip_reset_followup_dodge_events.clear();
        let current_time = frame.time;
        let player_ids: Vec<_> = processor.iter_player_ids_in_order().cloned().collect();
        self.prune_grounded_flip_reset_candidates(processor, &player_ids);
        self.update_followup_touch_candidates(processor, frame_index);

        let dodge_edges = self.current_frame_dodge_rising_edges.clone();
        for player_id in dodge_edges {
            self.maybe_emit_flip_reset_followup_dodge(
                processor,
                current_time,
                frame_index,
                &player_id,
            );
        }

        Ok(())
    }

    fn update_followup_touch_candidates(
        &mut self,
        processor: &dyn ProcessorView,
        frame_index: usize,
    ) {
        for touch_event in processor.current_frame_touch_events() {
            let Some(event) =
                self.build_flip_reset_followup_touch_candidate(processor, touch_event, frame_index)
            else {
                continue;
            };
            self.recent_flip_reset_candidates
                .insert(event.player.clone(), event);
        }
    }
}
