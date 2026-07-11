use super::*;

pub(crate) use subtr_actor_live::{
    LiveEventGenerator as SaLiveEventGenerator, LiveEventHistory as SaLiveEventHistory, player_id,
    player_index,
};

#[cfg(test)]
pub(crate) fn frame_info(frame: &SaLiveFrame) -> FrameInfo {
    subtr_actor_live::frame_info(&live_frame_data(frame))
}

#[cfg(test)]
pub(crate) fn player_state(players: &[SaPlayerFrame]) -> PlayerFrameState {
    subtr_actor_live::player_state(&live_player_frames(players))
}

#[cfg(test)]
pub(crate) fn explicit_demolish_events(
    frame: &FrameInfo,
    players: &PlayerFrameState,
    events: &[SaDemolishEvent],
) -> Vec<DemolishInfo> {
    let events: Vec<_> = events.iter().map(live_demolish_event).collect();
    subtr_actor_live::explicit_demolish_events(frame, players, &events)
}

pub(crate) fn frame_input_from_live_state(
    live_events: &mut SaLiveEventGenerator,
    live_event_history: &mut SaLiveEventHistory,
    replay_meta: Option<&ReplayMeta>,
    frame: &SaLiveFrame,
    sampled_players: &[SaPlayerFrame],
    explicit_events: &SaFrameEventSlices<'_>,
) -> FrameInput {
    let live_frame = live_frame_from_abi(frame, sampled_players, explicit_events);
    subtr_actor_live::frame_input_from_live_frame(
        live_events,
        live_event_history,
        replay_meta,
        live_frame,
    )
    .0
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

pub(crate) fn live_replay_meta_signature(
    players: &[SaPlayerFrame],
) -> Vec<(RemoteId, bool, Option<String>)> {
    LiveMatchMeta::from_player_frames(&live_player_frames(players)).signature()
}

pub(crate) fn live_replay_meta(players: &[SaPlayerFrame]) -> ReplayMeta {
    LiveMatchMeta::from_player_frames(&live_player_frames(players)).replay_meta()
}

pub(crate) fn sync_live_replay_meta(
    engine: &mut SaEngine,
    players: &[SaPlayerFrame],
) -> subtr_actor::SubtrActorResult<()> {
    let signature = live_replay_meta_signature(players);
    if engine.live_replay_meta_initialized && engine.live_replay_meta_signature == signature {
        return Ok(());
    }

    let replay_meta = live_replay_meta(players);
    engine.graph.on_replay_meta(&replay_meta)?;
    engine.live_replay_meta_initialized = true;
    engine.live_replay_meta = Some(replay_meta);
    engine.live_replay_meta_signature = signature;
    Ok(())
}

pub(crate) fn has_duplicate_player_indices(players: &[SaPlayerFrame]) -> bool {
    let mut seen = HashSet::new();
    players
        .iter()
        .any(|player| !seen.insert(player.player_index))
}
