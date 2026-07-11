use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, moment};
use crate::*;

/// Detects half-flips from player frame state during live play.
pub struct HalfFlipNode {
    calculator: HalfFlipCalculator,
}

impl HalfFlipNode {
    pub fn new() -> Self {
        Self {
            calculator: HalfFlipCalculator::new(),
        }
    }
}

impl Default for HalfFlipNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for HalfFlipNode {
    type State = HalfFlipCalculator;

    fn name(&self) -> &'static str {
        "half_flip"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::HALF_FLIP_EMITTED_EVENTS
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

    fn finish(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.finalize(ctx.get::<FrameInfo>()?);
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
fn projected_timeline_events(calculator: &HalfFlipCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    for event in calculator.events() {
        assembler.push(
            "half_flip",
            event.frame,
            EventLifecycle::Finalized,
            moment(event.frame, event.time),
            EventPayload::HalfFlip(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            Some(event.end_position),
            None,
            Some(event.confidence),
        );
    }
    assembler.into_events()
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(HalfFlipNode::new())
}
