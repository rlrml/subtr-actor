use std::collections::HashSet;
use std::path::Path;
use subtr_actor::*;

/// Helper to parse a replay file
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

fn max_abs_player_position_from_replay_data(replay_data: &ReplayData) -> f32 {
    replay_data
        .frame_data
        .players
        .iter()
        .flat_map(|(_, player_data)| player_data.frames().iter())
        .filter_map(|frame| match frame {
            PlayerFrame::Data { rigid_body, .. } => Some(rigid_body.location),
            PlayerFrame::Empty => None,
        })
        .flat_map(|location| [location.x.abs(), location.y.abs(), location.z.abs()])
        .fold(0.0f32, f32::max)
}

fn max_abs_position_from_ndarray(
    replay: &boxcars::Replay,
    global_feature_adders: &[&str],
    player_feature_adders: &[&str],
) -> f32 {
    let collector =
        NDArrayCollector::<f32>::from_strings(global_feature_adders, player_feature_adders)
            .expect("Should create collector");
    let (meta, array) = collector
        .process_replay(replay)
        .expect("Should process replay")
        .get_meta_and_ndarray()
        .expect("Should get ndarray");

    let headers = meta.headers_vec();
    let mut max_abs_position = 0.0f32;
    for (index, header) in headers.iter().enumerate() {
        if !header.contains("position ") {
            continue;
        }
        for value in array.column(index).iter().copied() {
            max_abs_position = max_abs_position.max(value.abs());
        }
    }
    max_abs_position
}

