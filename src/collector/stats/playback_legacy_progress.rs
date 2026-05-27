use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub fn into_legacy_replay_stats_timeline_with_progress<F>(
        self,
        frame_interval: usize,
        mut on_progress: F,
    ) -> SubtrActorResult<ReplayStatsTimeline>
    where
        F: FnMut(usize, usize) -> SubtrActorResult<()>,
    {
        let frame_interval = frame_interval.max(1);
        let total_frames = self.frames.len();
        on_progress(0, total_frames)?;
        let frames = self
            .frames
            .iter()
            .enumerate()
            .map(|(frame_index, frame)| {
                let replay_frame = self.replay_stats_frame(frame)?;
                let processed_frames = frame_index + 1;
                if processed_frames == total_frames
                    || processed_frames.is_multiple_of(frame_interval)
                {
                    on_progress(processed_frames, total_frames)?;
                }
                Ok(replay_frame)
            })
            .collect::<SubtrActorResult<Vec<_>>>()?;
        self.to_replay_stats_timeline_with_frames(frames)
    }

    #[deprecated(
        note = "use into_legacy_replay_stats_timeline_with_progress for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn into_stats_timeline_with_progress<F>(
        self,
        frame_interval: usize,
        on_progress: F,
    ) -> SubtrActorResult<ReplayStatsTimeline>
    where
        F: FnMut(usize, usize) -> SubtrActorResult<()>,
    {
        self.into_legacy_replay_stats_timeline_with_progress(frame_interval, on_progress)
    }
}
