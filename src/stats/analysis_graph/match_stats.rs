use super::graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct MatchStatsNode {
    calculator: MatchStatsCalculator,
}

impl MatchStatsNode {
    pub fn new() -> Self {
        Self {
            calculator: MatchStatsCalculator::new(),
        }
    }
}

impl_analysis_node! {
    node = MatchStatsNode,
    state = MatchStatsCalculator,
    name = "match_stats",
    dependencies = [
        frame_info_dependency() => FrameInfo,
        gameplay_state_dependency() => GameplayState,
        ball_frame_state_dependency() => BallFrameState,
        player_frame_state_dependency() => PlayerFrameState,
        frame_events_state_dependency() => FrameEventsState,
        live_play_dependency() => LivePlayState,
    ],
    call = calculator.update_parts,
}
