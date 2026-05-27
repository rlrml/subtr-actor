use std::path::Path;

use reqwest::blocking::Client;
use serde_json::{json, Value};
use subtr_actor::playlist_generation::PlaylistManifestReplayLocator;

use super::config::Config;
use super::source_api::download_ballchasing_replay;
use super::source_ids::{ballchasing_api_key, normalize_ballchasing_id};
use super::source_types::{BallchasingReplaySummary, ReplaySourceInput};

pub(crate) fn add_ballchasing_source(
    index: usize,
    summary: BallchasingReplaySummary,
    config: &Config,
    client: &Client,
    cache_dir: &Path,
    api_key: &mut Option<String>,
    sources: &mut Vec<ReplaySourceInput>,
) -> anyhow::Result<()> {
    let replay_id = normalize_ballchasing_id(&summary.id);
    let cache_path = cache_dir.join(format!("ballchasing-{replay_id}.replay"));
    if !cache_path.exists() {
        let key = match &api_key {
            Some(key) => key,
            None => {
                *api_key = Some(ballchasing_api_key()?);
                api_key.as_ref().expect("api key just set")
            }
        };
        download_ballchasing_replay(client, key, &replay_id, &cache_path)?;
        if index + 1 < config.count {
            std::thread::sleep(config.download_delay);
        }
    }

    let label = summary
        .replay_title
        .clone()
        .unwrap_or_else(|| format!("Ballchasing {replay_id}"));
    sources.push(ReplaySourceInput {
        source_id: format!("ballchasing:{replay_id}"),
        locator: PlaylistManifestReplayLocator::ballchasing(
            replay_id,
            cache_path.display().to_string(),
        ),
        bytes_path: cache_path,
        label,
        meta: serde_json::to_value(summary_meta(&summary))?,
    });
    Ok(())
}

fn summary_meta(summary: &BallchasingReplaySummary) -> Value {
    json!({
        "source": "ballchasing",
        "ballchasingId": summary.id,
        "title": summary.replay_title,
        "date": summary.date,
        "playlistId": summary.playlist_id,
        "playlistName": summary.playlist_name,
        "duration": summary.duration,
    })
}
