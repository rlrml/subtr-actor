use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, moment, span};
use crate::*;

/// Classifies ball touches (with rotation/possession/50-50/vertical context) into touch events/stats.
pub struct TouchNode {
    calculator: TouchCalculator,
}

impl TouchNode {
    pub fn new() -> Self {
        Self {
            calculator: TouchCalculator::new(),
        }
    }
}

impl Default for TouchNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for TouchNode {
    type State = TouchCalculator;

    fn name(&self) -> &'static str {
        "touch"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::TOUCH_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            player_vertical_state_dependency(),
            rotation_dependency(),
            touch_state_dependency(),
            possession_state_dependency(),
            fifty_fifty_state_dependency(),
            frame_events_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let touch_state = ctx.get::<TouchState>()?;
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<PlayerVerticalState>()?,
            ctx.get::<RotationCalculator>()?,
            touch_state,
            ctx.get::<PossessionState>()?,
            ctx.get::<FiftyFiftyState>()?,
            ctx.get::<FrameEventsState>()?,
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
fn projected_timeline_events(calculator: &TouchCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    // Touches keep accreting evidence long after the contact: ball-movement
    // credit merges/extends until it finalizes (retiming the presented span),
    // and intention/outcome tags upgrade promote-only as late as a goal many
    // seconds on. The touch moment itself — the id anchor — never moves, so
    // the touch is Confirmed at commit and only finalizes at finish.
    for event in calculator.events() {
        let timing =
            event
                .ball_movement
                .as_ref()
                .map_or(moment(event.frame, event.time), |movement| {
                    span(
                        movement.start_frame,
                        movement.end_frame,
                        movement.start_time,
                        movement.end_time,
                    )
                });
        assembler.push(
            "touch",
            event.frame,
            EventLifecycle::Confirmed,
            timing,
            EventPayload::Touch(event.clone()),
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
    Box::new(TouchNode::new())
}
