use super::*;

impl SpeedFlipCalculator {
    pub(super) fn apply_event(&mut self, event: SpeedFlipEvent) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_speed_flip = false;
        }

        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.record_event(&event);
        stats.is_last_speed_flip = true;
        stats.time_since_last_speed_flip = Some(0.0);
        stats.frames_since_last_speed_flip = Some(0);

        self.current_last_speed_flip_player = Some(event.player.clone());
        self.events.push(event);
    }
}
