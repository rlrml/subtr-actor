use super::*;

pub(super) fn team_calc<C, T>(
    ctx: &AnalysisStateContext<'_>,
    is_team_zero: bool,
    team_zero: impl Fn(&C) -> &T,
    team_one: impl Fn(&C) -> &T,
) -> SubtrActorResult<T>
where
    C: 'static,
    T: Clone,
{
    let calculator = ctx.get::<C>()?;
    Ok(team_value(
        team_zero(calculator),
        team_one(calculator),
        is_team_zero,
    ))
}

pub(super) fn team_value<T: Clone>(team_zero: &T, team_one: &T, is_team_zero: bool) -> T {
    if is_team_zero {
        team_zero.clone()
    } else {
        team_one.clone()
    }
}
