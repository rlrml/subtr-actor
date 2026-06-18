use super::*;

fn rigid_body(position: glam::Vec3) -> boxcars::RigidBody {
    rigid_body_with_velocity(position, glam::Vec3::ZERO)
}

fn rigid_body_with_velocity(position: glam::Vec3, velocity: glam::Vec3) -> boxcars::RigidBody {
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

fn ball(z: f32) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(glam::Vec3::new(0.0, 0.0, z)),
    })
}

fn ball_with_state(position: glam::Vec3, velocity: glam::Vec3) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body_with_velocity(position, velocity),
    })
}

fn frame(frame_number: usize, time: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 0.1,
        seconds_remaining: None,
    }
}

fn player(player_id: PlayerId, is_team_0: bool, goals: i32) -> PlayerSample {
    PlayerSample {
        player_id,
        is_team_0,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body(glam::Vec3::new(0.0, 0.0, 17.0))),
        boost_amount: Some(33.0),
        last_boost_amount: None,
        boost_active: false,
        dodge_active: false,
        powerslide_active: false,
        match_goals: Some(goals),
        match_assists: Some(0),
        match_saves: Some(0),
        match_shots: Some(0),
        match_score: Some(0),
    }
}

fn goal_event(time: f32, frame: usize, scorer: PlayerId) -> GoalEvent {
    GoalEvent {
        time,
        frame,
        scoring_team_is_team_0: true,
        player: Some(scorer),
        player_position: None,
        team_zero_score: Some(1),
        team_one_score: Some(0),
    }
}

fn unattributed_goal_event(time: f32, frame: usize) -> GoalEvent {
    GoalEvent {
        time,
        frame,
        scoring_team_is_team_0: true,
        player: None,
        player_position: None,
        team_zero_score: Some(1),
        team_one_score: Some(0),
    }
}

fn buildup_sample(time: f32, ball_y: f32) -> GoalBuildupSample {
    GoalBuildupSample {
        time,
        dt: 1.0,
        ball_y,
    }
}

fn shot_pressure(time: f32, is_team_0: bool) -> GoalBuildupPressureEvent {
    GoalBuildupPressureEvent { time, is_team_0 }
}

fn update(
    calculator: &mut MatchStatsCalculator,
    frame: FrameInfo,
    ball: BallFrameState,
    player_goals: i32,
    goal_events: Vec<GoalEvent>,
) {
    let scorer = PlayerId::Steam(1);
    calculator
        .update_parts(
            &frame,
            &GameplayState {
                ball_has_been_hit: Some(true),
                team_zero_score: Some(player_goals),
                team_one_score: Some(0),
                ..GameplayState::default()
            },
            &ball,
            &PlayerFrameState {
                players: vec![player(scorer, true, player_goals)],
            },
            &FrameEventsState {
                goal_events,
                ..FrameEventsState::default()
            },
            &LivePlayState {
                gameplay_phase: GameplayPhase::ActivePlay,
                is_live_play: true,
            },
            &TouchState::default(),
        )
        .unwrap();
}

#[test]
fn fills_missing_goal_context_scorer_from_goal_delta() {
    let scorer = PlayerId::Steam(1);
    let mut calculator = MatchStatsCalculator::new();

    update(
        &mut calculator,
        frame(0, 0.0),
        ball(BALL_GROUND_CONTACT_MAX_Z),
        0,
        Vec::new(),
    );
    update(
        &mut calculator,
        frame(1, 1.0),
        ball(BALL_GROUND_CONTACT_MAX_Z + 200.0),
        0,
        Vec::new(),
    );
    update(
        &mut calculator,
        frame(2, 2.5),
        ball(BALL_GROUND_CONTACT_MAX_Z + 300.0),
        1,
        vec![unattributed_goal_event(2.5, 2)],
    );

    assert_eq!(
        calculator.goal_context_events()[0].scorer,
        Some(scorer.clone())
    );
    let scorer_stats = calculator.player_stats().get(&scorer).unwrap();
    assert_eq!(
        scorer_stats
            .scoring_context
            .goal_ball_air_time
            .goal_ball_air_time_sample_count,
        1
    );
    assert_eq!(scorer_stats.average_goal_ball_air_time(), 2.5);
}

#[test]
fn goal_context_links_scorer_touch_with_ball_location_and_speeds() {
    let scorer = PlayerId::Steam(1);
    let mut calculator = MatchStatsCalculator::new();
    let touch_ball_position = glam::Vec3::new(100.0, 200.0, 300.0);

    calculator
        .update_parts(
            &frame(10, 1.0),
            &GameplayState {
                ball_has_been_hit: Some(true),
                team_zero_score: Some(0),
                team_one_score: Some(0),
                ..GameplayState::default()
            },
            &ball_with_state(touch_ball_position, glam::Vec3::new(1200.0, 0.0, 0.0)),
            &PlayerFrameState {
                players: vec![player(scorer.clone(), true, 0)],
            },
            &FrameEventsState::default(),
            &LivePlayState {
                gameplay_phase: GameplayPhase::ActivePlay,
                is_live_play: true,
            },
            &TouchState {
                touch_events: vec![TouchEvent {
                    touch_id: None,
                    time: 1.0,
                    frame: 10,
                    team_is_team_0: true,
                    player: Some(scorer.clone()),
                    player_position: None,
                    closest_approach_distance: Some(0.0),
                    contact_local_ball_position: None,
                    contact_local_hitbox_point: None,
                    contact_world_hitbox_point: None,
                    dodge_contact: false,
                }],
                ..TouchState::default()
            },
        )
        .unwrap();

    calculator
        .update_parts(
            &frame(20, 2.0),
            &GameplayState {
                ball_has_been_hit: Some(true),
                team_zero_score: Some(1),
                team_one_score: Some(0),
                ..GameplayState::default()
            },
            &ball_with_state(
                glam::Vec3::new(0.0, 5200.0, 100.0),
                glam::Vec3::new(0.0, 1600.0, 0.0),
            ),
            &PlayerFrameState {
                players: vec![player(scorer.clone(), true, 1)],
            },
            &FrameEventsState {
                goal_events: vec![goal_event(2.0, 20, scorer.clone())],
                ..FrameEventsState::default()
            },
            &LivePlayState {
                gameplay_phase: GameplayPhase::ActivePlay,
                is_live_play: true,
            },
            &TouchState::default(),
        )
        .unwrap();

    let goal_context = &calculator.goal_context_events()[0];
    assert_eq!(goal_context.ball_speed_at_goal, Some(1600.0));
    let scorer_last_touch = goal_context.scorer_last_touch.as_ref().unwrap();
    assert_eq!(scorer_last_touch.frame, 10);
    assert_eq!(scorer_last_touch.time, 1.0);
    assert_eq!(
        scorer_last_touch.ball_position,
        Some(GoalContextPosition::from(touch_ball_position))
    );
    assert_eq!(scorer_last_touch.ball_speed_after_touch, Some(1200.0));
}

#[test]
fn ball_speed_at_goal_falls_back_to_last_velocity_when_explosion_frame_has_none() {
    // Real replays record the goal on the ball explosion frame, where the
    // interpolated ball rigid body carries no velocity (reads as zero). The
    // speed at goal should fall back to the most recent in-flight velocity
    // rather than reporting 0.
    let mut calculator = MatchStatsCalculator::new();

    update(
        &mut calculator,
        frame(10, 1.0),
        ball_with_state(
            glam::Vec3::new(0.0, 4000.0, 100.0),
            glam::Vec3::new(0.0, 2500.0, 0.0),
        ),
        0,
        Vec::new(),
    );
    update(
        &mut calculator,
        frame(20, 1.1),
        // Explosion frame: ball present but with no usable velocity.
        ball_with_state(glam::Vec3::new(0.0, 5200.0, 100.0), glam::Vec3::ZERO),
        1,
        vec![goal_event(1.1, 20, PlayerId::Steam(1))],
    );

    assert_eq!(
        calculator.goal_context_events()[0].ball_speed_at_goal,
        Some(2500.0)
    );
}

#[test]
fn rewrites_misattributed_goal_context_scorer_from_goal_delta() {
    let scorer = PlayerId::Steam(1);
    let stale_touch_player = PlayerId::Steam(2);
    let mut calculator = MatchStatsCalculator::new();

    let update_players = |calculator: &mut MatchStatsCalculator,
                          frame: FrameInfo,
                          player_goals: i32,
                          goal_events| {
        calculator
            .update_parts(
                &frame,
                &GameplayState {
                    ball_has_been_hit: Some(true),
                    team_zero_score: Some(player_goals),
                    team_one_score: Some(0),
                    ..GameplayState::default()
                },
                &ball(BALL_GROUND_CONTACT_MAX_Z + 200.0),
                &PlayerFrameState {
                    players: vec![
                        player(scorer.clone(), true, player_goals),
                        player(stale_touch_player.clone(), true, 0),
                    ],
                },
                &FrameEventsState {
                    goal_events,
                    ..FrameEventsState::default()
                },
                &LivePlayState {
                    gameplay_phase: GameplayPhase::ActivePlay,
                    is_live_play: true,
                },
                &TouchState::default(),
            )
            .unwrap();
    };

    update_players(&mut calculator, frame(0, 0.0), 0, Vec::new());
    update_players(
        &mut calculator,
        frame(1, 1.5),
        1,
        vec![goal_event(1.5, 1, stale_touch_player.clone())],
    );

    assert_eq!(
        calculator.goal_context_events()[0].scorer,
        Some(scorer.clone())
    );
    assert_eq!(
        calculator
            .timeline()
            .iter()
            .find(|event| event.kind == TimelineEventKind::Goal)
            .and_then(|event| event.player_id.as_ref()),
        Some(&scorer)
    );
    let stale_stats = calculator
        .player_stats()
        .get(&stale_touch_player)
        .cloned()
        .unwrap_or_default();
    assert_eq!(
        stale_stats
            .scoring_context
            .goal_ball_air_time
            .goal_ball_air_time_sample_count,
        0
    );
}

#[test]
fn finish_flushes_attributed_goal_without_goal_counter_delta() {
    let scorer = PlayerId::Steam(1);
    let mut calculator = MatchStatsCalculator::new();
    let mut scorer_sample = player(scorer.clone(), true, 0);
    scorer_sample.match_goals = None;

    calculator
        .update_parts(
            &frame(1, 1.5),
            &GameplayState {
                ball_has_been_hit: Some(true),
                team_zero_score: Some(1),
                team_one_score: Some(0),
                ..GameplayState::default()
            },
            &ball(BALL_GROUND_CONTACT_MAX_Z + 200.0),
            &PlayerFrameState {
                players: vec![scorer_sample],
            },
            &FrameEventsState {
                goal_events: vec![goal_event(1.5, 1, scorer.clone())],
                ..FrameEventsState::default()
            },
            &LivePlayState {
                gameplay_phase: GameplayPhase::ActivePlay,
                is_live_play: true,
            },
            &TouchState::default(),
        )
        .unwrap();

    assert!(
        calculator
            .timeline()
            .iter()
            .all(|event| event.kind != TimelineEventKind::Goal)
    );

    calculator.finish().unwrap();

    assert_eq!(
        calculator
            .timeline()
            .iter()
            .filter(|event| event.kind == TimelineEventKind::Goal)
            .count(),
        1
    );
    assert_eq!(
        calculator
            .timeline()
            .iter()
            .find(|event| event.kind == TimelineEventKind::Goal)
            .and_then(|event| event.player_id.as_ref()),
        Some(&scorer)
    );
    assert_eq!(
        calculator
            .player_stats()
            .get(&scorer)
            .map(|stats| stats.goals),
        Some(1)
    );
}

#[test]
fn records_goal_ball_air_time_from_last_ground_contact() {
    let scorer = PlayerId::Steam(1);
    let mut calculator = MatchStatsCalculator::new();

    update(
        &mut calculator,
        frame(0, 0.0),
        ball(BALL_GROUND_CONTACT_MAX_Z),
        0,
        Vec::new(),
    );
    update(
        &mut calculator,
        frame(1, 1.0),
        ball(BALL_GROUND_CONTACT_MAX_Z + 200.0),
        0,
        Vec::new(),
    );
    update(
        &mut calculator,
        frame(2, 2.5),
        ball(BALL_GROUND_CONTACT_MAX_Z + 300.0),
        1,
        vec![goal_event(2.5, 2, scorer.clone())],
    );

    assert_eq!(
        calculator.goal_context_events()[0].ball_air_time_before_goal,
        Some(2.5)
    );

    let scorer_stats = calculator.player_stats().get(&scorer).unwrap();
    assert_eq!(
        scorer_stats
            .scoring_context
            .goal_ball_air_time
            .goal_ball_air_time_sample_count,
        1
    );
    assert_eq!(scorer_stats.average_goal_ball_air_time(), 2.5);
    assert_eq!(
        calculator.team_zero_stats().average_goal_ball_air_time(),
        2.5
    );
}

#[test]
fn leaves_goal_ball_air_time_empty_without_observed_ground_contact() {
    let scorer = PlayerId::Steam(1);
    let mut calculator = MatchStatsCalculator::new();

    update(
        &mut calculator,
        frame(2, 2.5),
        ball(BALL_GROUND_CONTACT_MAX_Z + 300.0),
        1,
        vec![goal_event(2.5, 2, scorer.clone())],
    );

    assert_eq!(
        calculator.goal_context_events()[0].ball_air_time_before_goal,
        None
    );

    let scorer_stats = calculator.player_stats().get(&scorer).unwrap();
    assert_eq!(
        scorer_stats
            .scoring_context
            .goal_ball_air_time
            .goal_ball_air_time_sample_count,
        0
    );
}

#[test]
fn counter_attack_buildup_accepts_defensive_half_pressure_without_defensive_third_time() {
    let mut calculator = MatchStatsCalculator::new();
    calculator.goal_buildup_samples = vec![
        buildup_sample(2.0, -500.0),
        buildup_sample(3.0, -500.0),
        buildup_sample(4.0, -500.0),
        buildup_sample(5.0, -500.0),
        buildup_sample(6.0, 600.0),
        buildup_sample(7.0, 600.0),
        buildup_sample(8.0, 600.0),
        buildup_sample(9.0, 600.0),
    ];

    assert_eq!(
        calculator.classify_goal_buildup(10.0, true),
        GoalBuildupKind::CounterAttack
    );
}

#[test]
fn counter_attack_buildup_ignores_defensive_pressure_before_the_kickoff() {
    // Regression: a clean kickoff goal must not be classified as a
    // counter-attack just because the lookback window reaches back across the
    // kickoff into the previous possession. The defensive-half presence here all
    // predates the kickoff first touch, so it must not count toward this goal.
    let mut calculator = MatchStatsCalculator::new();
    calculator.goal_buildup_samples = vec![
        buildup_sample(2.0, -500.0),
        buildup_sample(3.0, -500.0),
        buildup_sample(4.0, -500.0),
        buildup_sample(5.0, -500.0),
        buildup_sample(6.0, 600.0),
        buildup_sample(7.0, 600.0),
        buildup_sample(8.0, 600.0),
        buildup_sample(9.0, 600.0),
    ];
    // Kickoff first touch lands after the (prior-possession) defensive pressure.
    calculator.active_kickoff_touch_time = Some(5.5);

    assert_eq!(
        calculator.classify_goal_buildup(10.0, true),
        GoalBuildupKind::Other
    );
}

#[test]
fn counter_attack_buildup_ignores_opponent_shot_before_the_kickoff() {
    let mut calculator = MatchStatsCalculator::new();
    calculator.goal_buildup_samples = vec![
        buildup_sample(6.0, 600.0),
        buildup_sample(7.0, 600.0),
        buildup_sample(8.0, 600.0),
        buildup_sample(9.0, 600.0),
    ];
    // Opponent shot belongs to the possession before this kickoff.
    calculator.goal_buildup_pressure_events = vec![shot_pressure(5.0, false)];
    calculator.active_kickoff_touch_time = Some(5.5);

    assert_eq!(
        calculator.classify_goal_buildup(10.0, true),
        GoalBuildupKind::Other
    );
}

#[test]
fn counter_attack_buildup_accepts_opponent_shot_as_pressure_signal() {
    let mut calculator = MatchStatsCalculator::new();
    calculator.goal_buildup_samples = vec![
        buildup_sample(6.0, 600.0),
        buildup_sample(7.0, 600.0),
        buildup_sample(8.0, 600.0),
        buildup_sample(9.0, 600.0),
    ];
    calculator.goal_buildup_pressure_events = vec![shot_pressure(7.5, false)];

    assert_eq!(
        calculator.classify_goal_buildup(10.0, true),
        GoalBuildupKind::CounterAttack
    );
}

#[test]
fn counter_attack_buildup_requires_a_defensive_pressure_signal() {
    let mut calculator = MatchStatsCalculator::new();
    calculator.goal_buildup_samples = vec![
        buildup_sample(6.0, 600.0),
        buildup_sample(7.0, 600.0),
        buildup_sample(8.0, 600.0),
        buildup_sample(9.0, 600.0),
    ];

    assert_eq!(
        calculator.classify_goal_buildup(10.0, true),
        GoalBuildupKind::Other
    );
}

#[test]
fn sustained_pressure_buildup_waits_past_kickoff_goal_window() {
    let mut calculator = MatchStatsCalculator::new();
    calculator.goal_buildup_samples = (2..=11)
        .map(|time| buildup_sample(time as f32, FIELD_ZONE_BOUNDARY_Y + 500.0))
        .collect();
    calculator.active_kickoff_touch_time = Some(0.0);

    assert_eq!(
        calculator.classify_goal_buildup(11.8, true),
        GoalBuildupKind::Other
    );
}

#[test]
fn sustained_pressure_buildup_applies_after_kickoff_goal_window() {
    let mut calculator = MatchStatsCalculator::new();
    calculator.goal_buildup_samples = (2..=13)
        .map(|time| buildup_sample(time as f32, FIELD_ZONE_BOUNDARY_Y + 500.0))
        .collect();
    calculator.active_kickoff_touch_time = Some(0.0);

    assert_eq!(
        calculator.classify_goal_buildup(13.0, true),
        GoalBuildupKind::SustainedPressure
    );
}

#[test]
fn time_after_kickoff_uses_kickoff_first_touch_not_latest_touch() {
    let scorer = PlayerId::Steam(1);
    let mut calculator = MatchStatsCalculator::new();

    let touch_event = |time: f32, frame: usize| TouchEvent {
        touch_id: None,
        time,
        frame,
        team_is_team_0: true,
        player: Some(scorer.clone()),
        player_position: None,
        closest_approach_distance: Some(0.0),
        contact_local_ball_position: None,
        contact_local_hitbox_point: None,
        contact_world_hitbox_point: None,
        dodge_contact: false,
    };

    fn update_with(
        calculator: &mut MatchStatsCalculator,
        frame_info: FrameInfo,
        kickoff_phase: bool,
        player_goals: i32,
        touch_events: Vec<TouchEvent>,
        goal_events: Vec<GoalEvent>,
    ) {
        let scorer = PlayerId::Steam(1);
        calculator
            .update_parts(
                &frame_info,
                &GameplayState {
                    ball_has_been_hit: Some(!kickoff_phase),
                    team_zero_score: Some(player_goals),
                    team_one_score: Some(0),
                    ..GameplayState::default()
                },
                &ball(BALL_GROUND_CONTACT_MAX_Z),
                &PlayerFrameState {
                    players: vec![player(scorer, true, player_goals)],
                },
                &FrameEventsState {
                    touch_events,
                    goal_events,
                    ..FrameEventsState::default()
                },
                &LivePlayState {
                    gameplay_phase: GameplayPhase::ActivePlay,
                    is_live_play: !kickoff_phase,
                },
                &TouchState::default(),
            )
            .unwrap();
    }

    // Kickoff countdown: waiting for the kickoff's first touch.
    update_with(
        &mut calculator,
        frame(0, 0.0),
        true,
        0,
        Vec::new(),
        Vec::new(),
    );
    // Kickoff first touch at t=1.0 establishes the reference.
    update_with(
        &mut calculator,
        frame(10, 1.0),
        false,
        0,
        vec![touch_event(1.0, 10)],
        Vec::new(),
    );
    // A mid-rally touch much later must not reset the kickoff reference.
    update_with(
        &mut calculator,
        frame(200, 20.0),
        false,
        0,
        vec![touch_event(20.0, 200)],
        Vec::new(),
    );
    // Goal at t=21.0: time after kickoff is 20s, not 1s.
    update_with(
        &mut calculator,
        frame(210, 21.0),
        false,
        1,
        Vec::new(),
        vec![goal_event(21.0, 210, scorer.clone())],
    );

    let core_event = calculator
        .core_player_goal_context_events()
        .last()
        .expect("goal should emit a core goal context event");
    let time_after_kickoff = core_event
        .time_after_kickoff
        .expect("time_after_kickoff should be set");
    assert!(
        (time_after_kickoff - 20.0).abs() < 1e-4,
        "expected time_after_kickoff to measure from the kickoff first touch, got {time_after_kickoff}"
    );
}
