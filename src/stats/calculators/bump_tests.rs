use super::*;

fn rigid_body(position: glam::Vec3, velocity: glam::Vec3) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: false,
        location: glam_to_vec(&position),
        rotation: boxcars::Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: Some(glam_to_vec(&velocity)),
        angular_velocity: Some(glam_to_vec(&glam::Vec3::ZERO)),
    }
}

fn player(id: u64, is_team_0: bool, position: glam::Vec3, velocity: glam::Vec3) -> PlayerSample {
    PlayerSample {
        player_id: boxcars::RemoteId::Steam(id),
        is_team_0,
        rigid_body: Some(rigid_body(position, velocity)),
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

fn frame(frame_number: usize, time: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 0.1,
        seconds_remaining: None,
    }
}

fn active_fifty_fifty(team_zero_player: PlayerId, team_one_player: PlayerId) -> FiftyFiftyState {
    FiftyFiftyState {
        active_event: Some(ActiveFiftyFifty {
            start_time: 0.0,
            start_frame: 0,
            last_touch_time: 0.1,
            last_touch_frame: 1,
            is_kickoff: false,
            team_zero_player: Some(team_zero_player),
            team_one_player: Some(team_one_player),
            team_zero_position: [100.0, 0.0, 17.0],
            team_one_position: [280.0, 0.0, 17.0],
            midpoint: [190.0, 0.0, 17.0],
            plane_normal: [1.0, 0.0, 0.0],
        }),
        ..FiftyFiftyState::default()
    }
}

fn players(players: Vec<PlayerSample>) -> PlayerFrameState {
    PlayerFrameState { players }
}

#[test]
fn bump_detector_credits_player_with_clear_directional_impulse() {
    let mut calculator = BumpCalculator::new();
    let initiator_id = boxcars::RemoteId::Steam(1);
    let victim_id = boxcars::RemoteId::Steam(2);

    calculator
        .update(
            &frame(0, 0.0),
            &players(vec![
                player(
                    1,
                    true,
                    glam::Vec3::new(0.0, 0.0, 17.0),
                    glam::Vec3::new(1200.0, 0.0, 0.0),
                ),
                player(
                    2,
                    false,
                    glam::Vec3::new(260.0, 0.0, 17.0),
                    glam::Vec3::ZERO,
                ),
            ]),
            &FrameEventsState::default(),
            true,
        )
        .unwrap();

    calculator
        .update(
            &frame(1, 0.1),
            &players(vec![
                player(
                    1,
                    true,
                    glam::Vec3::new(120.0, 0.0, 17.0),
                    glam::Vec3::new(650.0, 0.0, 0.0),
                ),
                player(
                    2,
                    false,
                    glam::Vec3::new(300.0, 0.0, 17.0),
                    glam::Vec3::new(700.0, 0.0, 0.0),
                ),
            ]),
            &FrameEventsState::default(),
            true,
        )
        .unwrap();

    assert_eq!(calculator.events().len(), 1);
    let event = &calculator.events()[0];
    assert_eq!(event.initiator, initiator_id);
    assert_eq!(event.victim, victim_id);
    assert!(!event.is_team_bump);
    assert!(event.confidence > 0.4);

    assert_eq!(
        calculator
            .player_stats()
            .get(&boxcars::RemoteId::Steam(1))
            .unwrap()
            .bumps_inflicted,
        1
    );
    assert_eq!(
        calculator
            .player_stats()
            .get(&boxcars::RemoteId::Steam(2))
            .unwrap()
            .bumps_taken,
        1
    );
    assert_eq!(calculator.team_zero_stats().bumps_inflicted, 1);
}

#[test]
fn bump_detector_requires_clear_victim_impulse() {
    let mut calculator = BumpCalculator::new();

    calculator
        .update(
            &frame(0, 0.0),
            &players(vec![
                player(
                    1,
                    true,
                    glam::Vec3::new(0.0, 0.0, 17.0),
                    glam::Vec3::new(1000.0, 0.0, 0.0),
                ),
                player(
                    2,
                    false,
                    glam::Vec3::new(260.0, 0.0, 17.0),
                    glam::Vec3::ZERO,
                ),
            ]),
            &FrameEventsState::default(),
            true,
        )
        .unwrap();

    calculator
        .update(
            &frame(1, 0.1),
            &players(vec![
                player(
                    1,
                    true,
                    glam::Vec3::new(100.0, 0.0, 17.0),
                    glam::Vec3::new(800.0, 0.0, 0.0),
                ),
                player(
                    2,
                    false,
                    glam::Vec3::new(280.0, 0.0, 17.0),
                    glam::Vec3::new(150.0, 0.0, 0.0),
                ),
            ]),
            &FrameEventsState::default(),
            true,
        )
        .unwrap();

    assert!(calculator.events().is_empty());
    assert!(calculator.player_stats().is_empty());
}

#[test]
fn bump_detector_requires_initiator_slowdown() {
    let mut calculator = BumpCalculator::new();

    calculator
        .update(
            &frame(0, 0.0),
            &players(vec![
                player(
                    1,
                    true,
                    glam::Vec3::new(0.0, 0.0, 17.0),
                    glam::Vec3::new(1200.0, 0.0, 0.0),
                ),
                player(
                    2,
                    false,
                    glam::Vec3::new(260.0, 0.0, 17.0),
                    glam::Vec3::ZERO,
                ),
            ]),
            &FrameEventsState::default(),
            true,
        )
        .unwrap();

    calculator
        .update(
            &frame(1, 0.1),
            &players(vec![
                player(
                    1,
                    true,
                    glam::Vec3::new(120.0, 0.0, 17.0),
                    glam::Vec3::new(1200.0, 0.0, 0.0),
                ),
                player(
                    2,
                    false,
                    glam::Vec3::new(300.0, 0.0, 17.0),
                    glam::Vec3::new(700.0, 0.0, 0.0),
                ),
            ]),
            &FrameEventsState::default(),
            true,
        )
        .unwrap();

    assert!(calculator.events().is_empty());
    assert!(calculator.player_stats().is_empty());
}

#[test]
fn bump_detector_suppresses_active_fifty_fifty_pair() {
    let mut calculator = BumpCalculator::new();
    let initiator_id = boxcars::RemoteId::Steam(1);
    let victim_id = boxcars::RemoteId::Steam(2);

    calculator
        .update_with_fifty_fifty_state(
            &frame(0, 0.0),
            &players(vec![
                player(
                    1,
                    true,
                    glam::Vec3::new(0.0, 0.0, 17.0),
                    glam::Vec3::new(1200.0, 0.0, 0.0),
                ),
                player(
                    2,
                    false,
                    glam::Vec3::new(260.0, 0.0, 17.0),
                    glam::Vec3::ZERO,
                ),
            ]),
            &FrameEventsState::default(),
            &active_fifty_fifty(initiator_id.clone(), victim_id.clone()),
            true,
        )
        .unwrap();

    calculator
        .update_with_fifty_fifty_state(
            &frame(1, 0.1),
            &players(vec![
                player(
                    1,
                    true,
                    glam::Vec3::new(120.0, 0.0, 17.0),
                    glam::Vec3::new(650.0, 0.0, 0.0),
                ),
                player(
                    2,
                    false,
                    glam::Vec3::new(300.0, 0.0, 17.0),
                    glam::Vec3::new(700.0, 0.0, 0.0),
                ),
            ]),
            &FrameEventsState::default(),
            &active_fifty_fifty(initiator_id, victim_id),
            true,
        )
        .unwrap();

    assert!(calculator.events().is_empty());
    assert!(calculator.player_stats().is_empty());
}

#[test]
fn bump_detector_ignores_ambiguous_or_weak_contacts() {
    let mut calculator = BumpCalculator::new();

    calculator
        .update(
            &frame(0, 0.0),
            &players(vec![
                player(
                    1,
                    true,
                    glam::Vec3::new(0.0, 0.0, 17.0),
                    glam::Vec3::new(200.0, 0.0, 0.0),
                ),
                player(
                    2,
                    false,
                    glam::Vec3::new(240.0, 0.0, 17.0),
                    glam::Vec3::new(-200.0, 0.0, 0.0),
                ),
            ]),
            &FrameEventsState::default(),
            true,
        )
        .unwrap();

    calculator
        .update(
            &frame(1, 0.1),
            &players(vec![
                player(
                    1,
                    true,
                    glam::Vec3::new(20.0, 0.0, 17.0),
                    glam::Vec3::new(120.0, 0.0, 0.0),
                ),
                player(
                    2,
                    false,
                    glam::Vec3::new(220.0, 0.0, 17.0),
                    glam::Vec3::new(-120.0, 0.0, 0.0),
                ),
            ]),
            &FrameEventsState::default(),
            true,
        )
        .unwrap();

    assert!(calculator.events().is_empty());
    assert!(calculator.player_stats().is_empty());
}

#[test]
fn bump_detector_suppresses_same_pair_repeats() {
    let mut calculator = BumpCalculator::new();

    let samples = [
        (
            glam::Vec3::new(0.0, 0.0, 17.0),
            glam::Vec3::new(1200.0, 0.0, 0.0),
            glam::Vec3::new(260.0, 0.0, 17.0),
            glam::Vec3::ZERO,
        ),
        (
            glam::Vec3::new(120.0, 0.0, 17.0),
            glam::Vec3::new(1200.0, 0.0, 0.0),
            glam::Vec3::new(300.0, 0.0, 17.0),
            glam::Vec3::new(700.0, 0.0, 0.0),
        ),
        (
            glam::Vec3::new(240.0, 0.0, 17.0),
            glam::Vec3::new(650.0, 0.0, 0.0),
            glam::Vec3::new(420.0, 0.0, 17.0),
            glam::Vec3::new(1400.0, 0.0, 0.0),
        ),
    ];

    for (
        frame_number,
        (initiator_position, initiator_velocity, victim_position, victim_velocity),
    ) in samples.into_iter().enumerate()
    {
        calculator
            .update(
                &frame(frame_number, frame_number as f32 * 0.1),
                &players(vec![
                    player(1, true, initiator_position, initiator_velocity),
                    player(2, false, victim_position, victim_velocity),
                ]),
                &FrameEventsState::default(),
                true,
            )
            .unwrap();
    }

    assert_eq!(calculator.events().len(), 1);
}
