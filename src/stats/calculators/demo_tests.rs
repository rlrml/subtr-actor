use super::*;
use boxcars::RemoteId;

fn player(id: u32, is_team_0: bool) -> PlayerSample {
    PlayerSample {
        player_id: RemoteId::SplitScreen(id),
        is_team_0,
        hitbox: default_car_hitbox(),
        rigid_body: None,
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

fn frame(frame_number: usize, time: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 1.0,
        seconds_remaining: None,
    }
}

fn active_demo() -> DemoEventSample {
    DemoEventSample {
        attacker: RemoteId::SplitScreen(0),
        victim: RemoteId::SplitScreen(1),
    }
}

#[test]
fn persistent_active_demo_samples_do_not_double_count() {
    let mut calculator = DemoCalculator::new();
    let players = PlayerFrameState {
        players: vec![player(0, true), player(1, false)],
    };

    for frame_number in 1..=12 {
        calculator
            .update(
                &frame(frame_number, frame_number as f32),
                &players,
                &FrameEventsState {
                    active_demos: vec![active_demo()],
                    ..FrameEventsState::default()
                },
            )
            .unwrap();
    }

    assert_eq!(
        calculator
            .player_stats()
            .get(&RemoteId::SplitScreen(0))
            .map(|stats| stats.demos_inflicted),
        Some(1)
    );
    assert_eq!(
        calculator
            .player_stats()
            .get(&RemoteId::SplitScreen(1))
            .map(|stats| stats.demos_taken),
        Some(1)
    );
    assert_eq!(calculator.timeline().len(), 2);

    calculator
        .update(&frame(13, 13.0), &players, &FrameEventsState::default())
        .unwrap();
    calculator
        .update(
            &frame(30, 30.0),
            &players,
            &FrameEventsState {
                active_demos: vec![active_demo()],
                ..FrameEventsState::default()
            },
        )
        .unwrap();

    assert_eq!(
        calculator
            .player_stats()
            .get(&RemoteId::SplitScreen(0))
            .map(|stats| stats.demos_inflicted),
        Some(2)
    );
    assert_eq!(calculator.timeline().len(), 4);
}
