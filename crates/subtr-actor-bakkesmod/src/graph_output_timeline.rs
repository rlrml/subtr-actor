use super::*;

pub(crate) fn current_timeline_frame(graph: &AnalysisGraph) -> Option<ReplayStatsFrame> {
    graph
        .state::<StatsTimelineFrameState>()
        .and_then(|state| state.frame.clone())
}

pub(crate) fn record_timeline_frame(frames: &mut Vec<ReplayStatsFrame>, frame: ReplayStatsFrame) {
    if let Some(last_frame) = frames.last_mut() {
        if last_frame.frame_number == frame.frame_number {
            *last_frame = frame;
            return;
        }
    }
    frames.push(frame);
}

pub(crate) fn serialize_live_timeline(
    replay_meta: Option<&ReplayMeta>,
    events: ReplayStatsTimelineEvents,
    frames: Vec<ReplayStatsFrame>,
) -> Vec<u8> {
    let Some(replay_meta) = replay_meta else {
        return Vec::new();
    };
    serde_json::to_vec(&ReplayStatsTimeline {
        config: default_stats_timeline_config(),
        replay_meta: replay_meta.clone(),
        events,
        frames,
    })
    .unwrap_or_default()
}

pub(crate) fn current_timeline_events(graph: &AnalysisGraph) -> Option<ReplayStatsTimelineEvents> {
    graph
        .state::<StatsTimelineEventsState>()
        .map(|state| state.events.clone())
}

pub(crate) fn refresh_timeline_graph_state(engine: &mut SaEngine) {
    let Some(events) = engine
        .graph
        .state::<StatsTimelineEventsState>()
        .map(|state| state.events.clone())
    else {
        return;
    };
    push_drainable_events_from_timeline(
        &mut engine.pending_events,
        &mut engine.emitted_mechanic_ids,
        &mut engine.pending_team_events,
        &mut engine.emitted_team_event_ids,
        &mut engine.pending_goal_context_events,
        &mut engine.emitted_goal_context_ids,
        &events,
    );
    if let Some(frame) = current_timeline_frame(&engine.graph) {
        record_timeline_frame(&mut engine.timeline_frames, frame.clone());
    }
}
