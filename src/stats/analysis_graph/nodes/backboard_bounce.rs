use super::*;
use crate::stats::calculators::*;
use crate::*;

/// Tracks ball bounces off the backboard from ball/player/touch state; exposes them as shared state.
pub struct BackboardBounceStateNode {
    calculator: BackboardBounceCalculator,
    state: BackboardBounceState,
}

impl BackboardBounceStateNode {
    pub fn new() -> Self {
        Self {
            calculator: BackboardBounceCalculator::new(),
            state: BackboardBounceState::default(),
        }
    }
}

impl_analysis_node! {
    node = BackboardBounceStateNode,
    state = BackboardBounceState,
    name = "backboard_bounce_state",
    emitted_events = crate::stats::calculators::BACKBOARD_BOUNCE_STATE_EMITTED_EVENTS,
    dependencies = [
        frame_info_dependency() => FrameInfo,
        ball_frame_state_dependency() => BallFrameState,
        player_frame_state_dependency() => PlayerFrameState,
        touch_state_dependency() => TouchState,
        live_play_dependency() => LivePlayState,
    ],
    update_state = calculator.update,
}
