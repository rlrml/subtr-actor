use super::analysis_graph::AnalysisDependency;
use super::{backboard_bounce, fifty_fifty_state, possession_state, touch_state};
use crate::stats::calculators::{
    BackboardBounceState, CoreSample, FiftyFiftyState, PossessionState, TouchState,
};

pub(crate) type NodeDependencies = Vec<AnalysisDependency>;

pub(crate) fn core_sample_dependency() -> AnalysisDependency {
    AnalysisDependency::required::<CoreSample>()
}

pub(crate) fn touch_state_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<TouchState>(touch_state::boxed_default)
}

pub(crate) fn possession_state_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<PossessionState>(possession_state::boxed_default)
}

pub(crate) fn backboard_bounce_state_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<BackboardBounceState>(backboard_bounce::boxed_default)
}

pub(crate) fn fifty_fifty_state_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<FiftyFiftyState>(fifty_fifty_state::boxed_default)
}
