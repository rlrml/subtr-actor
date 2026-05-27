use super::stats_timeline_frame_dependencies::stats_timeline_frame_dependencies;
use super::*;

impl AnalysisNode for StatsTimelineFrameNode {
    type State = StatsTimelineFrameState;

    fn name(&self) -> &'static str {
        "stats_timeline_frame"
    }

    fn on_replay_meta(&mut self, meta: &ReplayMeta) -> SubtrActorResult<()> {
        self.replay_meta = Some(meta.clone());
        Ok(())
    }

    fn dependencies(&self) -> NodeDependencies {
        stats_timeline_frame_dependencies()
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.update_snapshot(ctx)
    }

    fn finish(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.update_snapshot(ctx)
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}
