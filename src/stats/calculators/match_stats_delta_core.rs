use super::match_stats_delta_scoring::{player_scoring_context_delta, team_scoring_context_delta};
use super::*;

pub(super) fn core_player_stats_delta(
    current: &CorePlayerStats,
    previous: &CorePlayerStats,
) -> CorePlayerStats {
    CorePlayerStats {
        score: current.score - previous.score,
        goals: current.goals - previous.goals,
        assists: current.assists - previous.assists,
        saves: current.saves - previous.saves,
        shots: current.shots - previous.shots,
        scoring_context: player_scoring_context_delta(
            &current.scoring_context,
            &previous.scoring_context,
        ),
    }
}

pub(super) fn core_team_stats_delta(
    current: &CoreTeamStats,
    previous: &CoreTeamStats,
) -> CoreTeamStats {
    CoreTeamStats {
        score: current.score - previous.score,
        goals: current.goals - previous.goals,
        assists: current.assists - previous.assists,
        saves: current.saves - previous.saves,
        shots: current.shots - previous.shots,
        scoring_context: team_scoring_context_delta(
            &current.scoring_context,
            &previous.scoring_context,
        ),
    }
}
