use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Derives team/possession stats from ball-frame and shared possession state.
pub struct PossessionNode {
    calculator: PossessionCalculator,
}

impl PossessionNode {
    pub fn new() -> Self {
        Self {
            calculator: PossessionCalculator::new(),
        }
    }
}

impl Default for PossessionNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for PossessionNode {
    type State = PossessionCalculator;

    fn name(&self) -> &'static str {
        "possession"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::POSSESSION_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            possession_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<PossessionState>()?,
            ctx.get::<LivePlayState>()?,
        )
    }

    fn finish(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.flush_pending_event();
        Ok(())
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
fn projected_timeline_events(calculator: &PossessionCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    // Possession segments are emitted only once their terminating boundary is
    // decided (any loss backdating happens inside the resolver *before* the
    // segment is pushed), so committed segments — including their start, the
    // id anchor — are immutable.
    for event in calculator.events() {
        assembler.push(
            "possession",
            event.frame,
            EventLifecycle::Finalized,
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::Possession(event.clone()),
            event.player_id.clone(),
            None,
            None,
            None,
            None,
            None,
        );
    }
    assembler.into_events()
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(PossessionNode::new())
}
