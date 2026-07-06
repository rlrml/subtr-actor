use super::*;
use crate::stats::calculators::ball_control_test_support::{ball, frame, player, rigid_body};

fn live_play() -> LivePlayState {
    LivePlayState {
        is_live_play: true,
        ..LivePlayState::default()
    }
}

fn player_with_id(
    player_id: PlayerId,
    is_team_0: bool,
    position: glam::Vec3,
    velocity: glam::Vec3,
) -> PlayerSample {
    PlayerSample {
        player_id,
        is_team_0,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body(position, velocity)),
        boost_amount: None,
        last_boost_amount: None,
        boost_active: false,
        dodge_active: false,
        dodge_torque: None,
        powerslide_active: false,
        match_goals: None,
        match_assists: None,
        match_saves: None,
        match_shots: None,
        match_score: None,
    }
}

fn touch(
    frame: usize,
    time: f32,
    player: (PlayerId, bool),
    tags: (&str, &str),
    positions: (glam::Vec3, glam::Vec3),
) -> TouchClassificationEvent {
    let (player_position, ball_position) = positions;
    TouchClassificationEvent {
        touch_id: None,
        time,
        frame,
        sample_time: time,
        sample_frame: frame,
        player: player.0,
        player_position: Some(player_position.to_array()),
        ball_position: Some(ball_position.to_array()),
        is_team_0: player.1,
        tags: TouchClassificationEvent::classification_tags(
            tags.0, "high", tags.1, "none", None, false, false,
        ),
        role: RoleState::Unknown,
        play_depth: PlayDepthState::Unknown,
        ball_speed_change: 120.0,
        ball_movement: None,
    }
}

fn update(
    calculator: &mut AirDribbleCalculator,
    frame_number: usize,
    time: f32,
    player_position: glam::Vec3,
    ball_position: glam::Vec3,
    touches: &[TouchClassificationEvent],
) {
    calculator
        .update_with_touch_classification_events(
            &frame(frame_number, time),
            &ball(ball_position, glam::Vec3::new(200.0, 0.0, 0.0)),
            &PlayerFrameState {
                players: vec![player(player_position, glam::Vec3::new(200.0, 0.0, 0.0))],
            },
            &live_play(),
            touches,
        )
        .unwrap();
}

#[test]
fn low_ball_at_ground_opener_does_not_break_before_first_qualifying_air_touch() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = AirDribbleCalculator::new();
    let mut touches = Vec::new();

    let ground_player_position = glam::Vec3::new(0.0, 0.0, 17.0);
    let ground_ball_position = glam::Vec3::new(30.0, 0.0, 98.0);
    touches.push(touch(
        1,
        0.0,
        (player_id.clone(), true),
        ("control", "ground"),
        (ground_player_position, ground_ball_position),
    ));
    update(
        &mut calculator,
        1,
        0.0,
        ground_player_position,
        ground_ball_position,
        &touches,
    );
    update(
        &mut calculator,
        2,
        0.2,
        glam::Vec3::new(60.0, 0.0, 17.0),
        glam::Vec3::new(90.0, 0.0, 120.0),
        &touches,
    );

    for (frame_number, time, x, kind, player_z, ball_z) in [
        (3, 0.4, 120.0, "medium_hit", 360.0, 649.0),
        (5, 0.8, 260.0, "control", 365.0, 655.0),
    ] {
        let player_position = glam::Vec3::new(x, 0.0, player_z);
        let ball_position = glam::Vec3::new(x + 30.0, 0.0, ball_z);
        touches.push(touch(
            frame_number,
            time,
            (player_id.clone(), true),
            (kind, "air"),
            (player_position, ball_position),
        ));
        update(
            &mut calculator,
            frame_number,
            time,
            player_position,
            ball_position,
            &touches,
        );
    }

    calculator.finish().unwrap();

    assert_eq!(calculator.events().len(), 1);
    let event = &calculator.events()[0];
    assert_eq!(event.player_id, player_id);
    assert_eq!(event.kind, BallCarryKind::AirDribble);
    assert_eq!(event.touch_count, 3);
    assert_eq!(event.air_touch_count, 2);
    assert_eq!(
        event.air_dribble_origin,
        Some(AirDribbleOrigin::GroundToAir)
    );
}

#[test]
fn low_ball_after_qualifying_air_touches_breaks_sequence() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = AirDribbleCalculator::new();
    let mut touches = Vec::new();

    for (frame_number, time, x, surface, player_z, ball_z) in [
        (1, 0.0, 0.0, "ground", 17.0, 98.0),
        (3, 0.4, 120.0, "air", 360.0, 520.0),
        (5, 0.8, 260.0, "air", 365.0, 530.0),
    ] {
        let player_position = glam::Vec3::new(x, 0.0, player_z);
        let ball_position = glam::Vec3::new(x + 30.0, 0.0, ball_z);
        touches.push(touch(
            frame_number,
            time,
            (player_id.clone(), true),
            ("control", surface),
            (player_position, ball_position),
        ));
        update(
            &mut calculator,
            frame_number,
            time,
            player_position,
            ball_position,
            &touches,
        );
    }

    update(
        &mut calculator,
        6,
        1.0,
        glam::Vec3::new(320.0, 0.0, 365.0),
        glam::Vec3::new(350.0, 0.0, 120.0),
        &touches,
    );

    assert_eq!(calculator.events().len(), 1);
    let event = &calculator.events()[0];
    assert_eq!(event.touch_count, 3);
    assert_eq!(event.air_touch_count, 2);
    assert_eq!(event.end_frame, 5);

    let player_position = glam::Vec3::new(260.0, 0.0, 360.0);
    let ball_position = glam::Vec3::new(290.0, 0.0, 520.0);
    touches.push(touch(
        7,
        1.2,
        (player_id, true),
        ("control", "air"),
        (player_position, ball_position),
    ));
    update(
        &mut calculator,
        7,
        1.2,
        player_position,
        ball_position,
        &touches,
    );

    calculator.finish().unwrap();

    assert_eq!(calculator.events().len(), 1);
}

#[test]
fn fewer_than_two_qualifying_air_touches_does_not_emit() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = AirDribbleCalculator::new();
    let mut touches = Vec::new();

    for (frame_number, time, surface) in [(1, 0.0, "ground"), (3, 0.4, "ground"), (5, 0.8, "air")] {
        let x = frame_number as f32 * 60.0;
        let player_position = glam::Vec3::new(x, 0.0, 360.0);
        let ball_position = glam::Vec3::new(x + 30.0, 0.0, 520.0);
        touches.push(touch(
            frame_number,
            time,
            (player_id.clone(), true),
            ("control", surface),
            (player_position, ball_position),
        ));
        update(
            &mut calculator,
            frame_number,
            time,
            player_position,
            ball_position,
            &touches,
        );
    }

    calculator.finish().unwrap();

    assert!(calculator.events().is_empty());
}

#[test]
fn wall_touches_do_not_count_toward_air_touch_threshold() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = AirDribbleCalculator::new();
    let mut touches = Vec::new();

    for (frame_number, time, surface, player_position, ball_position) in [
        (
            1,
            0.0,
            "wall",
            glam::Vec3::new(3300.0, 0.0, 180.0),
            glam::Vec3::new(3328.0, 0.0, 390.0),
        ),
        (
            3,
            0.4,
            "wall",
            glam::Vec3::new(3330.0, 0.0, 260.0),
            glam::Vec3::new(3358.0, 0.0, 460.0),
        ),
        (
            5,
            0.8,
            "air",
            glam::Vec3::new(340.0, 0.0, 360.0),
            glam::Vec3::new(370.0, 0.0, 540.0),
        ),
    ] {
        touches.push(touch(
            frame_number,
            time,
            (player_id.clone(), true),
            ("control", surface),
            (player_position, ball_position),
        ));
        update(
            &mut calculator,
            frame_number,
            time,
            player_position,
            ball_position,
            &touches,
        );
    }

    calculator.finish().unwrap();

    assert!(calculator.events().is_empty());
}

#[test]
fn wall_opener_plus_two_air_touches_emits_with_two_air_touches() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = AirDribbleCalculator::new();
    let mut touches = Vec::new();

    for (frame_number, time, surface, player_position, ball_position) in [
        (
            1,
            0.0,
            "wall",
            glam::Vec3::new(3300.0, 0.0, 180.0),
            glam::Vec3::new(3328.0, 0.0, 390.0),
        ),
        (
            3,
            0.4,
            "air",
            glam::Vec3::new(220.0, 0.0, 360.0),
            glam::Vec3::new(250.0, 0.0, 540.0),
        ),
        (
            5,
            0.8,
            "air",
            glam::Vec3::new(360.0, 0.0, 365.0),
            glam::Vec3::new(390.0, 0.0, 560.0),
        ),
    ] {
        touches.push(touch(
            frame_number,
            time,
            (player_id.clone(), true),
            ("control", surface),
            (player_position, ball_position),
        ));
        update(
            &mut calculator,
            frame_number,
            time,
            player_position,
            ball_position,
            &touches,
        );
    }

    calculator.finish().unwrap();

    assert_eq!(calculator.events().len(), 1);
    let event = &calculator.events()[0];
    assert_eq!(event.touch_count, 3);
    assert_eq!(event.air_touch_count, 2);
    assert_eq!(event.air_dribble_origin, Some(AirDribbleOrigin::WallToAir));
}

#[test]
fn different_player_touch_starts_a_new_sequence() {
    let first_player = boxcars::RemoteId::Steam(1);
    let second_player = boxcars::RemoteId::Steam(2);
    let mut calculator = AirDribbleCalculator::new();
    let mut touches = Vec::new();

    for (frame_number, time, player_id) in [
        (1, 0.0, first_player.clone()),
        (3, 0.4, first_player),
        (5, 0.8, second_player),
    ] {
        let x = frame_number as f32 * 60.0;
        let player_position = glam::Vec3::new(x, 0.0, 360.0);
        let ball_position = glam::Vec3::new(x + 30.0, 0.0, 520.0);
        touches.push(touch(
            frame_number,
            time,
            (player_id.clone(), true),
            ("control", "air"),
            (player_position, ball_position),
        ));
        calculator
            .update_with_touch_classification_events(
                &frame(frame_number, time),
                &ball(ball_position, glam::Vec3::new(200.0, 0.0, 0.0)),
                &PlayerFrameState {
                    players: vec![player_with_id(
                        player_id,
                        true,
                        player_position,
                        glam::Vec3::new(200.0, 0.0, 0.0),
                    )],
                },
                &live_play(),
                &touches,
            )
            .unwrap();
    }

    calculator.finish().unwrap();

    assert!(calculator.events().is_empty());
}
