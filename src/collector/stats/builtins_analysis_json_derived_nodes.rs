use super::*;

pub(super) fn derived_analysis_node_json(
    node_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Option<Value>> {
    match node_name {
        "continuous_ball_control" => continuous_ball_control_json(graph, node_name).map(Some),
        "fifty_fifty_state" => fifty_fifty_state_json(graph, node_name).map(Some),
        "player_vertical_state" => Ok(Some(player_vertical_state_json(graph_state::<
            PlayerVerticalState,
        >(
            graph, node_name
        )?))),
        "settings" => Ok(Some(settings_json(graph_state::<SettingsCalculator>(
            graph, node_name,
        )?))),
        _ => Ok(None),
    }
}

fn continuous_ball_control_json(graph: &AnalysisGraph, node_name: &str) -> SubtrActorResult<Value> {
    let state = graph_state::<ContinuousBallControlState>(graph, node_name)?;
    Ok(json!({
        "completed_sequences": state.completed_sequences.iter().map(|sequence| {
            json!({
                "player_id": sequence.player_id,
                "is_team_0": sequence.is_team_0,
                "kind": sequence.kind,
                "start_frame": sequence.start_frame,
                "end_frame": sequence.end_frame,
                "start_time": sequence.start_time,
                "end_time": sequence.end_time,
                "duration": sequence.duration,
                "straight_line_distance": sequence.straight_line_distance,
                "path_distance": sequence.path_distance,
                "average_horizontal_gap": sequence.average_horizontal_gap,
                "average_vertical_gap": sequence.average_vertical_gap,
                "average_speed": sequence.average_speed,
                "start_position": {
                    "x": sequence.start_position.x,
                    "y": sequence.start_position.y,
                    "z": sequence.start_position.z,
                },
                "end_position": {
                    "x": sequence.end_position.x,
                    "y": sequence.end_position.y,
                    "z": sequence.end_position.z,
                },
                "touch_count": sequence.touch_count,
                "air_touch_count": sequence.air_touch_count,
            })
        }).collect::<Vec<_>>(),
    }))
}

fn fifty_fifty_state_json(graph: &AnalysisGraph, node_name: &str) -> SubtrActorResult<Value> {
    let state = graph_state::<FiftyFiftyState>(graph, node_name)?;
    Ok(json!({
        "active_event": state.active_event.as_ref().map(|event| {
            json!({
                "start_time": event.start_time,
                "start_frame": event.start_frame,
                "last_touch_time": event.last_touch_time,
                "last_touch_frame": event.last_touch_frame,
                "is_kickoff": event.is_kickoff,
                "team_zero_player": event.team_zero_player,
                "team_one_player": event.team_one_player,
                "team_zero_position": event.team_zero_position,
                "team_one_position": event.team_one_position,
                "midpoint": event.midpoint,
                "plane_normal": event.plane_normal,
            })
        }),
        "resolved_events": state.resolved_events,
        "last_resolved_event": state.last_resolved_event,
    }))
}
