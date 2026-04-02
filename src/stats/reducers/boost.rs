pub use crate::stats::calculators::boost::*;
pub type BoostReducer = BoostCalculator;
pub type BoostReducerConfig = BoostCalculatorConfig;

use super::*;

impl StatsReducer for BoostReducer {
    fn on_sample(&mut self, sample: &CoreSample) -> SubtrActorResult<()> {
        self.update(sample)
    }
}
