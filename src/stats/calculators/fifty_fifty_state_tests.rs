use super::*;

fn frame(frame_number: usize, time: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 0.1,
        seconds_remaining: None,
    }
}

fn touch(frame: usize, time: f32, player: PlayerId, team_is_team_0: bool) -> TouchEvent {
    TouchEvent {
        touch_id: None,
        time,
        frame,
        team_is_team_0,
        player: Some(player),
        player_position: None,
        closest_approach_distance: Some(0.0),
        contact_local_ball_position: None,
        contact_local_hitbox_point: None,
        contact_world_hitbox_point: None,
        dodge_contact: false,
    }
}

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
        dodge_torque: None,
        powerslide_active: false,
        match_goals: None,
        match_assists: None,
        match_saves: None,
        match_shots: None,
        match_score: None,
    }
}

fn two_player_state(team_zero_player: &PlayerId, team_one_player: &PlayerId) -> PlayerFrameState {
    PlayerFrameState {
        players: vec![
            player(
                team_zero_player.clone(),
                true,
                glam::Vec3::new(0.0, -100.0, 0.0),
            ),
            player(
                team_one_player.clone(),
                false,
                glam::Vec3::new(0.0, 100.0, 0.0),
            ),
        ],
    }
}

fn active_event(team_zero_player: PlayerId) -> ActiveFiftyFifty {
    ActiveFiftyFifty {
        start_time: 1.0,
        start_frame: 100,
        last_touch_time: 1.0,
        last_touch_frame: 100,
        is_kickoff: false,
        team_zero_player: Some(team_zero_player),
        team_one_player: None,
        team_zero_touch_time: Some(1.0),
        team_zero_touch_frame: Some(100),
        team_zero_dodge_contact: false,
        team_one_touch_time: None,
        team_one_touch_frame: None,
        team_one_dodge_contact: false,
        team_zero_position: [0.0, 0.0, 0.0],
        team_one_position: [100.0, 0.0, 0.0],
        midpoint: [50.0, 0.0, 0.0],
        plane_normal: [1.0, 0.0, 0.0],
    }
}

#[test]
fn sequential_opposing_touches_start_fifty_fifty_within_short_window() {
    let team_zero_player = PlayerId::Steam(1);
    let team_one_player = PlayerId::Steam(2);
    let players = two_player_state(&team_zero_player, &team_one_player);
    let mut calculator = FiftyFiftyStateCalculator::new();

    let first_state = calculator.update(
        &frame(100, 1.0),
        &GameplayState::default(),
        &BallFrameState::default(),
        &players,
        &TouchState {
            touch_events: vec![touch(100, 1.0, team_zero_player.clone(), true)],
            ..TouchState::default()
        },
        &PossessionState::default(),
        &LivePlayState::active_play(),
    );
    assert!(first_state.active_event.is_none());

    let second_state = calculator.update(
        &frame(103, 1.08),
        &GameplayState::default(),
        &BallFrameState::default(),
        &players,
        &TouchState {
            touch_events: vec![touch(103, 1.08, team_one_player.clone(), false)],
            ..TouchState::default()
        },
        &PossessionState::default(),
        &LivePlayState::active_play(),
    );

    let active = second_state
        .active_event
        .expect("expected sequential opposing touches to start a 50/50");
    assert_eq!(active.start_frame, 100);
    assert_eq!(active.start_time, 1.0);
    assert_eq!(active.team_zero_player, Some(team_zero_player));
    assert_eq!(active.team_one_player, Some(team_one_player));
    assert_eq!(active.team_zero_touch_frame, Some(100));
    assert_eq!(active.team_one_touch_frame, Some(103));
}

#[test]
fn sequential_kickoff_touches_keep_kickoff_phase_from_first_touch() {
    let team_zero_player = PlayerId::Steam(1);
    let team_one_player = PlayerId::Steam(2);
    let players = two_player_state(&team_zero_player, &team_one_player);
    let mut calculator = FiftyFiftyStateCalculator::new();
    let kickoff_gameplay = GameplayState {
        ball_has_been_hit: Some(false),
        ..GameplayState::default()
    };

    calculator.update(
        &frame(100, 1.0),
        &kickoff_gameplay,
        &BallFrameState::default(),
        &players,
        &TouchState {
            touch_events: vec![touch(100, 1.0, team_zero_player, true)],
            ..TouchState::default()
        },
        &PossessionState::default(),
        &LivePlayState {
            gameplay_phase: GameplayPhase::KickoffWaitingForTouch,
            is_live_play: false,
        },
    );
    let state = calculator.update(
        &frame(103, 1.08),
        &GameplayState::default(),
        &BallFrameState::default(),
        &players,
        &TouchState {
            touch_events: vec![touch(103, 1.08, team_one_player, false)],
            ..TouchState::default()
        },
        &PossessionState::default(),
        &LivePlayState::active_play(),
    );

    assert!(
        state
            .active_event
            .expect("expected delayed kickoff contact to start a 50/50")
            .is_kickoff
    );
}

#[test]
fn continuation_touch_updates_last_touch_from_latest_touch_event_not_sample_frame() {
    let player = PlayerId::Steam(1);
    let mut calculator = FiftyFiftyStateCalculator {
        active_event: Some(active_event(player.clone())),
        last_resolved_event: None,
        pending_initial_touch: None,
        kickoff_touch_window_open: false,
    };

    let state = calculator.update(
        &frame(110, 1.1),
        &GameplayState::default(),
        &BallFrameState::default(),
        &PlayerFrameState::default(),
        &TouchState {
            touch_events: vec![
                touch(105, 1.05, player.clone(), true),
                touch(102, 1.02, player, true),
            ],
            ..TouchState::default()
        },
        &PossessionState::default(),
        &LivePlayState::active_play(),
    );

    let active = state
        .active_event
        .expect("expected the fifty-fifty to remain active");
    assert_eq!(active.last_touch_time, 1.05);
    assert_eq!(active.last_touch_frame, 105);
}
