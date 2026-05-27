use super::builtins::*;
use super::traits::*;
use crate::collector::{Collector, TimeAdvance};
use crate::*;
use ::ndarray;
use boxcars;

#[path = "collector_default.rs"]
mod collector_default;
#[path = "collector_headers.rs"]
mod collector_headers;
#[path = "collector_lifecycle.rs"]
mod collector_lifecycle;
#[path = "collector_process.rs"]
mod collector_process;
#[path = "collector_registry.rs"]
mod collector_registry;

pub use collector_headers::{NDArrayColumnHeaders, ReplayMetaWithHeaders};

/// Collects replay frames into a dense 2D feature matrix.
pub struct NDArrayCollector<F> {
    feature_adders: FeatureAdders<F>,
    player_feature_adders: PlayerFeatureAdders<F>,
    data: Vec<F>,
    replay_meta: Option<ReplayMeta>,
    frames_added: usize,
}
