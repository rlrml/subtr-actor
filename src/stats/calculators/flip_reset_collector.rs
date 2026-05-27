use super::*;

impl Collector for FlipResetTracker {
    fn process_frame(
        &mut self,
        processor: &dyn ProcessorView,
        frame: &boxcars::Frame,
        frame_number: usize,
        _current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        self.on_frame(processor, frame, frame_number)?;
        Ok(TimeAdvance::NextFrame)
    }
}
