pub mod decorator;
pub mod ndarray;
pub mod replay_data;

pub use self::ndarray::*;
pub use decorator::*;
pub use replay_data::*;

use crate::{processor::ReplayProcessor, ReplayProcessorResult};
use boxcars;

pub trait Collector: Sized {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_number: usize,
    ) -> Result<(), String>;

    fn process_replay(mut self, replay: &boxcars::Replay) -> ReplayProcessorResult<Self> {
        ReplayProcessor::new(replay).process(&mut self)?;
        Ok(self)
    }
}

impl<G> Collector for G
where
    G: FnMut(&ReplayProcessor, &boxcars::Frame, usize) -> Result<(), String>,
{
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_number: usize,
    ) -> Result<(), String> {
        self(processor, frame, frame_number)
    }
}
