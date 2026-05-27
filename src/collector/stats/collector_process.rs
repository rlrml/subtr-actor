use crate::collector::frame_resolution::FinalStatsFrameAction;
use crate::{Collector, FrameInfo, FrameInput, ProcessorView, SubtrActorResult, TimeAdvance};

use super::{FrameTransform, SampleMode, StatsCollector};

impl<T, F> Collector for StatsCollector<T, F>
where
    F: FrameTransform<Output = T>,
{
    fn process_frame(
        &mut self,
        processor: &dyn ProcessorView,
        _frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        self.refresh_replay_meta(processor)?;

        let dt = self
            .last_sample_time
            .map(|last_time| (current_time - last_time).max(0.0))
            .unwrap_or(0.0);
        let frame_input = match self.sample_mode {
            SampleMode::Aggregate => {
                self.aggregate_frame_input(processor, frame_number, current_time, dt)
            }
            SampleMode::Timeline => FrameInput::timeline(processor, frame_number, current_time, dt),
        };
        self.graph.evaluate_with_state(&frame_input)?;
        self.maybe_capture_current_frame(frame_number, current_time)?;
        self.finish_frame(processor, current_time);
        Ok(TimeAdvance::NextFrame)
    }

    fn finish_replay(&mut self, _processor: &dyn ProcessorView) -> SubtrActorResult<()> {
        self.graph.finish()?;
        let Some(replay_meta) = self.replay_meta.as_ref().cloned() else {
            return Ok(());
        };
        let Some(_) = self.graph.state::<FrameInfo>() else {
            return Ok(());
        };
        let mut final_snapshot = self.modules.snapshot_frame(&self.graph, &replay_meta)?;
        if self.captured_frames.is_some() {
            match self
                .frame_persistence
                .final_frame_action(final_snapshot.frame_number, final_snapshot.time)
            {
                Some(FinalStatsFrameAction::Append { dt }) => {
                    final_snapshot.dt = dt;
                    self.capture_frame_snapshot(&replay_meta, final_snapshot)?;
                }
                Some(FinalStatsFrameAction::ReplaceLast { dt }) => {
                    final_snapshot.dt = dt;
                    self.replace_last_frame_snapshot(&replay_meta, final_snapshot)?;
                }
                None => {}
            }
        }
        Ok(())
    }
}
