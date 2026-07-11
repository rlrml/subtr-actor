use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, moment};
use crate::*;

/// Tracks per-player boost usage and pickups, accumulating boost stats from frame/event state.
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
    emitted_events = crate::stats::calculators::BOOST_EMITTED_EVENTS,
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
    project_events = |node| { projected_timeline_events(&node.calculator) },
    evaluate = |node| {
        node.calculator.update_parts(
            frame_info,
            gameplay_state,
            player_frame_state,
            frame_events_state,
            player_vertical_state,
            live_play_state,
        )
    },
    finish = |node| {
        node.calculator.finish_calculation()
    },
    state_ref = |node| &node.calculator,
}

/// Projects this node's committed events for the stats timeline (see
/// `AnalysisNode::project_events`). The inline comments state the stream's
/// interim lifecycle rule.
fn projected_timeline_events(calculator: &BoostCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    for event in calculator.pickup_events() {
        assembler.push(
            "boost_pickups",
            event.frame,
            EventLifecycle::Finalized,
            moment(event.frame, event.time),
            EventPayload::BoostPickup(event.clone()),
            Some(event.player_id.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        );
    }

    for event in calculator.respawn_events() {
        assembler.push(
            "boost_respawn",
            event.frame,
            EventLifecycle::Finalized,
            moment(event.frame, event.time),
            EventPayload::Respawn(event.clone()),
            Some(event.player_id.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        );
    }
    assembler.into_events()
}
