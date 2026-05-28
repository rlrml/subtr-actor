use super::*;

impl TouchCalculator {
    pub(crate) fn apply_touch_events(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        vertical_state: &PlayerVerticalState,
        touch_events: &[TouchEvent],
    ) {
        let ball_speed_change = Self::ball_speed_change(frame, ball, self.previous_ball_velocity);

        for touch_event in touch_events {
            let Some(player_id) = touch_event.player.as_ref() else {
                continue;
            };
            let classification = Self::touch_classification_for_event(
                touch_event,
                ball,
                players,
                vertical_state,
                player_id,
                ball_speed_change,
            );
            self.events.push(touch_stats_event(
                frame,
                touch_event,
                player_id,
                classification,
                ball_speed_change,
            ));
            self.apply_touch_stats(
                frame,
                touch_event,
                player_id,
                classification,
                ball_speed_change,
            );
        }

        self.record_last_touch(frame, touch_events);
        self.mark_current_last_touch();
    }

    fn touch_classification_for_event(
        touch_event: &TouchEvent,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        vertical_state: &PlayerVerticalState,
        player_id: &PlayerId,
        ball_speed_change: f32,
    ) -> TouchClassification {
        let height_band = Self::height_band_for_touch(vertical_state.sample(player_id));
        let surface =
            Self::surface_for_touch(Self::player_position(players, player_id), height_band);
        let dodge_state = TouchDodgeState::from_dodge_active(
            touch_event.dodge_contact || Self::player_dodge_active(players, player_id),
        );
        let controlled_touch_kind = Self::controlled_touch_kind(ball, players, player_id);
        Self::classify_touch(
            height_band,
            surface,
            dodge_state,
            ball_speed_change,
            controlled_touch_kind,
        )
    }
}
