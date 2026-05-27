use super::*;

impl FlipResetTracker {
    pub(crate) fn update_flip_reset_events(
        &mut self,
        processor: &dyn ProcessorView,
        current_time: f32,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        self.current_frame_flip_reset_events.clear();
        for touch_event in processor.current_frame_touch_events() {
            let event = self
                .build_flip_reset_event(processor, touch_event, frame_index)
                .or_else(|| self.best_team_touch_candidate(processor, touch_event, frame_index));
            let Some(event) = event else {
                continue;
            };
            self.current_frame_flip_reset_events.push(event.clone());
            self.flip_reset_events.push(event);
        }

        self.update_flip_reset_proximity_events(processor, current_time, frame_index);
        Ok(())
    }

    fn best_team_touch_candidate(
        &self,
        processor: &dyn ProcessorView,
        touch_event: &TouchEvent,
        frame_index: usize,
    ) -> Option<FlipResetEvent> {
        let ball_rigid_body = processor.get_normalized_ball_rigid_body().ok()?;
        let ball_position = vec_to_glam(&ball_rigid_body.location);
        processor
            .iter_player_ids_in_order()
            .filter(|player| {
                processor.get_player_is_team_0(player).ok() == Some(touch_event.team_is_team_0)
            })
            .filter_map(|player| {
                let player_rigid_body = processor.get_normalized_player_rigid_body(player).ok()?;
                let fallback_touch_distance =
                    (ball_position - vec_to_glam(&player_rigid_body.location)).length();
                self.build_flip_reset_event_for_player(
                    processor,
                    player,
                    touch_event.time,
                    frame_index,
                    touch_event.team_is_team_0,
                    fallback_touch_distance,
                )
            })
            .max_by(|left, right| {
                left.confidence
                    .partial_cmp(&right.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}
