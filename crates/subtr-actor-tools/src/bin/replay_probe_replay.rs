use subtr_actor::{ReplayData, ReplayDataCollector};

pub(crate) fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"));
    boxcars::ParserBuilder::new(&data[..])
        .must_parse_network_data()
        .on_error_check_crc()
        .parse()
        .unwrap_or_else(|error| panic!("failed to parse {path}: {error}"))
}

pub(crate) fn collect_replay_data(path: &str) -> ReplayData {
    let replay = parse_replay(path);
    ReplayDataCollector::new()
        .get_replay_data(&replay)
        .unwrap_or_else(|error| panic!("failed to collect replay data for {path}: {error:?}"))
}
