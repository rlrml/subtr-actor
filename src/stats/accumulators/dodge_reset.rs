use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DodgeResetStats {
    pub count: u32,
    pub on_ball_count: u32,
    /// On-ball resets (flip resets) converted by a dodge-powered touch.
    pub flip_reset_used_count: u32,
    /// On-ball resets that resolved without being used (landed, superseded by
    /// another reset, or cut off by a goal / play ending / the replay ending).
    pub flip_reset_unused_count: u32,
    /// Total seconds between reset and use, summed over all used flip resets.
    pub flip_reset_total_time_to_use: f32,
    /// Fastest reset-to-use latency in seconds, if any flip reset was used.
    pub flip_reset_min_time_to_use: Option<f32>,
}

impl DodgeResetStats {
    /// Mean seconds between getting a flip reset and using it, over used
    /// resets. Zero when no reset was used.
    pub fn flip_reset_mean_time_to_use(&self) -> f32 {
        if self.flip_reset_used_count == 0 {
            0.0
        } else {
            self.flip_reset_total_time_to_use / self.flip_reset_used_count as f32
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DodgeResetStatsAccumulator {
    player_stats: HashMap<PlayerId, DodgeResetStats>,
}

impl DodgeResetStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, DodgeResetStats> {
        &self.player_stats
    }

    pub fn apply_event(&mut self, event: &DodgeResetEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.count += 1;
        if event.on_ball {
            stats.on_ball_count += 1;
        }
    }

    pub fn apply_flip_reset_outcome_event(&mut self, event: &FlipResetOutcomeEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        if let Some(time_to_use) = event.time_to_use {
            stats.flip_reset_used_count += 1;
            stats.flip_reset_total_time_to_use += time_to_use;
            stats.flip_reset_min_time_to_use = Some(
                stats
                    .flip_reset_min_time_to_use
                    .map_or(time_to_use, |current| current.min(time_to_use)),
            );
        } else {
            stats.flip_reset_unused_count += 1;
        }
    }
}
