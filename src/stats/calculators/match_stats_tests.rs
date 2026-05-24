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
