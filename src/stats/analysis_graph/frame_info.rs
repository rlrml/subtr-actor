use super::graph::*;
use crate::stats::calculators::*;
use crate::*;

pub struct FrameInfoNode {
    state: FrameInfo,
}

impl FrameInfoNode {
    pub fn new() -> Self {
        Self {
            state: FrameInfo::default(),
        }
    }
}

impl Default for FrameInfoNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for FrameInfoNode {
    type State = FrameInfo;

    fn name(&self) -> &'static str {
        "frame_info"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![AnalysisDependency::required::<FrameInput>()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.state = ctx.get::<FrameInput>()?.frame_info();
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(FrameInfoNode::new())
}
