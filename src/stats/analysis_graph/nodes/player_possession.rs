use super::*;
use crate::stats::calculators::*;
use crate::*;

/// Tracks per-player possession from ball/player/possession/touch state.
pub struct PlayerPossessionNode {
    calculator: PlayerPossessionCalculator,
}

impl PlayerPossessionNode {
    pub fn new() -> Self {
        Self {
            calculator: PlayerPossessionCalculator::new(),
        }
    }
}

impl Default for PlayerPossessionNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for PlayerPossessionNode {
    type State = PlayerPossessionCalculator;

    fn name(&self) -> &'static str {
        "player_possession"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            possession_state_dependency(),
            touch_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<PossessionState>()?,
            ctx.get::<TouchState>()?,
            ctx.get::<LivePlayState>()?,
        )
    }

    fn finish(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.finish();
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(PlayerPossessionNode::new())
}
