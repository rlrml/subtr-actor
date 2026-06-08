use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DodgeResetStats {
    pub count: u32,
    pub on_ball_count: u32,
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
}
