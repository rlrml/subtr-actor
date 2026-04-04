use super::graph::*;
use crate::stats::calculators::*;
use crate::*;

pub struct GameplayStateNode {
    state: GameplayState,
}

impl GameplayStateNode {
    pub fn new() -> Self {
        Self {
            state: GameplayState::default(),
        }
    }
}

impl_analysis_node! {
    node = GameplayStateNode,
    state = GameplayState,
    name = "gameplay_state",
    dependencies = [AnalysisDependency::required::<FrameInput>()],
    inputs = { frame_input: FrameInput },
    evaluate = |node| {
        node.state = frame_input.gameplay_state();
        Ok(())
    },
    state_ref = |node| &node.state,
}
