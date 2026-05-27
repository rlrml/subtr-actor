use super::*;

impl OneTimerCalculator {
    pub(super) fn one_timer_event_for_pass(
        pass: &PassEvent,
        ball: &BallFrameState,
    ) -> Option<OneTimerEvent> {
        let ball = ball.sample()?;
        let ball_position = ball.position();
        let ball_velocity = ball.velocity();
        let ball_speed = ball_velocity.length();
        if ball_speed < ONE_TIMER_MIN_BALL_SPEED {
            return None;
        }

        let target_y = if pass.is_team_0 {
            GOAL_CENTER_Y
        } else {
            -GOAL_CENTER_Y
        };
        let goal_direction = glam::Vec3::new(0.0, target_y, ball_position.z) - ball_position;
        let goal_alignment = goal_direction
            .normalize_or_zero()
            .dot(ball_velocity.normalize_or_zero());
        if goal_alignment < ONE_TIMER_MIN_GOAL_ALIGNMENT_COSINE {
            return None;
        }

        Some(OneTimerEvent {
            time: pass.time,
            frame: pass.frame,
            player: pass.receiver.clone(),
            passer: pass.passer.clone(),
            is_team_0: pass.is_team_0,
            pass_start_time: pass.start_time,
            pass_start_frame: pass.start_frame,
            pass_duration: pass.duration,
            pass_travel_distance: pass.ball_travel_distance,
            pass_advance_distance: pass.ball_advance_distance,
            ball_speed,
            goal_alignment,
        })
    }
}
