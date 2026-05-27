use super::*;

impl WavedashCalculator {
    pub(super) fn apply_event(&mut self, event: WavedashEvent) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_wavedash = false;
        }

        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.record_event(&event);
        stats.is_last_wavedash = true;
        stats.time_since_last_wavedash = Some(0.0);
        stats.frames_since_last_wavedash = Some(0);

        self.current_last_wavedash_player = Some(event.player.clone());
        self.events.push(event);
    }

    pub(super) fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_wavedash = false;
            stats.time_since_last_wavedash = stats
                .last_wavedash_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_wavedash = stats
                .last_wavedash_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }

        if let Some(player_id) = self.current_last_wavedash_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_wavedash = true;
            }
        }
    }
}
