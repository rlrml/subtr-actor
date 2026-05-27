use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub fn into_legacy_replay_stats_timeline(self) -> SubtrActorResult<ReplayStatsTimeline> {
        self.to_legacy_replay_stats_timeline()
    }

    #[deprecated(
        note = "use into_legacy_replay_stats_timeline for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn into_stats_timeline(self) -> SubtrActorResult<ReplayStatsTimeline> {
        self.into_legacy_replay_stats_timeline()
    }

    pub fn to_legacy_replay_stats_timeline(&self) -> SubtrActorResult<ReplayStatsTimeline> {
        self.to_replay_stats_timeline_with_frames(
            self.frames
                .iter()
                .map(|frame| self.replay_stats_frame(frame))
                .collect::<SubtrActorResult<Vec<_>>>()?,
        )
    }

    #[deprecated(
        note = "use to_legacy_replay_stats_timeline for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn to_stats_timeline(&self) -> SubtrActorResult<ReplayStatsTimeline> {
        self.to_legacy_replay_stats_timeline()
    }

    pub(crate) fn into_replay_stats_timeline_with_frames(
        self,
        frames: Vec<ReplayStatsFrame>,
    ) -> SubtrActorResult<ReplayStatsTimeline> {
        self.to_replay_stats_timeline_with_frames(frames)
    }

    pub(in crate::collector::stats::playback) fn to_replay_stats_timeline_with_frames(
        &self,
        frames: Vec<ReplayStatsFrame>,
    ) -> SubtrActorResult<ReplayStatsTimeline> {
        Ok(ReplayStatsTimeline {
            config: self.timeline_config(),
            replay_meta: self.replay_meta.clone(),
            events: self.timeline_event_sets_typed()?,
            frames,
        })
    }
}

impl CapturedStatsData<ReplayStatsFrame> {
    pub fn into_legacy_replay_stats_timeline(self) -> SubtrActorResult<ReplayStatsTimeline> {
        let CapturedStatsData {
            replay_meta,
            config,
            modules,
            frames,
        } = self;
        CapturedStatsData::<StatsSnapshotFrame> {
            replay_meta,
            config,
            modules,
            frames: Vec::new(),
        }
        .into_replay_stats_timeline_with_frames(frames)
    }

    #[deprecated(
        note = "use into_legacy_replay_stats_timeline for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn into_replay_stats_timeline(self) -> SubtrActorResult<ReplayStatsTimeline> {
        self.into_legacy_replay_stats_timeline()
    }
}
