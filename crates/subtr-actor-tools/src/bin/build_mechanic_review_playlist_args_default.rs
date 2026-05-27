use std::path::PathBuf;

use super::args::Args;
use super::constants::{
    DEFAULT_AFTER_SECONDS, DEFAULT_BEFORE_SECONDS, DEFAULT_COUNT, DEFAULT_DOWNLOAD_DELAY_MS,
    DEFAULT_GOAL_LOOKAHEAD_SECONDS, DEFAULT_GOAL_TAIL_SECONDS, DEFAULT_MIN_CLIP_SECONDS,
    DEFAULT_MIN_CONFIDENCE, DEFAULT_PLAYLIST,
};

impl Default for Args {
    fn default() -> Self {
        Self {
            ids: Vec::new(),
            replay_paths: Vec::new(),
            ids_file: None,
            output: None,
            cache_dir: PathBuf::from(".cache/mechanic-review-replays"),
            count: DEFAULT_COUNT,
            playlist: DEFAULT_PLAYLIST.to_owned(),
            sort_by: "replay-date".to_owned(),
            sort_dir: "desc".to_owned(),
            query_params: Vec::new(),
            min_confidence: DEFAULT_MIN_CONFIDENCE,
            before_seconds: DEFAULT_BEFORE_SECONDS,
            after_seconds: DEFAULT_AFTER_SECONDS,
            goal_lookahead_seconds: DEFAULT_GOAL_LOOKAHEAD_SECONDS,
            goal_tail_seconds: DEFAULT_GOAL_TAIL_SECONDS,
            min_clip_seconds: DEFAULT_MIN_CLIP_SECONDS,
            max_items: None,
            download_delay_ms: DEFAULT_DOWNLOAD_DELAY_MS,
            mechanic: Vec::new(),
            mechanics: Vec::new(),
            list_mechanics: false,
        }
    }
}
