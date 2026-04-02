pub use crate::stats::calculators::demo::*;
pub type DemoReducer = DemoCalculator;

use super::*;

impl StatsReducer for DemoReducer {
    fn on_sample(&mut self, sample: &CoreSample) -> SubtrActorResult<()> {
        self.update(sample)
    }
}
