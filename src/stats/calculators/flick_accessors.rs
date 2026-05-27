use super::*;

impl FlickCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, FlickStats> {
        &self.player_stats
    }

    pub fn events(&self) -> &[FlickEvent] {
        &self.events
    }

    pub(super) fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_flick = false;
            stats.time_since_last_flick = stats
                .last_flick_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_flick = stats
                .last_flick_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }

        if let Some(player_id) = self.current_last_flick_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_flick = true;
            }
        }
    }
}
