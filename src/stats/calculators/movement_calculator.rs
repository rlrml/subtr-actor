use super::*;

#[derive(Debug, Clone, Default)]
pub struct MovementCalculator {
    pub(super) player_stats: HashMap<PlayerId, MovementStats>,
    pub(super) player_teams: HashMap<PlayerId, bool>,
    pub(super) previous_positions: HashMap<PlayerId, glam::Vec3>,
    pub(super) team_zero_stats: MovementStats,
    pub(super) team_one_stats: MovementStats,
    pub(super) events: Vec<MovementEvent>,
}

impl MovementCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, MovementStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &MovementStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &MovementStats {
        &self.team_one_stats
    }

    pub fn events(&self) -> &[MovementEvent] {
        &self.events
    }
}
