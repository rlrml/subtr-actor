use std::path::Path;

use anyhow::Context;
use reqwest::blocking::Client;

use super::config::Config;
use super::constants::BALLCHASING_API_BASE_URL;
use super::source_types::{BallchasingReplayList, BallchasingReplaySummary};

pub(crate) fn search_ballchasing_replays(
    client: &Client,
    api_key: &str,
    config: &Config,
) -> anyhow::Result<Vec<BallchasingReplaySummary>> {
    let mut request = client
        .get(format!("{BALLCHASING_API_BASE_URL}/replays"))
        .header("Authorization", api_key)
        .query(&[
            ("playlist", config.playlist.as_str()),
            ("count", &config.count.to_string()),
            ("sort-by", config.sort_by.as_str()),
            ("sort-dir", config.sort_dir.as_str()),
        ]);

    for (key, value) in &config.query_params {
        request = request.query(&[(key.as_str(), value.as_str())]);
    }

    let response = request
        .send()
        .context("failed to search Ballchasing replays")?
        .error_for_status()
        .context("Ballchasing replay search returned an error")?
        .json::<BallchasingReplayList>()
        .context("failed to decode Ballchasing replay search response")?;
    Ok(response.list)
}

pub(crate) fn download_ballchasing_replay(
    client: &Client,
    api_key: &str,
    replay_id: &str,
    path: &Path,
) -> anyhow::Result<()> {
    let bytes = client
        .get(format!(
            "{BALLCHASING_API_BASE_URL}/replays/{replay_id}/file"
        ))
        .header("Authorization", api_key)
        .send()
        .with_context(|| format!("failed to download Ballchasing replay {replay_id}"))?
        .error_for_status()
        .with_context(|| format!("Ballchasing replay download failed for {replay_id}"))?
        .bytes()
        .with_context(|| format!("failed to read replay bytes for {replay_id}"))?;
    std::fs::write(path, &bytes)
        .with_context(|| format!("failed to write replay cache {}", path.display()))?;
    Ok(())
}
