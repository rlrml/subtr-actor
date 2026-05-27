use super::*;

#[derive(Debug, Clone, Default)]
pub struct CenterCalculator {
    pub(super) player_stats: HashMap<PlayerId, CenterPlayerStats>,
    pub(super) team_zero_stats: CenterTeamStats,
    pub(super) team_one_stats: CenterTeamStats,
    pub(super) events: Vec<CenterEvent>,
    pub(super) pending_touch: Option<PendingCenterTouch>,
    pub(super) current_last_center_player: Option<PlayerId>,
}

impl CenterCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, CenterPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &CenterTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &CenterTeamStats {
        &self.team_one_stats
    }

    pub fn events(&self) -> &[CenterEvent] {
        &self.events
    }
}
