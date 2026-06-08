pub(crate) use super::graph::*;
use crate::stats::calculators::{
    AerialGoalCalculator, AirDribbleGoalCalculator, BackboardBounceState, BackboardCalculator,
    BallCarryCalculator, BallFrameState, BoostCalculator, BumpCalculator, BumpGoalCalculator,
    CeilingShotCalculator, CenterCalculator, ContinuousBallControlState, ControlledPlayCalculator,
    CounterAttackGoalCalculator, DemoCalculator, DemoGoalCalculator, DodgeResetCalculator,
    DoubleTapCalculator, DoubleTapGoalCalculator, EmptyNetGoalCalculator, FiftyFiftyCalculator,
    FiftyFiftyState, FlickCalculator, FlickGoalCalculator, FlipImpulseCalculator,
    FlipResetGoalCalculator, FrameEventsState, FrameInfo, GameplayState, HalfFlipCalculator,
    HalfVolleyCalculator, HalfVolleyGoalCalculator, HighAerialGoalCalculator, LivePlayState,
    LongDistanceGoalCalculator, MatchStatsCalculator, MovementCalculator, MustyFlickCalculator,
    OneTimerCalculator, OneTimerGoalCalculator, OwnHalfGoalCalculator, PassCalculator,
    PassingGoalCalculator, PlayerFrameState, PlayerVerticalState, PositioningCalculator,
    PossessionCalculator, PossessionState, PowerslideCalculator, PressureCalculator,
    RotationCalculator, RushCalculator, SpeedFlipCalculator, TerritorialPressureCalculator,
    TouchCalculator, TouchState, WallAerialCalculator, WallAerialShotCalculator,
    WavedashCalculator, WhiffCalculator,
};

pub(crate) mod backboard;
pub(crate) mod backboard_bounce;
pub(crate) mod ball_carry;
pub(crate) mod ball_frame_state;
pub(crate) mod boost;
pub(crate) mod bump;
pub(crate) mod ceiling_shot;
pub(crate) mod center;
pub(crate) mod continuous_ball_control;
pub(crate) mod controlled_play;
pub(crate) mod demo;
pub(crate) mod dodge_reset;
pub(crate) mod double_tap;
pub(crate) mod fifty_fifty;
pub(crate) mod fifty_fifty_state;
pub(crate) mod flick;
pub(crate) mod flip_impulse;
pub(crate) mod frame_events_state;
pub(crate) mod frame_info;
pub(crate) mod gameplay_state;
pub(crate) mod goal_tags;
pub(crate) mod half_flip;
pub(crate) mod half_volley;
pub(crate) mod live_play;
pub(crate) mod match_stats;
pub(crate) mod movement;
pub(crate) mod musty_flick;
pub(crate) mod one_timer;
pub(crate) mod pass;
pub(crate) mod player_frame_state;
pub(crate) mod player_vertical_state;
pub(crate) mod positioning;
pub(crate) mod possession;
pub(crate) mod possession_state;
pub(crate) mod powerslide;
pub(crate) mod pressure;
pub(crate) mod rotation;
pub(crate) mod rush;
pub(crate) mod settings;
pub(crate) mod speed_flip;
pub(crate) mod stats_projection;
pub(crate) mod stats_timeline_events;
pub(crate) mod stats_timeline_frame;
pub(crate) mod territorial_pressure;
pub(crate) mod touch;
pub(crate) mod touch_state;
pub(crate) mod wall_aerial;
pub(crate) mod wall_aerial_shot;
pub(crate) mod wavedash;
pub(crate) mod whiff;

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
pub use bump::BumpNode;
#[allow(unused_imports)]
pub use ceiling_shot::CeilingShotNode;
#[allow(unused_imports)]
pub use center::CenterNode;
#[allow(unused_imports)]
pub use continuous_ball_control::ContinuousBallControlNode;
#[allow(unused_imports)]
pub use controlled_play::ControlledPlayNode;
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
pub use flick::FlickNode;
#[allow(unused_imports)]
pub use flip_impulse::FlipImpulseNode;
#[allow(unused_imports)]
pub use frame_events_state::FrameEventsStateNode;
#[allow(unused_imports)]
pub use frame_info::FrameInfoNode;
#[allow(unused_imports)]
pub use gameplay_state::GameplayStateNode;
#[allow(unused_imports)]
pub use goal_tags::{
    AerialGoalNode, AirDribbleGoalNode, BumpGoalNode, CounterAttackGoalNode, DemoGoalNode,
    EmptyNetGoalNode, FlickGoalNode, FlipResetGoalNode, HalfVolleyGoalNode, HighAerialGoalNode,
    LongDistanceGoalNode, OneTimerGoalNode, OwnHalfGoalNode, PassingGoalNode,
};
#[allow(unused_imports)]
pub use half_flip::HalfFlipNode;
#[allow(unused_imports)]
pub use half_volley::HalfVolleyNode;
#[allow(unused_imports)]
pub use live_play::LivePlayNode;
#[allow(unused_imports)]
pub use match_stats::MatchStatsNode;
#[allow(unused_imports)]
pub use movement::MovementNode;
#[allow(unused_imports)]
pub use musty_flick::MustyFlickNode;
#[allow(unused_imports)]
pub use one_timer::OneTimerNode;
#[allow(unused_imports)]
pub use pass::PassNode;
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
pub use rotation::RotationNode;
#[allow(unused_imports)]
pub use rush::RushNode;
#[allow(unused_imports)]
pub use settings::SettingsNode;
#[allow(unused_imports)]
pub use speed_flip::SpeedFlipNode;
#[allow(unused_imports)]
pub use stats_projection::{StatsProjectionNode, StatsProjectionState};
#[allow(unused_imports)]
pub use stats_timeline_events::{
    StatsTimelineEventsNode, StatsTimelineEventsState, STATS_TIMELINE_MECHANIC_KINDS,
};
#[allow(unused_imports)]
pub use stats_timeline_frame::{StatsTimelineFrameNode, StatsTimelineFrameState};
#[allow(unused_imports)]
pub use territorial_pressure::TerritorialPressureNode;
#[allow(unused_imports)]
pub use touch::TouchNode;
#[allow(unused_imports)]
pub use touch_state::TouchStateNode;
#[allow(unused_imports)]
pub use wall_aerial::WallAerialNode;
#[allow(unused_imports)]
pub use wall_aerial_shot::WallAerialShotNode;
#[allow(unused_imports)]
pub use wavedash::WavedashNode;
#[allow(unused_imports)]
pub use whiff::WhiffNode;

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

pub(crate) fn flip_impulse_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<FlipImpulseCalculator>(flip_impulse::boxed_default)
}

pub(crate) fn backboard_bounce_state_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<BackboardBounceState>(backboard_bounce::boxed_default)
}

pub(crate) fn continuous_ball_control_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<ContinuousBallControlState>(
        continuous_ball_control::boxed_default,
    )
}

pub(crate) fn fifty_fifty_state_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<FiftyFiftyState>(fifty_fifty_state::boxed_default)
}

pub(crate) fn match_stats_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<MatchStatsCalculator>(match_stats::boxed_default)
}

pub(crate) fn controlled_play_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<ControlledPlayCalculator>(controlled_play::boxed_default)
}

pub(crate) fn aerial_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<AerialGoalCalculator>(goal_tags::boxed_aerial_goal)
}

pub(crate) fn high_aerial_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<HighAerialGoalCalculator>(goal_tags::boxed_high_aerial_goal)
}

pub(crate) fn long_distance_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<LongDistanceGoalCalculator>(
        goal_tags::boxed_long_distance_goal,
    )
}

pub(crate) fn own_half_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<OwnHalfGoalCalculator>(goal_tags::boxed_own_half_goal)
}

pub(crate) fn empty_net_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<EmptyNetGoalCalculator>(goal_tags::boxed_empty_net_goal)
}

pub(crate) fn counter_attack_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<CounterAttackGoalCalculator>(
        goal_tags::boxed_counter_attack_goal,
    )
}

pub(crate) fn flick_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<FlickGoalCalculator>(goal_tags::boxed_flick_goal)
}

pub(crate) fn double_tap_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<DoubleTapGoalCalculator>(goal_tags::boxed_double_tap_goal)
}

pub(crate) fn one_timer_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<OneTimerGoalCalculator>(goal_tags::boxed_one_timer_goal)
}

pub(crate) fn passing_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<PassingGoalCalculator>(goal_tags::boxed_passing_goal)
}

pub(crate) fn air_dribble_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<AirDribbleGoalCalculator>(goal_tags::boxed_air_dribble_goal)
}

pub(crate) fn flip_reset_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<FlipResetGoalCalculator>(goal_tags::boxed_flip_reset_goal)
}

pub(crate) fn bump_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<BumpGoalCalculator>(goal_tags::boxed_bump_goal)
}

pub(crate) fn demo_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<DemoGoalCalculator>(goal_tags::boxed_demo_goal)
}

pub(crate) fn half_volley_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<HalfVolleyGoalCalculator>(goal_tags::boxed_half_volley_goal)
}

pub(crate) fn backboard_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<BackboardCalculator>(backboard::boxed_default)
}

pub(crate) fn ceiling_shot_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<CeilingShotCalculator>(ceiling_shot::boxed_default)
}

pub(crate) fn center_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<CenterCalculator>(center::boxed_default)
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

pub(crate) fn territorial_pressure_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<TerritorialPressureCalculator>(
        territorial_pressure::boxed_default,
    )
}

pub(crate) fn rotation_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<RotationCalculator>(rotation::boxed_default)
}

pub(crate) fn rush_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<RushCalculator>(rush::boxed_default)
}

pub(crate) fn touch_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<TouchCalculator>(touch::boxed_default)
}

pub(crate) fn wall_aerial_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<WallAerialCalculator>(wall_aerial::boxed_default)
}

pub(crate) fn wall_aerial_shot_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<WallAerialShotCalculator>(wall_aerial_shot::boxed_default)
}

pub(crate) fn whiff_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<WhiffCalculator>(whiff::boxed_default)
}

pub(crate) fn wavedash_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<WavedashCalculator>(wavedash::boxed_default)
}

pub(crate) fn speed_flip_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<SpeedFlipCalculator>(speed_flip::boxed_default)
}

pub(crate) fn half_flip_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<HalfFlipCalculator>(half_flip::boxed_default)
}

pub(crate) fn half_volley_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<HalfVolleyCalculator>(half_volley::boxed_default)
}

pub(crate) fn musty_flick_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<MustyFlickCalculator>(musty_flick::boxed_default)
}

pub(crate) fn pass_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<PassCalculator>(pass::boxed_default)
}

pub(crate) fn one_timer_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<OneTimerCalculator>(one_timer::boxed_default)
}

pub(crate) fn flick_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<FlickCalculator>(flick::boxed_default)
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

pub(crate) fn bump_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<BumpCalculator>(bump::boxed_default)
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

pub(crate) fn stats_projection_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<StatsProjectionState>(stats_projection::boxed_default)
}
