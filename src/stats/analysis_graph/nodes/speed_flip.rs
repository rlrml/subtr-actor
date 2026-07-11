use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Detects speed-flips from gameplay/ball/player state during live play.
pub struct SpeedFlipNode {
    calculator: SpeedFlipCalculator,
}

impl SpeedFlipNode {
    pub fn new() -> Self {
        Self {
            calculator: SpeedFlipCalculator::new(),
        }
    }
}

impl Default for SpeedFlipNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for SpeedFlipNode {
    type State = SpeedFlipCalculator;

    fn name(&self) -> &'static str {
        "speed_flip"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::SPEED_FLIP_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            gameplay_state_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update_parts(
            ctx.get::<FrameInfo>()?,
            ctx.get::<GameplayState>()?,
            ctx.get::<BallFrameState>()?,
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
fn projected_timeline_events(calculator: &SpeedFlipCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    for event in calculator.events() {
        assembler.push(
            "speed_flip",
            event.frame,
            EventLifecycle::Finalized,
            span(
                event.frame,
                event.resolved_frame,
                event.time,
                event.resolved_time,
            ),
            EventPayload::SpeedFlip(event.clone()),
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
    Box::new(SpeedFlipNode::new())
}
