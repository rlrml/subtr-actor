pub mod decorator;
pub mod ndarray;
pub mod replay_data;

pub use self::ndarray::*;
pub use decorator::*;
pub use replay_data::*;

use crate::{processor::ReplayProcessor, ReplayProcessorResult};
use boxcars;

// collector is just something that can process a frame
// each one has a time and deltatime
// resolution downsampler wrapper
// implements Collector, takes another, remembers last time, make sure at least that amount of time has passed
// only then tell the nexted collector to make a frame
// currently 30 frames a second
pub trait Collector: Sized {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_number: usize,
    ) -> ReplayProcessorResult<()>;

    fn process_replay(mut self, replay: &boxcars::Replay) -> ReplayProcessorResult<Self> {
        ReplayProcessor::new(replay)?.process(&mut self)?;
        Ok(self)
    }
}

impl<G> Collector for G
where
    G: FnMut(&ReplayProcessor, &boxcars::Frame, usize) -> ReplayProcessorResult<()>,
{
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_number: usize,
    ) -> ReplayProcessorResult<()> {
        self(processor, frame, frame_number)
    }
}
