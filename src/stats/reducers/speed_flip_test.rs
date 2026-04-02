use boxcars::RemoteId;

use super::*;
use crate::stats::reducers::StatsReducer;

fn rigid_body(
    position: glam::Vec3,
    rotation: glam::Quat,
    velocity: glam::Vec3,
    angular_velocity: glam::Vec3,
) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: false,
        location: glam_to_vec(&position),
        rotation: glam_to_quat(&rotation),
        linear_velocity: Some(glam_to_vec(&velocity)),
        angular_velocity: Some(glam_to_vec(&angular_velocity)),
    }
}

fn sample(
    frame_number: usize,
    time: f32,
    player_rigid_body: boxcars::RigidBody,
    dodge_active: bool,
    ball_position: Option<glam::Vec3>,
) -> FrameState {
    FrameState {
        frame_number,
        time,
        dt: if frame_number == 0 { 0.0 } else { 1.0 / 120.0 },
        seconds_remaining: None,
        game_state: Some(0),
        ball_has_been_hit: Some(false),
        kickoff_countdown_time: None,
        team_zero_score: None,
        team_one_score: None,
        possession_team_is_team_0: None,
        scored_on_team_is_team_0: None,
        current_in_game_team_player_counts: None,
        ball: ball_position.map(|ball_position| BallSample {
            rigid_body: rigid_body(
                ball_position,
                glam::Quat::IDENTITY,
                glam::Vec3::ZERO,
                glam::Vec3::ZERO,
            ),
        }),
        players: vec![PlayerSample {
            player_id: RemoteId::Steam(1),
            is_team_0: true,
            rigid_body: Some(player_rigid_body),
            boost_amount: Some(50.0),
            last_boost_amount: Some(50.0),
            boost_active: true,
            dodge_active,
            powerslide_active: false,
            match_goals: None,
            match_assists: None,
            match_saves: None,
            match_shots: None,
            match_score: None,
        }],
        active_demos: Vec::new(),
        demo_events: Vec::new(),
        boost_pad_events: Vec::new(),
        touch_events: Vec::new(),
        dodge_refreshed_events: Vec::new(),
        player_stat_events: Vec::new(),
        goal_events: Vec::new(),
    }
}

#[test]
fn detects_high_confidence_kickoff_speed_flip() {
    let mut reducer = SpeedFlipCalculator::new();
    let ball_position = Some(glam::Vec3::new(4000.0, 420.0, 92.75));

    reducer
        .on_sample(&sample(
            0,
            0.0,
            rigid_body(
                glam::Vec3::new(0.0, 0.0, 17.0),
                glam::Quat::IDENTITY,
                glam::Vec3::new(1280.0, 165.0, 0.0),
                glam::Vec3::ZERO,
            ),
            false,
            ball_position,
        ))
        .unwrap();
    reducer
        .on_sample(&sample(
            1,
            0.05,
            rigid_body(
                glam::Vec3::new(65.0, 6.0, 17.0),
                glam::Quat::IDENTITY,
                glam::Vec3::new(1440.0, 180.0, 0.0),
                glam::Vec3::new(1.1, 7.2, 3.0),
            ),
            true,
            ball_position,
        ))
        .unwrap();
    reducer
        .on_sample(&sample(
            2,
            0.13,
            rigid_body(
                glam::Vec3::new(250.0, 28.0, 17.0),
                glam::Quat::from_rotation_y(0.72),
                glam::Vec3::new(1775.0, 205.0, 0.0),
                glam::Vec3::new(0.8, 5.8, 2.2),
            ),
            true,
            ball_position,
        ))
        .unwrap();
    reducer
        .on_sample(&sample(
            3,
            0.27,
            rigid_body(
                glam::Vec3::new(610.0, 72.0, 17.0),
                glam::Quat::from_rotation_y(0.26),
                glam::Vec3::new(1875.0, 230.0, 0.0),
                glam::Vec3::new(0.3, 1.4, 0.9),
            ),
            true,
            ball_position,
        ))
        .unwrap();

    let stats = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
    assert_eq!(stats.count, 1);
    assert_eq!(stats.high_confidence_count, 1);
    assert_eq!(reducer.events().len(), 1);
    assert!(reducer.events()[0].confidence >= SPEED_FLIP_HIGH_CONFIDENCE);
}

#[test]
fn rejects_diagonal_kickoff_flip_without_cancel_recovery() {
    let mut reducer = SpeedFlipCalculator::new();
    let ball_position = Some(glam::Vec3::new(4000.0, 420.0, 92.75));

    reducer
        .on_sample(&sample(
            0,
            0.0,
            rigid_body(
                glam::Vec3::new(0.0, 0.0, 17.0),
                glam::Quat::IDENTITY,
                glam::Vec3::new(1260.0, 160.0, 0.0),
                glam::Vec3::ZERO,
            ),
            false,
            ball_position,
        ))
        .unwrap();
    reducer
        .on_sample(&sample(
            1,
            0.05,
            rigid_body(
                glam::Vec3::new(65.0, 6.0, 17.0),
                glam::Quat::IDENTITY,
                glam::Vec3::new(1430.0, 180.0, 0.0),
                glam::Vec3::new(1.0, 7.0, 2.9),
            ),
            true,
            ball_position,
        ))
        .unwrap();
    reducer
        .on_sample(&sample(
            2,
            0.13,
            rigid_body(
                glam::Vec3::new(250.0, 28.0, 17.0),
                glam::Quat::from_rotation_y(0.76),
                glam::Vec3::new(1690.0, 210.0, 0.0),
                glam::Vec3::new(0.8, 5.9, 2.3),
            ),
            true,
            ball_position,
        ))
        .unwrap();
    reducer
        .on_sample(&sample(
            3,
            0.27,
            rigid_body(
                glam::Vec3::new(540.0, 66.0, 17.0),
                glam::Quat::from_rotation_y(1.08),
                glam::Vec3::new(1710.0, 220.0, 0.0),
                glam::Vec3::new(0.6, 4.8, 1.8),
            ),
            true,
            ball_position,
        ))
        .unwrap();

    assert!(reducer.events().is_empty());
    assert!(reducer.player_stats().is_empty());
}

#[test]
fn detects_high_confidence_kickoff_speed_flip_with_sleeping_ball() {
    let mut reducer = SpeedFlipCalculator::new();
    let ball_position = None;

    reducer
        .on_sample(&sample(
            0,
            0.0,
            rigid_body(
                glam::Vec3::new(-1500.0, 0.0, 17.0),
                glam::Quat::IDENTITY,
                glam::Vec3::new(1280.0, 0.0, 0.0),
                glam::Vec3::ZERO,
            ),
            false,
            ball_position,
        ))
        .unwrap();
    reducer
        .on_sample(&sample(
            1,
            0.05,
            rigid_body(
                glam::Vec3::new(-1435.0, 0.0, 17.0),
                glam::Quat::IDENTITY,
                glam::Vec3::new(1440.0, 0.0, 0.0),
                glam::Vec3::new(1.1, 7.2, 3.0),
            ),
            true,
            ball_position,
        ))
        .unwrap();
    reducer
        .on_sample(&sample(
            2,
            0.13,
            rigid_body(
                glam::Vec3::new(-1250.0, 12.0, 17.0),
                glam::Quat::from_rotation_y(0.72),
                glam::Vec3::new(1775.0, 35.0, 0.0),
                glam::Vec3::new(0.8, 5.8, 2.2),
            ),
            true,
            ball_position,
        ))
        .unwrap();
    reducer
        .on_sample(&sample(
            3,
            0.27,
            rigid_body(
                glam::Vec3::new(-890.0, 24.0, 17.0),
                glam::Quat::from_rotation_y(0.26),
                glam::Vec3::new(1875.0, 45.0, 0.0),
                glam::Vec3::new(0.3, 1.4, 0.9),
            ),
            true,
            ball_position,
        ))
        .unwrap();

    let stats = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
    assert_eq!(stats.count, 1);
    assert_eq!(stats.high_confidence_count, 1);
    assert_eq!(reducer.events().len(), 1);
    assert!(reducer.events()[0].confidence >= SPEED_FLIP_HIGH_CONFIDENCE);
}

fn active_candidate(
    player_id: u64,
    start_time: f32,
    start_frame: usize,
) -> (PlayerId, ActiveSpeedFlipCandidate) {
    (
        RemoteId::Steam(player_id),
        ActiveSpeedFlipCandidate {
            is_team_0: player_id == 1,
            is_kickoff: false,
            kickoff_start_time: None,
            start_time,
            start_frame,
            start_position: [0.0, 0.0, 17.0],
            end_position: [100.0, 0.0, 17.0],
            start_speed: 1200.0,
            max_speed: 1700.0,
            best_alignment: 0.95,
            best_diagonal_score: 0.95,
            min_forward_z: -0.6,
            latest_forward_z: -0.1,
            latest_time: start_time + SPEED_FLIP_EVALUATION_SECONDS,
            latest_frame: start_frame + 1,
        },
    )
}

#[test]
fn finalize_candidates_orders_simultaneous_events_deterministically() {
    let mut reducer = SpeedFlipCalculator::new();
    let (player_one, candidate_one) = active_candidate(1, 0.5, 10);
    let (player_two, candidate_two) = active_candidate(2, 0.5, 10);
    reducer
        .active_candidates
        .insert(player_two.clone(), candidate_two);
    reducer
        .active_candidates
        .insert(player_one.clone(), candidate_one);

    reducer.finalize_candidates(
        &sample(
            100,
            1.0,
            rigid_body(
                glam::Vec3::ZERO,
                glam::Quat::IDENTITY,
                glam::Vec3::ZERO,
                glam::Vec3::ZERO,
            ),
            false,
            None,
        ),
        true,
    );

    assert_eq!(
        reducer
            .events()
            .iter()
            .map(|event| &event.player)
            .collect::<Vec<_>>(),
        vec![&player_one, &player_two]
    );
    assert_eq!(
        reducer.current_last_speed_flip_player.as_ref(),
        Some(&player_two)
    );
    assert_eq!(
        reducer
            .player_stats()
            .get(&player_one)
            .unwrap()
            .is_last_speed_flip,
        false
    );
    assert_eq!(
        reducer
            .player_stats()
            .get(&player_two)
            .unwrap()
            .is_last_speed_flip,
        true
    );
}

#[test]
fn detects_high_confidence_non_kickoff_speed_flip() {
    let mut reducer = SpeedFlipCalculator::new();
    let ball_position = Some(glam::Vec3::new(2000.0, 1200.0, 92.75));

    let mut frame0 = sample(
        0,
        10.0,
        rigid_body(
            glam::Vec3::new(-1500.0, 0.0, 17.0),
            glam::Quat::IDENTITY,
            glam::Vec3::new(1280.0, 0.0, 0.0),
            glam::Vec3::ZERO,
        ),
        false,
        ball_position,
    );
    frame0.ball_has_been_hit = Some(true);
    reducer.on_sample(&frame0).unwrap();

    let mut frame1 = sample(
        1,
        10.05,
        rigid_body(
            glam::Vec3::new(-1435.0, 0.0, 17.0),
            glam::Quat::IDENTITY,
            glam::Vec3::new(1440.0, 0.0, 0.0),
            glam::Vec3::new(1.1, 7.2, 3.0),
        ),
        true,
        ball_position,
    );
    frame1.ball_has_been_hit = Some(true);
    reducer.on_sample(&frame1).unwrap();

    let mut frame2 = sample(
        2,
        10.13,
        rigid_body(
            glam::Vec3::new(-1250.0, 12.0, 17.0),
            glam::Quat::from_rotation_y(0.72),
            glam::Vec3::new(1775.0, 35.0, 0.0),
            glam::Vec3::new(0.8, 5.8, 2.2),
        ),
        true,
        ball_position,
    );
    frame2.ball_has_been_hit = Some(true);
    reducer.on_sample(&frame2).unwrap();

    let mut frame3 = sample(
        3,
        10.27,
        rigid_body(
            glam::Vec3::new(-890.0, 24.0, 17.0),
            glam::Quat::from_rotation_y(0.26),
            glam::Vec3::new(1875.0, 45.0, 0.0),
            glam::Vec3::new(0.3, 1.4, 0.9),
        ),
        true,
        ball_position,
    );
    frame3.ball_has_been_hit = Some(true);
    reducer.on_sample(&frame3).unwrap();

    let stats = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
    assert_eq!(stats.count, 1);
    assert_eq!(stats.high_confidence_count, 1);
    assert_eq!(reducer.events().len(), 1);
    assert!(reducer.events()[0].confidence >= SPEED_FLIP_HIGH_CONFIDENCE);
    assert_eq!(reducer.events()[0].time_since_kickoff_start, 0.0);
}
