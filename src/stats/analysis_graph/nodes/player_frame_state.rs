use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct PlayerFrameStateNode {
    state: PlayerFrameState,
}

impl PlayerFrameStateNode {
    pub fn new() -> Self {
        Self {
            state: PlayerFrameState::default(),
        }
    }
}

impl_analysis_node! {
    node = PlayerFrameStateNode,
    state = PlayerFrameState,
    name = "player_frame_state",
    dependencies = [AnalysisDependency::required::<FrameInput>()],
    inputs = { frame_input: FrameInput },
    evaluate = |node| {
        node.state = frame_input.player_frame_state();
        Ok(())
    },
    state_ref = |node| &node.state,
}
