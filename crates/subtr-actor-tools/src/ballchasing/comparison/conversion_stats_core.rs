use subtr_actor::{CorePlayerStats, CoreTeamStats};

use super::super::super::comparable_types::ComparableCoreStats;

pub(crate) fn comparable_core_from_player(stats: &CorePlayerStats) -> ComparableCoreStats {
    ComparableCoreStats {
        score: Some(stats.score as f64),
        goals: Some(stats.goals as f64),
        assists: Some(stats.assists as f64),
        saves: Some(stats.saves as f64),
        shots: Some(stats.shots as f64),
        shooting_percentage: Some(stats.shooting_percentage() as f64),
    }
}

pub(crate) fn comparable_core_from_team(stats: &CoreTeamStats) -> ComparableCoreStats {
    ComparableCoreStats {
        score: Some(stats.score as f64),
        goals: Some(stats.goals as f64),
        assists: Some(stats.assists as f64),
        saves: Some(stats.saves as f64),
        shots: Some(stats.shots as f64),
        shooting_percentage: Some(stats.shooting_percentage() as f64),
    }
}
