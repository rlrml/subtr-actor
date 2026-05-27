use super::*;

impl MustyFlickCalculator {
    pub(super) fn record_touch_event(
        &mut self,
        frame: &FrameInfo,
        player_id: &PlayerId,
        event: MustyFlickEvent,
    ) {
        let stats = self.player_stats.entry(player_id.clone()).or_default();
        stats.record_event(&event, event.aerial);
        stats.is_last_musty = true;
        stats.time_since_last_musty = Some((frame.time - event.time).max(0.0));
        stats.frames_since_last_musty = Some(frame.frame_number.saturating_sub(event.frame));

        self.current_last_musty_player = Some(player_id.clone());
        self.events.push(event);
    }
}
