pub use crate::stats::calculators::powerslide::*;
pub type PowerslideReducer = PowerslideCalculator;

use super::*;

impl StatsReducer for PowerslideReducer {
    fn on_sample(&mut self, sample: &CoreSample) -> SubtrActorResult<()> {
        self.update(sample)
    }
}
