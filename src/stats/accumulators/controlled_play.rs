use super::*;

/// Accumulated controlled-play stats: counts, times, and ball advance.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct ControlledPlayStats {
    pub count: u32,
    pub total_time: f32,
    pub longest_time: f32,
    pub touch_count: u32,
    pub total_advance_distance: f32,
}

impl ControlledPlayStats {
    pub fn avg_time(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_time / self.count as f32
        }
    }
}

/// Accumulates controlled-play stats over the replay.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ControlledPlayStatsAccumulator {
    player_stats: HashMap<PlayerId, ControlledPlayStats>,
    team_zero_stats: ControlledPlayStats,
    team_one_stats: ControlledPlayStats,
}

impl ControlledPlayStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, ControlledPlayStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &ControlledPlayStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &ControlledPlayStats {
        &self.team_one_stats
    }

    pub fn apply_event(&mut self, event: &ControlledPlayEvent) {
        Self::apply_event_to_stats(
            self.player_stats
                .entry(event.player_id.clone())
                .or_default(),
            event,
        );
        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        Self::apply_event_to_stats(team_stats, event);
    }

    fn apply_event_to_stats(stats: &mut ControlledPlayStats, event: &ControlledPlayEvent) {
        stats.count += 1;
        stats.total_time += event.duration;
        stats.longest_time = stats.longest_time.max(event.duration);
        stats.touch_count += event.touch_count;
        stats.total_advance_distance += event.total_advance_distance;
    }
}
