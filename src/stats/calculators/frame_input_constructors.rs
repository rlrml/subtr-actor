use super::*;

impl FrameInput {
    pub fn timeline(
        processor: &dyn ProcessorView,
        frame_number: usize,
        current_time: f32,
        dt: f32,
    ) -> Self {
        Self {
            frame_info: Self::build_frame_info(processor, frame_number, current_time, dt),
            gameplay_state: Self::build_gameplay_state(processor),
            ball_frame_state: Self::build_ball_frame_state(processor, current_time),
            player_frame_state: Self::build_player_frame_state(processor, current_time),
            frame_events_state: Self::build_current_frame_events_state(processor),
            live_play_state: None,
        }
    }

    pub fn timeline_with_live_play_state(
        processor: &dyn ProcessorView,
        frame_number: usize,
        current_time: f32,
        dt: f32,
        live_play_state: LivePlayState,
    ) -> Self {
        let mut input = Self::timeline(processor, frame_number, current_time, dt);
        input.live_play_state = Some(live_play_state);
        input
    }

    #[allow(clippy::too_many_arguments)]
    pub fn aggregate(
        processor: &dyn ProcessorView,
        frame_number: usize,
        current_time: f32,
        dt: f32,
        last_demolish_count: usize,
        last_boost_pad_event_count: usize,
        last_touch_event_count: usize,
        last_dodge_refreshed_event_count: usize,
        last_player_stat_event_count: usize,
        last_goal_event_count: usize,
    ) -> Self {
        Self {
            frame_info: Self::build_frame_info(processor, frame_number, current_time, dt),
            gameplay_state: Self::build_gameplay_state(processor),
            ball_frame_state: Self::build_ball_frame_state(processor, current_time),
            player_frame_state: Self::build_player_frame_state(processor, current_time),
            frame_events_state: Self::build_events_since_last_sample(
                processor,
                last_demolish_count,
                last_boost_pad_event_count,
                last_touch_event_count,
                last_dodge_refreshed_event_count,
                last_player_stat_event_count,
                last_goal_event_count,
            ),
            live_play_state: None,
        }
    }
}
