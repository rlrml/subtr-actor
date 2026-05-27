use super::*;

impl SaLiveEventGenerator {
    pub(crate) fn frame_events(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        gameplay: &GameplayState,
        explicit_live_play: Option<LivePlayState>,
        explicit_events: &SaFrameEventSlices<'_>,
    ) -> (FrameEventsState, LivePlayState) {
        let explicit_touch_events = explicit_touch_events(frame, explicit_events.touches);
        let has_explicit_touch_events = !explicit_touch_events.is_empty();
        let explicit_dodge_refresh_keys =
            explicit_dodge_refresh_keys(frame, explicit_events.dodge_refreshes);
        let has_explicit_dodge_refreshed_events = !explicit_dodge_refresh_keys.is_empty();
        let explicit_dodge_refreshed_events =
            self.explicit_dodge_refreshed_events(frame, explicit_events.dodge_refreshes);
        let explicit_demolishes = self.explicit_demolish_events(frame, explicit_events.demolishes);
        let demo_events = explicit_demolish_events(frame, &explicit_demolishes);
        let active_demos = self.sync_active_demos(frame, &explicit_demolishes);
        let boost_pad_events =
            self.explicit_boost_pad_events(frame, explicit_events.boost_pad_events);
        let player_stat_events =
            explicit_player_stat_events(frame, explicit_events.player_stat_events);
        let goal_events = self.explicit_goal_events(frame, explicit_events.goals);
        let base_events = FrameEventsState {
            active_demos,
            demo_events,
            boost_pad_events,
            player_stat_events,
            goal_events,
            ..FrameEventsState::default()
        };
        let live_play = explicit_live_play.unwrap_or_else(|| {
            let mut gameplay = gameplay.clone();
            if has_explicit_touch_events || has_explicit_dodge_refreshed_events {
                if gameplay.ball_has_been_hit == Some(false) {
                    gameplay.ball_has_been_hit = Some(true);
                }
                if gameplay.kickoff_countdown_time.is_some_and(|time| time > 0) {
                    gameplay.kickoff_countdown_time = Some(0);
                    gameplay.game_state = None;
                }
            }
            self.live_play_tracker.state_parts(&gameplay, &base_events)
        });
        let touch_tracker_events = FrameEventsState {
            touch_events: explicit_touch_events,
            dodge_refreshed_events: explicit_dodge_refreshed_events.clone(),
            ..FrameEventsState::default()
        };
        let touch_state =
            self.touch_state
                .update(frame, ball, players, &touch_tracker_events, &live_play);
        let mut touch_events = touch_state.touch_events;
        if touch_events.is_empty() && has_explicit_touch_events {
            touch_events = touch_tracker_events.touch_events.clone();
        }
        let mut dodge_refreshed_events = explicit_dodge_refreshed_events;
        dodge_refreshed_events.sort_by_key(|event| event.counter_value);

        (
            FrameEventsState {
                touch_events,
                dodge_refreshed_events,
                ..base_events
            },
            live_play,
        )
    }
}
