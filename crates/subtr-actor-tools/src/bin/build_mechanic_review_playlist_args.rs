use std::path::PathBuf;

use clap::Parser;

use super::args_query::parse_query_param;
use super::constants::{
    DEFAULT_AFTER_SECONDS, DEFAULT_BEFORE_SECONDS, DEFAULT_COUNT, DEFAULT_DOWNLOAD_DELAY_MS,
    DEFAULT_GOAL_LOOKAHEAD_SECONDS, DEFAULT_GOAL_TAIL_SECONDS, DEFAULT_MIN_CLIP_SECONDS,
    DEFAULT_MIN_CONFIDENCE, DEFAULT_PLAYLIST,
};

#[derive(Debug, Parser)]
#[command(about = "Build a mechanic-review playlist from heuristic mechanic events.")]
pub(crate) struct Args {
    /// Add one Ballchasing replay id or URL.
    #[arg(long = "id", value_name = "ballchasing-id-or-url")]
    pub(crate) ids: Vec<String>,
    /// Add Ballchasing replay ids or URLs from a file, one per line.
    #[arg(long, value_name = "path")]
    pub(crate) ids_file: Option<PathBuf>,
    /// Add a local .replay file.
    #[arg(long = "replay-path", value_name = "path")]
    pub(crate) replay_paths: Vec<PathBuf>,
    /// Write playlist JSON to path. Defaults to stdout.
    #[arg(short, long, value_name = "path")]
    pub(crate) output: Option<PathBuf>,

    /// Replay cache directory.
    #[arg(
        long,
        value_name = "path",
        default_value = ".cache/mechanic-review-replays"
    )]
    pub(crate) cache_dir: PathBuf,

    /// Number of Ballchasing replays to search/download when no sources are given.
    #[arg(long, default_value_t = DEFAULT_COUNT)]
    pub(crate) count: usize,

    /// Ballchasing playlist filter.
    #[arg(long, default_value = DEFAULT_PLAYLIST)]
    pub(crate) playlist: String,

    /// Ballchasing sort field.
    #[arg(long, default_value = "replay-date")]
    pub(crate) sort_by: String,

    /// Ballchasing sort direction.
    #[arg(long, default_value = "desc", value_name = "asc|desc")]
    pub(crate) sort_dir: String,

    /// Extra Ballchasing /replays query param. Repeatable.
    #[arg(long = "query", value_name = "key=value", value_parser = parse_query_param)]
    pub(crate) query_params: Vec<(String, String)>,

    /// Minimum detector confidence for scored events.
    #[arg(long, default_value_t = DEFAULT_MIN_CONFIDENCE)]
    pub(crate) min_confidence: f32,

    /// Clip lead-in before setup start.
    #[arg(long, default_value_t = DEFAULT_BEFORE_SECONDS)]
    pub(crate) before_seconds: f32,

    /// Clip tail after mechanic event.
    #[arg(long, default_value_t = DEFAULT_AFTER_SECONDS)]
    pub(crate) after_seconds: f32,

    /// Extend clips through same-team goals this many seconds after the mechanic event.
    #[arg(long, default_value_t = DEFAULT_GOAL_LOOKAHEAD_SECONDS)]
    pub(crate) goal_lookahead_seconds: f32,

    /// Clip tail after an included goal explosion.
    #[arg(long, default_value_t = DEFAULT_GOAL_TAIL_SECONDS)]
    pub(crate) goal_tail_seconds: f32,

    /// Minimum emitted clip duration, extended within replay bounds.
    #[arg(long, default_value_t = DEFAULT_MIN_CLIP_SECONDS)]
    pub(crate) min_clip_seconds: f32,

    /// Limit emitted candidates.
    #[arg(long)]
    pub(crate) max_items: Option<usize>,

    /// Delay between uncached Ballchasing downloads.
    #[arg(long, default_value_t = DEFAULT_DOWNLOAD_DELAY_MS)]
    pub(crate) download_delay_ms: u64,

    /// Include a mechanic detector. Repeatable.
    #[arg(long = "mechanic", value_name = "name")]
    pub(crate) mechanic: Vec<String>,

    /// Include comma-separated mechanic detectors.
    #[arg(long = "mechanics", value_name = "a,b,c", value_delimiter = ',')]
    pub(crate) mechanics: Vec<String>,

    /// Print supported mechanic detector names.
    #[arg(long)]
    pub(crate) list_mechanics: bool,
}
