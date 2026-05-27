use super::*;

pub(crate) fn explicit_dodge_refresh_keys(
    frame: &FrameInfo,
    events: &[SaDodgeRefreshedEvent],
) -> HashSet<(RemoteId, usize)> {
    events
        .iter()
        .map(|event| {
            let (frame_number, _) = event_frame_and_time(frame, event.timing);
            (player_id(event.player_index), frame_number)
        })
        .collect()
}

#[path = "live_event_dedupe.rs"]
mod live_event_dedupe;
pub(crate) use live_event_dedupe::*;
#[path = "live_player_stat_events.rs"]
mod live_player_stat_events;
pub(crate) use live_player_stat_events::*;
#[path = "live_demolish_events.rs"]
mod live_demolish_events;
pub(crate) use live_demolish_events::*;
#[path = "live_boost_pad_events.rs"]
mod live_boost_pad_events;
#[path = "live_demo_state.rs"]
mod live_demo_state;
#[path = "live_dodge_refresh_events.rs"]
mod live_dodge_refresh_events;
#[path = "live_frame_events.rs"]
mod live_frame_events;
#[path = "live_goal_events.rs"]
mod live_goal_events;
