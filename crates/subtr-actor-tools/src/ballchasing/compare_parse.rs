use std::path::Path;

use anyhow::Context;

pub fn parse_replay_bytes(data: &[u8]) -> anyhow::Result<boxcars::Replay> {
    boxcars::ParserBuilder::new(data)
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .context("Failed to parse replay")
}

pub fn parse_replay_file(path: impl AsRef<Path>) -> anyhow::Result<boxcars::Replay> {
    let path = path.as_ref();
    let data = std::fs::read(path)
        .with_context(|| format!("Failed to read replay file: {}", path.display()))?;
    parse_replay_bytes(&data).with_context(|| format!("Failed to parse replay: {}", path.display()))
}
