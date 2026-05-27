use super::*;

impl ProcessorGameView for ReplayProcessor<'_> {
    fn get_replay_meta(&self) -> SubtrActorResult<ReplayMeta> {
        ReplayProcessor::get_replay_meta(self)
    }

    fn player_count(&self) -> usize {
        ReplayProcessor::player_count(self)
    }

    fn iter_player_ids_in_order(&self) -> Box<dyn Iterator<Item = &PlayerId> + '_> {
        Box::new(ReplayProcessor::iter_player_ids_in_order(self))
    }

    fn current_in_game_team_player_counts(&self) -> [usize; 2] {
        ReplayProcessor::current_in_game_team_player_counts(self)
    }

    fn get_seconds_remaining(&self) -> SubtrActorResult<i32> {
        ReplayProcessor::get_seconds_remaining(self)
    }

    fn get_replicated_state_name(&self) -> SubtrActorResult<i32> {
        ReplayProcessor::get_replicated_state_name(self)
    }

    fn get_replicated_game_state_time_remaining(&self) -> SubtrActorResult<i32> {
        ReplayProcessor::get_replicated_game_state_time_remaining(self)
    }

    fn get_ball_has_been_hit(&self) -> SubtrActorResult<bool> {
        ReplayProcessor::get_ball_has_been_hit(self)
    }
}
