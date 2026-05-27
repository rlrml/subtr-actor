use super::*;

#[path = "core_player_buildup.rs"]
mod core_player_buildup;
#[path = "core_player_positions.rs"]
mod core_player_positions;
#[path = "core_player_timing.rs"]
mod core_player_timing;

use core_player_buildup::visit_goal_buildup;
use core_player_positions::{visit_goal_against_position, visit_scoring_touch_position};
use core_player_timing::visit_goal_timing;

impl StatFieldProvider for CorePlayerStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visit_match_totals(self, visitor);
        visit_defensive_goal_context(self, visitor);
        visit_scoring_goal_context(self, visitor);
        visit_goal_timing(self, visitor);
        visit_goal_buildup(self, visitor);
    }
}

fn visit_match_totals(stats: &CorePlayerStats, visitor: &mut dyn FnMut(ExportedStat)) {
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

fn visit_defensive_goal_context(stats: &CorePlayerStats, visitor: &mut dyn FnMut(ExportedStat)) {
    let context = &stats.scoring_context;
    visitor(ExportedStat::unsigned(
        "core",
        "goals_conceded_while_last_defender",
        StatUnit::Count,
        context.goals_conceded_while_last_defender,
    ));
    visitor(ExportedStat::unsigned(
        "core",
        "goals_for_while_most_back",
        StatUnit::Count,
        context.goals_for_while_most_back,
    ));
    visitor(ExportedStat::unsigned(
        "core",
        "goals_against_while_most_back",
        StatUnit::Count,
        context.goals_against_while_most_back,
    ));
}

fn visit_scoring_goal_context(stats: &CorePlayerStats, visitor: &mut dyn FnMut(ExportedStat)) {
    visit_goal_against_boost(stats, visitor);
    visit_goal_against_position(stats, visitor);
    visit_scoring_touch_position(stats, visitor);
}

fn visit_goal_against_boost(stats: &CorePlayerStats, visitor: &mut dyn FnMut(ExportedStat)) {
    let context = &stats.scoring_context;
    visitor(ExportedStat::unsigned(
        "core",
        "goal_against_boost_sample_count",
        StatUnit::Count,
        context.goal_against_boost_sample_count,
    ));
    visitor(ExportedStat::float(
        "core",
        "average_boost_on_goals_against",
        StatUnit::Boost,
        stats.average_boost_on_goals_against(),
    ));
    visitor(ExportedStat::unsigned(
        "core",
        "goal_against_boost_leadup_sample_count",
        StatUnit::Count,
        context.goal_against_boost_leadup_sample_count,
    ));
    visitor(ExportedStat::float(
        "core",
        "average_boost_in_goal_against_leadup",
        StatUnit::Boost,
        stats.average_boost_in_goal_against_leadup(),
    ));
    visitor(ExportedStat::float(
        "core",
        "average_min_boost_in_goal_against_leadup",
        StatUnit::Boost,
        stats.average_min_boost_in_goal_against_leadup(),
    ));
}
