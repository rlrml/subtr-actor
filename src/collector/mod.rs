pub mod decorator;
pub mod ndarray;
pub mod replay_data;

pub use self::ndarray::*;
pub use decorator::*;
pub use replay_data::*;

use crate::*;
use boxcars;

pub enum TimeAdvance {
    Time(f32),
    NextFrame,
}

pub trait Collector: Sized {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_number: usize,
        target_time: f32,
    ) -> BoxcarsResult<TimeAdvance>;

    fn process_replay(mut self, replay: &boxcars::Replay) -> BoxcarsResult<Self> {
        ReplayProcessor::new(replay)?.process(&mut self)?;
        Ok(self)
    }
}

impl<G> Collector for G
where
    G: FnMut(&ReplayProcessor, &boxcars::Frame, usize, f32) -> BoxcarsResult<TimeAdvance>,
{
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_number: usize,
        target_time: f32,
    ) -> BoxcarsResult<TimeAdvance> {
        self(processor, frame, frame_number, target_time)
    }
}
