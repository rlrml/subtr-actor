use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct BackboardNode {
    calculator: BackboardCalculator,
}

impl BackboardNode {
    pub fn new() -> Self {
        Self {
            calculator: BackboardCalculator::new(),
        }
    }
}

impl_analysis_node! {
    node = BackboardNode,
    state = BackboardCalculator,
    name = "backboard",
    dependencies = [
        frame_info_dependency() => FrameInfo,
        backboard_bounce_state_dependency() => BackboardBounceState,
    ],
    call = calculator.update,
}
