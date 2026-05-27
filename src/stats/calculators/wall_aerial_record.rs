use super::*;

impl WallAerialCalculator {
    pub(super) fn record_event(&mut self, frame: &FrameInfo, mut event: WallAerialEvent) {
        event.sample_time = frame.time;
        event.sample_frame = frame.frame_number;
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.count += 1;
        if event.confidence >= WALL_AERIAL_HIGH_CONFIDENCE {
            stats.high_confidence_count += 1;
        }
        stats.is_last_wall_aerial = true;
        stats.last_wall_aerial_time = Some(event.time);
        stats.last_wall_aerial_frame = Some(event.frame);
        stats.time_since_last_wall_aerial = Some((frame.time - event.time).max(0.0));
        stats.frames_since_last_wall_aerial = Some(frame.frame_number.saturating_sub(event.frame));
        stats.last_confidence = Some(event.confidence);
        stats.best_confidence = stats.best_confidence.max(event.confidence);
        stats.cumulative_confidence += event.confidence;
        stats.cumulative_setup_duration += event.setup_duration;
        stats.cumulative_takeoff_to_touch_time += event.time_since_takeoff;
        stats.cumulative_touch_height += event.player_position[2];

        self.current_last_wall_aerial_player = Some(event.player.clone());
        self.recent_event_times
            .insert(event.player.clone(), event.time);
        self.events.push(event);
    }

    pub(super) fn mark_current_last_wall_aerial_player(&mut self) {
        if let Some(player_id) = self.current_last_wall_aerial_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_wall_aerial = true;
            }
        }
    }
}
