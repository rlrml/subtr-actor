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

    let build_version = replay
        .properties
        .iter()
        .find(|(key, _)| key == "BuildVersion")
        .and_then(|(_, value)| value.as_string());
    let num_frames = replay
        .properties
        .iter()
        .find(|(key, _)| key == "NumFrames")
        .and_then(|(_, value)| value.as_i32());
    let match_type = replay
        .properties
        .iter()
        .find(|(key, _)| key == "MatchType")
        .and_then(|(_, value)| value.as_string());

    println!(
        "replay={path} major_version={} minor_version={} net_version={:?} build_version={:?} match_type={:?} num_frames={:?}",
        replay.major_version,
        replay.minor_version,
        replay.net_version,
        build_version,
        match_type,
        num_frames
    );
}
