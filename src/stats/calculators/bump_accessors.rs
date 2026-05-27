use super::*;

impl BumpCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, BumpPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &BumpTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &BumpTeamStats {
        &self.team_one_stats
    }

    pub fn events(&self) -> &[BumpEvent] {
        &self.events
    }
}
