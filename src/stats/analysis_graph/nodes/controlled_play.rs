use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Detects stretches of controlled play from ball/player positions and touches.
pub struct ControlledPlayNode {
    calculator: ControlledPlayCalculator,
}

impl ControlledPlayNode {
    pub fn new() -> Self {
        Self {
            calculator: ControlledPlayCalculator::new(),
        }
    }
}

impl Default for ControlledPlayNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for ControlledPlayNode {
    type State = ControlledPlayCalculator;

    fn name(&self) -> &'static str {
        "controlled_play"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::CONTROLLED_PLAY_EMITTED_EVENTS
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
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
fn projected_timeline_events(calculator: &ControlledPlayCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    // Controlled-play runs commit when the run finalizes (gap/boundary).
    for event in calculator.events() {
        assembler.push(
            "controlled_play",
            event.start_frame,
            EventLifecycle::Finalized,
            span(
                event.start_frame,
                event.end_frame,
                event.start_time,
                event.end_time,
            ),
            EventPayload::ControlledPlay(event.clone()),
            Some(event.player_id.clone()),
            None,
            Some(event.is_team_0),
            None,
            None,
            None,
        );
    }
    assembler.into_events()
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(ControlledPlayNode::new())
}
