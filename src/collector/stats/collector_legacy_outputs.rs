use serde_json::Value;

use crate::{ReplayStatsTimeline, SubtrActorResult};

use super::super::playback::{StatsSnapshotData, StatsSnapshotFrame};
use super::super::types::serialize_to_json_value;
use super::transform::ReplayStatsFrameTransform;
use super::{FrameTransform, IdentityFrameTransform, StatsCollector};

impl StatsCollector<StatsSnapshotFrame, IdentityFrameTransform> {
    pub fn get_snapshot_data(self, replay: &boxcars::Replay) -> SubtrActorResult<StatsSnapshotData>
    where
        IdentityFrameTransform: FrameTransform<Output = StatsSnapshotFrame>,
    {
        self.capture_frames().get_captured_data(replay)
    }

    /// Collect the legacy full per-frame stats timeline as JSON.
    ///
    /// This serializes cumulative team/player partial sums on every captured
    /// frame. Prefer `StatsTimelineEventCollector` for compact event-backed
    /// timeline transfer.
    pub fn get_legacy_stats_timeline_value(
        self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<Value> {
        serialize_to_json_value(&self.get_legacy_replay_stats_timeline(replay)?)
    }

    /// Collect the legacy full per-frame stats timeline.
    ///
    /// This preserves the pre-event-transfer snapshot shape for compatibility
    /// and parity checks.
    pub fn get_legacy_replay_stats_timeline(
        self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<ReplayStatsTimeline> {
        self.with_frame_transform(ReplayStatsFrameTransform)
            .capture_frames()
            .get_captured_data(replay)?
            .into_legacy_replay_stats_timeline()
    }

    #[deprecated(
        note = "use get_legacy_stats_timeline_value for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn get_stats_timeline_value(self, replay: &boxcars::Replay) -> SubtrActorResult<Value> {
        self.get_legacy_stats_timeline_value(replay)
    }

    #[deprecated(
        note = "use get_legacy_replay_stats_timeline for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn get_replay_stats_timeline(
        self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<ReplayStatsTimeline> {
        self.get_legacy_replay_stats_timeline(replay)
    }

    pub fn into_snapshot_data(self) -> SubtrActorResult<StatsSnapshotData> {
        self.into_captured_data()
    }

    pub fn into_legacy_stats_timeline_value(self) -> SubtrActorResult<Value> {
        self.into_snapshot_data()?.to_legacy_stats_timeline_value()
    }

    pub fn into_legacy_replay_stats_timeline(self) -> SubtrActorResult<ReplayStatsTimeline> {
        self.into_snapshot_data()?
            .into_legacy_replay_stats_timeline()
    }

    #[deprecated(
        note = "use into_legacy_stats_timeline_value for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn into_stats_timeline_value(self) -> SubtrActorResult<Value> {
        self.into_legacy_stats_timeline_value()
    }

    #[deprecated(
        note = "use into_legacy_replay_stats_timeline for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn into_replay_stats_timeline(self) -> SubtrActorResult<ReplayStatsTimeline> {
        self.into_legacy_replay_stats_timeline()
    }
}
