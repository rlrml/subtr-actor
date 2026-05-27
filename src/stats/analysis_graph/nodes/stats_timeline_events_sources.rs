use super::*;

pub(super) struct StatsTimelineEventSources<'a> {
    pub(super) match_stats: &'a MatchStatsCalculator,
    pub(super) possession: &'a PossessionCalculator,
    pub(super) pressure: &'a PressureCalculator,
    pub(super) territorial_pressure: &'a TerritorialPressureCalculator,
    pub(super) movement: &'a MovementCalculator,
    pub(super) positioning: &'a PositioningCalculator,
    pub(super) rotation: &'a RotationCalculator,
    pub(super) demo: &'a DemoCalculator,
    pub(super) backboard: &'a BackboardCalculator,
    pub(super) fifty_fifty: &'a FiftyFiftyCalculator,
    pub(super) rush: &'a RushCalculator,
    pub(super) whiff: &'a WhiffCalculator,
    pub(super) powerslide: &'a PowerslideCalculator,
    pub(super) touch: &'a TouchCalculator,
    pub(super) boost: &'a BoostCalculator,
    pub(super) bump: &'a BumpCalculator,
    pub(super) mechanics: MechanicEventSources<'a>,
    pub(super) goal_tags: GoalTagEventSources<'a>,
}

impl<'a> StatsTimelineEventSources<'a> {
    pub(super) fn from_context(ctx: &'a AnalysisStateContext<'_>) -> SubtrActorResult<Self> {
        Ok(Self {
            match_stats: ctx.get::<MatchStatsCalculator>()?,
            possession: ctx.get::<PossessionCalculator>()?,
            pressure: ctx.get::<PressureCalculator>()?,
            territorial_pressure: ctx.get::<TerritorialPressureCalculator>()?,
            movement: ctx.get::<MovementCalculator>()?,
            positioning: ctx.get::<PositioningCalculator>()?,
            rotation: ctx.get::<RotationCalculator>()?,
            demo: ctx.get::<DemoCalculator>()?,
            backboard: ctx.get::<BackboardCalculator>()?,
            fifty_fifty: ctx.get::<FiftyFiftyCalculator>()?,
            rush: ctx.get::<RushCalculator>()?,
            whiff: ctx.get::<WhiffCalculator>()?,
            powerslide: ctx.get::<PowerslideCalculator>()?,
            touch: ctx.get::<TouchCalculator>()?,
            boost: ctx.get::<BoostCalculator>()?,
            bump: ctx.get::<BumpCalculator>()?,
            mechanics: MechanicEventSources::from_context(ctx)?,
            goal_tags: GoalTagEventSources::from_context(ctx)?,
        })
    }
}
