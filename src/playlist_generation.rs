use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum PlaybackBoundKind {
    Frame,
    Time,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PlaybackBound {
    pub kind: PlaybackBoundKind,
    pub value: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub enum PlaylistAdvanceMode {
    Auto,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub enum PlaylistEndMode {
    Stop,
    Loop,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct PlaylistPlaybackOptions {
    pub advance_mode: PlaylistAdvanceMode,
    pub end_mode: PlaylistEndMode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct PlaylistManifestReplayLocator {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub cache_path: Option<String>,
}

impl PlaylistManifestReplayLocator {
    pub fn ballchasing(id: String, cache_path: String) -> Self {
        Self {
            kind: "ballchasing".to_owned(),
            id: Some(id),
            path: None,
            cache_path: Some(cache_path),
        }
    }

    pub fn path(path: String) -> Self {
        Self {
            kind: "path".to_owned(),
            id: None,
            path: Some(path),
            cache_path: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct PlaylistManifestReplay {
    pub id: String,
    pub label: String,
    pub locator: PlaylistManifestReplayLocator,
    pub path: String,
    #[ts(type = "Record<string, unknown>")]
    pub meta: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct PlaylistManifestItem {
    pub id: String,
    pub replay: String,
    pub start: PlaybackBound,
    pub end: PlaybackBound,
    pub label: String,
    #[ts(type = "Record<string, unknown>")]
    pub meta: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct PlaylistManifestPage {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub previous: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(type = "number")]
    #[ts(optional)]
    pub total: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct PlaylistManifest {
    pub version: u32,
    pub kind: String,
    pub label: String,
    pub playback: PlaylistPlaybackOptions,
    pub replays: Vec<PlaylistManifestReplay>,
    pub items: Vec<PlaylistManifestItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub page: Option<PlaylistManifestPage>,
    #[ts(type = "Record<string, unknown>")]
    pub meta: Value,
}
