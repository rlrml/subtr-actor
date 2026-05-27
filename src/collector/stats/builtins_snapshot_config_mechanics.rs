use super::*;

pub(super) fn mechanic_config_json(
    module_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Option<Value>> {
    let value = match module_name {
        "half_volley" => {
            let calculator = graph_state::<HalfVolleyCalculator>(graph, module_name)?;
            serde_json::json!({
                "half_volley_max_bounce_to_touch_seconds": calculator.config().max_bounce_to_touch_seconds,
                "half_volley_min_ball_speed": calculator.config().min_ball_speed,
            })
        }
        _ => return Ok(None),
    };
    Ok(Some(serialize_to_json_value(&value)?))
}
