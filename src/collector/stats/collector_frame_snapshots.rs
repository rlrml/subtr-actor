use crate::{ProcessorView, ReplayMeta, StatsSnapshotFrame, SubtrActorResult};

use super::{FrameTransform, StatsCollector};

impl<T, F> StatsCollector<T, F> {
    pub(super) fn capture_frame_snapshot(
        &mut self,
        replay_meta: &ReplayMeta,
        frame: StatsSnapshotFrame,
    ) -> SubtrActorResult<()>
    where
        F: FrameTransform<Output = T>,
    {
        if let Some(frames) = &mut self.captured_frames {
            frames.push(self.frame_transform.transform(replay_meta, frame)?);
        }
        Ok(())
    }

    pub(super) fn replace_last_frame_snapshot(
        &mut self,
        replay_meta: &ReplayMeta,
        frame: StatsSnapshotFrame,
    ) -> SubtrActorResult<()>
    where
        F: FrameTransform<Output = T>,
    {
        if let Some(frames) = &mut self.captured_frames {
            if let Some(last_frame) = frames.last_mut() {
                *last_frame = self.frame_transform.transform(replay_meta, frame)?;
            }
        }
        Ok(())
    }

    pub(super) fn refresh_replay_meta(
        &mut self,
        processor: &dyn ProcessorView,
    ) -> SubtrActorResult<()> {
        let player_count = processor.player_count();
        if self.last_replay_meta_player_count == Some(player_count) {
            return Ok(());
        }

        let replay_meta = processor.get_replay_meta()?;
        self.graph.on_replay_meta(&replay_meta)?;
        self.replay_meta = Some(replay_meta);
        self.last_replay_meta_player_count = Some(player_count);
        Ok(())
    }
}
