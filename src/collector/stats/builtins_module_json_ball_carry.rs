use super::*;

pub(super) fn ball_carry_module_json(
    module_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Option<Value>> {
    match module_name {
        "ball_carry" => {
            let calculator = graph_state::<BallCarryCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.carry_events(),
            })
            .map(Some)
        }
        "air_dribble" => {
            let calculator = graph_state::<BallCarryCalculator>(graph, module_name)?;
            let events = calculator
                .carry_events()
                .iter()
                .filter(|event| event.kind == BallCarryKind::AirDribble)
                .collect::<Vec<_>>();
            serialize_to_json_value(&TeamPlayerStatsWithCollectedEventsExport {
                team_zero: calculator.team_zero_air_dribble_stats(),
                team_one: calculator.team_one_air_dribble_stats(),
                player_stats: player_stats_entries(calculator.player_air_dribble_stats()),
                events,
            })
            .map(Some)
        }
        _ => Ok(None),
    }
}
