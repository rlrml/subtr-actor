use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Tracks which half of the field the ball is in from ball-frame and live-play state.
pub struct BallHalfNode {
    calculator: BallHalfCalculator,
}

impl BallHalfNode {
    pub fn new() -> Self {
        Self::with_config(BallHalfCalculatorConfig::default())
    }

    pub fn with_config(config: BallHalfCalculatorConfig) -> Self {
        Self {
            calculator: BallHalfCalculator::with_config(config),
        }
    }
}

impl Default for BallHalfNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for BallHalfNode {
    type State = BallHalfCalculator;

    fn name(&self) -> &'static str {
        "ball_half"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::BALL_HALF_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
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
fn projected_timeline_events(calculator: &BallHalfCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    // Ball-half/third spans commit when the label changes; the coalescing
    // pending span is not part of `events()`.
    for event in calculator.events() {
        assembler.push(
            "ball_half",
            event.frame,
            EventLifecycle::Finalized,
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::BallHalf(event.clone()),
            None,
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
    Box::new(BallHalfNode::new())
}
