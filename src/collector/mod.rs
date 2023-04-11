pub mod decorator;
pub mod ndarray;
pub mod replay_data;

pub use self::ndarray::*;
pub use decorator::*;
pub use replay_data::*;

use crate::processor::ReplayProcessor;
use boxcars;

pub trait Collector {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_number: usize,
    ) -> Result<(), String>;
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
