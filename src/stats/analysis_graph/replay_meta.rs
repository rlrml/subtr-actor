use super::AnalysisGraph;
use crate::{PlayerId, ProcessorView, ReplayMeta, SubtrActorResult};

#[derive(Default)]
pub(crate) struct ReplayMetaTracker {
    player_ids: Option<Vec<PlayerId>>,
}

impl ReplayMetaTracker {
    pub(crate) fn needs_refresh(&self, processor: &dyn ProcessorView) -> bool {
        !self
            .player_ids
            .as_ref()
            .is_some_and(|ids| ids.iter().eq(processor.iter_player_ids_in_order()))
    }

    pub(crate) fn refresh(
        &mut self,
        processor: &dyn ProcessorView,
        graph: &mut AnalysisGraph,
    ) -> SubtrActorResult<Option<ReplayMeta>> {
        if !self.needs_refresh(processor) {
            return Ok(None);
        }
        let meta = processor.get_replay_meta()?;
        graph.on_replay_meta(&meta)?;
        self.player_ids = Some(processor.iter_player_ids_in_order().cloned().collect());
        Ok(Some(meta))
    }
}

#[cfg(test)]
#[path = "replay_meta_tests.rs"]
mod tests;
