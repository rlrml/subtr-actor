use super::collector_config::default_stats_timeline_config;
use super::collector_graph::build_legacy_timeline_graph;
use crate::collector::frame_resolution::{StatsFramePersistenceController, StatsFrameResolution};
use crate::stats::analysis_graph::{AnalysisGraph, StatsTimelineEventsState};
use crate::*;

pub struct StatsTimelineCollector {
    pub(super) graph: AnalysisGraph,
    pub(super) replay_meta: Option<ReplayMeta>,
    pub(super) last_replay_meta_player_count: Option<usize>,
    pub(super) frames: Vec<ReplayStatsFrame>,
    pub(super) last_sample_time: Option<f32>,
    pub(super) frame_persistence: StatsFramePersistenceController,
}

impl Default for StatsTimelineCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsTimelineCollector {
    /// Create the legacy full-snapshot timeline collector.
    ///
    /// This evaluates and stores cumulative team/player stat modules for every
    /// captured frame. Prefer [`StatsTimelineEventCollector`] for compact
    /// event-backed transfer.
    pub fn new() -> Self {
        let graph = build_legacy_timeline_graph();
        Self {
            graph,
            replay_meta: None,
            last_replay_meta_player_count: None,
            frames: Vec::new(),
            last_sample_time: None,
            frame_persistence: StatsFramePersistenceController::new(StatsFrameResolution::default()),
        }
    }

    fn timeline_config(&self) -> StatsTimelineConfig {
        default_stats_timeline_config()
    }

    pub fn into_legacy_replay_stats_timeline(self) -> SubtrActorResult<ReplayStatsTimeline> {
        let replay_meta = self
            .replay_meta
            .clone()
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::CouldNotBuildReplayMeta))?;
        let mut events = self
            .graph
            .state::<StatsTimelineEventsState>()
            .map(|state| state.events.clone())
            .unwrap_or_default();
        if let Some(boost) = self.graph.state::<BoostCalculator>() {
            events.boost_pickups = boost.pickup_comparison_events().to_vec();
            events.boost_ledger = boost.ledger_events().to_vec();
            events.boost_state = boost.state_events().to_vec();
        }
        Ok(ReplayStatsTimeline {
            config: self.timeline_config(),
            replay_meta,
            events,
            frames: self.frames,
        })
    }

    #[deprecated(
        note = "use into_legacy_replay_stats_timeline for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn into_replay_stats_timeline(self) -> SubtrActorResult<ReplayStatsTimeline> {
        self.into_legacy_replay_stats_timeline()
    }
}
