use super::*;
use crate::stats::calculators::WALL_AERIAL_HIGH_CONFIDENCE;

/// Per-player accumulated wall-aerial stats with confidence.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct WallAerialStats {
    pub count: u32,
    pub high_confidence_count: u32,
    pub is_last_wall_aerial: bool,
    pub last_wall_aerial_time: Option<f32>,
    pub last_wall_aerial_frame: Option<usize>,
    pub time_since_last_wall_aerial: Option<f32>,
    pub frames_since_last_wall_aerial: Option<usize>,
    pub last_confidence: Option<f32>,
    pub best_confidence: f32,
    pub cumulative_confidence: f32,
    pub cumulative_setup_duration: f32,
    pub cumulative_takeoff_to_touch_time: f32,
    pub cumulative_touch_height: f32,
}

impl WallAerialStats {
    fn average(&self, value: f32) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            value / self.count as f32
        }
    }

    pub fn average_confidence(&self) -> f32 {
        self.average(self.cumulative_confidence)
    }

    pub fn average_setup_duration(&self) -> f32 {
        self.average(self.cumulative_setup_duration)
    }

    pub fn average_takeoff_to_touch_time(&self) -> f32 {
        self.average(self.cumulative_takeoff_to_touch_time)
    }

    pub fn average_touch_height(&self) -> f32 {
        self.average(self.cumulative_touch_height)
    }
}

/// Accumulates wall-aerial stats over the replay.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WallAerialStatsAccumulator {
    player_stats: HashMap<PlayerId, WallAerialStats>,
    current_last_wall_aerial_player: Option<PlayerId>,
}

impl WallAerialStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, WallAerialStats> {
        &self.player_stats
    }

    pub fn begin_sample(&mut self, frame: &FrameInfo) {
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

    pub fn apply_event(&mut self, event: &WallAerialEvent, frame: &FrameInfo) {
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
    }

    pub fn restore_current_last_event_marker(&mut self) {
        if let Some(player_id) = self.current_last_wall_aerial_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_wall_aerial = true;
            }
        }
    }

    pub fn reset_current_last_event_marker(&mut self) {
        self.current_last_wall_aerial_player = None;
    }
}
