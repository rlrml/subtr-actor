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
#[path = "build_mechanic_review_playlist_source_items.rs"]
mod source_items;
#[path = "build_mechanic_review_playlist_source_parse.rs"]
mod source_parse;
#[path = "build_mechanic_review_playlist_source_types.rs"]
mod source_types;

use config::parse_args;
use manifest::{build_manifest, write_manifest};

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
