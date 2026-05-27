use super::*;

pub(super) struct GoalTagEventSources<'a> {
    aerial: &'a AerialGoalCalculator,
    high_aerial: &'a HighAerialGoalCalculator,
    long_distance: &'a LongDistanceGoalCalculator,
    own_half: &'a OwnHalfGoalCalculator,
    empty_net: &'a EmptyNetGoalCalculator,
    counter_attack: &'a CounterAttackGoalCalculator,
    flick: &'a FlickGoalCalculator,
    double_tap: &'a DoubleTapGoalCalculator,
    one_timer: &'a OneTimerGoalCalculator,
    passing: &'a PassingGoalCalculator,
    air_dribble: &'a AirDribbleGoalCalculator,
    flip_reset: &'a FlipResetGoalCalculator,
    half_volley: &'a HalfVolleyGoalCalculator,
}

impl<'a> GoalTagEventSources<'a> {
    pub(super) fn from_context(ctx: &'a AnalysisStateContext<'_>) -> SubtrActorResult<Self> {
        Ok(Self {
            aerial: ctx.get::<AerialGoalCalculator>()?,
            high_aerial: ctx.get::<HighAerialGoalCalculator>()?,
            long_distance: ctx.get::<LongDistanceGoalCalculator>()?,
            own_half: ctx.get::<OwnHalfGoalCalculator>()?,
            empty_net: ctx.get::<EmptyNetGoalCalculator>()?,
            counter_attack: ctx.get::<CounterAttackGoalCalculator>()?,
            flick: ctx.get::<FlickGoalCalculator>()?,
            double_tap: ctx.get::<DoubleTapGoalCalculator>()?,
            one_timer: ctx.get::<OneTimerGoalCalculator>()?,
            passing: ctx.get::<PassingGoalCalculator>()?,
            air_dribble: ctx.get::<AirDribbleGoalCalculator>()?,
            flip_reset: ctx.get::<FlipResetGoalCalculator>()?,
            half_volley: ctx.get::<HalfVolleyGoalCalculator>()?,
        })
    }

    pub(super) fn combined_events(&self) -> Vec<GoalTagEvent> {
        combined_goal_tag_events(&[
            self.aerial.events(),
            self.high_aerial.events(),
            self.long_distance.events(),
            self.own_half.events(),
            self.empty_net.events(),
            self.counter_attack.events(),
            self.flick.events(),
            self.double_tap.events(),
            self.one_timer.events(),
            self.passing.events(),
            self.air_dribble.events(),
            self.flip_reset.events(),
            self.half_volley.events(),
        ])
    }
}
