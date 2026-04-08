use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct PossessionStateNode {
    calculator: PossessionStateCalculator,
    state: PossessionState,
}

impl PossessionStateNode {
    pub fn new() -> Self {
        Self {
            calculator: PossessionStateCalculator::new(),
            state: PossessionState::default(),
        }
    }
}

impl_analysis_node! {
    node = PossessionStateNode,
    state = PossessionState,
    name = "possession_state",
    dependencies = [
        frame_info_dependency() => FrameInfo,
        touch_state_dependency() => TouchState,
        live_play_dependency() => LivePlayState,
    ],
    update_state = calculator.update,
}
