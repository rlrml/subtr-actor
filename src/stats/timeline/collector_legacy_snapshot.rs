use super::collector_legacy::StatsTimelineCollector;
use crate::stats::analysis_graph::StatsTimelineFrameState;
use crate::*;

impl StatsTimelineCollector {
    pub(super) fn snapshot_frame(&self) -> SubtrActorResult<ReplayStatsFrame> {
        self.graph
            .state::<StatsTimelineFrameState>()
            .and_then(|state| state.frame.clone())
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
                    "missing StatsTimelineFrame state while building timeline frame".to_owned(),
                ))
            })
    }
}
