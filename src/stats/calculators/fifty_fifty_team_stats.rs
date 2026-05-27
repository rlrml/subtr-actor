use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FiftyFiftyTeamStats {
    pub count: u32,
    pub wins: u32,
    pub losses: u32,
    pub neutral_outcomes: u32,
    pub kickoff_count: u32,
    pub kickoff_wins: u32,
    pub kickoff_losses: u32,
    pub kickoff_neutral_outcomes: u32,
    pub possession_after_count: u32,
    pub opponent_possession_after_count: u32,
    pub neutral_possession_after_count: u32,
    pub kickoff_possession_after_count: u32,
    pub kickoff_opponent_possession_after_count: u32,
    pub kickoff_neutral_possession_after_count: u32,
}

impl FiftyFiftyStats {
    pub fn for_team(&self, is_team_zero: bool) -> FiftyFiftyTeamStats {
        let stats = fifty_fifty_team_counts(self, is_team_zero);
        FiftyFiftyTeamStats {
            count: self.count,
            wins: stats.0,
            losses: stats.1,
            neutral_outcomes: self.neutral_outcomes,
            kickoff_count: self.kickoff_count,
            kickoff_wins: stats.2,
            kickoff_losses: stats.3,
            kickoff_neutral_outcomes: self.kickoff_neutral_outcomes,
            possession_after_count: stats.4,
            opponent_possession_after_count: stats.5,
            neutral_possession_after_count: self.neutral_possession_after_count,
            kickoff_possession_after_count: stats.6,
            kickoff_opponent_possession_after_count: stats.7,
            kickoff_neutral_possession_after_count: self.kickoff_neutral_possession_after_count,
        }
    }
}

fn fifty_fifty_team_counts(
    stats: &FiftyFiftyStats,
    is_team_zero: bool,
) -> (u32, u32, u32, u32, u32, u32, u32, u32) {
    if is_team_zero {
        (
            stats.team_zero_wins,
            stats.team_one_wins,
            stats.kickoff_team_zero_wins,
            stats.kickoff_team_one_wins,
            stats.team_zero_possession_after_count,
            stats.team_one_possession_after_count,
            stats.kickoff_team_zero_possession_after_count,
            stats.kickoff_team_one_possession_after_count,
        )
    } else {
        (
            stats.team_one_wins,
            stats.team_zero_wins,
            stats.kickoff_team_one_wins,
            stats.kickoff_team_zero_wins,
            stats.team_one_possession_after_count,
            stats.team_zero_possession_after_count,
            stats.kickoff_team_one_possession_after_count,
            stats.kickoff_team_zero_possession_after_count,
        )
    }
}
