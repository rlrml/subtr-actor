use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Derives loose team possession (last team to touch owns the ball until the
/// opponent takes it away) directly from touch and live-play state.
pub struct LoosePossessionNode {
    calculator: LoosePossessionCalculator,
}

impl LoosePossessionNode {
    pub fn new() -> Self {
        Self {
            calculator: LoosePossessionCalculator::new(),
        }
    }
}

impl Default for LoosePossessionNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for LoosePossessionNode {
    type State = LoosePossessionCalculator;

    fn name(&self) -> &'static str {
        "loose_possession"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::LOOSE_POSSESSION_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            touch_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<TouchState>()?,
            ctx.get::<LivePlayState>()?,
        )
    }

    fn finish(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.finish();
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
fn projected_timeline_events(calculator: &LoosePossessionCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    // Same emission discipline as `possession`.
    for event in calculator.events() {
        assembler.push(
            "loose_possession",
            event.frame,
            EventLifecycle::Finalized,
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::LoosePossession(event.clone()),
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
    Box::new(LoosePossessionNode::new())
}
