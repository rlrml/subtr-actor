use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
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
    project_events = |node| { projected_timeline_events(&node.calculator) },
    call = calculator.update,
}

/// Projects this node's committed events for the stats timeline (see
/// `AnalysisNode::project_events`). The inline comments state the stream's
/// interim lifecycle rule.
fn projected_timeline_events(calculator: &FlickCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    for event in calculator.events() {
        assembler.push(
            "flick",
            event.setup_start_frame,
            EventLifecycle::Finalized,
            span(
                event.setup_start_frame,
                event.frame,
                event.setup_start_time,
                event.time,
            ),
            EventPayload::Flick(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            Some(event.confidence),
        );
    }
    assembler.into_events()
}
