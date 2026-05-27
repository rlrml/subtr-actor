use super::collector_config::default_stats_timeline_config;
use super::collector_graph::build_timeline_event_graph;
use crate::collector::frame_resolution::{StatsFramePersistenceController, StatsFrameResolution};
use crate::stats::analysis_graph::{AnalysisGraph, StatsTimelineEventsState};
use crate::*;

pub struct StatsTimelineEventCollector {
    pub(super) graph: AnalysisGraph,
    pub(super) replay_meta: Option<ReplayMeta>,
    pub(super) last_replay_meta_player_count: Option<usize>,
    pub(super) frames: Vec<ReplayStatsFrameScaffold>,
    pub(super) last_sample_time: Option<f32>,
    pub(super) frame_persistence: StatsFramePersistenceController,
}

impl Default for StatsTimelineEventCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsTimelineEventCollector {
    pub fn new() -> Self {
        Self {
            graph: build_timeline_event_graph(),
            replay_meta: None,
            last_replay_meta_player_count: None,
            frames: Vec::new(),
            last_sample_time: None,
            frame_persistence: StatsFramePersistenceController::new(StatsFrameResolution::default()),
        }
    }

    pub fn with_frame_resolution(mut self, resolution: StatsFrameResolution) -> Self {
        self.frame_persistence = StatsFramePersistenceController::new(resolution);
        self
    }

    pub fn into_replay_stats_timeline_scaffold(
        self,
    ) -> SubtrActorResult<ReplayStatsTimelineScaffold> {
        let replay_meta = self
            .replay_meta
            .clone()
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::CouldNotBuildReplayMeta))?;
        let events = self
            .graph
            .state::<StatsTimelineEventsState>()
            .map(|state| state.events.clone())
            .unwrap_or_default();
        Ok(ReplayStatsTimelineScaffold {
            config: default_stats_timeline_config(),
            replay_meta,
            events,
            frames: self.frames,
        })
    }

    pub fn get_replay_stats_timeline_scaffold(
        mut self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<ReplayStatsTimelineScaffold> {
        let mut processor = ReplayProcessor::new(replay)?;
        processor.process(&mut self)?;
        self.into_replay_stats_timeline_scaffold()
    }

    #[deprecated(
        note = "use get_replay_stats_timeline_scaffold for compact event-backed timelines"
    )]
    pub fn get_replay_data(
        self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<ReplayStatsTimelineScaffold> {
        self.get_replay_stats_timeline_scaffold(replay)
    }
}
