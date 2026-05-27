use super::{PlaybackBound, PlaylistManifestPage, PlaylistPlaybackOptions};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

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
