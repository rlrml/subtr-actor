use super::analysis_graph::AnalysisDependency;
use super::{
    backboard_bounce, ball_frame_state, fifty_fifty_state, frame_events_state, frame_info,
    frame_state, gameplay_state, player_frame_state, possession_state, touch_state,
};
use crate::stats::calculators::{
    BackboardBounceState, BallFrameState, FiftyFiftyState, FrameEventsState, FrameInfo, FrameState,
    GameplayState, PlayerFrameState, PossessionState, TouchState,
};

pub(crate) type NodeDependencies = Vec<AnalysisDependency>;

pub(crate) fn full_frame_dependencies() -> NodeDependencies {
    vec![
        frame_info_dependency(),
        gameplay_state_dependency(),
        ball_frame_state_dependency(),
        player_frame_state_dependency(),
        frame_events_state_dependency(),
    ]
}

pub(crate) fn frame_state_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<FrameState>(frame_state::boxed_default)
}

pub(crate) fn frame_info_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<FrameInfo>(frame_info::boxed_default)
}

pub(crate) fn gameplay_state_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<GameplayState>(gameplay_state::boxed_default)
}

pub(crate) fn ball_frame_state_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<BallFrameState>(ball_frame_state::boxed_default)
}

pub(crate) fn player_frame_state_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<PlayerFrameState>(player_frame_state::boxed_default)
}

pub(crate) fn frame_events_state_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<FrameEventsState>(frame_events_state::boxed_default)
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
