use super::*;
use crate::stats::calculators::*;
use crate::*;

/// Detects flicks from ball/player state and touches during live play.
pub struct FlickNode {
    calculator: FlickCalculator,
}

impl FlickNode {
    pub fn new() -> Self {
        Self {
            calculator: FlickCalculator::new(),
        }
    }
}

impl_analysis_node! {
    node = FlickNode,
    state = FlickCalculator,
    name = "flick",
    emitted_events = crate::stats::calculators::FLICK_EMITTED_EVENTS,
    dependencies = [
        frame_info_dependency() => FrameInfo,
        ball_frame_state_dependency() => BallFrameState,
        player_frame_state_dependency() => PlayerFrameState,
        touch_state_dependency() => TouchState,
        touch_dependency() => TouchCalculator,
        live_play_dependency() => LivePlayState,
    ],
    call = calculator.update,
}
