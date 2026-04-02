pub use crate::stats::calculators::pressure::*;
pub type PressureReducer = PressureCalculator;
pub type PressureReducerConfig = PressureCalculatorConfig;

use super::*;

impl StatsReducer for PressureReducer {
    fn on_sample(&mut self, sample: &CoreSample) -> SubtrActorResult<()> {
        self.update(sample)
    }
}
