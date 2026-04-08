pub(crate) use super::graph::*;
use crate::stats::calculators::{
    BackboardBounceState, BackboardCalculator, BallCarryCalculator, BallFrameState,
    BoostCalculator, CeilingShotCalculator, DemoCalculator, DodgeResetCalculator,
    DoubleTapCalculator, FiftyFiftyCalculator, FiftyFiftyState, FrameEventsState, FrameInfo,
    GameplayState, LivePlayState, MatchStatsCalculator, MovementCalculator, MustyFlickCalculator,
    PlayerFrameState, PlayerVerticalState, PositioningCalculator, PossessionCalculator,
    PossessionState, PowerslideCalculator, PressureCalculator, RushCalculator, SpeedFlipCalculator,
    TouchCalculator, TouchState,
};

pub(crate) mod backboard;
pub(crate) mod backboard_bounce;
pub(crate) mod ball_carry;
pub(crate) mod ball_frame_state;
pub(crate) mod boost;
pub(crate) mod ceiling_shot;
pub(crate) mod demo;
pub(crate) mod dodge_reset;
pub(crate) mod double_tap;
pub(crate) mod fifty_fifty;
pub(crate) mod fifty_fifty_state;
pub(crate) mod frame_events_state;
pub(crate) mod frame_info;
pub(crate) mod gameplay_state;
pub(crate) mod live_play;
pub(crate) mod match_stats;
pub(crate) mod movement;
pub(crate) mod musty_flick;
pub(crate) mod player_frame_state;
pub(crate) mod player_vertical_state;
pub(crate) mod positioning;
pub(crate) mod possession;
pub(crate) mod possession_state;
pub(crate) mod powerslide;
pub(crate) mod pressure;
pub(crate) mod rush;
pub(crate) mod settings;
pub(crate) mod speed_flip;
pub(crate) mod stats_timeline_events;
pub(crate) mod stats_timeline_frame;
pub(crate) mod touch;
pub(crate) mod touch_state;

#[allow(unused_imports)]
pub use backboard::BackboardNode;
#[allow(unused_imports)]
pub use backboard_bounce::BackboardBounceStateNode;
#[allow(unused_imports)]
pub use ball_carry::BallCarryNode;
#[allow(unused_imports)]
pub use ball_frame_state::BallFrameStateNode;
#[allow(unused_imports)]
pub use boost::BoostNode;
#[allow(unused_imports)]
pub use ceiling_shot::CeilingShotNode;
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
pub use frame_events_state::FrameEventsStateNode;
#[allow(unused_imports)]
pub use frame_info::FrameInfoNode;
#[allow(unused_imports)]
pub use gameplay_state::GameplayStateNode;
#[allow(unused_imports)]
pub use live_play::LivePlayNode;
#[allow(unused_imports)]
pub use match_stats::MatchStatsNode;
#[allow(unused_imports)]
pub use movement::MovementNode;
#[allow(unused_imports)]
pub use musty_flick::MustyFlickNode;
#[allow(unused_imports)]
pub use player_frame_state::PlayerFrameStateNode;
#[allow(unused_imports)]
pub use player_vertical_state::PlayerVerticalStateNode;
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
pub use stats_timeline_events::{StatsTimelineEventsNode, StatsTimelineEventsState};
#[allow(unused_imports)]
pub use stats_timeline_frame::{StatsTimelineFrameNode, StatsTimelineFrameState};
#[allow(unused_imports)]
pub use touch::TouchNode;
#[allow(unused_imports)]
pub use touch_state::TouchStateNode;

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

pub(crate) fn player_vertical_state_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<PlayerVerticalState>(player_vertical_state::boxed_default)
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

pub(crate) fn live_play_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<LivePlayState>(live_play::boxed_default)
}

pub(crate) fn backboard_bounce_state_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<BackboardBounceState>(backboard_bounce::boxed_default)
}

pub(crate) fn fifty_fifty_state_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<FiftyFiftyState>(fifty_fifty_state::boxed_default)
}

pub(crate) fn match_stats_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<MatchStatsCalculator>(match_stats::boxed_default)
}

pub(crate) fn backboard_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<BackboardCalculator>(backboard::boxed_default)
}

pub(crate) fn ceiling_shot_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<CeilingShotCalculator>(ceiling_shot::boxed_default)
}

pub(crate) fn double_tap_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<DoubleTapCalculator>(double_tap::boxed_default)
}

pub(crate) fn fifty_fifty_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<FiftyFiftyCalculator>(fifty_fifty::boxed_default)
}

pub(crate) fn possession_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<PossessionCalculator>(possession::boxed_default)
}

pub(crate) fn pressure_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<PressureCalculator>(pressure::boxed_default)
}

pub(crate) fn rush_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<RushCalculator>(rush::boxed_default)
}

pub(crate) fn touch_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<TouchCalculator>(touch::boxed_default)
}

pub(crate) fn speed_flip_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<SpeedFlipCalculator>(speed_flip::boxed_default)
}

pub(crate) fn musty_flick_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<MustyFlickCalculator>(musty_flick::boxed_default)
}

pub(crate) fn dodge_reset_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<DodgeResetCalculator>(dodge_reset::boxed_default)
}

pub(crate) fn ball_carry_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<BallCarryCalculator>(ball_carry::boxed_default)
}

pub(crate) fn boost_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<BoostCalculator>(boost::boxed_default)
}

pub(crate) fn movement_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<MovementCalculator>(movement::boxed_default)
}

pub(crate) fn positioning_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<PositioningCalculator>(positioning::boxed_default)
}

pub(crate) fn powerslide_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<PowerslideCalculator>(powerslide::boxed_default)
}

pub(crate) fn demo_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<DemoCalculator>(demo::boxed_default)
}
