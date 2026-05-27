use super::*;

pub trait ProcessorPlayerStatsView {
    fn get_player_id_from_car_id(&self, actor_id: &boxcars::ActorId) -> SubtrActorResult<PlayerId>;
    fn get_player_boost_level(&self, player_id: &PlayerId) -> SubtrActorResult<f32>;
    fn get_player_last_boost_level(&self, player_id: &PlayerId) -> SubtrActorResult<f32>;
    fn get_player_boost_percentage(&self, player_id: &PlayerId) -> SubtrActorResult<f32>;
    fn get_boost_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8>;
    fn get_jump_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8>;
    fn get_double_jump_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8>;
    fn get_dodge_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8>;
    fn get_powerslide_active(&self, player_id: &PlayerId) -> SubtrActorResult<bool>;
    fn get_player_match_assists(&self, player_id: &PlayerId) -> SubtrActorResult<i32>;
}
