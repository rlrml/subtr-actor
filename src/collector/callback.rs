use crate::{Collector, ReplayProcessor, SubtrActorResult, TimeAdvance};
use boxcars;

/// A lightweight collector that invokes a callback at a configurable frame cadence.
///
/// This is useful for side effects that should observe replay traversal without
/// owning replay-derived state, such as progress reporting, instrumentation, or
/// debugging hooks.
pub struct CallbackCollector<C> {
    callback: C,
    frame_interval: usize,
}

impl<C> CallbackCollector<C> {
    /// Returns the configured callback cadence in processed frames.
    pub fn frame_interval(&self) -> usize {
        self.frame_interval
    }
}

impl<C> CallbackCollector<C>
where
    C: FnMut(&boxcars::Frame, usize, f32) -> SubtrActorResult<()>,
{
    /// Creates a collector that invokes the callback for every processed frame.
    pub fn new(callback: C) -> Self {
        Self::with_frame_interval(callback, 1)
    }

    /// Creates a collector that invokes the callback every `frame_interval`
    /// processed frames.
    ///
    /// A `frame_interval` of `0` is normalized to `1`.
    pub fn with_frame_interval(callback: C, frame_interval: usize) -> Self {
        Self {
            callback,
            frame_interval: frame_interval.max(1),
        }
    }
}

impl<C> Collector for CallbackCollector<C>
where
    C: FnMut(&boxcars::Frame, usize, f32) -> SubtrActorResult<()>,
{
    fn process_frame(
        &mut self,
        _processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        if frame_number.is_multiple_of(self.frame_interval) {
            (self.callback)(frame, frame_number, current_time)?;
        }

        Ok(TimeAdvance::NextFrame)
    }
}
