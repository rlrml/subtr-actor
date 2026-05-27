use super::collector_legacy::StatsTimelineCollector;
use crate::collector::frame_resolution::{StatsFramePersistenceController, StatsFrameResolution};
use crate::*;

impl StatsTimelineCollector {
    pub fn with_frame_resolution(mut self, resolution: StatsFrameResolution) -> Self {
        self.frame_persistence = StatsFramePersistenceController::new(resolution);
        self
    }

    pub fn get_legacy_replay_stats_timeline(
        mut self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<ReplayStatsTimeline> {
        let mut processor = ReplayProcessor::new(replay)?;
        processor.process(&mut self)?;
        self.into_legacy_replay_stats_timeline()
    }

    #[deprecated(
        note = "use get_legacy_replay_stats_timeline for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn get_replay_data(
        self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<ReplayStatsTimeline> {
        self.get_legacy_replay_stats_timeline(replay)
    }

    #[deprecated(
        note = "use into_legacy_timeline for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn into_timeline(self) -> ReplayStatsTimeline {
        self.into_legacy_timeline()
    }

    pub fn into_legacy_timeline(self) -> ReplayStatsTimeline {
        self.into_legacy_replay_stats_timeline()
            .expect("analysis-node timeline collector should build typed stats frames")
    }
}
