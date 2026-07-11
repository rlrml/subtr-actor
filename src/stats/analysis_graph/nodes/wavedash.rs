use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Detects wavedashes from player frame state during live play.
pub struct WavedashNode {
    calculator: WavedashCalculator,
}

impl WavedashNode {
    pub fn new() -> Self {
        Self {
            calculator: WavedashCalculator::new(),
        }
    }
}

impl Default for WavedashNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for WavedashNode {
    type State = WavedashCalculator;

    fn name(&self) -> &'static str {
        "wavedash"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::WAVEDASH_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            player_frame_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<LivePlayState>()?,
        )
    }

    fn project_events(&self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<Vec<Event>> {
        Ok(projected_timeline_events(&self.calculator))
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

/// Projects this node's committed events for the stats timeline (see
/// `AnalysisNode::project_events`). The inline comments state the stream's
/// interim lifecycle rule.
fn projected_timeline_events(calculator: &WavedashCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    for event in calculator.events() {
        assembler.push(
            "wavedash",
            event.dodge_frame,
            EventLifecycle::Finalized,
            span(event.dodge_frame, event.frame, event.dodge_time, event.time),
            EventPayload::Wavedash(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            Some(event.landing_position),
            None,
            Some(event.confidence),
        );
    }
    assembler.into_events()
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(WavedashNode::new())
}
