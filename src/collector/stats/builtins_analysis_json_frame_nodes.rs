use super::*;

pub(super) fn frame_analysis_node_json(
    node_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Option<Value>> {
    match node_name {
        "stats_timeline_events" => serialize_to_json_value(
            &graph_state::<StatsTimelineEventsState>(graph, node_name)?.events,
        )
        .map(Some),
        "stats_timeline_frame" => graph_state::<StatsTimelineFrameState>(graph, node_name)?
            .frame
            .as_ref()
            .map(serialize_to_json_value)
            .transpose()
            .map(|value| Some(value.unwrap_or(Value::Null))),
        "frame_info" => {
            let state = graph_state::<FrameInfo>(graph, node_name)?;
            Ok(Some(json!({
                "frame_number": state.frame_number,
                "time": state.time,
                "dt": state.dt,
                "seconds_remaining": state.seconds_remaining,
            })))
        }
        "gameplay_state" => gameplay_state_json(graph, node_name).map(Some),
        "ball_frame_state" => Ok(Some(ball_frame_state_json(graph_state::<BallFrameState>(
            graph, node_name,
        )?))),
        "player_frame_state" => {
            let state = graph_state::<PlayerFrameState>(graph, node_name)?;
            Ok(Some(json!({
                "players": state.players.iter().map(player_sample_json).collect::<Vec<_>>(),
            })))
        }
        "frame_events_state" => frame_events_state_json(graph, node_name).map(Some),
        "live_play" => {
            serialize_to_json_value(graph_state::<LivePlayState>(graph, node_name)?).map(Some)
        }
        "touch_state" => touch_state_json(graph, node_name).map(Some),
        "possession_state" => possession_state_json(graph, node_name).map(Some),
        "backboard_bounce_state" => backboard_bounce_state_json(graph, node_name).map(Some),
        _ => Ok(None),
    }
}

fn gameplay_state_json(graph: &AnalysisGraph, node_name: &str) -> SubtrActorResult<Value> {
    let state = graph_state::<GameplayState>(graph, node_name)?;
    Ok(json!({
        "game_state": state.game_state,
        "ball_has_been_hit": state.ball_has_been_hit,
        "kickoff_countdown_time": state.kickoff_countdown_time,
        "team_zero_score": state.team_zero_score,
        "team_one_score": state.team_one_score,
        "possession_team_is_team_0": state.possession_team_is_team_0,
        "scored_on_team_is_team_0": state.scored_on_team_is_team_0,
        "current_in_game_team_player_counts": state.current_in_game_team_player_counts,
        "is_live_play": state.is_live_play(),
        "kickoff_phase_active": state.kickoff_phase_active(),
    }))
}

fn frame_events_state_json(graph: &AnalysisGraph, node_name: &str) -> SubtrActorResult<Value> {
    let state = graph_state::<FrameEventsState>(graph, node_name)?;
    Ok(json!({
        "active_demos": state.active_demos.iter().map(demo_event_sample_json).collect::<Vec<_>>(),
        "demo_events": state.demo_events,
        "boost_pad_events": state.boost_pad_events,
        "touch_events": state.touch_events,
        "dodge_refreshed_events": state.dodge_refreshed_events,
        "player_stat_events": state.player_stat_events,
        "goal_events": state.goal_events,
    }))
}
