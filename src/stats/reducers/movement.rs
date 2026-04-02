pub use crate::stats::calculators::movement::*;
pub type MovementReducer = MovementCalculator;

use super::*;

impl StatsReducer for MovementReducer {
    fn on_sample(&mut self, sample: &CoreSample) -> SubtrActorResult<()> {
        self.update(sample)
    }
}
