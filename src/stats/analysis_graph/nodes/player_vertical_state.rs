use super::*;
use crate::stats::calculators::*;
use crate::*;

/// Tracks per-player airborne/vertical state derived from player frame state.
pub struct PlayerVerticalStateNode {
    calculator: PlayerVerticalStateCalculator,
    state: PlayerVerticalState,
}

impl PlayerVerticalStateNode {
    pub fn new() -> Self {
        Self {
            calculator: PlayerVerticalStateCalculator::new(),
            state: PlayerVerticalState::default(),
        }
    }
}

impl_analysis_node! {
    node = PlayerVerticalStateNode,
    state = PlayerVerticalState,
    name = "player_vertical_state",
    dependencies = [player_frame_state_dependency() => PlayerFrameState],
    update_state = calculator.update,
}
