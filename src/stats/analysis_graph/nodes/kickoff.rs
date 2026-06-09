use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct KickoffNode {
    calculator: KickoffCalculator,
}

impl KickoffNode {
    pub fn new() -> Self {
        Self {
            calculator: KickoffCalculator::new(),
        }
    }
}

impl Default for KickoffNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for KickoffNode {
    type State = KickoffCalculator;

    fn name(&self) -> &'static str {
        "kickoff"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            gameplay_state_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            touch_state_dependency(),
            frame_events_state_dependency(),
            speed_flip_dependency(),
            boost_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let speed_flip = ctx.get::<SpeedFlipCalculator>()?;
        let boost_ledger_events = ctx.get::<BoostCalculator>()?.projected_ledger_events();
        self.calculator
            .update_with_speed_flips(KickoffUpdateContext {
                frame: ctx.get::<FrameInfo>()?,
                gameplay: ctx.get::<GameplayState>()?,
                ball: ctx.get::<BallFrameState>()?,
                players: ctx.get::<PlayerFrameState>()?,
                touch_state: ctx.get::<TouchState>()?,
                events: ctx.get::<FrameEventsState>()?,
                speed_flip_events: speed_flip.events(),
                boost_ledger_events: &boost_ledger_events,
            })
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(KickoffNode::new())
}
