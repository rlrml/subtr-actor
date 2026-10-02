use subtr_actor::*;

// `ReplicatedStateName` indexes this replay's name table, where index 67 is
// "Active" rather than the goal-replay state.
const REPLAY_PATH: &str = "assets/ranked-standard-state-name-index-collision-2024-08-25.replay";

#[test]
fn game_state_resolves_through_the_replay_name_table() {
    let data = std::fs::read(REPLAY_PATH).expect("Failed to read replay file");
    let replay = boxcars::ParserBuilder::new(&data[..])
        .must_parse_network_data()
        .parse()
        .expect("Failed to parse replay");
    assert_eq!(replay.names[67], "Active");

    let captured = StatsCollector::only_modules(["movement"])
        .capture_frames()
        .get_captured_data(&replay)
        .expect("Failed to collect stats");
    let live_play_seconds: f32 = captured
        .frames
        .iter()
        .filter(|frame| frame.is_live_play)
        .map(|frame| frame.dt)
        .sum();

    assert!(
        live_play_seconds > 150.0,
        "expected most of the match to be live play, got {live_play_seconds}s"
    );
}
