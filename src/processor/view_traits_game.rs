use super::*;

pub trait ProcessorGameView {
    fn get_replay_meta(&self) -> SubtrActorResult<ReplayMeta>;
    fn player_count(&self) -> usize;
    fn iter_player_ids_in_order(&self) -> Box<dyn Iterator<Item = &PlayerId> + '_>;
    fn current_in_game_team_player_counts(&self) -> [usize; 2];
    fn get_seconds_remaining(&self) -> SubtrActorResult<i32>;
    fn get_replicated_state_name(&self) -> SubtrActorResult<i32>;
    fn get_replicated_game_state_time_remaining(&self) -> SubtrActorResult<i32>;
    fn get_ball_has_been_hit(&self) -> SubtrActorResult<bool>;
}
