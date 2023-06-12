use crate::*;

pub struct FrameRateDecorator<'a, C> {
    collector: &'a mut C,
    target_frame_duration: f32,
}

impl<'a, C> FrameRateDecorator<'a, C> {
    pub fn new(target_frame_duration: f32, collector: &'a mut C) -> Self {
        Self {
            collector,
            target_frame_duration,
        }
    }

    pub fn new_from_fps(fps: f32, collector: &'a mut C) -> Self {
        Self::new(1.0 / fps, collector)
    }
}

impl<'a, C: Collector> Collector for FrameRateDecorator<'a, C> {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> BoxcarsResult<TimeAdvance> {
        let original_advance =
            self.collector
                .process_frame(processor, frame, frame_number, current_time)?;

        let next_target = current_time + self.target_frame_duration;

        let next_target_time = match original_advance {
            TimeAdvance::NextFrame => TimeAdvance::Time(next_target),
            TimeAdvance::Time(t) => {
                TimeAdvance::Time(f32::max(t, current_time + self.target_frame_duration))
            }
        };

        Ok(next_target_time)
    }
}
