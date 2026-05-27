use super::*;

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
