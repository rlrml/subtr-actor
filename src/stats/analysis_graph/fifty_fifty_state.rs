use super::graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct FiftyFiftyStateNode {
    calculator: FiftyFiftyStateCalculator,
    state: FiftyFiftyState,
}

impl FiftyFiftyStateNode {
    pub fn new() -> Self {
        Self {
            calculator: FiftyFiftyStateCalculator::new(),
            state: FiftyFiftyState::default(),
        }
    }
}

impl_analysis_node! {
    node = FiftyFiftyStateNode,
    state = FiftyFiftyState,
    name = "fifty_fifty_state",
    dependencies = [
        frame_info_dependency() => FrameInfo,
        gameplay_state_dependency() => GameplayState,
        ball_frame_state_dependency() => BallFrameState,
        player_frame_state_dependency() => PlayerFrameState,
        touch_state_dependency() => TouchState,
        possession_state_dependency() => PossessionState,
        live_play_dependency() => LivePlayState,
    ],
    update_state = calculator.update,
}
