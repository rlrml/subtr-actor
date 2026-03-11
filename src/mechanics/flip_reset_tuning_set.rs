use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlipResetTuningReplay {
    pub replay_id: String,
    pub replay_path: String,
    pub exact_dodge_refresh_count: usize,
    pub date: Option<String>,
    pub title: Option<String>,
    pub uploader: Option<String>,
    pub player_names: Vec<String>,
}

impl FlipResetTuningReplay {
    pub fn replay_path_from_manifest(&self, manifest_path: &Path) -> PathBuf {
        let replay_path = Path::new(&self.replay_path);
        if replay_path.is_absolute() {
            replay_path.to_path_buf()
        } else {
            manifest_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(replay_path)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlipResetTuningManifest {
    pub playlist: String,
    pub min_rank: String,
    pub replays: Vec<FlipResetTuningReplay>,
}
