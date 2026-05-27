use std::path::Path;

use anyhow::{bail, Context};
use serde_json::json;
use subtr_actor::playlist_generation::{
    PlaylistAdvanceMode, PlaylistEndMode, PlaylistManifest, PlaylistManifestReplay,
    PlaylistPlaybackOptions,
};

use super::config::Config;
use super::mechanics::resolve_mechanics;
use super::source_collect::collect_sources;
use super::source_items::build_items_for_source;
use super::source_parse::parse_replay_file;

pub(crate) fn build_manifest(config: &Config) -> anyhow::Result<PlaylistManifest> {
    let mechanics = resolve_mechanics(config)?;
    let sources = collect_sources(config)?;
    if sources.is_empty() {
        bail!("no replay sources were selected");
    }

    let mut replays = Vec::new();
    let mut items = Vec::new();
    for source in &sources {
        let replay = parse_replay_file(&source.bytes_path)?;
        replays.push(PlaylistManifestReplay {
            id: source.source_id.clone(),
            label: source.label.clone(),
            path: source.bytes_path.display().to_string(),
            locator: source.locator.clone(),
            meta: source.meta.clone(),
        });
        items.extend(build_items_for_source(source, &replay, config, &mechanics)?);
        if let Some(max_items) = config.max_items {
            if items.len() >= max_items {
                items.truncate(max_items);
                break;
            }
        }
    }

    let candidate_count = items.len();

    Ok(PlaylistManifest {
        version: 1,
        kind: "mechanic-review-playlist".to_owned(),
        label: "Mechanic review candidates".to_owned(),
        playback: PlaylistPlaybackOptions {
            advance_mode: PlaylistAdvanceMode::Manual,
            end_mode: PlaylistEndMode::Stop,
        },
        replays,
        items,
        page: None,
        meta: json!({
            "mechanics": mechanics,
            "sourceReplayCount": sources.len(),
            "candidateCount": candidate_count,
            "minConfidence": config.min_confidence,
            "clipPadding": {
                "beforeSeconds": config.before_seconds,
                "afterSeconds": config.after_seconds,
                "goalLookaheadSeconds": config.goal_lookahead_seconds,
                "goalTailSeconds": config.goal_tail_seconds,
                "minClipSeconds": config.min_clip_seconds,
            },
            "generatedBy": "build_mechanic_review_playlist",
        }),
    })
}

pub(crate) fn write_manifest(
    manifest: &PlaylistManifest,
    output: Option<&Path>,
) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(manifest)?;
    match output {
        Some(path) => {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent).with_context(|| {
                        format!("failed to create output dir {}", parent.display())
                    })?;
                }
            }
            std::fs::write(path, format!("{json}\n"))
                .with_context(|| format!("failed to write playlist {}", path.display()))?;
        }
        None => println!("{json}"),
    }
    Ok(())
}
