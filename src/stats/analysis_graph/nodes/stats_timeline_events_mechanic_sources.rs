use super::*;

pub(super) struct MechanicEventSources<'a> {
    pub(super) ball_carry: &'a BallCarryCalculator,
    pub(super) ceiling_shot: &'a CeilingShotCalculator,
    pub(super) wall_aerial: &'a WallAerialCalculator,
    pub(super) wall_aerial_shot: &'a WallAerialShotCalculator,
    pub(super) center: &'a CenterCalculator,
    pub(super) dodge_reset: &'a DodgeResetCalculator,
    pub(super) double_tap: &'a DoubleTapCalculator,
    pub(super) flick: &'a FlickCalculator,
    pub(super) musty_flick: &'a MustyFlickCalculator,
    pub(super) one_timer: &'a OneTimerCalculator,
    pub(super) pass: &'a PassCalculator,
    pub(super) speed_flip: &'a SpeedFlipCalculator,
    pub(super) half_flip: &'a HalfFlipCalculator,
    pub(super) half_volley: &'a HalfVolleyCalculator,
    pub(super) wavedash: &'a WavedashCalculator,
}

impl<'a> MechanicEventSources<'a> {
    pub(super) fn from_context(ctx: &'a AnalysisStateContext<'_>) -> SubtrActorResult<Self> {
        Ok(Self {
            ball_carry: ctx.get::<BallCarryCalculator>()?,
            ceiling_shot: ctx.get::<CeilingShotCalculator>()?,
            wall_aerial: ctx.get::<WallAerialCalculator>()?,
            wall_aerial_shot: ctx.get::<WallAerialShotCalculator>()?,
            center: ctx.get::<CenterCalculator>()?,
            dodge_reset: ctx.get::<DodgeResetCalculator>()?,
            double_tap: ctx.get::<DoubleTapCalculator>()?,
            flick: ctx.get::<FlickCalculator>()?,
            musty_flick: ctx.get::<MustyFlickCalculator>()?,
            one_timer: ctx.get::<OneTimerCalculator>()?,
            pass: ctx.get::<PassCalculator>()?,
            speed_flip: ctx.get::<SpeedFlipCalculator>()?,
            half_flip: ctx.get::<HalfFlipCalculator>()?,
            half_volley: ctx.get::<HalfVolleyCalculator>()?,
            wavedash: ctx.get::<WavedashCalculator>()?,
        })
    }
}
