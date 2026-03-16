use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct DodgeResetStats {
    pub count: u32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DodgeResetReducer {
    player_stats: HashMap<PlayerId, DodgeResetStats>,
}

impl DodgeResetReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, DodgeResetStats> {
        &self.player_stats
    }
}

impl StatsReducer for DodgeResetReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        for event in &sample.dodge_refreshed_events {
            self.player_stats
                .entry(event.player.clone())
                .or_default()
                .count += 1;
        }
        Ok(())
    }
}
