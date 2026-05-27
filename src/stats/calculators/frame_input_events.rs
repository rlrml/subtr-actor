use super::*;

impl FrameInput {
    pub(super) fn build_active_demo_events(processor: &dyn ProcessorView) -> Vec<DemoEventSample> {
        let active_demo_events = processor.current_frame_active_demo_events();
        if !active_demo_events.is_empty() {
            active_demo_events.to_vec()
        } else if let Ok(demos) = processor.get_active_demos() {
            demos
                .into_iter()
                .filter_map(|demo| {
                    let attacker = processor
                        .get_player_id_from_car_id(&demo.attacker_actor_id())
                        .ok()?;
                    let victim = processor
                        .get_player_id_from_car_id(&demo.victim_actor_id())
                        .ok()?;
                    Some(DemoEventSample { attacker, victim })
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub(super) fn build_current_frame_events_state(
        processor: &dyn ProcessorView,
    ) -> FrameEventsState {
        FrameEventsState {
            active_demos: Self::build_active_demo_events(processor),
            demo_events: processor.current_frame_demolish_events().to_vec(),
            boost_pad_events: processor.current_frame_boost_pad_events().to_vec(),
            touch_events: processor.current_frame_touch_events().to_vec(),
            dodge_refreshed_events: processor.current_frame_dodge_refreshed_events().to_vec(),
            player_stat_events: processor.current_frame_player_stat_events().to_vec(),
            goal_events: processor.current_frame_goal_events().to_vec(),
        }
    }

    pub(super) fn build_events_since_last_sample(
        processor: &dyn ProcessorView,
        last_demolish_count: usize,
        last_boost_pad_event_count: usize,
        last_touch_event_count: usize,
        last_dodge_refreshed_event_count: usize,
        last_player_stat_event_count: usize,
        last_goal_event_count: usize,
    ) -> FrameEventsState {
        FrameEventsState {
            active_demos: Self::build_active_demo_events(processor),
            demo_events: processor.demolishes()[last_demolish_count..].to_vec(),
            boost_pad_events: processor.boost_pad_events()[last_boost_pad_event_count..].to_vec(),
            touch_events: processor.touch_events()[last_touch_event_count..].to_vec(),
            dodge_refreshed_events: processor.dodge_refreshed_events()
                [last_dodge_refreshed_event_count..]
                .to_vec(),
            player_stat_events: processor.player_stat_events()[last_player_stat_event_count..]
                .to_vec(),
            goal_events: processor.goal_events()[last_goal_event_count..].to_vec(),
        }
    }
}
