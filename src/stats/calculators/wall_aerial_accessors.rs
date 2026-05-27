use super::*;

impl WallAerialCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, WallAerialStats> {
        &self.player_stats
    }

    pub fn events(&self) -> &[WallAerialEvent] {
        &self.events
    }

    pub(super) fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_wall_aerial = false;
            stats.time_since_last_wall_aerial = stats
                .last_wall_aerial_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_wall_aerial = stats
                .last_wall_aerial_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
    }
}
