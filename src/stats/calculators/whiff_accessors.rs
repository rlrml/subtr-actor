use super::*;

impl WhiffCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, WhiffStats> {
        &self.player_stats
    }

    pub fn events(&self) -> &[WhiffEvent] {
        &self.events
    }

    pub(super) fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_whiff = false;
            stats.time_since_last_whiff = stats
                .last_whiff_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_whiff = stats
                .last_whiff_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
    }
}
