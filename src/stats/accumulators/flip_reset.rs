use super::*;

/// Per-player accumulated confirmed flip-reset stats.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FlipResetStats {
    pub count: u32,
    /// Total seconds between getting the reset and using it with a dodge touch.
    pub total_time_to_use: f32,
    /// Fastest reset-to-use latency in seconds, if any flip reset was confirmed.
    pub min_time_to_use: Option<f32>,
}

impl FlipResetStats {
    /// Mean seconds between getting a flip reset and using it. Zero when no
    /// reset was confirmed.
    pub fn mean_time_to_use(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_time_to_use / self.count as f32
        }
    }
}

/// Accumulates confirmed flip-reset stats over the replay.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FlipResetStatsAccumulator {
    player_stats: HashMap<PlayerId, FlipResetStats>,
}

impl FlipResetStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, FlipResetStats> {
        &self.player_stats
    }

    pub fn apply_event(&mut self, event: &FlipResetEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.count += 1;
        stats.total_time_to_use += event.time_since_reset;
        stats.min_time_to_use = Some(
            stats
                .min_time_to_use
                .map_or(event.time_since_reset, |current| {
                    current.min(event.time_since_reset)
                }),
        );
    }
}
