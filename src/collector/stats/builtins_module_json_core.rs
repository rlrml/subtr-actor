use super::*;

#[derive(Serialize)]
struct CoreStatsExport<'a> {
    team_zero: CoreTeamStats,
    team_one: CoreTeamStats,
    player_stats: Vec<PlayerStatsEntry<'a, CorePlayerStats>>,
    timeline: &'a [TimelineEvent],
    goal_context: &'a [GoalContextEvent],
    player_events: &'a [CorePlayerStatsEvent],
    team_events: &'a [CoreTeamStatsEvent],
}

#[derive(Serialize)]
struct DemoStatsExport<'a> {
    team_zero: &'a DemoTeamStats,
    team_one: &'a DemoTeamStats,
    player_stats: Vec<PlayerStatsEntry<'a, DemoPlayerStats>>,
    timeline: &'a [TimelineEvent],
}

pub(super) fn core_module_json(
    module_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Option<Value>> {
    match module_name {
        "core" => {
            let calculator = graph_state::<MatchStatsCalculator>(graph, module_name)?;
            serialize_to_json_value(&CoreStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                timeline: calculator.timeline(),
                goal_context: calculator.goal_context_events(),
                player_events: calculator.core_player_events(),
                team_events: calculator.core_team_events(),
            })
            .map(Some)
        }
        "demo" => {
            let calculator = graph_state::<DemoCalculator>(graph, module_name)?;
            serialize_to_json_value(&DemoStatsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                timeline: calculator.timeline(),
            })
            .map(Some)
        }
        _ => Ok(None),
    }
}
