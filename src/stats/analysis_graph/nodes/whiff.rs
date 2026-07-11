use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Detects whiffs and beaten-to-ball attempts from ball/player state and touches during live play.
pub struct WhiffNode {
    calculator: WhiffCalculator,
}

impl WhiffNode {
    pub fn new() -> Self {
        Self {
            calculator: WhiffCalculator::new(),
        }
    }
}

impl Default for WhiffNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for WhiffNode {
    type State = WhiffCalculator;

    fn name(&self) -> &'static str {
        "whiff"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::WHIFF_EMITTED_EVENTS
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
fn projected_timeline_events(calculator: &WhiffCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    // Whiffs commit once resolved (Whiff/BeatenToBall decided at push time).
    for event in calculator.events() {
        assembler.push(
            "whiff",
            event.frame,
            EventLifecycle::Finalized,
            span(
                event.frame,
                event.resolved_frame,
                event.time,
                event.resolved_time,
            ),
            EventPayload::Whiff(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        );
    }
    assembler.into_events()
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(WhiffNode::new())
}
