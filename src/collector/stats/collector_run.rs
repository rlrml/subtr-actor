use crate::{ReplayProcessor, SubtrActorError, SubtrActorErrorVariant, SubtrActorResult};

use super::super::playback::CapturedStatsData;
use super::super::types::CollectedStats;
use super::{FrameTransform, SampleMode, StatsCollector};

impl<T, F> StatsCollector<T, F> {
    pub fn get_stats(mut self, replay: &boxcars::Replay) -> SubtrActorResult<CollectedStats>
    where
        F: FrameTransform<Output = T>,
    {
        self.sample_mode = SampleMode::Aggregate;
        let mut processor = ReplayProcessor::new(replay)?;
        processor.process(&mut self)?;
        if self.replay_meta.is_none() {
            self.replay_meta = Some(processor.get_replay_meta()?);
        }
        self.into_stats()
    }

    pub fn get_captured_data(
        mut self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<CapturedStatsData<T>>
    where
        F: FrameTransform<Output = T>,
    {
        let mut processor = ReplayProcessor::new(replay)?;
        processor.process(&mut self)?;
        if self.replay_meta.is_none() {
            self.replay_meta = Some(processor.get_replay_meta()?);
        }
        self.into_captured_data()
    }

    pub fn into_stats(self) -> SubtrActorResult<CollectedStats> {
        let replay_meta = self
            .replay_meta
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::CouldNotBuildReplayMeta))?;
        Ok(CollectedStats {
            replay_meta,
            modules: self.modules.collected_modules(&self.graph)?,
        })
    }

    pub fn into_captured_data(self) -> SubtrActorResult<CapturedStatsData<T>> {
        let replay_meta = self
            .replay_meta
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::CouldNotBuildReplayMeta))?;
        Ok(CapturedStatsData {
            replay_meta: replay_meta.clone(),
            config: self.modules.snapshot_config_json(&self.graph)?,
            modules: self.modules.modules_json(&self.graph)?,
            frames: self.captured_frames.unwrap_or_default(),
        })
    }
}
