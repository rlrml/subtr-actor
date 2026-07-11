use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Detects ceiling shots from ball/player positions and touch events during live play.
pub struct CeilingShotNode {
    calculator: CeilingShotCalculator,
}

impl CeilingShotNode {
    pub fn new() -> Self {
        Self {
            calculator: CeilingShotCalculator::new(),
        }
    }
}

impl Default for CeilingShotNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for CeilingShotNode {
    type State = CeilingShotCalculator;

    fn name(&self) -> &'static str {
        "ceiling_shot"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::CEILING_SHOT_EMITTED_EVENTS
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
        let frame = ctx.get::<FrameInfo>()?;
        let ball = ctx.get::<BallFrameState>()?;
        let players = ctx.get::<PlayerFrameState>()?;
        let touch_state = ctx.get::<TouchState>()?;
        let live_play_state = ctx.get::<LivePlayState>()?;
        self.calculator.update_parts(
            frame,
            ball,
            players,
            &touch_state.touch_events,
            live_play_state,
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
fn projected_timeline_events(calculator: &CeilingShotCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    for event in calculator.events() {
        assembler.push(
            "ceiling_shot",
            event.ceiling_contact_frame,
            EventLifecycle::Finalized,
            span(
                event.ceiling_contact_frame,
                event.frame,
                event.ceiling_contact_time,
                event.time,
            ),
            EventPayload::CeilingShot(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            Some(event.touch_position),
            Some(event.confidence),
        );
    }
    assembler.into_events()
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(CeilingShotNode::new())
}
