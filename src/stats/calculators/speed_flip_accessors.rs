use super::*;

impl SpeedFlipCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, SpeedFlipStats> {
        &self.player_stats
    }

    pub fn events(&self) -> &[SpeedFlipEvent] {
        &self.events
    }

    pub(super) fn player_by_id<'a>(
        players: &'a PlayerFrameState,
        player_id: &PlayerId,
    ) -> Option<&'a PlayerSample> {
        players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
    }

    pub(super) fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_speed_flip = false;
            stats.time_since_last_speed_flip = stats
                .last_speed_flip_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_speed_flip = stats
                .last_speed_flip_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }

        if let Some(player_id) = self.current_last_speed_flip_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_speed_flip = true;
            }
        }
    }
}
