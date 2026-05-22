use super::*;

#[test]
fn kickoff_approach_stays_active_before_first_touch_even_when_not_live_play() {
    let mut calculator = SpeedFlipCalculator::default();
    let frame = FrameInfo {
        frame_number: 1,
        time: 0.5,
        dt: 0.1,
        seconds_remaining: None,
    };
    let gameplay = GameplayState {
        ball_has_been_hit: Some(false),
        ..Default::default()
    };

    calculator
        .update_parts(
            &frame,
            &gameplay,
            &BallFrameState::default(),
            &PlayerFrameState::default(),
            false,
        )
        .unwrap();

    assert!(calculator.kickoff_approach_active_last_frame);
    assert_eq!(calculator.current_kickoff_start_time, Some(frame.time));
}
