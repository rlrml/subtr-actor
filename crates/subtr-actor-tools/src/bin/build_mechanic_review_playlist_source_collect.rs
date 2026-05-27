use anyhow::Context;
use reqwest::blocking::Client;
use serde_json::json;
use subtr_actor::playlist_generation::PlaylistManifestReplayLocator;

use super::config::Config;
use super::source_api::search_ballchasing_replays;
use super::source_ballchasing::add_ballchasing_source;
use super::source_ids::{ballchasing_api_key, load_ids_file, normalize_ballchasing_id};
use super::source_types::{BallchasingReplaySummary, ReplaySourceInput};

pub(crate) fn collect_sources(config: &Config) -> anyhow::Result<Vec<ReplaySourceInput>> {
    std::fs::create_dir_all(&config.cache_dir)
        .with_context(|| format!("failed to create cache dir {}", config.cache_dir.display()))?;
    let cache_dir = std::fs::canonicalize(&config.cache_dir).with_context(|| {
        format!(
            "failed to canonicalize cache dir {}",
            config.cache_dir.display()
        )
    })?;
    let client = Client::new();

    let mut ids = config.ids.clone();
    if let Some(ids_file) = &config.ids_file {
        ids.extend(load_ids_file(ids_file)?);
    }

    let summaries = if ids.is_empty() && config.replay_paths.is_empty() {
        let api_key = ballchasing_api_key()?;
        search_ballchasing_replays(&client, &api_key, config)?
    } else {
        ids.into_iter()
            .map(|id| BallchasingReplaySummary {
                id: normalize_ballchasing_id(&id),
                replay_title: None,
                date: None,
                playlist_id: None,
                playlist_name: None,
                duration: None,
            })
            .collect()
    };

    let mut sources = Vec::new();
    let mut api_key = None;
    for (index, summary) in summaries.into_iter().enumerate() {
        add_ballchasing_source(
            index,
            summary,
            config,
            &client,
            &cache_dir,
            &mut api_key,
            &mut sources,
        )?;
    }
    for path in &config.replay_paths {
        let canonical = std::fs::canonicalize(path)
            .with_context(|| format!("failed to canonicalize replay path {}", path.display()))?;
        sources.push(ReplaySourceInput {
            source_id: format!("path:{}", canonical.display()),
            locator: PlaylistManifestReplayLocator::path(canonical.display().to_string()),
            bytes_path: canonical.clone(),
            label: canonical
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("local replay")
                .to_owned(),
            meta: json!({ "source": "local" }),
        });
    }
    Ok(sources)
}
