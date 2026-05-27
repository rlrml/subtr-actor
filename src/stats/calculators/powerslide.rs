use super::*;

#[path = "powerslide_update.rs"]
mod update;
#[path = "powerslide_types.rs"]
mod types;

pub use self::types::*;

#[derive(Debug, Clone, Default)]
pub struct PowerslideCalculator {
    player_stats: HashMap<PlayerId, PowerslideStats>,
    team_zero_stats: PowerslideStats,
    team_one_stats: PowerslideStats,
    last_active: HashMap<PlayerId, bool>,
    events: Vec<PowerslideEvent>,
}

impl PowerslideCalculator {
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

    pub fn events(&self) -> &[PowerslideEvent] {
        &self.events
    }
}
