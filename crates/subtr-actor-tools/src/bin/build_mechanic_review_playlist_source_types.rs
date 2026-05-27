use std::path::PathBuf;

use serde::Deserialize;
use serde_json::Value;
use subtr_actor::playlist_generation::PlaylistManifestReplayLocator;

#[derive(Debug, Clone)]
pub(crate) struct ReplaySourceInput {
    pub(crate) source_id: String,
    pub(crate) locator: PlaylistManifestReplayLocator,
    pub(crate) bytes_path: PathBuf,
    pub(crate) label: String,
    pub(crate) meta: Value,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BallchasingReplayList {
    pub(crate) list: Vec<BallchasingReplaySummary>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BallchasingReplaySummary {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) replay_title: Option<String>,
    #[serde(default)]
    pub(crate) date: Option<String>,
    #[serde(default)]
    pub(crate) playlist_id: Option<String>,
    #[serde(default)]
    pub(crate) playlist_name: Option<String>,
    #[serde(default)]
    pub(crate) duration: Option<f32>,
}
