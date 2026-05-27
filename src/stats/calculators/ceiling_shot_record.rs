use super::*;

impl CeilingShotCalculator {
    pub(super) fn record_touch_event(
        &mut self,
        frame: &FrameInfo,
        player_id: &PlayerId,
        event: CeilingShotEvent,
    ) {
        let stats = self.player_stats.entry(player_id.clone()).or_default();
        stats.record_event(&event);
        stats.is_last_ceiling_shot = true;
        stats.time_since_last_ceiling_shot = Some((frame.time - event.time).max(0.0));
        stats.frames_since_last_ceiling_shot = Some(frame.frame_number.saturating_sub(event.frame));

        self.current_last_ceiling_shot_player = Some(player_id.clone());
        self.events.push(event);
    }
}
