use super::*;

pub trait ProcessorFrameEventView {
    fn current_frame_active_demo_events(&self) -> &[DemoEventSample] {
        &[]
    }

    fn current_frame_demolish_events(&self) -> &[DemolishInfo] {
        &[]
    }

    fn current_frame_boost_pad_events(&self) -> &[BoostPadEvent];
    fn current_frame_touch_events(&self) -> &[TouchEvent];
    fn current_frame_dodge_refreshed_events(&self) -> &[DodgeRefreshedEvent];
    fn current_frame_player_stat_events(&self) -> &[PlayerStatEvent];
    fn current_frame_goal_events(&self) -> &[GoalEvent];
}
