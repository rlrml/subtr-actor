use super::*;

pub(crate) fn frame_input_from_live_state(
    live_events: &mut SaLiveEventGenerator,
    live_event_history: &mut SaLiveEventHistory,
    replay_meta: Option<&ReplayMeta>,
    frame: &SaLiveFrame,
    sampled_players: &[SaPlayerFrame],
    explicit_events: &SaFrameEventSlices<'_>,
) -> FrameInput {
    let frame_info = frame_info(frame);
    let ball = ball_state(frame);
    let players = player_state(sampled_players);
    let gameplay = gameplay_state(frame, sampled_players);
    let explicit_live_play = explicit_live_play_state(frame);
    let (frame_events, live_play) = live_events.frame_events(
        &frame_info,
        &ball,
        &players,
        &gameplay,
        explicit_live_play,
        explicit_events,
    );
    live_event_history.append_frame_events(&frame_events);
    let processor = SaLiveProcessorView::new(
        replay_meta,
        frame,
        sampled_players,
        frame_events,
        live_event_history,
    );
    FrameInput::timeline_with_live_play_state(
        &processor,
        frame.frame_number as usize,
        frame.time,
        frame.dt,
        live_play,
    )
}

#[cfg(test)]
pub(crate) fn frame_input(
    engine: &mut SaEngine,
    frame: &SaLiveFrame,
    sampled_players: &[SaPlayerFrame],
    explicit_events: &SaFrameEventSlices<'_>,
) -> FrameInput {
    frame_input_from_live_state(
        &mut engine.live_events,
        &mut engine.live_event_history,
        engine.live_replay_meta.as_ref(),
        frame,
        sampled_players,
        explicit_events,
    )
}
