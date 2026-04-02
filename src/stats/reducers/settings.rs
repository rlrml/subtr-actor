pub use crate::stats::calculators::settings::*;
pub type SettingsReducer = SettingsCalculator;

use super::*;

impl StatsReducer for SettingsReducer {
    fn on_replay_meta(&mut self, meta: &ReplayMeta) -> SubtrActorResult<()> {
        self.apply_replay_meta(meta)
    }

    fn on_sample(&mut self, sample: &FrameState) -> SubtrActorResult<()> {
        let _ = sample;
        self.update()
    }
}
