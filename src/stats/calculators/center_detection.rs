use super::*;

impl CenterCalculator {
    pub(super) fn center_event_for_position(
        pending: &PendingCenterTouch,
        frame: &FrameInfo,
        ball_position: glam::Vec3,
    ) -> Option<CenterEvent> {
        let duration = frame.time - pending.time;
        if !(0.0..=CENTER_MAX_DURATION_SECONDS).contains(&duration)
            || !is_centering_lane_change(pending, ball_position)
        {
            return None;
        }

        let ball_delta = ball_position - pending.ball_position;
        let ball_travel_distance = ball_delta.length();
        if ball_travel_distance < CENTER_MIN_BALL_TRAVEL_DISTANCE {
            return None;
        }

        Some(CenterEvent {
            time: frame.time,
            frame: frame.frame_number,
            player: pending.player.clone(),
            is_team_0: pending.is_team_0,
            start_time: pending.time,
            start_frame: pending.frame,
            duration,
            start_ball_position: pending.ball_position.to_array(),
            end_ball_position: ball_position.to_array(),
            ball_travel_distance,
            ball_advance_distance: ball_delta.y * team_forward_sign(pending.is_team_0),
            lateral_centering_distance: pending.ball_position.x.abs() - ball_position.x.abs(),
        })
    }
}

fn is_centering_lane_change(pending: &PendingCenterTouch, ball_position: glam::Vec3) -> bool {
    let start_normalized_y = normalized_y(pending.is_team_0, pending.ball_position);
    let end_normalized_y = normalized_y(pending.is_team_0, ball_position);
    if start_normalized_y < CENTER_MIN_START_ATTACKING_Y
        || end_normalized_y < CENTER_MIN_END_ATTACKING_Y
    {
        return false;
    }

    let start_abs_x = pending.ball_position.x.abs();
    let end_abs_x = ball_position.x.abs();
    start_abs_x >= CENTER_MIN_START_ABS_X
        && end_abs_x <= CENTER_MAX_END_ABS_X
        && start_abs_x - end_abs_x >= CENTER_MIN_LATERAL_DISTANCE
}

fn team_forward_sign(is_team_0: bool) -> f32 {
    if is_team_0 {
        1.0
    } else {
        -1.0
    }
}
