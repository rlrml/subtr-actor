use super::*;

pub(super) fn game_state_module_json(
    module_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Option<Value>> {
    macro_rules! stats_events_json {
        ($calculator:ty) => {{
            let calculator = graph_state::<$calculator>(graph, module_name)?;
            serialize_to_json_value(&StatsWithEventsExport {
                stats: calculator.stats(),
                events: calculator.events(),
            })
            .map(Some)
        }};
    }

    match module_name {
        "fifty_fifty" => {
            let calculator = graph_state::<FiftyFiftyCalculator>(graph, module_name)?;
            serialize_to_json_value(&StatsWithPlayerEventsExport {
                stats: calculator.stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
            .map(Some)
        }
        "possession" => stats_events_json!(PossessionCalculator),
        "pressure" => stats_events_json!(PressureCalculator),
        "territorial_pressure" => stats_events_json!(TerritorialPressureCalculator),
        "rotation" => {
            let calculator = graph_state::<RotationCalculator>(graph, module_name)?;
            serialize_to_json_value(&serde_json::json!({
                "team_zero": calculator.team_zero_stats(),
                "team_one": calculator.team_one_stats(),
                "player_stats": player_stats_entries(calculator.player_stats()),
                "player_events": calculator.player_events(),
                "team_events": calculator.team_events(),
            }))
            .map(Some)
        }
        "rush" => stats_events_json!(RushCalculator),
        "touch" => {
            let calculator = graph_state::<TouchCalculator>(graph, module_name)?;
            serialize_to_json_value(&serde_json::json!({
                "player_stats": player_stats_entries(calculator.player_stats()),
                "events": calculator.events(),
                "ball_movement_events": calculator.ball_movement_events(),
                "last_touch_events": calculator.last_touch_events(),
            }))
            .map(Some)
        }
        "boost" => {
            let calculator = graph_state::<BoostCalculator>(graph, module_name)?;
            serialize_to_json_value(&serde_json::json!({
                "team_zero": calculator.team_zero_stats(),
                "team_one": calculator.team_one_stats(),
                "player_stats": player_stats_entries(calculator.player_stats()),
                "events": calculator.pickup_comparison_events(),
                "ledger_events": calculator.ledger_events(),
                "state_events": calculator.state_events(),
            }))
            .map(Some)
        }
        _ => Ok(None),
    }
}
