use super::*;

pub trait ProcessorEventHistoryView {
    fn get_player_match_goals(&self, player_id: &PlayerId) -> SubtrActorResult<i32>;
    fn get_player_match_saves(&self, player_id: &PlayerId) -> SubtrActorResult<i32>;
    fn get_player_match_score(&self, player_id: &PlayerId) -> SubtrActorResult<i32>;
    fn get_player_match_shots(&self, player_id: &PlayerId) -> SubtrActorResult<i32>;

    fn get_active_demos(&self) -> SubtrActorResult<Vec<DemolishAttribute>>;
    fn demolishes(&self) -> &[DemolishInfo];
    fn boost_pad_events(&self) -> &[BoostPadEvent];
    fn touch_events(&self) -> &[TouchEvent];
    fn dodge_refreshed_events(&self) -> &[DodgeRefreshedEvent];
    fn player_stat_events(&self) -> &[PlayerStatEvent];
    fn goal_events(&self) -> &[GoalEvent];
}
