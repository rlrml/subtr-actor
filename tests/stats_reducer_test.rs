use subtr_actor::*;

fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).unwrap_or_else(|_| panic!("Failed to read replay file: {path}"));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay: {path}"))
}

#[test]
fn test_powerslide_reducer_collects_duration_and_presses() {
    let replay = parse_replay("assets/replays/test/rlcs.replay");
    let reducer = ReducerCollector::new(PowerslideReducer::new())
        .process_replay(&replay)
        .expect("Failed to process replay with powerslide reducer")
        .into_inner();

    assert!(
        reducer
            .player_stats()
            .values()
            .any(|stats| stats.total_duration > 0.0),
        "Expected at least one player to have non-zero powerslide duration"
    );
    assert!(
        reducer
            .player_stats()
            .values()
            .any(|stats| stats.press_count > 0),
        "Expected at least one player to have non-zero powerslide press count"
    );
    assert!(
        reducer.team_zero_stats().total_duration > 0.0
            || reducer.team_one_stats().total_duration > 0.0,
        "Expected at least one team to have non-zero powerslide duration"
    );
}

#[test]
fn test_pressure_reducer_tracks_ball_side_time() {
    let replay = parse_replay("assets/replays/test/rlcs.replay");
    let reducer = ReducerCollector::new(PressureReducer::new())
        .process_replay(&replay)
        .expect("Failed to process replay with pressure reducer")
        .into_inner();

    assert!(
        reducer.team_zero_side_duration() > 0.0,
        "Expected non-zero tracked time on team zero side"
    );
    assert!(
        reducer.team_one_side_duration() > 0.0,
        "Expected non-zero tracked time on team one side"
    );
    assert!(
        reducer.total_tracked_duration() > 0.0,
        "Expected pressure reducer to track some ball time"
    );
}

#[test]
fn test_tuple_reducers_compose_under_frame_rate_decorator() {
    let replay = parse_replay("assets/replays/test/rlcs.replay");
    let mut collector = ReducerCollector::new((PowerslideReducer::new(), PressureReducer::new()));

    FrameRateDecorator::new_from_fps(10.0, &mut collector)
        .process_replay(&replay)
        .expect("Failed to process replay with composed reducers");

    let (powerslide, pressure) = collector.into_inner();

    assert!(
        powerslide
            .player_stats()
            .values()
            .any(|stats| stats.press_count > 0),
        "Expected composed powerslide reducer to record presses"
    );
    assert!(
        pressure.total_tracked_duration() > 0.0,
        "Expected composed pressure reducer to track ball-side time"
    );
}
