macro_rules! sa_live_processor_frame_event_methods {
    () => {
        fn current_frame_active_demo_events(&self) -> &[DemoEventSample] {
            &self.events.active_demos
        }

        fn current_frame_demolish_events(&self) -> &[DemolishInfo] {
            &self.events.demo_events
        }

        fn current_frame_boost_pad_events(&self) -> &[BoostPadEvent] {
            &self.events.boost_pad_events
        }

        fn current_frame_touch_events(&self) -> &[TouchEvent] {
            &self.events.touch_events
        }

        fn current_frame_dodge_refreshed_events(&self) -> &[DodgeRefreshedEvent] {
            &self.events.dodge_refreshed_events
        }

        fn current_frame_player_stat_events(&self) -> &[PlayerStatEvent] {
            &self.events.player_stat_events
        }

        fn current_frame_goal_events(&self) -> &[GoalEvent] {
            &self.events.goal_events
        }
    };
}
