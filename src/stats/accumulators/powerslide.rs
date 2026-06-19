use super::*;

/// Per-player accumulated powerslide press count and total duration.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PowerslideStats {
    pub total_duration: f32,
    pub press_count: u32,
}

impl PowerslideStats {
    pub fn average_duration(&self) -> f32 {
        if self.press_count == 0 {
            0.0
        } else {
            self.total_duration / self.press_count as f32
        }
    }
}

/// Accumulates powerslide stats over the replay.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PowerslideStatsAccumulator {
    player_stats: HashMap<PlayerId, PowerslideStats>,
    team_zero_stats: PowerslideStats,
    team_one_stats: PowerslideStats,
}

impl PowerslideStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, PowerslideStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &PowerslideStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &PowerslideStats {
        &self.team_one_stats
    }

    pub fn apply_sample(
        &mut self,
        player_id: &PlayerId,
        is_team_0: bool,
        active: bool,
        previous_active: bool,
        dt: f32,
        live_play: bool,
    ) {
        let stats = self.player_stats.entry(player_id.clone()).or_default();
        let team_stats = if is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };

        if live_play && active {
            stats.total_duration += dt;
            team_stats.total_duration += dt;
        }

        if live_play && active && !previous_active {
            stats.press_count += 1;
            team_stats.press_count += 1;
        }
    }
}
