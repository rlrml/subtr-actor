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
        time,
        frame,
        team_is_team_0,
        player: Some(player),
        player_position: None,
        closest_approach_distance: Some(0.0),
        dodge_contact: false,
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
fn continuation_touch_updates_last_touch_from_latest_touch_event_not_sample_frame() {
    let player = PlayerId::Steam(1);
    let mut calculator = FiftyFiftyStateCalculator {
        active_event: Some(active_event(player.clone())),
        last_resolved_event: None,
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
