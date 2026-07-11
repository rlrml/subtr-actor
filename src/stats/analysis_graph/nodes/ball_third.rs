use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

pub struct BallThirdNode {
    calculator: BallThirdCalculator,
}

impl BallThirdNode {
    pub fn new() -> Self {
        Self::with_config(BallThirdCalculatorConfig::default())
    }

    pub fn with_config(config: BallThirdCalculatorConfig) -> Self {
        Self {
            calculator: BallThirdCalculator::with_config(config),
        }
    }
}

impl Default for BallThirdNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for BallThirdNode {
    type State = BallThirdCalculator;

    fn name(&self) -> &'static str {
        "ball_third"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::BALL_THIRD_EMITTED_EVENTS
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
fn projected_timeline_events(calculator: &BallThirdCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    for event in calculator.events() {
        assembler.push(
            "ball_third",
            event.frame,
            EventLifecycle::Finalized,
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::BallThird(event.clone()),
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
    Box::new(BallThirdNode::new())
}
