use super::*;

impl MatchStatsCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, CorePlayerStats> {
        &self.player_stats
    }

    pub fn timeline(&self) -> &[TimelineEvent] {
        &self.timeline
    }

    pub fn goal_context_events(&self) -> &[GoalContextEvent] {
        &self.goal_context_events
    }

    pub fn core_player_events(&self) -> &[CorePlayerStatsEvent] {
        &self.core_player_events
    }

    pub fn core_team_events(&self) -> &[CoreTeamStatsEvent] {
        &self.core_team_events
    }
}
