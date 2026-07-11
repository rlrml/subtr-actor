use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Detects ball carries and air dribbles from continuous ball-control sequences.
pub struct BallCarryNode {
    calculator: BallCarryCalculator,
}

impl BallCarryNode {
    pub fn new() -> Self {
        Self {
            calculator: BallCarryCalculator::new(),
        }
    }

    fn update_from_control_state(
        &mut self,
        ctx: &AnalysisStateContext<'_>,
    ) -> SubtrActorResult<()> {
        self.calculator
            .update(ctx.get::<ContinuousBallControlState>()?)
    }
}

impl Default for BallCarryNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for BallCarryNode {
    type State = BallCarryCalculator;

    fn name(&self) -> &'static str {
        "ball_carry"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::BALL_CARRY_EMITTED_EVENTS
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![continuous_ball_control_dependency()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.update_from_control_state(ctx)
    }

    fn finish(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.update_from_control_state(ctx)
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
fn projected_timeline_events(calculator: &BallCarryCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    // Carries/air dribbles commit only when the control sequence completes.
    for event in calculator.carry_events() {
        assembler.push(
            "ball_carry",
            event.start_frame,
            EventLifecycle::Finalized,
            span(
                event.start_frame,
                event.end_frame,
                event.start_time,
                event.end_time,
            ),
            EventPayload::BallCarry(event.clone()),
            Some(event.player_id.clone()),
            None,
            Some(event.is_team_0),
            Some(event.end_position),
            Some(event.end_position),
            None,
        );
    }
    assembler.into_events()
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(BallCarryNode::new())
}
