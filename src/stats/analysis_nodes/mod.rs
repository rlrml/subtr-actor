#![allow(dead_code)]

pub mod analysis_graph;

mod backboard;
mod backboard_bounce;
mod ball_carry;
mod boost;
mod ceiling_shot;
mod collector;
mod demo;
mod dodge_reset;
mod double_tap;
mod fifty_fifty;
mod fifty_fifty_state;
mod match_stats;
mod movement;
mod musty_flick;
mod nodes;
mod positioning;
mod possession;
mod possession_state;
mod powerslide;
mod pressure;
mod rush;
mod settings;
mod speed_flip;
mod touch;
mod touch_state;

use crate::stats::calculators::CoreSample;

#[allow(unused_imports)]
pub use backboard::BackboardNode;
#[allow(unused_imports)]
pub use backboard_bounce::BackboardBounceStateNode;
#[allow(unused_imports)]
pub use ball_carry::BallCarryNode;
#[allow(unused_imports)]
pub use boost::BoostNode;
#[allow(unused_imports)]
pub use ceiling_shot::CeilingShotNode;
#[allow(unused_imports)]
pub use collector::AnalysisNodeCollector;
#[allow(unused_imports)]
pub use demo::DemoNode;
#[allow(unused_imports)]
pub use dodge_reset::DodgeResetNode;
#[allow(unused_imports)]
pub use double_tap::DoubleTapNode;
#[allow(unused_imports)]
pub use fifty_fifty::FiftyFiftyNode;
#[allow(unused_imports)]
pub use fifty_fifty_state::FiftyFiftyStateNode;
#[allow(unused_imports)]
pub use match_stats::MatchStatsNode;
#[allow(unused_imports)]
pub use movement::MovementNode;
#[allow(unused_imports)]
pub use musty_flick::MustyFlickNode;
#[allow(unused_imports)]
pub use positioning::PositioningNode;
#[allow(unused_imports)]
pub use possession::PossessionNode;
#[allow(unused_imports)]
pub use possession_state::PossessionStateNode;
#[allow(unused_imports)]
pub use powerslide::PowerslideNode;
#[allow(unused_imports)]
pub use pressure::PressureNode;
#[allow(unused_imports)]
pub use rush::RushNode;
#[allow(unused_imports)]
pub use settings::SettingsNode;
#[allow(unused_imports)]
pub use speed_flip::SpeedFlipNode;
#[allow(unused_imports)]
pub use touch::TouchNode;
#[allow(unused_imports)]
pub use touch_state::TouchStateNode;

pub fn all_analysis_nodes() -> Vec<Box<dyn analysis_graph::AnalysisNodeDyn>> {
    vec![
        backboard::boxed_default(),
        ball_carry::boxed_default(),
        boost::boxed_default(),
        ceiling_shot::boxed_default(),
        demo::boxed_default(),
        dodge_reset::boxed_default(),
        double_tap::boxed_default(),
        fifty_fifty::boxed_default(),
        match_stats::boxed_default(),
        movement::boxed_default(),
        musty_flick::boxed_default(),
        positioning::boxed_default(),
        possession::boxed_default(),
        powerslide::boxed_default(),
        pressure::boxed_default(),
        rush::boxed_default(),
        settings::boxed_default(),
        speed_flip::boxed_default(),
        touch::boxed_default(),
    ]
}

pub fn graph_with_all_analysis_nodes() -> analysis_graph::AnalysisGraph {
    let mut graph = analysis_graph::AnalysisGraph::new().with_root_state_type::<CoreSample>();
    for node in all_analysis_nodes() {
        graph.push_boxed_node(node);
    }
    graph
}

#[cfg(test)]
#[path = "analysis_nodes_tests.rs"]
mod tests;
