use super::*;

pub(super) fn goal_config_json(
    module_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Option<Value>> {
    let value = match module_name {
        "aerial_goal" => {
            let calculator = graph_state::<AerialGoalCalculator>(graph, module_name)?;
            serde_json::json!({
                "aerial_goal_min_ball_z": calculator.config().min_ball_z,
            })
        }
        "high_aerial_goal" => {
            let calculator = graph_state::<HighAerialGoalCalculator>(graph, module_name)?;
            serde_json::json!({
                "high_aerial_goal_min_ball_z": calculator.config().min_ball_z,
            })
        }
        "long_distance_goal" => {
            let calculator = graph_state::<LongDistanceGoalCalculator>(graph, module_name)?;
            serde_json::json!({
                "long_distance_goal_max_attacking_y": calculator.config().max_attacking_y,
            })
        }
        "own_half_goal" => {
            let calculator = graph_state::<OwnHalfGoalCalculator>(graph, module_name)?;
            serde_json::json!({
                "own_half_goal_max_attacking_y": calculator.config().max_attacking_y,
            })
        }
        "empty_net_goal" => empty_net_config_json(graph, module_name)?,
        "flick_goal" => {
            let calculator = graph_state::<FlickGoalCalculator>(graph, module_name)?;
            serde_json::json!({
                "flick_goal_max_event_to_goal_seconds": calculator.config().max_event_to_goal_seconds,
            })
        }
        "double_tap_goal" => {
            let calculator = graph_state::<DoubleTapGoalCalculator>(graph, module_name)?;
            serde_json::json!({
                "double_tap_goal_max_event_to_goal_seconds": calculator.config().max_event_to_goal_seconds,
            })
        }
        "one_timer_goal" => {
            let calculator = graph_state::<OneTimerGoalCalculator>(graph, module_name)?;
            serde_json::json!({
                "one_timer_goal_max_event_to_goal_seconds": calculator.config().max_event_to_goal_seconds,
            })
        }
        "passing_goal" => {
            let calculator = graph_state::<PassingGoalCalculator>(graph, module_name)?;
            serde_json::json!({
                "passing_goal_max_pass_to_goal_seconds": calculator.config().max_pass_to_goal_seconds,
            })
        }
        "air_dribble_goal" => {
            let calculator = graph_state::<AirDribbleGoalCalculator>(graph, module_name)?;
            serde_json::json!({
                "air_dribble_goal_max_end_to_goal_seconds": calculator.config().max_end_to_goal_seconds,
            })
        }
        "flip_reset_goal" => {
            let calculator = graph_state::<FlipResetGoalCalculator>(graph, module_name)?;
            serde_json::json!({
                "flip_reset_goal_max_event_to_goal_seconds": calculator.config().max_event_to_goal_seconds,
            })
        }
        "half_volley_goal" => {
            let calculator = graph_state::<HalfVolleyGoalCalculator>(graph, module_name)?;
            serde_json::json!({
                "half_volley_goal_max_touch_to_goal_seconds": calculator.config().max_touch_to_goal_seconds,
                "half_volley_goal_min_goal_alignment": calculator.config().min_goal_alignment,
            })
        }
        _ => return Ok(None),
    };
    Ok(Some(serialize_to_json_value(&value)?))
}

fn empty_net_config_json(graph: &AnalysisGraph, module_name: &str) -> SubtrActorResult<Value> {
    let calculator = graph_state::<EmptyNetGoalCalculator>(graph, module_name)?;
    Ok(serde_json::json!({
        "empty_net_min_defender_y_margin": calculator.config().min_defender_y_margin,
        "empty_net_min_defender_distance": calculator.config().min_defender_distance,
        "empty_net_max_touch_attacking_y": calculator.config().max_touch_attacking_y,
    }))
}
