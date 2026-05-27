use super::*;

pub(super) fn team_player_module_json(
    module_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Option<Value>> {
    match module_name {
        "backboard" => {
            let calculator = graph_state::<BackboardCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
            .map(Some)
        }
        "center" => {
            let calculator = graph_state::<CenterCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
            .map(Some)
        }
        "double_tap" => {
            let calculator = graph_state::<DoubleTapCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
            .map(Some)
        }
        "one_timer" => {
            let calculator = graph_state::<OneTimerCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
            .map(Some)
        }
        "half_volley" => {
            let calculator = graph_state::<HalfVolleyCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
            .map(Some)
        }
        "pass" => {
            let calculator = graph_state::<PassCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
            .map(Some)
        }
        "bump" => {
            let calculator = graph_state::<BumpCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
            .map(Some)
        }
        "movement" => {
            let calculator = graph_state::<MovementCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
            .map(Some)
        }
        "powerslide" => {
            let calculator = graph_state::<PowerslideCalculator>(graph, module_name)?;
            serialize_to_json_value(&TeamPlayerStatsWithEventsExport {
                team_zero: calculator.team_zero_stats(),
                team_one: calculator.team_one_stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
            .map(Some)
        }
        _ => Ok(None),
    }
}
