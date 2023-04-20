use crate::*;

static EXPECTED_FRAME_LENGTH: f32 = 1.0 / 30.0;

pub struct FrameRateDecorator<'a, C> {
    collector: &'a mut C,
    target_frame_duration: f32,
    actual_last_frame_time: Option<f32>,
    targeted_last_frame_time: f32,
}

impl<'a, C> FrameRateDecorator<'a, C> {
    pub fn new(target_frame_duration: f32, collector: &'a mut C) -> Self {
        Self {
            collector,
            target_frame_duration,
            actual_last_frame_time: None,
            targeted_last_frame_time: 0.0,
        }
    }

    pub fn new_from_fps(fps: f32, collector: &'a mut C) -> Self {
        Self::new(1.0 / fps, collector)
    }

    fn next_target_frame_time(&self) -> f32 {
        self.targeted_last_frame_time + self.target_frame_duration
    }

    fn should_make_frame(&self, frame: &boxcars::Frame) -> bool {
        if let None = self.actual_last_frame_time {
            return true;
        }
        let target = self.next_target_frame_time();
        let this_frame_inaccuracy = frame.time - target;
        let next_expected_frame_time = frame.time + EXPECTED_FRAME_LENGTH;
        let next_frame_inaccuracy = next_expected_frame_time - target;
        this_frame_inaccuracy.abs() < next_frame_inaccuracy.abs()
    }

    fn update_frame_times(&mut self, frame: &boxcars::Frame) {
        self.targeted_last_frame_time = self.next_target_frame_time();
        self.actual_last_frame_time = Some(frame.time);
    }
}

impl<'a, C: Collector> Collector for FrameRateDecorator<'a, C> {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_number: usize,
    ) -> ReplayProcessorResult<()> {
        if self.should_make_frame(frame) {
            self.collector
                .process_frame(processor, frame, frame_number)?;
            self.update_frame_times(frame);
        }
        Ok(())
    }
}

pub fn require_ball_rigid_body_exists(
    processor: &ReplayProcessor,
    _f: &boxcars::Frame,
    _: usize,
) -> Result<bool, String> {
    Ok(processor
        .get_ball_rigid_body()
        .map(|rb| !rb.sleeping)
        .unwrap_or(false))
}
