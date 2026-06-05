use super::*;

fn rigid_body(position: glam::Vec3) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: false,
        location: glam_to_vec(&position),
        rotation: boxcars::Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: None,
        angular_velocity: None,
    }
}

fn ball(position: glam::Vec3) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(position),
    })
}

fn player(player_id: PlayerId, is_team_0: bool, position: glam::Vec3) -> PlayerSample {
    PlayerSample {
        player_id,
        is_team_0,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body(position)),
        boost_amount: None,
        last_boost_amount: None,
        boost_active: false,
        dodge_active: false,
        powerslide_active: false,
        match_goals: None,
        match_assists: None,
        match_saves: None,
        match_shots: None,
        match_score: None,
    }
}

fn frame(frame_number: usize, time: f32, dt: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt,
        seconds_remaining: None,
    }
}

fn gameplay_2v2() -> GameplayState {
    GameplayState {
        ball_has_been_hit: Some(true),
        current_in_game_team_player_counts: [2, 2],
        ..GameplayState::default()
    }
}

fn update_team_zero(
    calculator: &mut RotationCalculator,
    frame_number: usize,
    time: f32,
    blue_a_position: glam::Vec3,
    blue_b_position: glam::Vec3,
) {
    calculator
        .update(
            &frame(frame_number, time, 0.1),
            &gameplay_2v2(),
            &ball(glam::Vec3::ZERO),
            &PlayerFrameState {
                players: vec![
                    player(PlayerId::Steam(1), true, blue_a_position),
                    player(PlayerId::Steam(2), true, blue_b_position),
                    player(
                        PlayerId::Steam(3),
                        false,
                        glam::Vec3::new(3000.0, 3000.0, 0.0),
                    ),
                    player(
                        PlayerId::Steam(4),
                        false,
                        glam::Vec3::new(-3000.0, 3000.0, 0.0),
                    ),
                ],
            },
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .expect("rotation update should succeed");
}

#[test]
fn debounces_first_man_changes_before_counting_rotations() {
    let mut calculator = RotationCalculator::with_config(RotationCalculatorConfig {
        first_man_debounce_seconds: 0.3,
        first_man_ambiguity_margin: 50.0,
        ..RotationCalculatorConfig::default()
    });

    update_team_zero(
        &mut calculator,
        1,
        0.1,
        glam::Vec3::new(100.0, 0.0, 0.0),
        glam::Vec3::new(1000.0, 0.0, 0.0),
    );
    assert_eq!(calculator.team_zero_stats().rotation_count, 0);

    update_team_zero(
        &mut calculator,
        2,
        0.2,
        glam::Vec3::new(1000.0, 0.0, 0.0),
        glam::Vec3::new(100.0, 0.0, 0.0),
    );
    update_team_zero(
        &mut calculator,
        3,
        0.3,
        glam::Vec3::new(1000.0, 0.0, 0.0),
        glam::Vec3::new(100.0, 0.0, 0.0),
    );
    assert_eq!(calculator.team_zero_stats().rotation_count, 0);

    update_team_zero(
        &mut calculator,
        4,
        0.4,
        glam::Vec3::new(1000.0, 0.0, 0.0),
        glam::Vec3::new(100.0, 0.0, 0.0),
    );
    assert_eq!(calculator.team_zero_stats().rotation_count, 1);
    assert_eq!(
        calculator
            .player_stats()
            .get(&PlayerId::Steam(1))
            .expect("player one stats")
            .lost_first_man_count,
        1
    );
    assert_eq!(
        calculator
            .player_stats()
            .get(&PlayerId::Steam(2))
            .expect("player two stats")
            .became_first_man_count,
        1
    );
}

#[test]
fn ambiguous_first_man_frames_do_not_create_rotations() {
    let mut calculator = RotationCalculator::with_config(RotationCalculatorConfig {
        first_man_debounce_seconds: 0.2,
        first_man_ambiguity_margin: 300.0,
        ..RotationCalculatorConfig::default()
    });

    update_team_zero(
        &mut calculator,
        1,
        0.1,
        glam::Vec3::new(100.0, 0.0, 0.0),
        glam::Vec3::new(1000.0, 0.0, 0.0),
    );
    update_team_zero(
        &mut calculator,
        2,
        0.2,
        glam::Vec3::new(100.0, 0.0, 0.0),
        glam::Vec3::new(250.0, 0.0, 0.0),
    );
    update_team_zero(
        &mut calculator,
        3,
        0.3,
        glam::Vec3::new(100.0, 0.0, 0.0),
        glam::Vec3::new(250.0, 0.0, 0.0),
    );

    assert_eq!(calculator.team_zero_stats().rotation_count, 0);
    let player_one_stats = calculator
        .player_stats()
        .get(&PlayerId::Steam(1))
        .expect("player one stats");
    assert_eq!(player_one_stats.current_role_state, RoleState::Ambiguous);
    assert_eq!(player_one_stats.time_ambiguous_role, 0.2);
}

#[test]
fn records_role_and_depth_time() {
    let mut calculator = RotationCalculator::with_config(RotationCalculatorConfig {
        first_man_ambiguity_margin: 50.0,
        ..RotationCalculatorConfig::default()
    });

    update_team_zero(
        &mut calculator,
        1,
        0.1,
        glam::Vec3::new(0.0, -500.0, 0.0),
        glam::Vec3::new(1000.0, 500.0, 0.0),
    );

    let first_man = calculator
        .player_stats()
        .get(&PlayerId::Steam(1))
        .expect("first man stats");
    assert_eq!(first_man.current_role_state, RoleState::FirstMan);
    assert_eq!(first_man.current_depth_state, PlayDepthState::BehindPlay);
    assert_eq!(first_man.time_first_man, 0.1);
    assert_eq!(first_man.time_behind_play, 0.1);

    let second_man = calculator
        .player_stats()
        .get(&PlayerId::Steam(2))
        .expect("second man stats");
    assert_eq!(second_man.current_role_state, RoleState::SecondMan);
    assert_eq!(second_man.current_depth_state, PlayDepthState::AheadOfPlay);
    assert_eq!(second_man.time_second_man, 0.1);
    assert_eq!(second_man.time_ahead_of_play, 0.1);
}

#[test]
fn rotation_player_events_emit_state_change_spans() {
    let mut calculator = RotationCalculator::with_config(RotationCalculatorConfig {
        first_man_ambiguity_margin: 50.0,
        ..RotationCalculatorConfig::default()
    });

    for frame_number in 1..=3 {
        update_team_zero(
            &mut calculator,
            frame_number,
            frame_number as f32 * 0.1,
            glam::Vec3::new(0.0, -500.0, 0.0),
            glam::Vec3::new(1000.0, 500.0, 0.0),
        );
    }
    calculator.flush_pending_player_events();

    let player_events = calculator.player_events();
    assert_eq!(player_events.len(), 4);
    let first_man = player_events
        .iter()
        .find(|event| event.player == PlayerId::Steam(1) && event.active)
        .expect("first man span should be emitted");
    assert_eq!(first_man.frame, 1);
    assert_eq!(first_man.end_frame, 3);
    assert!((first_man.duration - 0.3).abs() < 1e-6);
    assert_eq!(first_man.current_role_state, RoleState::FirstMan);
    assert_eq!(first_man.current_depth_state, PlayDepthState::BehindPlay);

    let second_man = player_events
        .iter()
        .find(|event| event.player == PlayerId::Steam(2) && event.active)
        .expect("second man span should be emitted");
    assert_eq!(second_man.frame, 1);
    assert_eq!(second_man.end_frame, 3);
    assert!((second_man.duration - 0.3).abs() < 1e-6);
    assert_eq!(second_man.current_role_state, RoleState::SecondMan);
    assert_eq!(second_man.current_depth_state, PlayDepthState::AheadOfPlay);
}

#[test]
fn derived_rotation_events_split_role_depth_and_stints() {
    let mut calculator = RotationCalculator::with_config(RotationCalculatorConfig {
        first_man_debounce_seconds: 0.25,
        first_man_ambiguity_margin: 50.0,
        ..RotationCalculatorConfig::default()
    });

    update_team_zero(
        &mut calculator,
        1,
        0.1,
        glam::Vec3::new(0.0, -300.0, 0.0),
        glam::Vec3::new(1000.0, 0.0, 0.0),
    );
    update_team_zero(
        &mut calculator,
        2,
        0.2,
        glam::Vec3::new(0.0, 300.0, 0.0),
        glam::Vec3::new(1000.0, 0.0, 0.0),
    );
    update_team_zero(
        &mut calculator,
        3,
        0.3,
        glam::Vec3::new(0.0, 300.0, 0.0),
        glam::Vec3::new(1000.0, 0.0, 0.0),
    );
    calculator.flush_pending_player_events();

    let player_one_raw = calculator
        .player_events()
        .iter()
        .filter(|event| event.player == PlayerId::Steam(1) && event.active)
        .collect::<Vec<_>>();
    let player_one_role_spans = calculator
        .role_span_events()
        .into_iter()
        .filter(|event| event.player == PlayerId::Steam(1))
        .collect::<Vec<_>>();
    let player_one_depth_spans = calculator
        .depth_span_events()
        .into_iter()
        .filter(|event| event.player == PlayerId::Steam(1))
        .collect::<Vec<_>>();
    let player_one_stints = calculator
        .first_man_stint_events()
        .into_iter()
        .filter(|event| event.player == PlayerId::Steam(1))
        .collect::<Vec<_>>();

    assert_eq!(player_one_raw.len(), 2);
    assert_eq!(player_one_role_spans.len(), 1);
    assert_eq!(
        player_one_role_spans[0].current_role_state,
        RoleState::FirstMan
    );
    assert!((player_one_role_spans[0].duration - 0.3).abs() < 1e-6);
    assert_eq!(player_one_depth_spans.len(), 2);
    assert_eq!(
        player_one_depth_spans[0].current_depth_state,
        PlayDepthState::BehindPlay
    );
    assert_eq!(
        player_one_depth_spans[1].current_depth_state,
        PlayDepthState::AheadOfPlay
    );
    assert_eq!(player_one_stints.len(), 1);
    assert_eq!(player_one_stints[0].frame, 1);
    assert_eq!(player_one_stints[0].end_frame, 3);
    assert!((player_one_stints[0].duration - 0.3).abs() < 1e-6);
}

#[test]
fn first_man_stints_survive_brief_interruptions() {
    let mut calculator = RotationCalculator::with_config(RotationCalculatorConfig {
        first_man_debounce_seconds: 0.25,
        first_man_ambiguity_margin: 50.0,
        ..RotationCalculatorConfig::default()
    });

    update_team_zero(
        &mut calculator,
        1,
        0.1,
        glam::Vec3::new(100.0, 0.0, 0.0),
        glam::Vec3::new(1000.0, 0.0, 0.0),
    );
    update_team_zero(
        &mut calculator,
        2,
        0.2,
        glam::Vec3::new(100.0, 0.0, 0.0),
        glam::Vec3::new(120.0, 0.0, 0.0),
    );
    update_team_zero(
        &mut calculator,
        3,
        0.3,
        glam::Vec3::new(100.0, 0.0, 0.0),
        glam::Vec3::new(1000.0, 0.0, 0.0),
    );

    let stats = calculator
        .player_stats()
        .get(&PlayerId::Steam(1))
        .expect("player one stats");
    assert_eq!(stats.first_man_stint_count, 1);
    assert_eq!(stats.time_first_man, 0.2);
    assert_eq!(stats.longest_first_man_stint_time, 0.2);
    assert_eq!(stats.average_first_man_stint_time(), 0.2);

    update_team_zero(
        &mut calculator,
        4,
        0.4,
        glam::Vec3::new(100.0, 0.0, 0.0),
        glam::Vec3::new(120.0, 0.0, 0.0),
    );
    update_team_zero(
        &mut calculator,
        5,
        0.5,
        glam::Vec3::new(100.0, 0.0, 0.0),
        glam::Vec3::new(120.0, 0.0, 0.0),
    );
    update_team_zero(
        &mut calculator,
        6,
        0.6,
        glam::Vec3::new(100.0, 0.0, 0.0),
        glam::Vec3::new(120.0, 0.0, 0.0),
    );
    update_team_zero(
        &mut calculator,
        7,
        0.7,
        glam::Vec3::new(100.0, 0.0, 0.0),
        glam::Vec3::new(1000.0, 0.0, 0.0),
    );

    let stats = calculator
        .player_stats()
        .get(&PlayerId::Steam(1))
        .expect("player one stats");
    assert_eq!(stats.first_man_stint_count, 2);
    assert!((stats.time_first_man - 0.3).abs() < f32::EPSILON);
    assert!((stats.longest_first_man_stint_time - 0.2).abs() < f32::EPSILON);
    assert!((stats.average_first_man_stint_time() - 0.15).abs() < f32::EPSILON);
}
