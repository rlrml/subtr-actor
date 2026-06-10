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

fn player_at(
    id: u64,
    position: glam::Vec3,
    velocity: glam::Vec3,
    dodge_active: bool,
) -> PlayerSample {
    PlayerSample {
        player_id: boxcars::RemoteId::Steam(id),
        is_team_0: true,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body(position, velocity)),
        boost_amount: None,
        last_boost_amount: None,
        boost_active: false,
        dodge_active,
        powerslide_active: false,
        match_goals: None,
        match_assists: None,
        match_saves: None,
        match_shots: None,
        match_score: None,
    }
}

fn player(id: u64, x: f32, dodge_active: bool) -> PlayerSample {
    player_at(
        id,
        glam::Vec3::new(x, 0.0, 17.0),
        glam::Vec3::new(900.0, 0.0, 0.0),
        dodge_active,
    )
}

fn frame(frame_number: usize, time: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 0.1,
        seconds_remaining: None,
    }
}

fn ball() -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(glam::Vec3::new(0.0, 0.0, 92.75), glam::Vec3::ZERO),
    })
}

fn moving_ball(velocity: glam::Vec3) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(glam::Vec3::new(0.0, 0.0, 92.75), velocity),
    })
}

#[test]
fn counts_near_miss_after_player_exits_ball_area() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = WhiffCalculator::new();
    let ball = ball();
    let touch_state = TouchState::default();

    calculator
        .update(
            &frame(1, 0.1),
            &ball,
            &PlayerFrameState {
                players: vec![player(1, -210.0, false)],
            },
            &touch_state,
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.2),
            &ball,
            &PlayerFrameState {
                players: vec![player(1, 460.0, false)],
            },
            &touch_state,
            &LivePlayState::active_play(),
        )
        .unwrap();

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.whiff_count, 1);
    assert_eq!(stats.grounded_whiff_count, 1);
    assert_eq!(calculator.events().len(), 1);
}

#[test]
fn touch_cancels_active_whiff_candidate() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = WhiffCalculator::new();
    let ball = ball();

    calculator
        .update(
            &frame(1, 0.1),
            &ball,
            &PlayerFrameState {
                players: vec![player(1, -210.0, false)],
            },
            &TouchState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.2),
            &ball,
            &PlayerFrameState {
                players: vec![player(1, 460.0, false)],
            },
            &TouchState {
                touch_events: vec![TouchEvent {
                    touch_id: None,
                    time: 0.2,
                    frame: 2,
                    team_is_team_0: true,
                    player: Some(player_id.clone()),
                    player_position: None,
                    closest_approach_distance: Some(0.0),
                    dodge_contact: false,
                }],
                ..TouchState::default()
            },
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player_id).is_none());
    assert!(calculator.events().is_empty());
}

#[test]
fn opponent_touch_counts_as_beaten_to_ball_not_whiff() {
    let player_id = boxcars::RemoteId::Steam(1);
    let opponent_id = boxcars::RemoteId::Steam(2);
    let mut calculator = WhiffCalculator::new();
    let ball = ball();

    calculator
        .update(
            &frame(1, 0.1),
            &ball,
            &PlayerFrameState {
                players: vec![player(1, -210.0, false)],
            },
            &TouchState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.2),
            &ball,
            &PlayerFrameState {
                players: vec![player(1, -120.0, false)],
            },
            &TouchState {
                touch_events: vec![TouchEvent {
                    touch_id: None,
                    time: 0.2,
                    frame: 2,
                    team_is_team_0: false,
                    player: Some(opponent_id),
                    player_position: None,
                    closest_approach_distance: Some(0.0),
                    dodge_contact: false,
                }],
                ..TouchState::default()
            },
            &LivePlayState::active_play(),
        )
        .unwrap();

    let stats = calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.whiff_count, 0);
    assert_eq!(stats.beaten_to_ball_count, 1);
    assert_eq!(calculator.events().len(), 1);
    assert_eq!(calculator.events()[0].kind, WhiffEventKind::BeatenToBall);
}

#[test]
fn teammate_touch_cancels_active_whiff_candidate() {
    let player_id = boxcars::RemoteId::Steam(1);
    let teammate_id = boxcars::RemoteId::Steam(3);
    let mut calculator = WhiffCalculator::new();
    let ball = ball();

    calculator
        .update(
            &frame(1, 0.1),
            &ball,
            &PlayerFrameState {
                players: vec![player(1, -210.0, false)],
            },
            &TouchState::default(),
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.2),
            &ball,
            &PlayerFrameState {
                players: vec![player(1, -120.0, false)],
            },
            &TouchState {
                touch_events: vec![TouchEvent {
                    touch_id: None,
                    time: 0.2,
                    frame: 2,
                    team_is_team_0: true,
                    player: Some(teammate_id),
                    player_position: None,
                    closest_approach_distance: Some(0.0),
                    dodge_contact: false,
                }],
                ..TouchState::default()
            },
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player_id).is_none());
    assert!(calculator.events().is_empty());
}

#[test]
fn lateral_drive_by_is_not_a_whiff() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = WhiffCalculator::new();
    let ball = ball();
    let touch_state = TouchState::default();

    calculator
        .update(
            &frame(1, 0.1),
            &ball,
            &PlayerFrameState {
                players: vec![player_at(
                    1,
                    glam::Vec3::new(-70.0, -170.0, 17.0),
                    glam::Vec3::new(900.0, 0.0, 0.0),
                    false,
                )],
            },
            &touch_state,
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.2),
            &ball,
            &PlayerFrameState {
                players: vec![player_at(
                    1,
                    glam::Vec3::new(460.0, -170.0, 17.0),
                    glam::Vec3::new(900.0, 0.0, 0.0),
                    false,
                )],
            },
            &touch_state,
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player_id).is_none());
    assert!(calculator.events().is_empty());
}

#[test]
fn matching_ball_velocity_is_not_a_whiff_attempt() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = WhiffCalculator::new();
    let ball = moving_ball(glam::Vec3::new(650.0, 0.0, 0.0));
    let touch_state = TouchState::default();

    calculator
        .update(
            &frame(1, 0.1),
            &ball,
            &PlayerFrameState {
                players: vec![player_at(
                    1,
                    glam::Vec3::new(-180.0, 0.0, 17.0),
                    glam::Vec3::new(900.0, 0.0, 0.0),
                    false,
                )],
            },
            &touch_state,
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.2),
            &ball,
            &PlayerFrameState {
                players: vec![player_at(
                    1,
                    glam::Vec3::new(460.0, 0.0, 17.0),
                    glam::Vec3::new(900.0, 0.0, 0.0),
                    false,
                )],
            },
            &touch_state,
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player_id).is_none());
    assert!(calculator.events().is_empty());
}

#[test]
fn side_dodge_is_not_a_whiff_attempt() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = WhiffCalculator::new();
    let ball = ball();
    let touch_state = TouchState::default();

    calculator
        .update(
            &frame(1, 0.1),
            &ball,
            &PlayerFrameState {
                players: vec![player_at(
                    1,
                    glam::Vec3::new(-70.0, -190.0, 17.0),
                    glam::Vec3::new(900.0, 0.0, 0.0),
                    true,
                )],
            },
            &touch_state,
            &LivePlayState::active_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 0.2),
            &ball,
            &PlayerFrameState {
                players: vec![player_at(
                    1,
                    glam::Vec3::new(460.0, -190.0, 17.0),
                    glam::Vec3::new(900.0, 0.0, 0.0),
                    true,
                )],
            },
            &touch_state,
            &LivePlayState::active_play(),
        )
        .unwrap();

    assert!(calculator.player_stats().get(&player_id).is_none());
    assert!(calculator.events().is_empty());
}

fn whiff_event(aerial: bool, dodge_active: bool, closest_approach_distance: f32) -> WhiffEvent {
    WhiffEvent {
        kind: WhiffEventKind::Whiff,
        time: 1.0,
        frame: 10,
        resolved_time: 1.0,
        resolved_frame: 10,
        player: boxcars::RemoteId::Steam(1),
        player_position: None,
        is_team_0: true,
        closest_approach_distance,
        forward_alignment: 0.9,
        approach_speed: 900.0,
        dodge_active,
        aerial,
    }
}

#[test]
fn whiff_stats_records_events_as_labeled_counts() {
    let mut stats = WhiffStats::default();

    stats.record_whiff(&whiff_event(false, false, 120.0));
    stats.record_whiff(&whiff_event(true, true, 90.0));
    stats.record_whiff(&whiff_event(true, true, 110.0));

    assert_eq!(
        stats.whiff_count_with_labels(&[StatLabel::new("vertical_state", "aerial")]),
        2
    );
    assert_eq!(
        stats.whiff_count_with_labels(&[
            StatLabel::new("vertical_state", "aerial"),
            StatLabel::new("dodge_state", "dodge"),
        ]),
        2
    );
    assert_eq!(
        stats.whiff_count_with_labels(&[
            StatLabel::new("vertical_state", "grounded"),
            StatLabel::new("dodge_state", "no_dodge"),
        ]),
        1
    );
}

#[test]
fn whiff_stats_keeps_legacy_fields_synced_from_labeled_counts() {
    let mut stats = WhiffStats::default();

    stats.record_whiff(&whiff_event(false, false, 120.0));
    stats.record_whiff(&whiff_event(true, true, 90.0));

    assert_eq!(stats.whiff_count, 2);
    assert_eq!(stats.grounded_whiff_count, 1);
    assert_eq!(stats.aerial_whiff_count, 1);
    assert_eq!(stats.dodge_whiff_count, 1);
    assert_eq!(stats.best_closest_approach_distance, Some(90.0));
    assert_eq!(stats.average_closest_approach_distance(), 105.0);
}
