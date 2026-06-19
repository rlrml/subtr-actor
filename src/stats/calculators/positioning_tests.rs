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
        linear_velocity: Some(glam_to_vec(&glam::Vec3::ZERO)),
        angular_velocity: Some(glam_to_vec(&glam::Vec3::ZERO)),
    }
}

fn ball(position: glam::Vec3) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(position),
    })
}

fn player(id: u64, is_team_0: bool, position: glam::Vec3) -> PlayerSample {
    PlayerSample {
        player_id: boxcars::RemoteId::Steam(id),
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

fn frame(frame_number: usize, time: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 0.1,
        seconds_remaining: None,
    }
}

fn active_gameplay() -> GameplayState {
    GameplayState {
        ball_has_been_hit: Some(true),
        current_in_game_team_player_counts: [2, 0],
        ..Default::default()
    }
}

fn active_gameplay_with_teams(team_zero: usize, team_one: usize) -> GameplayState {
    GameplayState {
        ball_has_been_hit: Some(true),
        current_in_game_team_player_counts: [team_zero, team_one],
        ..Default::default()
    }
}

fn ball_depth_segments(margin: f32, start: f32, end: f32) -> Vec<(BallDepthState, f32)> {
    scalar_state_segments(
        start,
        end,
        &[-margin, margin],
        &[
            BallDepthState::BehindBall,
            BallDepthState::LevelWithBall,
            BallDepthState::AheadOfBall,
        ],
    )
}

#[test]
fn ball_depth_segments_treat_near_ball_band_as_level() {
    let segments = ball_depth_segments(150.0, -100.0, 100.0);
    assert_eq!(segments, vec![(BallDepthState::LevelWithBall, 1.0)]);
}

#[test]
fn ball_depth_segments_split_crossing_time_across_all_three_buckets() {
    let segments = ball_depth_segments(150.0, -300.0, 300.0);
    assert_eq!(segments.len(), 3);
    assert_eq!(segments[0].0, BallDepthState::BehindBall);
    assert!((segments[0].1 - 0.25).abs() < 1e-6);
    assert_eq!(segments[1].0, BallDepthState::LevelWithBall);
    assert!((segments[1].1 - 0.5).abs() < 1e-6);
    assert_eq!(segments[2].0, BallDepthState::AheadOfBall);
    assert!((segments[2].1 - 0.25).abs() < 1e-6);
}

#[test]
fn ball_depth_segments_count_boundary_point_as_in_front_not_level() {
    let segments = ball_depth_segments(150.0, 150.0, 150.0);
    assert_eq!(segments, vec![(BallDepthState::AheadOfBall, 1.0)]);
}

#[test]
fn positioning_facets_emit_coalesced_spans() {
    let mut calculator = PositioningCalculator::new();
    let gameplay = active_gameplay();
    let ball = ball(glam::Vec3::ZERO);
    let players = PlayerFrameState {
        players: vec![
            player(1, true, glam::vec3(0.0, -1000.0, 0.0)),
            player(2, true, glam::vec3(0.0, 1000.0, 0.0)),
        ],
    };

    for frame_number in 0..3 {
        calculator
            .update(
                &frame(frame_number, frame_number as f32 * 0.1),
                &gameplay,
                &ball,
                &players,
                &FrameEventsState::default(),
                &LivePlayState::active_play(),
                None,
            )
            .expect("positioning update should succeed");
    }
    calculator.flush_pending_events();

    let activity_events = calculator.activity_events();
    assert_eq!(activity_events.len(), 2);
    for event in &activity_events {
        assert_eq!(event.state, ActivityState::Tracked);
        assert_eq!(event.frame, 0);
        assert_eq!(event.end_frame, 2);
        assert!((event.duration - 0.3).abs() < 1e-6);
    }

    let depth_role_events = calculator.depth_role_events();
    let back_event = depth_role_events
        .iter()
        .find(|event| event.player == boxcars::RemoteId::Steam(1))
        .expect("back player depth role span should be emitted");
    assert_eq!(back_event.state, DepthRoleState::MostBack);
    let forward_event = depth_role_events
        .iter()
        .find(|event| event.player == boxcars::RemoteId::Steam(2))
        .expect("forward player depth role span should be emitted");
    assert_eq!(forward_event.state, DepthRoleState::MostForward);
}

#[test]
fn closest_to_ball_requires_stable_challenger_before_switching() {
    let mut calculator = PositioningCalculator::with_config(PositioningCalculatorConfig {
        closest_to_ball_switch_margin: 0.0,
        closest_to_ball_switch_min_seconds: 0.3,
        ..PositioningCalculatorConfig::default()
    });
    let gameplay = active_gameplay();
    let ball = ball(glam::Vec3::ZERO);

    let closest_player_by_frame = [
        (glam::vec3(100.0, 0.0, 0.0), glam::vec3(120.0, 0.0, 0.0), 1),
        (glam::vec3(100.0, 0.0, 0.0), glam::vec3(50.0, 0.0, 0.0), 1),
        (glam::vec3(100.0, 0.0, 0.0), glam::vec3(50.0, 0.0, 0.0), 1),
        (glam::vec3(100.0, 0.0, 0.0), glam::vec3(50.0, 0.0, 0.0), 2),
    ];

    for (frame_number, (player_one_position, player_two_position, expected_closest)) in
        closest_player_by_frame.iter().enumerate()
    {
        let players = PlayerFrameState {
            players: vec![
                player(1, true, *player_one_position),
                player(2, true, *player_two_position),
            ],
        };
        calculator
            .update(
                &frame(frame_number, frame_number as f32 * 0.1),
                &gameplay,
                &ball,
                &players,
                &FrameEventsState::default(),
                &LivePlayState::active_play(),
                None,
            )
            .expect("positioning update should succeed");

        let events = calculator.ball_proximity_events();
        let closest = events
            .iter()
            .find(|event| event.end_frame == frame_number && event.state.closest_to_ball_team)
            .expect("team closest span should cover the current frame");
        assert_eq!(closest.player, boxcars::RemoteId::Steam(*expected_closest));
        let absolute_closest = events
            .iter()
            .find(|event| event.end_frame == frame_number && event.state.closest_to_ball_absolute)
            .expect("absolute closest span should cover the current frame");
        assert_eq!(
            absolute_closest.player,
            boxcars::RemoteId::Steam(*expected_closest)
        );
    }
}

#[test]
fn shadow_defense_detects_retreating_goal_side_defender() {
    let mut calculator = PositioningCalculator::new();
    let gameplay = active_gameplay_with_teams(1, 1);
    let defender_id = boxcars::RemoteId::Steam(1);
    let attacker_id = boxcars::RemoteId::Steam(2);

    let samples = [
        (-1900.0, -700.0, -400.0),
        (-2000.0, -800.0, -500.0),
        (-2100.0, -900.0, -600.0),
    ];
    for (frame_number, (defender_y, ball_y, attacker_y)) in samples.iter().enumerate() {
        let players = PlayerFrameState {
            players: vec![
                player(1, true, glam::vec3(0.0, *defender_y, 0.0)),
                player(2, false, glam::vec3(0.0, *attacker_y, 0.0)),
            ],
        };
        calculator
            .update(
                &frame(frame_number, frame_number as f32 * 0.1),
                &gameplay,
                &ball(glam::vec3(0.0, *ball_y, 0.0)),
                &players,
                &FrameEventsState::default(),
                &LivePlayState::active_play(),
                Some(&attacker_id),
            )
            .expect("positioning update should succeed");
    }
    calculator.flush_pending_events();

    let events = calculator.shadow_defense_events();
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_eq!(event.player, defender_id);
    assert_eq!(event.state, ShadowDefenseState::Shadowing);
    assert_eq!(event.frame, 1);
    assert_eq!(event.end_frame, 2);
    assert!((event.duration - 0.2).abs() < 1e-6);
}

#[test]
fn shadow_defense_requires_opponent_possession() {
    let mut calculator = PositioningCalculator::new();
    let gameplay = active_gameplay_with_teams(1, 1);

    let samples = [(-1900.0, -700.0), (-2000.0, -800.0)];
    for (frame_number, (defender_y, ball_y)) in samples.iter().enumerate() {
        let players = PlayerFrameState {
            players: vec![
                player(1, true, glam::vec3(0.0, *defender_y, 0.0)),
                player(2, false, glam::vec3(0.0, -500.0, 0.0)),
            ],
        };
        calculator
            .update(
                &frame(frame_number, frame_number as f32 * 0.1),
                &gameplay,
                &ball(glam::vec3(0.0, *ball_y, 0.0)),
                &players,
                &FrameEventsState::default(),
                &LivePlayState::active_play(),
                None,
            )
            .expect("positioning update should succeed");
    }
    calculator.flush_pending_events();

    assert!(calculator.shadow_defense_events().is_empty());
}

#[test]
fn shadow_defense_excludes_immediate_close_challenge() {
    let mut calculator = PositioningCalculator::new();
    let gameplay = active_gameplay_with_teams(1, 1);
    let attacker_id = boxcars::RemoteId::Steam(2);

    let samples = [(-850.0, -700.0, -400.0), (-950.0, -800.0, -500.0)];
    for (frame_number, (defender_y, ball_y, attacker_y)) in samples.iter().enumerate() {
        let players = PlayerFrameState {
            players: vec![
                player(1, true, glam::vec3(0.0, *defender_y, 0.0)),
                player(2, false, glam::vec3(0.0, *attacker_y, 0.0)),
            ],
        };
        calculator
            .update(
                &frame(frame_number, frame_number as f32 * 0.1),
                &gameplay,
                &ball(glam::vec3(0.0, *ball_y, 0.0)),
                &players,
                &FrameEventsState::default(),
                &LivePlayState::active_play(),
                Some(&attacker_id),
            )
            .expect("positioning update should succeed");
    }
    calculator.flush_pending_events();

    assert!(calculator.shadow_defense_events().is_empty());
}
