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
            team_zero_touch_time: Some(0.0),
            team_zero_touch_frame: Some(0),
            team_zero_dodge_contact: false,
            team_one_touch_time: Some(0.0),
            team_one_touch_frame: Some(0),
            team_one_dodge_contact: false,
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
            &LivePlayState::active_play(),
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
                    glam::Vec3::new(245.0, 0.0, 17.0),
                    glam::Vec3::new(700.0, 0.0, 0.0),
                ),
            ]),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
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
            &LivePlayState::active_play(),
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
                    glam::Vec3::new(235.0, 0.0, 17.0),
                    glam::Vec3::new(150.0, 0.0, 0.0),
                ),
            ]),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert!(calculator.events().is_empty());
    assert!(calculator.player_stats().is_empty());
}

#[test]
fn bump_detector_credits_clear_victim_impulse_without_sampled_initiator_slowdown() {
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
            &LivePlayState::active_play(),
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
                    glam::Vec3::new(245.0, 0.0, 17.0),
                    glam::Vec3::new(700.0, 0.0, 0.0),
                ),
            ]),
            &FrameEventsState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert_eq!(calculator.events().len(), 1);
    let event = &calculator.events()[0];
    assert_eq!(event.initiator, initiator_id);
    assert_eq!(event.victim, victim_id);
    assert!(event.victim_impulse >= BUMP_MIN_VICTIM_IMPULSE);
    assert!(event.closing_speed >= BUMP_MIN_CLOSING_SPEED);
}

#[test]
fn bump_detector_rejects_center_near_cars_with_separated_hitboxes() {
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
            &LivePlayState::active_play(),
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
            &LivePlayState::active_play(),
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
            &LivePlayState::active_play(),
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
                    glam::Vec3::new(245.0, 0.0, 17.0),
                    glam::Vec3::new(700.0, 0.0, 0.0),
                ),
            ]),
            &FrameEventsState::default(),
            &active_fifty_fifty(initiator_id, victim_id),
            &LivePlayState::active_play(),
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
            &LivePlayState::active_play(),
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
            &LivePlayState::active_play(),
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
            glam::Vec3::new(245.0, 0.0, 17.0),
            glam::Vec3::new(700.0, 0.0, 0.0),
        ),
        (
            glam::Vec3::new(240.0, 0.0, 17.0),
            glam::Vec3::new(650.0, 0.0, 0.0),
            glam::Vec3::new(365.0, 0.0, 17.0),
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
                &LivePlayState::active_play(),
            )
            .unwrap();
    }

    assert_eq!(calculator.events().len(), 1);
}

#[test]
fn bump_stats_accumulator_derives_summaries_from_events() {
    let events = vec![
        BumpEvent {
            time: 1.0,
            frame: 10,
            initiator: boxcars::RemoteId::Steam(1),
            victim: boxcars::RemoteId::Steam(2),
            initiator_is_team_0: true,
            victim_is_team_0: false,
            is_team_bump: false,
            strength: 800.0,
            confidence: 0.75,
            contact_distance: 120.0,
            closing_speed: 900.0,
            victim_impulse: 500.0,
            initiator_position: [0.0, 0.0, 17.0],
            victim_position: [150.0, 0.0, 17.0],
        },
        BumpEvent {
            time: 2.0,
            frame: 20,
            initiator: boxcars::RemoteId::Steam(1),
            victim: boxcars::RemoteId::Steam(3),
            initiator_is_team_0: true,
            victim_is_team_0: true,
            is_team_bump: true,
            strength: 1000.0,
            confidence: 0.8,
            contact_distance: 110.0,
            closing_speed: 950.0,
            victim_impulse: 600.0,
            initiator_position: [0.0, 0.0, 17.0],
            victim_position: [150.0, 0.0, 17.0],
        },
    ];

    let stats = BumpStatsAccumulator::from_events(&events);
    let initiator_stats = stats
        .player_stats()
        .get(&boxcars::RemoteId::Steam(1))
        .expect("initiator should have derived stats");
    assert_eq!(initiator_stats.bumps_inflicted, 2);
    assert_eq!(initiator_stats.team_bumps_inflicted, 1);
    assert_eq!(initiator_stats.last_bump_time, Some(2.0));
    assert_eq!(initiator_stats.last_bump_frame, Some(20));
    assert_eq!(initiator_stats.last_bump_strength, Some(1000.0));
    assert_eq!(initiator_stats.max_bump_strength, 1000.0);
    assert_eq!(initiator_stats.cumulative_bump_strength, 1800.0);
    assert_eq!(initiator_stats.average_bump_strength(), 900.0);

    assert_eq!(
        stats
            .player_stats()
            .get(&boxcars::RemoteId::Steam(2))
            .expect("first victim should have derived stats")
            .bumps_taken,
        1
    );
    assert_eq!(
        stats
            .player_stats()
            .get(&boxcars::RemoteId::Steam(3))
            .expect("team-bump victim should have derived stats")
            .team_bumps_taken,
        1
    );
    assert_eq!(stats.team_zero_stats().bumps_inflicted, 2);
    assert_eq!(stats.team_zero_stats().team_bumps_inflicted, 1);
    assert_eq!(stats.team_one_stats().bumps_inflicted, 0);
}
