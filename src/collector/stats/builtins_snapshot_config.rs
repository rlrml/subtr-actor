use super::*;

#[path = "builtins_snapshot_config_goals.rs"]
mod builtins_snapshot_config_goals;
#[path = "builtins_snapshot_config_mechanics.rs"]
mod builtins_snapshot_config_mechanics;
#[path = "builtins_snapshot_config_none.rs"]
mod builtins_snapshot_config_none;
#[path = "builtins_snapshot_config_state.rs"]
mod builtins_snapshot_config_state;

pub fn builtin_stats_module_config_json(
    module_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Value> {
    Ok(builtin_snapshot_config_json(module_name, graph)?.unwrap_or(Value::Null))
}

pub(crate) fn builtin_snapshot_config_json(
    module_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Option<Value>> {
    if builtins_snapshot_config_none::module_has_no_snapshot_config(module_name) {
        return Ok(None);
    }
    if let Some(value) = builtins_snapshot_config_state::state_config_json(module_name, graph)? {
        return Ok(Some(value));
    }
    if let Some(value) = builtins_snapshot_config_goals::goal_config_json(module_name, graph)? {
        return Ok(Some(value));
    }
    if let Some(value) =
        builtins_snapshot_config_mechanics::mechanic_config_json(module_name, graph)?
    {
        return Ok(Some(value));
    }
    SubtrActorError::new_result(SubtrActorErrorVariant::UnknownStatsModuleName(
        module_name.to_owned(),
    ))
}
