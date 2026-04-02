pub use crate::stats::calculators::dodge_reset::*;
pub type DodgeResetReducer = DodgeResetCalculator;

use super::*;

impl StatsReducer for DodgeResetReducer {
    fn on_sample(&mut self, sample: &CoreSample) -> SubtrActorResult<()> {
        self.update(sample)
    }
}
