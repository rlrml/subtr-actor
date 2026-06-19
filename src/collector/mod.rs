//! The output layer: turn a processed replay into structured data, numeric
//! features, or stat timelines.
//!
//! Everything here is built on the [`Collector`] trait — the core extension
//! point. A collector observes the replay frame by frame (driven by a
//! [`ReplayProcessor`]) and decides the sampling pace
//! via the [`TimeAdvance`] it returns. Run one with
//! [`Collector::process_replay`], or share a single pass across several with
//! [`ReplayProcessor::process_all`](crate::ReplayProcessor::process_all).
//!
//! # Built-in collectors
//!
//! - [`ReplayDataCollector`] — a serde-friendly payload of frame data, metadata,
//!   and derived event streams for JSON export and playback UIs.
//! - [`NDArrayCollector`] — a dense numeric feature matrix for ML; see the
//!   [`ndarray`] submodule for the feature-adder system.
//! - Stats collectors in [`stats`] — graph-backed accumulated stats; see also
//!   [`crate::stats::timeline`] for cumulative-over-time stat timelines.
//!
//! # Wrappers
//!
//! - [`FrameRateDecorator`] downsamples any collector to a target FPS.
//! - [`CallbackCollector`] attaches side-effecting hooks (progress, debugging)
//!   to a collector.

pub mod callback;
pub mod decorator;
pub(crate) mod frame_resolution;
pub mod ndarray;
pub mod replay_data;
pub mod stats;

pub use self::ndarray::*;
pub use callback::*;
pub use decorator::*;
pub use frame_resolution::StatsFrameResolution;
pub use replay_data::*;
pub use stats::*;

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
        processor: &dyn ProcessorView,
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

    /// Finalize replay-derived state after the last frame has been processed.
    ///
    /// Collectors that aggregate state across frame boundaries can override
    /// this to flush any in-progress segment once replay traversal is complete.
    fn finish_replay(&mut self, _processor: &dyn ProcessorView) -> SubtrActorResult<()> {
        Ok(())
    }
}

impl<G> Collector for G
where
    G: FnMut(&dyn ProcessorView, &boxcars::Frame, usize, f32) -> SubtrActorResult<TimeAdvance>,
{
    fn process_frame(
        &mut self,
        processor: &dyn ProcessorView,
        frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        self(processor, frame, frame_number, current_time)
    }
}
