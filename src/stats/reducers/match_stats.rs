pub use crate::stats::calculators::match_stats::*;
pub type MatchStatsReducer = MatchStatsCalculator;

use super::*;

impl StatsReducer for MatchStatsReducer {
    fn on_sample(&mut self, sample: &FrameState) -> SubtrActorResult<()> {
        self.update(sample)
    }
}
