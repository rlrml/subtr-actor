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

fn player(position: glam::Vec3, velocity: glam::Vec3) -> PlayerSample {
    PlayerSample {
        player_id: boxcars::RemoteId::Steam(1),
        is_team_0: true,
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

fn ball(position: glam::Vec3, velocity: glam::Vec3) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(position, velocity),
    })
}

fn frame(frame_number: usize, time: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 0.2,
        seconds_remaining: None,
    }
}

fn touch_state() -> TouchState {
    TouchState {
        last_touch_player: Some(boxcars::RemoteId::Steam(1)),
        ..TouchState::default()
    }
}

fn touch_state_with_touch(frame: usize, time: f32) -> TouchState {
    let player_id = boxcars::RemoteId::Steam(1);
    TouchState {
        touch_events: vec![TouchEvent {
            time,
            frame,
            team_is_team_0: true,
            player: Some(player_id.clone()),
            closest_approach_distance: None,
        }],
        last_touch: Some(TouchEvent {
            time,
            frame,
            team_is_team_0: true,
            player: Some(player_id.clone()),
            closest_approach_distance: None,
        }),
        last_touch_player: Some(player_id),
        last_touch_team_is_team_0: Some(true),
    }
}

#[derive(Default)]
struct BallCarryHarness {
    tracker: ContinuousBallControlTracker<BallCarryKind>,
    state: ContinuousBallControlState,
    calculator: BallCarryCalculator,
}

impl BallCarryHarness {
    fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_state: &TouchState,
        live_play_state: &LivePlayState,
    ) {
        let candidate = if frame.dt > 0.0 {
            BallCarryCalculator::control_candidate(
                ball,
                players,
                live_play_state.is_live_play,
                touch_state,
            )
        } else {
            None
        };

        self.state.completed_sequences.extend(self.tracker.update(
            frame,
            candidate,
            BallCarryCalculator::min_duration_for_kind,
        ));
        self.calculator.update(&self.state).unwrap();
    }

    fn finish(&mut self) {
        if let Some(sequence) = self
            .tracker
            .finish(BallCarryCalculator::min_duration_for_kind)
        {
            self.state.completed_sequences.push(sequence);
        }
        self.calculator.update(&self.state).unwrap();
    }
}

#[test]
fn counts_air_dribble_when_airborne_player_carries_airborne_ball() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut harness = BallCarryHarness::default();

    for i in 1..=5 {
        let x = i as f32 * 50.0;
        harness.update(
            &frame(i, i as f32 * 0.2),
            &ball(
                glam::Vec3::new(x, 0.0, 520.0),
                glam::Vec3::new(250.0, 0.0, 0.0),
            ),
            &PlayerFrameState {
                players: vec![player(
                    glam::Vec3::new(x, 0.0, 360.0),
                    glam::Vec3::new(250.0, 0.0, 0.0),
                )],
            },
            &touch_state(),
            &LivePlayState {
                is_live_play: true,
                ..LivePlayState::default()
            },
        );
    }

    harness.finish();

    assert!(harness.calculator.player_stats().get(&player_id).is_none());
    let stats = harness
        .calculator
        .player_air_dribble_stats()
        .get(&player_id)
        .unwrap();
    assert_eq!(stats.count, 1);
    assert_eq!(stats.ground_to_air_count, 1);
    assert_eq!(stats.wall_to_air_count, 0);
    assert!((stats.total_time - 1.0).abs() < f32::EPSILON);
    assert_eq!(
        harness.calculator.carry_events()[0].kind,
        BallCarryKind::AirDribble
    );
    assert_eq!(
        harness.calculator.carry_events()[0].air_dribble_origin,
        Some(AirDribbleOrigin::GroundToAir)
    );
}

#[test]
fn keeps_ground_carry_stats_separate_from_air_dribbles() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut harness = BallCarryHarness::default();

    for i in 1..=5 {
        let x = i as f32 * 50.0;
        harness.update(
            &frame(i, i as f32 * 0.2),
            &ball(
                glam::Vec3::new(x, 0.0, 120.0),
                glam::Vec3::new(250.0, 0.0, 0.0),
            ),
            &PlayerFrameState {
                players: vec![player(
                    glam::Vec3::new(x, 0.0, 20.0),
                    glam::Vec3::new(250.0, 0.0, 0.0),
                )],
            },
            &touch_state(),
            &LivePlayState {
                is_live_play: true,
                ..LivePlayState::default()
            },
        );
    }

    harness.finish();

    let stats = harness.calculator.player_stats().get(&player_id).unwrap();
    assert_eq!(stats.carry_count, 1);
    assert!(harness
        .calculator
        .player_air_dribble_stats()
        .get(&player_id)
        .is_none());
    assert_eq!(
        harness.calculator.carry_events()[0].kind,
        BallCarryKind::Carry
    );
}

#[test]
fn records_air_dribble_touch_count() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut harness = BallCarryHarness::default();

    for i in 1..=8 {
        let x = i as f32 * 50.0;
        let touch_state = if matches!(i, 1 | 4 | 7) {
            touch_state_with_touch(i, i as f32 * 0.2)
        } else {
            touch_state()
        };
        harness.update(
            &frame(i, i as f32 * 0.2),
            &ball(
                glam::Vec3::new(x, 0.0, 520.0),
                glam::Vec3::new(250.0, 0.0, 0.0),
            ),
            &PlayerFrameState {
                players: vec![player(
                    glam::Vec3::new(x, 0.0, 360.0),
                    glam::Vec3::new(250.0, 0.0, 0.0),
                )],
            },
            &touch_state,
            &LivePlayState {
                is_live_play: true,
                ..LivePlayState::default()
            },
        );
    }

    harness.finish();

    let stats = harness
        .calculator
        .player_air_dribble_stats()
        .get(&player_id)
        .unwrap();
    assert_eq!(stats.total_touch_count, 3);
    assert_eq!(stats.max_touch_count, 3);
    assert_eq!(stats.average_touch_count(), 3.0);
    assert_eq!(harness.calculator.carry_events()[0].touch_count, 3);
}

#[test]
fn records_wall_to_air_dribble_origin() {
    let player_id = boxcars::RemoteId::Steam(1);
    let mut harness = BallCarryHarness::default();

    for i in 1..=5 {
        harness.update(
            &frame(i, i as f32 * 0.2),
            &ball(
                glam::Vec3::new(3420.0, i as f32 * 50.0, 520.0),
                glam::Vec3::new(0.0, 250.0, 0.0),
            ),
            &PlayerFrameState {
                players: vec![player(
                    glam::Vec3::new(3300.0, i as f32 * 50.0, 360.0),
                    glam::Vec3::new(0.0, 250.0, 0.0),
                )],
            },
            &touch_state(),
            &LivePlayState {
                is_live_play: true,
                ..LivePlayState::default()
            },
        );
    }

    harness.finish();

    let stats = harness
        .calculator
        .player_air_dribble_stats()
        .get(&player_id)
        .unwrap();
    assert_eq!(stats.ground_to_air_count, 0);
    assert_eq!(stats.wall_to_air_count, 1);
    assert_eq!(
        harness.calculator.carry_events()[0].air_dribble_origin,
        Some(AirDribbleOrigin::WallToAir)
    );
}
