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

fn player_at_height(player_id: PlayerId, dodge_active: bool, height: f32) -> PlayerSample {
    PlayerSample {
        player_id,
        is_team_0: true,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body(glam::Vec3::new(0.0, 0.0, height))),
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

fn player(player_id: PlayerId, dodge_active: bool) -> PlayerSample {
    player_at_height(player_id, dodge_active, 100.0)
}

fn players(player_id: PlayerId, dodge_active: bool) -> PlayerFrameState {
    PlayerFrameState {
        players: vec![player(player_id, dodge_active)],
    }
}

fn grounded_players(player_id: PlayerId) -> PlayerFrameState {
    PlayerFrameState {
        players: vec![player_at_height(player_id, false, 17.0)],
    }
}

fn ball() -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(glam::Vec3::new(0.0, 0.0, 180.0)),
    })
}

fn frame_info(time: f32, frame_number: usize) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 0.0,
        seconds_remaining: None,
    }
}

fn reset_event(player: PlayerId) -> DodgeRefreshedEvent {
    reset_event_at(player, 1.0, 10, 1)
}

fn reset_event_at(
    player: PlayerId,
    time: f32,
    frame: usize,
    counter_value: i32,
) -> DodgeRefreshedEvent {
    DodgeRefreshedEvent {
        time,
        frame,
        player,
        is_team_0: true,
        player_position: None,
        counter_value,
    }
}

fn touch_event(player: PlayerId, time: f32, frame: usize) -> TouchEvent {
    TouchEvent {
        touch_id: None,
        time,
        frame,
        team_is_team_0: true,
        player: Some(player),
        player_position: None,
        closest_approach_distance: Some(0.0),
        contact_local_ball_position: None,
        contact_local_hitbox_point: None,
        contact_world_hitbox_point: None,
        dodge_contact: false,
    }
}

fn underside_touch_event(player: PlayerId, time: f32, frame: usize) -> TouchEvent {
    TouchEvent {
        player_position: Some(boxcars::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 500.0,
        }),
        contact_local_ball_position: Some([0.0, 0.0, -80.0]),
        ..touch_event(player, time, frame)
    }
}

fn raw_team_touch_event(time: f32, frame: usize) -> TouchEvent {
    TouchEvent {
        touch_id: None,
        time,
        frame,
        team_is_team_0: true,
        player: None,
        player_position: None,
        closest_approach_distance: None,
        contact_local_ball_position: None,
        contact_local_hitbox_point: None,
        contact_world_hitbox_point: None,
        dodge_contact: false,
    }
}

fn touch_state(touch_events: Vec<TouchEvent>) -> TouchState {
    TouchState {
        touch_events,
        ..TouchState::default()
    }
}

fn goal_event(time: f32, frame: usize) -> GoalEvent {
    GoalEvent {
        time,
        frame,
        scoring_team_is_team_0: true,
        player: None,
        player_position: None,
        team_zero_score: None,
        team_one_score: None,
    }
}

fn update_live(
    calculator: &mut DodgeResetCalculator,
    frame: &FrameInfo,
    players: &PlayerFrameState,
    events: &FrameEventsState,
    touch_state: &TouchState,
) {
    calculator
        .update(
            frame,
            &ball(),
            players,
            events,
            touch_state,
            &LivePlayState::active_play(),
        )
        .unwrap();
}

#[test]
fn on_ball_reset_alone_is_not_confirmed_flip_reset() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = DodgeResetCalculator::new();

    update_live(
        &mut calculator,
        &frame_info(1.0, 10),
        &players(player_id.clone(), false),
        &FrameEventsState {
            dodge_refreshed_events: vec![reset_event(player_id.clone())],
            ..FrameEventsState::default()
        },
        &TouchState::default(),
    );

    assert_eq!(calculator.player_stats()[&player_id].on_ball_count, 1);
    assert!(calculator.confirmed_flip_reset_events().is_empty());
    assert!(calculator.flip_reset_outcome_events().is_empty());
    let event = calculator.events().first().unwrap();
    assert!(event.on_ball);
    assert!(
        !event.used,
        "an unconfirmed on-ball reset must not be marked used"
    );
    assert!(event.outcome.is_none());
}

#[test]
fn touch_after_reset_requires_dodge_to_confirm_flip_reset() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = DodgeResetCalculator::new();

    update_live(
        &mut calculator,
        &frame_info(1.0, 10),
        &players(player_id.clone(), false),
        &FrameEventsState {
            dodge_refreshed_events: vec![reset_event(player_id.clone())],
            ..FrameEventsState::default()
        },
        &TouchState::default(),
    );
    update_live(
        &mut calculator,
        &frame_info(1.2, 12),
        &players(player_id.clone(), false),
        &FrameEventsState {
            touch_events: vec![raw_team_touch_event(1.2, 12)],
            ..FrameEventsState::default()
        },
        &touch_state(vec![touch_event(player_id.clone(), 1.2, 12)]),
    );

    assert!(calculator.confirmed_flip_reset_events().is_empty());
}

#[test]
fn underside_touch_can_seed_flip_reset_when_counter_is_absent() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = DodgeResetCalculator::new();

    update_live(
        &mut calculator,
        &frame_info(1.0, 10),
        &players(player_id.clone(), false),
        &FrameEventsState::default(),
        &touch_state(vec![underside_touch_event(player_id.clone(), 1.0, 10)]),
    );
    update_live(
        &mut calculator,
        &frame_info(1.5, 15),
        &players(player_id.clone(), true),
        &FrameEventsState::default(),
        &TouchState::default(),
    );
    update_live(
        &mut calculator,
        &frame_info(1.6, 16),
        &players(player_id.clone(), true),
        &FrameEventsState::default(),
        &touch_state(vec![touch_event(player_id.clone(), 1.6, 16)]),
    );

    let event = calculator
        .confirmed_flip_reset_events()
        .first()
        .expect("underside touch should seed a pending flip reset");
    assert_eq!(event.player, player_id);
    assert_eq!(event.reset_frame, 10);
    assert_eq!(event.frame, 16);

    let reset = calculator
        .events()
        .iter()
        .find(|event| event.frame == 10)
        .expect("fallback on-ball reset event should be emitted");
    assert!(reset.on_ball);
    assert!(reset.used);
    assert_eq!(reset.outcome, Some(FlipResetOutcome::Used));
}

#[test]
fn raw_replay_touch_after_reset_does_not_confirm_without_attributed_touch_state() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = DodgeResetCalculator::new();

    update_live(
        &mut calculator,
        &frame_info(1.0, 10),
        &players(player_id.clone(), false),
        &FrameEventsState {
            dodge_refreshed_events: vec![reset_event(player_id.clone())],
            ..FrameEventsState::default()
        },
        &TouchState::default(),
    );
    update_live(
        &mut calculator,
        &frame_info(1.1, 11),
        &players(player_id, true),
        &FrameEventsState::default(),
        &TouchState::default(),
    );
    update_live(
        &mut calculator,
        &frame_info(1.3, 13),
        &players(boxcars::RemoteId::Steam(1), true),
        &FrameEventsState {
            touch_events: vec![raw_team_touch_event(1.3, 13)],
            ..FrameEventsState::default()
        },
        &TouchState::default(),
    );

    assert!(calculator.confirmed_flip_reset_events().is_empty());
}

#[test]
fn dodge_touch_after_on_ball_reset_confirms_flip_reset() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = DodgeResetCalculator::new();

    update_live(
        &mut calculator,
        &frame_info(1.0, 10),
        &players(player_id.clone(), false),
        &FrameEventsState {
            dodge_refreshed_events: vec![reset_event(player_id.clone())],
            ..FrameEventsState::default()
        },
        &TouchState::default(),
    );
    update_live(
        &mut calculator,
        &frame_info(1.1, 11),
        &players(player_id.clone(), true),
        &FrameEventsState::default(),
        &TouchState::default(),
    );
    update_live(
        &mut calculator,
        &frame_info(1.3, 13),
        &players(player_id.clone(), true),
        &FrameEventsState {
            touch_events: vec![raw_team_touch_event(1.3, 13)],
            ..FrameEventsState::default()
        },
        &touch_state(vec![touch_event(player_id.clone(), 1.3, 13)]),
    );

    let event = calculator.confirmed_flip_reset_events().first().unwrap();
    assert_eq!(event.player, player_id);
    assert_eq!(event.reset_frame, 10);
    assert_eq!(event.frame, 13);
    assert!((event.time_since_reset - 0.3).abs() < 1e-5);

    // The confirming touch should retroactively mark the originating on-ball
    // dodge reset event as a used flip reset with its reset-to-use latency.
    let reset = calculator
        .events()
        .iter()
        .find(|event| event.frame == 10)
        .expect("on-ball dodge reset event should be emitted");
    assert!(reset.on_ball);
    assert!(
        reset.used,
        "confirmed flip reset should set `used` on the reset"
    );
    assert_eq!(reset.outcome, Some(FlipResetOutcome::Used));
    assert!((reset.time_to_use.unwrap() - 0.3).abs() < 1e-5);

    let outcome = calculator.flip_reset_outcome_events().first().unwrap();
    assert_eq!(outcome.player, player_id);
    assert_eq!(outcome.outcome, FlipResetOutcome::Used);
    assert_eq!(outcome.reset_frame, 10);
    assert_eq!(outcome.frame, 13);
    assert!((outcome.time_to_use.unwrap() - 0.3).abs() < 1e-5);

    let stats = &calculator.player_stats()[&player_id];
    assert_eq!(stats.flip_reset_used_count, 1);
    assert_eq!(stats.flip_reset_unused_count, 0);
    assert!((stats.flip_reset_total_time_to_use - 0.3).abs() < 1e-5);
    assert!((stats.flip_reset_min_time_to_use.unwrap() - 0.3).abs() < 1e-5);
    assert!((stats.flip_reset_mean_time_to_use() - 0.3).abs() < 1e-5);
}

#[test]
fn dodge_byte_lagging_conversion_touch_still_confirms_flip_reset() {
    // Mirrors a fast flip-into-ball finish (e.g. the goal-9 case): the
    // conversion touch is sampled a frame before the dodge component's active
    // byte replicates. The reset must still confirm once the dodge appears
    // within `FLIP_RESET_DODGE_TOUCH_LAG_TOLERANCE_SECONDS`, even though the
    // touch alone is under the minimum reset-to-touch delay.
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = DodgeResetCalculator::new();

    update_live(
        &mut calculator,
        &frame_info(1.0, 10),
        &players(player_id.clone(), false),
        &FrameEventsState {
            dodge_refreshed_events: vec![reset_event(player_id.clone())],
            ..FrameEventsState::default()
        },
        &TouchState::default(),
    );
    // Conversion touch lands while the dodge byte is still off.
    update_live(
        &mut calculator,
        &frame_info(1.1, 11),
        &players(player_id.clone(), false),
        &FrameEventsState::default(),
        &touch_state(vec![touch_event(player_id.clone(), 1.1, 11)]),
    );
    // Dodge byte flips on a frame later, within the lag tolerance.
    update_live(
        &mut calculator,
        &frame_info(1.15, 12),
        &players(player_id.clone(), true),
        &FrameEventsState::default(),
        &TouchState::default(),
    );

    let event = calculator
        .confirmed_flip_reset_events()
        .first()
        .expect("lagging dodge byte should still confirm the flip reset");
    assert_eq!(event.player, player_id);
    assert_eq!(event.reset_frame, 10);
    assert_eq!(event.frame, 11);
    // Latency stays measured to the conversion touch.
    assert!((event.time_since_reset - 0.1).abs() < 1e-5);

    let reset = calculator
        .events()
        .iter()
        .find(|event| event.frame == 10)
        .expect("on-ball dodge reset event should be emitted");
    assert!(reset.used);
    assert_eq!(reset.outcome, Some(FlipResetOutcome::Used));
}

#[test]
fn dodge_byte_after_lag_window_does_not_confirm_flip_reset() {
    // A touch and a much later dodge are not a conversion: the dodge appears
    // well past the lag tolerance with no fresh dodge-active touch, so the reset
    // stays unconfirmed.
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = DodgeResetCalculator::new();

    update_live(
        &mut calculator,
        &frame_info(1.0, 10),
        &players(player_id.clone(), false),
        &FrameEventsState {
            dodge_refreshed_events: vec![reset_event(player_id.clone())],
            ..FrameEventsState::default()
        },
        &TouchState::default(),
    );
    update_live(
        &mut calculator,
        &frame_info(1.1, 11),
        &players(player_id.clone(), false),
        &FrameEventsState::default(),
        &touch_state(vec![touch_event(player_id.clone(), 1.1, 11)]),
    );
    // Dodge byte only appears 0.3s after the touch, beyond the lag tolerance,
    // and no new touch lands while dodging.
    update_live(
        &mut calculator,
        &frame_info(1.4, 14),
        &players(player_id, true),
        &FrameEventsState::default(),
        &TouchState::default(),
    );

    assert!(calculator.confirmed_flip_reset_events().is_empty());
}

#[test]
fn landing_resolves_pending_flip_reset_as_unused() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = DodgeResetCalculator::new();

    update_live(
        &mut calculator,
        &frame_info(1.0, 10),
        &players(player_id.clone(), false),
        &FrameEventsState {
            dodge_refreshed_events: vec![reset_event(player_id.clone())],
            ..FrameEventsState::default()
        },
        &TouchState::default(),
    );
    update_live(
        &mut calculator,
        &frame_info(2.0, 20),
        &grounded_players(player_id.clone()),
        &FrameEventsState::default(),
        &TouchState::default(),
    );

    let outcome = calculator.flip_reset_outcome_events().first().unwrap();
    assert_eq!(outcome.outcome, FlipResetOutcome::Landed);
    assert_eq!(outcome.reset_frame, 10);
    assert_eq!(outcome.frame, 20);
    assert!(outcome.time_to_use.is_none());

    let reset = calculator.events().first().unwrap();
    assert_eq!(reset.outcome, Some(FlipResetOutcome::Landed));
    assert!(!reset.used);

    let stats = &calculator.player_stats()[&player_id];
    assert_eq!(stats.flip_reset_used_count, 0);
    assert_eq!(stats.flip_reset_unused_count, 1);
    assert!(stats.flip_reset_min_time_to_use.is_none());
}

#[test]
fn goal_resolves_pending_flip_reset_as_unused() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = DodgeResetCalculator::new();

    update_live(
        &mut calculator,
        &frame_info(1.0, 10),
        &players(player_id.clone(), false),
        &FrameEventsState {
            dodge_refreshed_events: vec![reset_event(player_id.clone())],
            ..FrameEventsState::default()
        },
        &TouchState::default(),
    );
    update_live(
        &mut calculator,
        &frame_info(1.5, 15),
        &players(player_id.clone(), false),
        &FrameEventsState {
            goal_events: vec![goal_event(1.5, 15)],
            ..FrameEventsState::default()
        },
        &TouchState::default(),
    );

    let outcome = calculator.flip_reset_outcome_events().first().unwrap();
    assert_eq!(outcome.outcome, FlipResetOutcome::GoalScored);
    assert_eq!(outcome.frame, 15);
    assert!(outcome.time_to_use.is_none());
    assert_eq!(
        calculator.player_stats()[&player_id].flip_reset_unused_count,
        1
    );
}

#[test]
fn live_play_ending_resolves_pending_flip_reset_as_unused() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = DodgeResetCalculator::new();

    update_live(
        &mut calculator,
        &frame_info(1.0, 10),
        &players(player_id.clone(), false),
        &FrameEventsState {
            dodge_refreshed_events: vec![reset_event(player_id.clone())],
            ..FrameEventsState::default()
        },
        &TouchState::default(),
    );
    calculator
        .update(
            &frame_info(1.5, 15),
            &ball(),
            &players(player_id.clone(), false),
            &FrameEventsState::default(),
            &TouchState::default(),
            &LivePlayState::new(GameplayPhase::PostGoal),
        )
        .unwrap();

    let outcome = calculator.flip_reset_outcome_events().first().unwrap();
    assert_eq!(outcome.outcome, FlipResetOutcome::PlayEnded);
    assert_eq!(outcome.frame, 15);
    assert_eq!(
        calculator.player_stats()[&player_id].flip_reset_unused_count,
        1
    );
}

#[test]
fn superseding_reset_marks_previous_reset_unused() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = DodgeResetCalculator::new();

    update_live(
        &mut calculator,
        &frame_info(1.0, 10),
        &players(player_id.clone(), false),
        &FrameEventsState {
            dodge_refreshed_events: vec![reset_event_at(player_id.clone(), 1.0, 10, 1)],
            ..FrameEventsState::default()
        },
        &TouchState::default(),
    );
    update_live(
        &mut calculator,
        &frame_info(1.4, 14),
        &players(player_id.clone(), false),
        &FrameEventsState {
            dodge_refreshed_events: vec![reset_event_at(player_id.clone(), 1.4, 14, 2)],
            ..FrameEventsState::default()
        },
        &TouchState::default(),
    );
    update_live(
        &mut calculator,
        &frame_info(1.5, 15),
        &players(player_id.clone(), true),
        &FrameEventsState::default(),
        &TouchState::default(),
    );
    update_live(
        &mut calculator,
        &frame_info(1.7, 17),
        &players(player_id.clone(), true),
        &FrameEventsState {
            touch_events: vec![raw_team_touch_event(1.7, 17)],
            ..FrameEventsState::default()
        },
        &touch_state(vec![touch_event(player_id.clone(), 1.7, 17)]),
    );

    let outcomes = calculator.flip_reset_outcome_events();
    assert_eq!(outcomes.len(), 2);
    assert_eq!(outcomes[0].outcome, FlipResetOutcome::Superseded);
    assert_eq!(outcomes[0].reset_frame, 10);
    assert!(outcomes[0].time_to_use.is_none());
    assert_eq!(outcomes[1].outcome, FlipResetOutcome::Used);
    assert_eq!(outcomes[1].reset_frame, 14);
    // Latency is measured from the superseding (most recent) reset.
    assert!((outcomes[1].time_to_use.unwrap() - 0.3).abs() < 1e-5);

    let first_reset = calculator
        .events()
        .iter()
        .find(|event| event.frame == 10)
        .unwrap();
    assert_eq!(first_reset.outcome, Some(FlipResetOutcome::Superseded));
    assert!(!first_reset.used);

    let stats = &calculator.player_stats()[&player_id];
    assert_eq!(stats.flip_reset_used_count, 1);
    assert_eq!(stats.flip_reset_unused_count, 1);
    assert!((stats.flip_reset_min_time_to_use.unwrap() - 0.3).abs() < 1e-5);
}

#[test]
fn finish_resolves_pending_flip_reset_as_replay_ended() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut calculator = DodgeResetCalculator::new();

    update_live(
        &mut calculator,
        &frame_info(1.0, 10),
        &players(player_id.clone(), false),
        &FrameEventsState {
            dodge_refreshed_events: vec![reset_event(player_id.clone())],
            ..FrameEventsState::default()
        },
        &TouchState::default(),
    );
    calculator.finish();

    let outcome = calculator.flip_reset_outcome_events().first().unwrap();
    assert_eq!(outcome.outcome, FlipResetOutcome::ReplayEnded);
    assert_eq!(outcome.reset_frame, 10);
    assert!(outcome.time_to_use.is_none());
    assert_eq!(
        calculator.player_stats()[&player_id].flip_reset_unused_count,
        1
    );
}
