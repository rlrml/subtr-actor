use super::*;

impl PassCalculator {
    pub(super) fn pass_event_for_touch(
        &self,
        touch: &TouchEvent,
        receiver: &PlayerId,
        ball_position: glam::Vec3,
        backboard_bounce_state: &BackboardBounceState,
    ) -> Option<PassEvent> {
        let previous = self.last_touch.as_ref()?;
        if previous.player == *receiver || previous.is_team_0 != touch.team_is_team_0 {
            return None;
        }

        let duration = touch.time - previous.time;
        if !(0.0..=PASS_MAX_DURATION_SECONDS).contains(&duration) {
            return None;
        }

        let ball_delta = ball_position - previous.ball_position;
        let ball_travel_distance = ball_delta.length();
        if ball_travel_distance < PASS_MIN_BALL_TRAVEL_DISTANCE {
            return None;
        }

        Some(Self::pass_event(
            touch,
            receiver,
            previous,
            ball_delta,
            ball_travel_distance,
            backboard_bounce_state,
        ))
    }

    fn pass_event(
        touch: &TouchEvent,
        receiver: &PlayerId,
        previous: &PendingPassTouch,
        ball_delta: glam::Vec3,
        ball_travel_distance: f32,
        backboard_bounce_state: &BackboardBounceState,
    ) -> PassEvent {
        let went_off_backboard = Self::has_backboard_bounce_between(
            previous,
            touch,
            backboard_bounce_state.last_bounce_event.as_ref(),
        );
        PassEvent {
            time: touch.time,
            frame: touch.frame,
            sample_time: touch.time,
            sample_frame: touch.frame,
            passer: previous.player.clone(),
            receiver: receiver.clone(),
            is_team_0: touch.team_is_team_0,
            start_time: previous.time,
            start_frame: previous.frame,
            duration: touch.time - previous.time,
            ball_travel_distance,
            ball_advance_distance: ball_delta.y * team_forward_sign(touch.team_is_team_0),
            pass_kind: Self::pass_kind(previous.from_fifty_fifty, went_off_backboard),
        }
    }

    fn has_backboard_bounce_between(
        previous: &PendingPassTouch,
        touch: &TouchEvent,
        bounce_event: Option<&BackboardBounceEvent>,
    ) -> bool {
        bounce_event.is_some_and(|event| {
            event.player == previous.player
                && event.is_team_0 == previous.is_team_0
                && event.time >= previous.time
                && event.time <= touch.time
        })
    }
}

fn team_forward_sign(is_team_0: bool) -> f32 {
    if is_team_0 {
        1.0
    } else {
        -1.0
    }
}
