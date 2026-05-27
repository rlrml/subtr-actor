use super::*;

impl ProcessorFrameEventView for ReplayProcessor<'_> {
    fn current_frame_boost_pad_events(&self) -> &[BoostPadEvent] {
        ReplayProcessor::current_frame_boost_pad_events(self)
    }

    fn current_frame_touch_events(&self) -> &[TouchEvent] {
        ReplayProcessor::current_frame_touch_events(self)
    }

    fn current_frame_dodge_refreshed_events(&self) -> &[DodgeRefreshedEvent] {
        ReplayProcessor::current_frame_dodge_refreshed_events(self)
    }

    fn current_frame_player_stat_events(&self) -> &[PlayerStatEvent] {
        ReplayProcessor::current_frame_player_stat_events(self)
    }

    fn current_frame_goal_events(&self) -> &[GoalEvent] {
        ReplayProcessor::current_frame_goal_events(self)
    }
}
