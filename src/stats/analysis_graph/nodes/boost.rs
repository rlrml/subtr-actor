use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct BoostNode {
    calculator: BoostCalculator,
}

impl BoostNode {
    pub fn new() -> Self {
        Self::with_config(BoostCalculatorConfig::default())
    }

    pub fn with_config(config: BoostCalculatorConfig) -> Self {
        Self {
            calculator: BoostCalculator::with_config(config),
        }
    }
}

impl_analysis_node! {
    node = BoostNode,
    state = BoostCalculator,
    name = "boost",
    dependencies = [
        frame_info_dependency(),
        gameplay_state_dependency(),
        player_frame_state_dependency(),
        frame_events_state_dependency(),
        player_vertical_state_dependency(),
        live_play_dependency(),
    ],
    inputs = {
        frame_info: FrameInfo,
        gameplay_state: GameplayState,
        player_frame_state: PlayerFrameState,
        frame_events_state: FrameEventsState,
        player_vertical_state: PlayerVerticalState,
        live_play_state: LivePlayState,
    },
    evaluate = |node| {
        node.calculator.update_parts(
            frame_info,
            gameplay_state,
            player_frame_state,
            frame_events_state,
            player_vertical_state,
            live_play_state.counts_toward_player_motion(),
        )
    },
    state_ref = |node| &node.calculator,
}
