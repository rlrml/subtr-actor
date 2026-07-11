use super::*;

pub(crate) struct SaFrameEventSlices<'a> {
    pub(crate) touches: &'a [SaTouchEvent],
    pub(crate) dodge_refreshes: &'a [SaDodgeRefreshedEvent],
    pub(crate) boost_pad_events: &'a [SaBoostPadEvent],
    pub(crate) goals: &'a [SaGoalEvent],
    pub(crate) player_stat_events: &'a [SaPlayerStatEvent],
    pub(crate) demolishes: &'a [SaDemolishEvent],
}

#[cfg(test)]
pub(crate) fn sa_live_processor_view<'a>(
    replay_meta: Option<&'a ReplayMeta>,
    frame: &SaLiveFrame,
    players: &[SaPlayerFrame],
    events: FrameEventsState,
    event_history: &'a SaLiveEventHistory,
) -> LiveProcessorView<'a> {
    let live_frame = LiveFrame {
        players: live_player_frames(players),
        ..live_frame_data(frame)
    };
    LiveProcessorView::new(replay_meta, live_frame, events, event_history)
}

pub(crate) unsafe fn checked_slice<'a, T>(items: *const T, count: usize) -> Result<&'a [T], ()> {
    // SAFETY: Forwarding the caller's slice validity guarantee.
    unsafe { raw_slice(items, count) }
}

pub(crate) unsafe fn frame_event_slices(frame: &SaLiveFrame) -> Result<SaFrameEventSlices<'_>, ()> {
    Ok(SaFrameEventSlices {
        touches: unsafe { checked_slice(frame.touches, frame.touch_count) }?,
        dodge_refreshes: unsafe {
            checked_slice(frame.dodge_refreshes, frame.dodge_refresh_count)
        }?,
        boost_pad_events: unsafe {
            checked_slice(frame.boost_pad_events, frame.boost_pad_event_count)
        }?,
        goals: unsafe { checked_slice(frame.goals, frame.goal_count) }?,
        player_stat_events: unsafe {
            checked_slice(frame.player_stat_events, frame.player_stat_event_count)
        }?,
        demolishes: unsafe { checked_slice(frame.demolishes, frame.demolish_count) }?,
    })
}
