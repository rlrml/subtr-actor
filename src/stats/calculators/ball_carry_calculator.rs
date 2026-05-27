use super::*;

#[derive(Debug, Clone, Default)]
pub struct BallCarryCalculator {
    pub(super) player_stats: HashMap<PlayerId, BallCarryStats>,
    pub(super) player_air_dribble_stats: HashMap<PlayerId, AirDribbleStats>,
    pub(super) team_zero_stats: BallCarryStats,
    pub(super) team_one_stats: BallCarryStats,
    pub(super) team_zero_air_dribble_stats: AirDribbleStats,
    pub(super) team_one_air_dribble_stats: AirDribbleStats,
    pub(super) carry_events: Vec<BallCarryEvent>,
    pub(super) processed_control_sequence_count: usize,
}

impl BallCarryCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, BallCarryStats> {
        &self.player_stats
    }

    pub fn player_air_dribble_stats(&self) -> &HashMap<PlayerId, AirDribbleStats> {
        &self.player_air_dribble_stats
    }

    pub fn team_zero_stats(&self) -> &BallCarryStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &BallCarryStats {
        &self.team_one_stats
    }

    pub fn team_zero_air_dribble_stats(&self) -> &AirDribbleStats {
        &self.team_zero_air_dribble_stats
    }

    pub fn team_one_air_dribble_stats(&self) -> &AirDribbleStats {
        &self.team_one_air_dribble_stats
    }

    pub fn carry_events(&self) -> &[BallCarryEvent] {
        &self.carry_events
    }
}
