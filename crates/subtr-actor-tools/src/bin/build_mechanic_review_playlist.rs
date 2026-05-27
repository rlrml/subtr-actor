use anyhow::anyhow;
use subtr_actor::{
    playlist_generation::PlaylistManifestItem,
    stats::analysis_graph::collect_builtin_analysis_graph_for_replay, ReplayProcessor,
};

#[path = "build_mechanic_review_playlist_args.rs"]
mod args;
#[path = "build_mechanic_review_playlist_args_default.rs"]
mod args_default;
#[path = "build_mechanic_review_playlist_args_query.rs"]
mod args_query;
#[path = "build_mechanic_review_playlist_candidate.rs"]
mod candidate;
#[path = "build_mechanic_review_playlist_config.rs"]
mod config;
#[path = "build_mechanic_review_playlist_constants.rs"]
mod constants;
#[path = "build_mechanic_review_playlist_extract.rs"]
mod extract;
#[path = "build_mechanic_review_playlist_extract_ceiling.rs"]
mod extract_ceiling;
#[path = "build_mechanic_review_playlist_extract_flick.rs"]
mod extract_flick;
#[path = "build_mechanic_review_playlist_extract_flip_reset.rs"]
mod extract_flip_reset;
#[path = "build_mechanic_review_playlist_extract_movement.rs"]
mod extract_movement;
#[path = "build_mechanic_review_playlist_extract_touch.rs"]
mod extract_touch;
#[path = "build_mechanic_review_playlist_goal_scan.rs"]
mod goal_scan;
#[path = "build_mechanic_review_playlist_item.rs"]
mod item;
#[path = "build_mechanic_review_playlist_manifest.rs"]
mod manifest;
#[path = "build_mechanic_review_playlist_mechanics.rs"]
mod mechanics;
#[path = "build_mechanic_review_playlist_players.rs"]
mod players;
#[path = "build_mechanic_review_playlist_source_api.rs"]
mod source_api;
#[path = "build_mechanic_review_playlist_source_ballchasing.rs"]
mod source_ballchasing;
#[path = "build_mechanic_review_playlist_source_collect.rs"]
mod source_collect;
#[path = "build_mechanic_review_playlist_source_ids.rs"]
mod source_ids;
#[path = "build_mechanic_review_playlist_source_parse.rs"]
mod source_parse;
#[path = "build_mechanic_review_playlist_source_types.rs"]
mod source_types;

use candidate::{enforce_min_clip_duration, followup_goal_for_candidate, replay_duration_seconds};
use config::{parse_args, Config};
use extract::extract_candidates;
use goal_scan::GoalScanCollector;
use item::build_playlist_item;
use manifest::{build_manifest, write_manifest};
use mechanics::graph_node_names_for_mechanics;
use players::player_display_map;
use source_types::ReplaySourceInput;

pub(crate) fn build_items_for_source(
    source: &ReplaySourceInput,
    replay: &boxcars::Replay,
    config: &Config,
    mechanics: &[&str],
) -> anyhow::Result<Vec<PlaylistManifestItem>> {
    let graph_nodes = graph_node_names_for_mechanics(mechanics);
    let graph = collect_builtin_analysis_graph_for_replay(replay, graph_nodes).map_err(|err| {
        anyhow!(
            "failed to collect mechanic stats for {}: {err:?}",
            source.label
        )
    })?;
    let mut processor = ReplayProcessor::new(replay).map_err(|err| {
        anyhow!(
            "failed to build replay processor for {}: {err:?}",
            source.label
        )
    })?;
    processor.process(&mut GoalScanCollector).map_err(|err| {
        anyhow!(
            "failed to process replay goals for {}: {err:?}",
            source.label
        )
    })?;
    let replay_meta = processor.get_replay_meta().map_err(|err| {
        anyhow!(
            "failed to read replay metadata for {}: {err:?}",
            source.label
        )
    })?;
    let replay_duration = replay_duration_seconds(replay);
    let players = player_display_map(&replay_meta);
    let candidates = extract_candidates(replay, &graph, mechanics, config)?;

    let mut items = Vec::new();
    for candidate in candidates {
        let player = candidate
            .player_id
            .as_ref()
            .and_then(|player_id| players.get(player_id));
        let start_time = (candidate.start_time - config.before_seconds).max(0.0);
        let followup_goal = followup_goal_for_candidate(&candidate, &processor.goal_events, config);
        let padded_end_time = followup_goal
            .map(|goal| goal.time + config.goal_tail_seconds)
            .unwrap_or(candidate.end_time + config.after_seconds)
            .max(candidate.end_time + config.after_seconds)
            .max(start_time);
        let (start_time, end_time) = enforce_min_clip_duration(
            start_time,
            padded_end_time,
            replay_duration,
            config.min_clip_seconds,
        );
        items.push(build_playlist_item(
            source,
            candidate,
            player,
            start_time,
            end_time,
            followup_goal,
        ));
    }

    Ok(items)
}

fn main() -> anyhow::Result<()> {
    let config = parse_args()?;
    let manifest = build_manifest(&config)?;
    write_manifest(&manifest, config.output.as_deref())?;
    eprintln!(
        "wrote {} mechanic candidates across {} replays",
        manifest.items.len(),
        manifest.replays.len()
    );
    Ok(())
}
