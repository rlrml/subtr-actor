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

fn ball_y(y: f32) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(glam::Vec3::new(0.0, y, 100.0)),
    })
}

fn frame(frame_number: usize, time: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 1.0,
        seconds_remaining: None,
    }
}

fn live_play() -> LivePlayState {
    LivePlayState {
        gameplay_phase: GameplayPhase::ActivePlay,
        is_live_play: true,
    }
}

fn possession(team_is_team_0: Option<bool>) -> PossessionState {
    PossessionState {
        active_team_before_sample: team_is_team_0,
        current_team_is_team_0: team_is_team_0,
        active_player_before_sample: None,
        current_player: None,
    }
}

fn config() -> TerritorialPressureCalculatorConfig {
    TerritorialPressureCalculatorConfig {
        min_establish_seconds: 2.0,
        min_establish_third_seconds: 1.0,
        relief_grace_seconds: 3.0,
        confirmed_relief_grace_seconds: 2.0,
        ..TerritorialPressureCalculatorConfig::default()
    }
}

#[test]
fn starts_session_after_sustained_opponent_half_time() {
    let mut calculator = TerritorialPressureCalculator::with_config(config());

    calculator
        .update(
            &frame(1, 1.0),
            &ball_y(500.0),
            &possession(None),
            &live_play(),
        )
        .unwrap();
    assert_eq!(calculator.stats().team_zero_session_count, 0);

    calculator
        .update(
            &frame(2, 2.0),
            &ball_y(600.0),
            &possession(None),
            &live_play(),
        )
        .unwrap();

    assert_eq!(calculator.stats().team_zero_session_count, 1);
    assert_eq!(calculator.stats().team_zero_session_time, 2.0);
    assert_eq!(calculator.stats().team_zero_offensive_half_time, 2.0);
}

#[test]
fn offensive_third_starts_session_quickly() {
    let mut calculator = TerritorialPressureCalculator::with_config(config());

    calculator
        .update(
            &frame(1, 1.0),
            &ball_y(FIELD_ZONE_BOUNDARY_Y + 100.0),
            &possession(None),
            &live_play(),
        )
        .unwrap();

    assert_eq!(calculator.stats().team_zero_session_count, 1);
    assert_eq!(calculator.stats().team_zero_offensive_third_time, 1.0);
}

#[test]
fn session_is_sticky_through_brief_midfield_clearance() {
    let mut calculator = TerritorialPressureCalculator::with_config(config());

    calculator
        .update(
            &frame(1, 1.0),
            &ball_y(500.0),
            &possession(None),
            &live_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 2.0),
            &ball_y(600.0),
            &possession(None),
            &live_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(3, 3.0),
            &ball_y(0.0),
            &possession(None),
            &live_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(4, 4.0),
            &ball_y(500.0),
            &possession(None),
            &live_play(),
        )
        .unwrap();

    assert!(calculator.events().is_empty());
    assert_eq!(calculator.stats().team_zero_session_count, 1);
    assert_eq!(calculator.stats().team_zero_session_time, 4.0);
}

#[test]
fn confirmed_opponent_possession_relieves_pressure() {
    let mut calculator = TerritorialPressureCalculator::with_config(config());

    calculator
        .update(
            &frame(1, 1.0),
            &ball_y(500.0),
            &possession(None),
            &live_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 2.0),
            &ball_y(600.0),
            &possession(None),
            &live_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(3, 3.0),
            &ball_y(0.0),
            &possession(Some(false)),
            &live_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(4, 4.0),
            &ball_y(-300.0),
            &possession(Some(false)),
            &live_play(),
        )
        .unwrap();

    assert_eq!(calculator.events().len(), 1);
    assert_eq!(
        calculator.events()[0].end_reason,
        TerritorialPressureEndReason::Relieved
    );
    assert_eq!(calculator.events()[0].duration, 4.0);
}

#[test]
fn live_play_stoppage_ends_active_session() {
    let mut calculator = TerritorialPressureCalculator::with_config(config());

    calculator
        .update(
            &frame(1, 1.0),
            &ball_y(500.0),
            &possession(None),
            &live_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(2, 2.0),
            &ball_y(600.0),
            &possession(None),
            &live_play(),
        )
        .unwrap();
    calculator
        .update(
            &frame(3, 3.0),
            &ball_y(600.0),
            &possession(None),
            &LivePlayState::default(),
        )
        .unwrap();

    assert_eq!(calculator.events().len(), 1);
    assert_eq!(
        calculator.events()[0].end_reason,
        TerritorialPressureEndReason::Stoppage
    );
}
