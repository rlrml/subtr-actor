use super::*;
use crate::stats::calculators::{
    BackboardCalculator, CeilingShotCalculator, CenterCalculator, DoubleTapCalculator,
    FiftyFiftyCalculator, HalfVolleyCalculator, OneTimerCalculator, PassCalculator,
    PossessionCalculator, PressureCalculator, RotationCalculator, RushCalculator,
    TerritorialPressureCalculator, TouchCalculator, WallAerialCalculator, WallAerialShotCalculator,
    WhiffCalculator,
};

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

pub(crate) fn half_volley_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<HalfVolleyCalculator>(half_volley::boxed_default)
}

pub(crate) fn pass_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<PassCalculator>(pass::boxed_default)
}

pub(crate) fn one_timer_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<OneTimerCalculator>(one_timer::boxed_default)
}
