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

impl_analysis_node! {
    node = FrameInfoNode,
    state = FrameInfo,
    name = "frame_info",
    dependencies = [AnalysisDependency::required::<FrameInput>()],
    inputs = { frame_input: FrameInput },
    evaluate = |node| {
        node.state = frame_input.frame_info();
        Ok(())
    },
    state_ref = |node| &node.state,
}
