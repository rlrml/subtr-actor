pub mod decorator;
pub mod ndarray;
pub mod replay_data;

pub use self::ndarray::*;
pub use decorator::*;
pub use replay_data::*;

use crate::*;
use boxcars;

/// Enum used to control the progress of time during replay processing.
pub enum TimeAdvance {
    /// Move forward in time by a specified amount.
    Time(f32),
    /// Advance to the next frame.
    NextFrame,
}

/// Trait for types that collect data from a replay.
///
/// A `Collector` processes frames from a replay, potentially using a
/// [`ReplayProcessor`] for access to additional replay data and context. It
/// determines the pace of replay progression via the [`TimeAdvance`] return
/// value.
pub trait Collector {
    /// Process a single frame from a replay.
    ///
    /// # Arguments
    ///
    /// * `processor` - The [`ReplayProcessor`] providing context for the replay.
    /// * `frame` - The [`boxcars::Frame`] to process.
    /// * `frame_number` - The number of the current frame.
    /// * `current_time` - The current target time in the replay.
    ///
    /// # Returns
    ///
    /// Returns a [`TimeAdvance`] enum which determines the next step in replay
    /// progression.
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance>;

    /// Process an entire replay.
    ///
    /// # Arguments
    ///
    /// * `replay` - The [`boxcars::Replay`] to process.
    ///
    /// # Returns
    ///
    /// Returns the [`Collector`] itself, potentially modified by the processing
    /// of the replay.
    fn process_replay(mut self, replay: &boxcars::Replay) -> SubtrActorResult<Self>
    where
        Self: Sized,
    {
        ReplayProcessor::new(replay)?.process(&mut self)?;
        Ok(self)
    }
}

impl<G> Collector for G
where
    G: FnMut(&ReplayProcessor, &boxcars::Frame, usize, f32) -> SubtrActorResult<TimeAdvance>,
{
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        self(processor, frame, frame_number, current_time)
    }
}
