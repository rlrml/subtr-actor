pub use crate::stats::calculators::speed_flip::*;
pub type SpeedFlipReducer = SpeedFlipCalculator;

use super::*;

impl StatsReducer for SpeedFlipReducer {
    fn on_sample(&mut self, sample: &FrameState) -> SubtrActorResult<()> {
        self.update(sample)
    }
}
