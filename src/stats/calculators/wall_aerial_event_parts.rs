use super::*;

pub(super) fn wall_aerial_event_from_parts(
    touch: &TouchEvent,
    armed: &ArmedWallAerial,
    player_position: glam::Vec3,
    ball: &BallSample,
    time_since_takeoff: f32,
    ball_speed_change: f32,
) -> WallAerialEvent {
    let ball_position = ball.position();
    let setup = &armed.controlled_setup;
    WallAerialEvent {
        time: touch.time,
        frame: touch.frame,
        sample_time: touch.time,
        sample_frame: touch.frame,
        player: touch
            .player
            .as_ref()
            .expect("touch player is required")
            .clone(),
        is_team_0: touch.team_is_team_0,
        wall: armed.wall,
        wall_contact_time: armed.wall_contact_time,
        wall_contact_frame: armed.wall_contact_frame,
        takeoff_time: armed.takeoff_time,
        takeoff_frame: armed.takeoff_frame,
        time_since_takeoff,
        wall_contact_position: armed.wall_contact_position.to_array(),
        takeoff_position: armed.takeoff_position.to_array(),
        player_position: player_position.to_array(),
        ball_position: ball_position.to_array(),
        setup_start_time: setup.start_time,
        setup_start_frame: setup.start_frame,
        setup_duration: setup.duration,
        ball_speed: ball.velocity().length(),
        ball_speed_change,
        goal_alignment: wall_aerial_goal_alignment(
            touch.team_is_team_0,
            ball_position,
            ball.velocity(),
        ),
        confidence: wall_aerial_event_confidence(
            setup.duration,
            time_since_takeoff,
            player_position.z,
            ball_speed_change,
        ),
    }
}

fn wall_aerial_event_confidence(
    setup_duration: f32,
    time_since_takeoff: f32,
    player_z: f32,
    ball_speed_change: f32,
) -> f32 {
    (0.30
        + 0.20 * wall_aerial_normalize_score(setup_duration, WALL_AERIAL_MIN_CONTROL_DURATION, 1.2)
        + 0.18
            * (1.0
                - wall_aerial_normalize_score(
                    time_since_takeoff,
                    0.15,
                    WALL_AERIAL_MAX_TAKEOFF_TO_TOUCH_SECONDS,
                ))
        + 0.16 * wall_aerial_normalize_score(player_z, WALL_AERIAL_MIN_TOUCH_PLAYER_Z, 850.0)
        + 0.16
            * wall_aerial_normalize_score(
                ball_speed_change,
                WALL_AERIAL_REFERENCE_BALL_SPEED_CHANGE,
                900.0,
            ))
    .clamp(0.0, 1.0)
}
