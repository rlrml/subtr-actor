use subtr_actor::{evaluate_replay_plausibility, ReplayDataCollector};

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "assets/rlcs.replay".to_string());
    let data =
        std::fs::read(&path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"));
    let replay = boxcars::ParserBuilder::new(&data[..])
        .must_parse_network_data()
        .on_error_check_crc()
        .parse()
        .unwrap_or_else(|error| panic!("failed to parse {path}: {error}"));
    let replay_data = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .unwrap_or_else(|error| panic!("failed to collect replay data for {path}: {error:?}"));
    let report = evaluate_replay_plausibility(&replay_data);
    println!("{report:#?}");
}
