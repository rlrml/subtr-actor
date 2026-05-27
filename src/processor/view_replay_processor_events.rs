use super::*;

impl ProcessorEventHistoryView for ReplayProcessor<'_> {
    fn get_player_match_goals(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        ReplayProcessor::get_player_match_goals(self, player_id)
    }

    fn get_player_match_saves(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        ReplayProcessor::get_player_match_saves(self, player_id)
    }

    fn get_player_match_score(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        ReplayProcessor::get_player_match_score(self, player_id)
    }

    fn get_player_match_shots(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        ReplayProcessor::get_player_match_shots(self, player_id)
    }

    fn get_active_demos(&self) -> SubtrActorResult<Vec<DemolishAttribute>> {
        ReplayProcessor::get_active_demos(self).map(Iterator::collect)
    }

    fn demolishes(&self) -> &[DemolishInfo] {
        &self.demolishes
    }

    fn boost_pad_events(&self) -> &[BoostPadEvent] {
        &self.boost_pad_events
    }

    fn touch_events(&self) -> &[TouchEvent] {
        &self.touch_events
    }

    fn dodge_refreshed_events(&self) -> &[DodgeRefreshedEvent] {
        &self.dodge_refreshed_events
    }

    fn player_stat_events(&self) -> &[PlayerStatEvent] {
        &self.player_stat_events
    }

    fn goal_events(&self) -> &[GoalEvent] {
        &self.goal_events
    }
}
