use std::collections::HashMap;

use subtr_actor::*;

fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).expect("Failed to read replay file");
    boxcars::ParserBuilder::new(&data[..])
        .must_parse_network_data()
        .parse()
        .expect("Failed to parse replay")
}

#[allow(clippy::result_large_err)]
fn state_names_by_frame(replay: &boxcars::Replay) -> HashMap<usize, String> {
    let mut names = HashMap::new();
    let mut collect = |processor: &dyn ProcessorView,
                       _frame: &boxcars::Frame,
                       frame_number: usize,
                       _current_time: f32| {
        if let Ok(index) = processor.get_replicated_state_name() {
            names.insert(frame_number, replay.names[index as usize].clone());
        }
        Ok(TimeAdvance::NextFrame)
    };
    ReplayProcessor::new(replay)
        .expect("Failed to create processor")
        .process(&mut collect)
        .expect("Failed to process replay");
    names
}

fn assert_game_states_classify_live_play(path: &str) {
    let replay = parse_replay(path);
    let state_names = state_names_by_frame(&replay);
    let captured = StatsCollector::only_modules(["movement"])
        .capture_frames()
        .get_captured_data(&replay)
        .expect("Failed to collect stats");

    let mut active_frames = 0;
    let mut live_active_frames = 0;
    for frame in &captured.frames {
        match state_names.get(&frame.frame_number).map(String::as_str) {
            Some("Countdown" | "PostGoalScored" | "ReplayPlayback") => assert!(
                !frame.is_live_play,
                "{path}: frame {} in state {:?} counted as live play",
                frame.frame_number, state_names[&frame.frame_number],
            ),
            Some("Active") => {
                active_frames += 1;
                live_active_frames += usize::from(frame.is_live_play);
            }
            _ => {}
        }
    }

    let live_fraction = live_active_frames as f32 / active_frames as f32;
    assert!(
        live_fraction > 0.8,
        "{path}: only {live_fraction:.2} of Active frames were live play"
    );
}

// Name-table index 67 is "Active" here, which the old hardcoded goal-replay
// code matched, collapsing live play to about one second.
#[test]
fn game_states_resolve_when_active_collides_with_goal_replay_index() {
    let path = "assets/ranked-standard-state-name-index-collision-2024-08-25.replay";
    assert_eq!(parse_replay(path).names[67], "Active");
    assert_game_states_classify_live_play(path);
}

// Countdown=52, Active=53, PostGoalScored=67, ReplayPlayback=69.
#[test]
fn game_states_resolve_when_active_collides_with_countdown_index() {
    assert_game_states_classify_live_play(
        "assets/replay-format-2026-06-02-v868-32-net11-worldcup-ball.replay",
    );
}

// Countdown=1, Active=2, PostGoalScored=3.
#[test]
fn game_states_resolve_in_small_name_table() {
    assert_game_states_classify_live_play(
        "assets/replay-format-2016-07-21-v868-12-net-none-lan.replay",
    );
}
