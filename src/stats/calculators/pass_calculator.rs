use super::*;

#[derive(Debug, Clone, Default)]
pub struct PassCalculator {
    pub(super) player_stats: HashMap<PlayerId, PassPlayerStats>,
    pub(super) team_zero_stats: PassTeamStats,
    pub(super) team_one_stats: PassTeamStats,
    pub(super) events: Vec<PassEvent>,
    pub(super) last_completed_events: Vec<PassLastCompletedEvent>,
    pub(super) last_touch: Option<PendingPassTouch>,
    pub(super) current_last_completed_pass_player: Option<PlayerId>,
    pub(super) emitted_last_completed_pass_player: Option<PlayerId>,
}

impl PassCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, PassPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &PassTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &PassTeamStats {
        &self.team_one_stats
    }

    pub fn events(&self) -> &[PassEvent] {
        &self.events
    }

    pub fn last_completed_events(&self) -> &[PassLastCompletedEvent] {
        &self.last_completed_events
    }
}
