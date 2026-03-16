use std::path::Path;
use subtr_actor::collector::replay_data::{BallFrame, ReplayDataCollector};

fn parse_replay(path: &str) -> boxcars::Replay {
    let replay_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let data = std::fs::read(&replay_path)
        .unwrap_or_else(|_| panic!("Failed to read replay file: {}", replay_path.display()));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay: {}", replay_path.display()))
}

#[test]
fn kickoff_ball_frames_are_present_before_first_touch() {
    let replay = parse_replay("assets/replays/rlcs.replay");
    let replay_data = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .expect("Failed to collect replay data");

    let first_touch_time = replay_data
        .touch_events
        .first()
        .map(|event| event.time)
        .expect("Expected replay to contain at least one touch event");

    let pre_touch_ball_frames = replay_data
        .frame_data
        .metadata_frames
        .iter()
        .zip(replay_data.frame_data.ball_data.frames().iter())
        .take_while(|(metadata, _)| metadata.time < first_touch_time);

    let non_empty_pre_touch_frames = pre_touch_ball_frames
        .filter(|(_, frame)| matches!(frame, BallFrame::Data { .. }))
        .count();

    assert!(
        non_empty_pre_touch_frames > 0,
        "Expected kickoff frames before the first touch to retain ball position data"
    );
}
