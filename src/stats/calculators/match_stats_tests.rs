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

fn ball(z: f32) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(glam::Vec3::new(0.0, 0.0, z)),
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
    let stale_stats = calculator.player_stats().get(&stale_touch_player).unwrap();
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

    assert!(calculator
        .timeline()
        .iter()
        .all(|event| event.kind != TimelineEventKind::Goal));

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
