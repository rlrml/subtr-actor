use crate::*;

/// A struct which decorates a [`Collector`] implementation with a target frame
/// duration, in order to control the frame rate of the replay processing. If a
/// frame is processed and the next desired frame time is before the
/// `target_frame_duration`, [`TimeAdvance::Time`] is used to ensure the next
/// frame will only be processed after the `target_frame_duration` has passed.
pub struct FrameRateDecorator<'a, C> {
    collector: &'a mut C,
    target_frame_duration: f32,
}

impl<'a, C> FrameRateDecorator<'a, C> {
    /// Constructs a new [`FrameRateDecorator`] instance with a given target
    /// frame duration and underlying [`Collector`] reference.
    ///
    /// # Arguments
    ///
    /// * `target_frame_duration`: The target duration for each frame in seconds.
    /// * `collector`: A mutable reference to the underlying [`Collector`] instance.
    pub fn new(target_frame_duration: f32, collector: &'a mut C) -> Self {
        Self {
            collector,
            target_frame_duration,
        }
    }

    /// Constructs a new [`FrameRateDecorator`] instance with a desired frames
    /// per second (fps) rate and underlying [`Collector`] reference. The target
    /// frame duration is computed as the reciprocal of the fps value.
    ///
    /// # Arguments
    ///
    /// * `fps`: The desired frame rate in frames per second.
    /// * `collector`: A mutable reference to the underlying [`Collector`] instance.
    pub fn new_from_fps(fps: f32, collector: &'a mut C) -> Self {
        Self::new(1.0 / fps, collector)
    }
}

impl<'a, C: Collector> Collector for FrameRateDecorator<'a, C> {
    /// Processes the given frame, delegating to the underlying [`Collector`]'s
    /// [`process_frame`](Collector::process_frame) method, then adjusts the
    /// returned [`TimeAdvance`] based on the target frame duration.
    ///
    /// If the original [`TimeAdvance`] was [`TimeAdvance::NextFrame`] or a
    /// [`TimeAdvance::Time`] less than the target frame duration, this method
    /// returns a [`TimeAdvance::Time`] equal to the current time plus the
    /// target frame duration.
    ///
    /// If the original [`TimeAdvance`] was a [`TimeAdvance::Time`] greater than
    /// the target frame duration, this method returns the original
    /// [`TimeAdvance::Time`].
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
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
