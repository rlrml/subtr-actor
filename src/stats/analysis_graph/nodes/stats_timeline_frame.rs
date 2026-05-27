use super::*;
use crate::stats::calculators::*;
use crate::*;

#[path = "stats_timeline_frame_dependencies.rs"]
mod stats_timeline_frame_dependencies;
#[path = "stats_timeline_frame_node.rs"]
mod stats_timeline_frame_node;
#[path = "stats_timeline_frame_player.rs"]
mod stats_timeline_frame_player;
#[path = "stats_timeline_frame_snapshot.rs"]
mod stats_timeline_frame_snapshot;
#[path = "stats_timeline_frame_team.rs"]
mod stats_timeline_frame_team;
#[path = "stats_timeline_frame_team_helpers.rs"]
mod stats_timeline_frame_team_helpers;

#[derive(Debug, Clone, Default)]
pub struct StatsTimelineFrameState {
    pub frame: Option<ReplayStatsFrame>,
}

pub struct StatsTimelineFrameNode {
    pub(super) replay_meta: Option<ReplayMeta>,
    pub(super) state: StatsTimelineFrameState,
}

impl StatsTimelineFrameNode {
    pub fn new() -> Self {
        Self {
            replay_meta: None,
            state: StatsTimelineFrameState::default(),
        }
    }

    pub(super) fn replay_meta(&self) -> SubtrActorResult<&ReplayMeta> {
        self.replay_meta.as_ref().ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
                "missing ReplayMeta state while building timeline frame".to_owned(),
            ))
        })
    }
}

impl Default for StatsTimelineFrameNode {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(StatsTimelineFrameNode::new())
}
