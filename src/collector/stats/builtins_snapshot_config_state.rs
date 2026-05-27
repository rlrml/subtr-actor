use super::*;

pub(super) fn state_config_json(
    module_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Option<Value>> {
    let value = match module_name {
        "positioning" => {
            let calculator = graph_state::<PositioningCalculator>(graph, module_name)?;
            serde_json::json!({
                "most_back_forward_threshold_y": calculator.config().most_back_forward_threshold_y,
                "level_ball_depth_margin": calculator.config().level_ball_depth_margin,
            })
        }
        "pressure" => {
            let calculator = graph_state::<PressureCalculator>(graph, module_name)?;
            serde_json::json!({
                "pressure_neutral_zone_half_width_y": calculator.config().neutral_zone_half_width_y,
            })
        }
        "territorial_pressure" => territorial_pressure_config_json(module_name, graph)?,
        "rotation" => {
            let calculator = graph_state::<RotationCalculator>(graph, module_name)?;
            serde_json::json!({
                "role_depth_margin": calculator.config().role_depth_margin,
                "first_man_ambiguity_margin": calculator.config().first_man_ambiguity_margin,
                "first_man_debounce_seconds": calculator.config().first_man_debounce_seconds,
            })
        }
        "rush" => {
            let calculator = graph_state::<RushCalculator>(graph, module_name)?;
            serde_json::json!({
                "rush_max_start_y": calculator.config().max_start_y,
                "rush_attack_support_distance_y": calculator.config().attack_support_distance_y,
                "rush_defender_distance_y": calculator.config().defender_distance_y,
                "rush_min_possession_retained_seconds": calculator.config().min_possession_retained_seconds,
            })
        }
        _ => return Ok(None),
    };
    Ok(Some(serialize_to_json_value(&value)?))
}

fn territorial_pressure_config_json(
    module_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Value> {
    let calculator = graph_state::<TerritorialPressureCalculator>(graph, module_name)?;
    Ok(serde_json::json!({
        "territorial_pressure_neutral_zone_half_width_y": calculator.config().neutral_zone_half_width_y,
        "territorial_pressure_min_establish_seconds": calculator.config().min_establish_seconds,
        "territorial_pressure_min_establish_third_seconds": calculator.config().min_establish_third_seconds,
        "territorial_pressure_relief_grace_seconds": calculator.config().relief_grace_seconds,
        "territorial_pressure_confirmed_relief_grace_seconds": calculator.config().confirmed_relief_grace_seconds,
    }))
}
