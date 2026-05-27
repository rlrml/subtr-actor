use super::*;

pub(crate) struct SaFrameEventSlices<'a> {
    pub(crate) touches: &'a [SaTouchEvent],
    pub(crate) dodge_refreshes: &'a [SaDodgeRefreshedEvent],
    pub(crate) boost_pad_events: &'a [SaBoostPadEvent],
    pub(crate) goals: &'a [SaGoalEvent],
    pub(crate) player_stat_events: &'a [SaPlayerStatEvent],
    pub(crate) demolishes: &'a [SaDemolishEvent],
}

pub(crate) unsafe fn checked_slice<'a, T>(items: *const T, count: usize) -> Result<&'a [T], ()> {
    if items.is_null() && count != 0 {
        return Err(());
    }
    if count == 0 {
        Ok(&[])
    } else {
        Ok(slice::from_raw_parts(items, count))
    }
}

pub(crate) unsafe fn frame_event_slices(frame: &SaLiveFrame) -> Result<SaFrameEventSlices<'_>, ()> {
    Ok(SaFrameEventSlices {
        touches: checked_slice(frame.touches, frame.touch_count)?,
        dodge_refreshes: checked_slice(frame.dodge_refreshes, frame.dodge_refresh_count)?,
        boost_pad_events: checked_slice(frame.boost_pad_events, frame.boost_pad_event_count)?,
        goals: checked_slice(frame.goals, frame.goal_count)?,
        player_stat_events: checked_slice(frame.player_stat_events, frame.player_stat_event_count)?,
        demolishes: checked_slice(frame.demolishes, frame.demolish_count)?,
    })
}
