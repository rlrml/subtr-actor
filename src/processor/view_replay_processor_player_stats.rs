use super::*;

impl ProcessorPlayerStatsView for ReplayProcessor<'_> {
    fn get_player_id_from_car_id(&self, actor_id: &boxcars::ActorId) -> SubtrActorResult<PlayerId> {
        ReplayProcessor::get_player_id_from_car_id(self, actor_id)
    }

    fn get_player_boost_level(&self, player_id: &PlayerId) -> SubtrActorResult<f32> {
        ReplayProcessor::get_player_boost_level(self, player_id)
    }

    fn get_player_last_boost_level(&self, player_id: &PlayerId) -> SubtrActorResult<f32> {
        ReplayProcessor::get_player_last_boost_level(self, player_id)
    }

    fn get_player_boost_percentage(&self, player_id: &PlayerId) -> SubtrActorResult<f32> {
        ReplayProcessor::get_player_boost_percentage(self, player_id)
    }

    fn get_boost_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        ReplayProcessor::get_boost_active(self, player_id)
    }

    fn get_jump_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        ReplayProcessor::get_jump_active(self, player_id)
    }

    fn get_double_jump_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        ReplayProcessor::get_double_jump_active(self, player_id)
    }

    fn get_dodge_active(&self, player_id: &PlayerId) -> SubtrActorResult<u8> {
        ReplayProcessor::get_dodge_active(self, player_id)
    }

    fn get_powerslide_active(&self, player_id: &PlayerId) -> SubtrActorResult<bool> {
        ReplayProcessor::get_powerslide_active(self, player_id)
    }

    fn get_player_match_assists(&self, player_id: &PlayerId) -> SubtrActorResult<i32> {
        ReplayProcessor::get_player_match_assists(self, player_id)
    }
}
