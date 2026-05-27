use super::*;

impl WallAerialShotCalculator {
    pub(super) fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_wall_aerial_shot = false;
            stats.time_since_last_wall_aerial_shot = stats
                .last_wall_aerial_shot_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_wall_aerial_shot = stats
                .last_wall_aerial_shot_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
    }

    pub(super) fn record_event(&mut self, frame: &FrameInfo, event: WallAerialShotEvent) {
        self.record_stats(frame, &event);
        self.current_last_wall_aerial_shot_player = Some(event.player.clone());
        self.recent_wall_contacts.remove(&event.player);
        self.armed_shots.remove(&event.player);
        self.events.push(event);
    }

    fn record_stats(&mut self, frame: &FrameInfo, event: &WallAerialShotEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.count += 1;
        if event.confidence >= WALL_AERIAL_HIGH_CONFIDENCE {
            stats.high_confidence_count += 1;
        }
        stats.is_last_wall_aerial_shot = true;
        stats.last_wall_aerial_shot_time = Some(event.time);
        stats.last_wall_aerial_shot_frame = Some(event.frame);
        stats.time_since_last_wall_aerial_shot = Some((frame.time - event.time).max(0.0));
        stats.frames_since_last_wall_aerial_shot =
            Some(frame.frame_number.saturating_sub(event.frame));
        stats.last_confidence = Some(event.confidence);
        stats.best_confidence = stats.best_confidence.max(event.confidence);
        stats.cumulative_confidence += event.confidence;
        stats.cumulative_takeoff_to_shot_time += event.time_since_takeoff;
        stats.cumulative_shot_height += event.player_position[2];
    }
}
