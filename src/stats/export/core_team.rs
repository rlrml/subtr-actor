use super::*;

#[path = "core_team_buildup.rs"]
mod core_team_buildup;
#[path = "core_team_timing.rs"]
mod core_team_timing;

use core_team_buildup::visit_team_goal_buildup;
use core_team_timing::visit_team_goal_timing;

impl StatFieldProvider for CoreTeamStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visit_team_match_totals(self, visitor);
        visit_team_goal_timing(self, visitor);
        visit_team_goal_buildup(self, visitor);
    }
}

fn visit_team_match_totals(stats: &CoreTeamStats, visitor: &mut dyn FnMut(ExportedStat)) {
    for (name, value) in [
        ("score", stats.score),
        ("goals", stats.goals),
        ("assists", stats.assists),
        ("saves", stats.saves),
        ("shots", stats.shots),
    ] {
        visitor(ExportedStat::signed("core", name, StatUnit::Count, value));
    }
}
