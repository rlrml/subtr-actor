use super::*;
use crate::stats::calculators::{
    BallCarryCalculator, BoostCalculator, BumpCalculator, DemoCalculator, DodgeResetCalculator,
    FlickCalculator, HalfFlipCalculator, MovementCalculator, MustyFlickCalculator,
    PositioningCalculator, PowerslideCalculator, SpeedFlipCalculator, WavedashCalculator,
};

pub(crate) fn wavedash_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<WavedashCalculator>(wavedash::boxed_default)
}

pub(crate) fn speed_flip_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<SpeedFlipCalculator>(speed_flip::boxed_default)
}

pub(crate) fn half_flip_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<HalfFlipCalculator>(half_flip::boxed_default)
}

pub(crate) fn musty_flick_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<MustyFlickCalculator>(musty_flick::boxed_default)
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
