use super::*;

impl FlipResetTracker {
    pub(crate) fn update_flip_reset_proximity_events(
        &mut self,
        processor: &dyn ProcessorView,
        current_time: f32,
        frame_index: usize,
    ) {
        const PROXIMITY_EVENT_DEBOUNCE_SECONDS: f32 = 0.35;

        let player_ids: Vec<_> = processor.iter_player_ids_in_order().cloned().collect();
        for player in player_ids {
            if self
                .current_frame_flip_reset_events
                .iter()
                .any(|event| event.player == player)
                || self
                    .recent_flip_reset_proximity_event_time
                    .get(&player)
                    .is_some_and(|previous| {
                        current_time - previous < PROXIMITY_EVENT_DEBOUNCE_SECONDS
                    })
            {
                continue;
            }

            let Some(event) = self.build_flip_reset_proximity_event(
                processor,
                &player,
                current_time,
                frame_index,
            ) else {
                continue;
            };
            self.recent_flip_reset_proximity_event_time
                .insert(player, current_time);
            self.current_frame_flip_reset_events.push(event.clone());
            self.flip_reset_events.push(event);
        }
    }
}
