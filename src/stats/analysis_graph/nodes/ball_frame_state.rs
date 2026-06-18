use super::*;
use crate::stats::calculators::*;
use crate::*;

/// Exposes per-frame ball state (position/velocity) extracted from raw frame input.
pub struct BallFrameStateNode {
    state: BallFrameState,
}

impl BallFrameStateNode {
    pub fn new() -> Self {
        Self {
            state: BallFrameState::default(),
        }
    }
}

impl_analysis_node! {
    node = BallFrameStateNode,
    state = BallFrameState,
    name = "ball_frame_state",
    dependencies = [AnalysisDependency::required::<FrameInput>()],
    inputs = { frame_input: FrameInput },
    evaluate = |node| {
        node.state = frame_input.ball_frame_state();
        Ok(())
    },
    state_ref = |node| &node.state,
}
