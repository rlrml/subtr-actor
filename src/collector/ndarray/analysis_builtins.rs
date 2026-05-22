use crate::stats::analysis_graph::{AnalysisDependency, AnalysisNodeDyn, TouchNode};
use crate::stats::calculators::TouchCalculator;
use crate::*;
use boxcars;

fn boxed_touch_node() -> Box<dyn AnalysisNodeDyn> {
    Box::new(TouchNode::new())
}

fn touch_dependency() -> Vec<AnalysisDependency> {
    vec![AnalysisDependency::with_default::<TouchCalculator>(
        boxed_touch_node,
    )]
}

build_analysis_player_feature_adder!(
    AnalysisPlayerTouches,
    |_self_: &AnalysisPlayerTouches<F>| touch_dependency(),
    |_self_: &AnalysisPlayerTouches<F>,
     context: &AnalysisFeatureContext<'_>,
     player_id: &PlayerId,
     _processor: &dyn ProcessorView,
     _frame: &boxcars::Frame,
     _frame_count: usize,
     _current_time: f32| {
        let touch_event = context
            .state::<TouchCalculator>()?
            .new_events()
            .iter()
            .any(|event| &event.player == player_id);
        convert_all_floats!(f32::from(touch_event))
    },
    "analysis touch event",
);
