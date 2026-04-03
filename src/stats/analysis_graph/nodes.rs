use super::graph::AnalysisDependency;
use super::{
    backboard, backboard_bounce, ball_carry, ball_frame_state, boost, ceiling_shot, demo,
    dodge_reset, double_tap, fifty_fifty, fifty_fifty_state, frame_events_state, frame_info,
    gameplay_state, live_play, match_stats, movement, musty_flick, player_frame_state,
    player_vertical_state, positioning, possession, possession_state, powerslide, pressure, rush,
    speed_flip, touch, touch_state,
};
use crate::stats::calculators::{
    BackboardBounceState, BackboardCalculator, BallCarryCalculator, BallFrameState,
    BoostCalculator, CeilingShotCalculator, DemoCalculator, DodgeResetCalculator,
    DoubleTapCalculator, FiftyFiftyCalculator, FiftyFiftyState, FrameEventsState, FrameInfo,
    GameplayState, LivePlayState, MatchStatsCalculator, MovementCalculator, MustyFlickCalculator,
    PlayerFrameState, PlayerVerticalState, PositioningCalculator, PossessionCalculator,
    PossessionState, PowerslideCalculator, PressureCalculator, RushCalculator, SpeedFlipCalculator,
    TouchCalculator, TouchState,
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
