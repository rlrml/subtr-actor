use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, moment};
use crate::*;

/// Detects players beaten to the ball by an opponent's touch during live play.
pub struct BeatenToBallNode {
    calculator: BeatenToBallCalculator,
}

impl BeatenToBallNode {
    pub fn new() -> Self {
        Self::with_calculator(BeatenToBallCalculator::new())
    }

    /// Wraps a pre-configured calculator (e.g. one with diagnostics enabled
    /// via [`BeatenToBallCalculator::enable_diagnostics`]) so audit tooling
    /// can run it inside an analysis graph.
    pub fn with_calculator(calculator: BeatenToBallCalculator) -> Self {
        Self { calculator }
    }
}

impl Default for BeatenToBallNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for BeatenToBallNode {
    type State = BeatenToBallCalculator;

    fn name(&self) -> &'static str {
        "beaten_to_ball"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::BEATEN_TO_BALL_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            touch_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<TouchState>()?,
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
fn projected_timeline_events(calculator: &BeatenToBallCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    // Beaten-to-ball is retrospective: an event commits fully decided at the
    // opponent's confirmed touch, so it is final as soon as it is pushed.
    for event in calculator.events() {
        assembler.push(
            "beaten_to_ball",
            event.frame,
            EventLifecycle::Finalized,
            moment(event.frame, event.time),
            EventPayload::BeatenToBall(event.clone()),
            Some(event.player.clone()),
            Some(event.winner.clone()),
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        );
    }
    assembler.into_events()
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(BeatenToBallNode::new())
}
