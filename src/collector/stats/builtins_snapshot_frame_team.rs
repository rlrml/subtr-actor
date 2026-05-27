use super::builtins_snapshot_frame_values::team_player_stats_snapshot_value;
use super::*;

pub(super) fn team_snapshot_frame_json(
    module_name: &str,
    graph: &AnalysisGraph,
    _replay_meta: &ReplayMeta,
) -> SubtrActorResult<Option<Value>> {
    macro_rules! team_module {
        ($calculator:ty) => {{
            let calculator = graph_state::<$calculator>(graph, module_name)?;
            team_player_stats_snapshot_value(
                calculator.team_zero_stats(),
                calculator.team_one_stats(),
                calculator.player_stats(),
            )
            .map(Some)
        }};
    }

    match module_name {
        "backboard" => team_module!(BackboardCalculator),
        "center" => team_module!(CenterCalculator),
        "double_tap" => team_module!(DoubleTapCalculator),
        "one_timer" => team_module!(OneTimerCalculator),
        "half_volley" => team_module!(HalfVolleyCalculator),
        "pass" => team_module!(PassCalculator),
        "rotation" => team_module!(RotationCalculator),
        "ball_carry" => team_module!(BallCarryCalculator),
        "boost" => team_module!(BoostCalculator),
        "bump" => team_module!(BumpCalculator),
        "powerslide" => team_module!(PowerslideCalculator),
        "demo" => team_module!(DemoCalculator),
        "air_dribble" => air_dribble_snapshot_frame_json(module_name, graph),
        _ => Ok(None),
    }
}

fn air_dribble_snapshot_frame_json(
    module_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Option<Value>> {
    let calculator = graph_state::<BallCarryCalculator>(graph, module_name)?;
    serialize_to_json_value(&TeamPlayerStatsExport {
        team_zero: calculator.team_zero_air_dribble_stats(),
        team_one: calculator.team_one_air_dribble_stats(),
        player_stats: player_stats_entries(calculator.player_air_dribble_stats()),
    })
    .map(Some)
}
