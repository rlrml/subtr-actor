use super::*;

impl ProcessorPlayerCoreView for ReplayProcessor<'_> {
    fn get_normalized_player_rigid_body(
        &self,
        player_id: &PlayerId,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        ReplayProcessor::get_normalized_player_rigid_body(self, player_id)
    }

    fn get_velocity_applied_player_rigid_body(
        &self,
        player_id: &PlayerId,
        target_time: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        ReplayProcessor::get_velocity_applied_player_rigid_body(self, player_id, target_time)
    }

    fn get_interpolated_player_rigid_body(
        &self,
        player_id: &PlayerId,
        target_time: f32,
        close_enough_to_frame_time: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        ReplayProcessor::get_interpolated_player_rigid_body(
            self,
            player_id,
            target_time,
            close_enough_to_frame_time,
        )
    }

    fn get_player_name(&self, player_id: &PlayerId) -> SubtrActorResult<String> {
        ReplayProcessor::get_player_name(self, player_id)
    }

    fn get_player_team_key(&self, player_id: &PlayerId) -> SubtrActorResult<String> {
        ReplayProcessor::get_player_team_key(self, player_id)
    }

    fn get_player_is_team_0(&self, player_id: &PlayerId) -> SubtrActorResult<bool> {
        ReplayProcessor::get_player_is_team_0(self, player_id)
    }
}
