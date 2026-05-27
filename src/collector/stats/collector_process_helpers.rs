use crate::{FrameInput, ProcessorView, SubtrActorResult};

use super::{FrameTransform, SampleMode, StatsCollector};

impl<T, F> StatsCollector<T, F> {
    pub(super) fn aggregate_frame_input(
        &self,
        processor: &dyn ProcessorView,
        frame_number: usize,
        current_time: f32,
        dt: f32,
    ) -> FrameInput {
        FrameInput::aggregate(
            processor,
            frame_number,
            current_time,
            dt,
            self.last_demolish_count,
            self.last_boost_pad_event_count,
            self.last_touch_event_count,
            self.last_dodge_refreshed_event_count,
            self.last_player_stat_event_count,
            self.last_goal_event_count,
        )
    }

    pub(super) fn maybe_capture_current_frame(
        &mut self,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<()>
    where
        F: FrameTransform<Output = T>,
    {
        if self.captured_frames.is_none() {
            return Ok(());
        }

        let replay_meta = self
            .replay_meta
            .as_ref()
            .expect("replay metadata should be initialized before snapshotting")
            .clone();
        if let Some(emitted_dt) = self.frame_persistence.on_frame(frame_number, current_time) {
            let mut frame = self.modules.snapshot_frame(&self.graph, &replay_meta)?;
            frame.dt = emitted_dt;
            self.capture_frame_snapshot(&replay_meta, frame)?;
        }
        Ok(())
    }

    pub(super) fn finish_frame(&mut self, processor: &dyn ProcessorView, current_time: f32) {
        self.last_sample_time = Some(current_time);
        if !matches!(self.sample_mode, SampleMode::Aggregate) {
            return;
        }

        self.last_demolish_count = processor.demolishes().len();
        self.last_boost_pad_event_count = processor.boost_pad_events().len();
        self.last_touch_event_count = processor.touch_events().len();
        self.last_dodge_refreshed_event_count = processor.dodge_refreshed_events().len();
        self.last_player_stat_event_count = processor.player_stat_events().len();
        self.last_goal_event_count = processor.goal_events().len();
    }
}
