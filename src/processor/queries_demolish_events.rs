use crate::{
    BoostPadEvent, DemolishAttribute, DemolishFormat, DodgeRefreshedEvent, GoalEvent,
    PlayerStatEvent, ReplayProcessor, TouchEvent, MAX_DEMOLISH_KNOWN_FRAMES_PASSED,
};

impl<'a> ReplayProcessor<'a> {
    /// Returns whether a demolish has already been recorded within the dedupe window.
    pub(crate) fn demolish_is_known(&self, demo: &DemolishAttribute, frame_index: usize) -> bool {
        self.known_demolishes
            .iter()
            .any(|(existing, existing_frame_index)| {
                existing == demo
                    && frame_index
                        .checked_sub(*existing_frame_index)
                        .or_else(|| existing_frame_index.checked_sub(frame_index))
                        .unwrap()
                        < MAX_DEMOLISH_KNOWN_FRAMES_PASSED
            })
    }

    /// Returns the demolish attribute encoding currently used by the replay, if known.
    pub fn get_demolish_format(&self) -> Option<DemolishFormat> {
        self.demolish_format
    }

    pub fn current_frame_boost_pad_events(&self) -> &[BoostPadEvent] {
        &self.current_frame_boost_pad_events
    }

    pub fn current_frame_touch_events(&self) -> &[TouchEvent] {
        &self.current_frame_touch_events
    }

    pub fn current_frame_dodge_refreshed_events(&self) -> &[DodgeRefreshedEvent] {
        &self.current_frame_dodge_refreshed_events
    }

    pub fn current_frame_goal_events(&self) -> &[GoalEvent] {
        &self.current_frame_goal_events
    }

    pub fn current_frame_player_stat_events(&self) -> &[PlayerStatEvent] {
        &self.current_frame_player_stat_events
    }
}
