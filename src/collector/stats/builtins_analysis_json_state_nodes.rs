use super::*;

pub(super) fn touch_state_json(graph: &AnalysisGraph, node_name: &str) -> SubtrActorResult<Value> {
    let state = graph_state::<TouchState>(graph, node_name)?;
    Ok(json!({
        "touch_events": state.touch_events,
        "last_touch": state.last_touch,
        "last_touch_player": state.last_touch_player,
        "last_touch_team_is_team_0": state.last_touch_team_is_team_0,
    }))
}

pub(super) fn possession_state_json(
    graph: &AnalysisGraph,
    node_name: &str,
) -> SubtrActorResult<Value> {
    let state = graph_state::<PossessionState>(graph, node_name)?;
    Ok(json!({
        "active_team_before_sample": state.active_team_before_sample,
        "current_team_is_team_0": state.current_team_is_team_0,
        "active_player_before_sample": state.active_player_before_sample,
        "current_player": state.current_player,
    }))
}

pub(super) fn backboard_bounce_state_json(
    graph: &AnalysisGraph,
    node_name: &str,
) -> SubtrActorResult<Value> {
    let state = graph_state::<BackboardBounceState>(graph, node_name)?;
    Ok(json!({
        "bounce_events": state.bounce_events,
        "last_bounce_event": state.last_bounce_event,
    }))
}
