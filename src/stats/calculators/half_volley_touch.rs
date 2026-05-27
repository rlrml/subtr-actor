use super::*;

impl HalfVolleyCalculator {
    pub(super) fn event_for_touch(
        &self,
        ball: &BallFrameState,
        touch: &TouchEvent,
    ) -> Option<HalfVolleyEvent> {
        let player = touch.player.clone()?;
        let bounce = self.last_floor_bounce.as_ref()?;
        let bounce_to_touch_seconds = touch.time - bounce.time;
        if !(0.0..=self.config.max_bounce_to_touch_seconds).contains(&bounce_to_touch_seconds) {
            return None;
        }
        self.validate_dodge_timing(touch, &player)?;

        let ball = ball.sample()?;
        let ball_position = ball.position();
        let ball_velocity = ball.velocity();
        let ball_speed = ball_velocity.length();
        if ball_speed < self.config.min_ball_speed {
            return None;
        }

        Some(HalfVolleyEvent {
            time: touch.time,
            frame: touch.frame,
            sample_time: touch.time,
            sample_frame: touch.frame,
            player,
            is_team_0: touch.team_is_team_0,
            bounce_time: bounce.time,
            bounce_frame: bounce.frame,
            bounce_to_touch_seconds,
            ball_speed,
            goal_alignment: goal_alignment(touch.team_is_team_0, ball_position, ball_velocity),
        })
    }

    fn validate_dodge_timing(&self, touch: &TouchEvent, player: &PlayerId) -> Option<()> {
        let dodge_start = self.recent_dodge_starts.get(player)?;
        let dodge_to_touch_seconds = touch.time - dodge_start.time;
        if !(0.0..=HALF_VOLLEY_MAX_DODGE_TO_TOUCH_SECONDS).contains(&dodge_to_touch_seconds) {
            return None;
        }
        let ground_to_dodge_seconds = dodge_start.time - dodge_start.ground_contact.time;
        (0.0..=HALF_VOLLEY_MAX_GROUND_TO_DODGE_SECONDS)
            .contains(&ground_to_dodge_seconds)
            .then_some(())
    }
}

fn goal_alignment(is_team_0: bool, ball_position: glam::Vec3, ball_velocity: glam::Vec3) -> f32 {
    let target_y = if is_team_0 {
        HALF_VOLLEY_GOAL_CENTER_Y
    } else {
        -HALF_VOLLEY_GOAL_CENTER_Y
    };
    let goal_direction = glam::Vec3::new(0.0, target_y, ball_position.z) - ball_position;
    goal_direction
        .normalize_or_zero()
        .dot(ball_velocity.normalize_or_zero())
}
