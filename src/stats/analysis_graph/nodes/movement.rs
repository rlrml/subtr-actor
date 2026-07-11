use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Tracks per-player movement classification/stats from player and vertical state during live play.
pub struct MovementNode {
    calculator: MovementCalculator,
}

impl MovementNode {
    pub fn new() -> Self {
        Self {
            calculator: MovementCalculator::new(),
        }
    }
}

impl Default for MovementNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for MovementNode {
    type State = MovementCalculator;

    fn name(&self) -> &'static str {
        "movement"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::MOVEMENT_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            player_frame_state_dependency(),
            player_vertical_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let live_play_state = ctx.get::<LivePlayState>()?;
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<PlayerVerticalState>()?,
            live_play_state,
        )
    }

    fn finish(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.flush_pending_events();
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
fn projected_timeline_events(calculator: &MovementCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    // Movement spans commit on classification change; the per-player pending
    // spans are not part of `events()`.
    for event in calculator.events() {
        assembler.push(
            "movement",
            event.frame,
            EventLifecycle::Finalized,
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::Movement(event.clone()),
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
    Box::new(MovementNode::new())
}
