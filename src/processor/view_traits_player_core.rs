use super::*;

pub trait ProcessorPlayerCoreView {
    fn get_normalized_player_rigid_body(
        &self,
        player_id: &PlayerId,
    ) -> SubtrActorResult<boxcars::RigidBody>;
    fn get_velocity_applied_player_rigid_body(
        &self,
        player_id: &PlayerId,
        target_time: f32,
    ) -> SubtrActorResult<boxcars::RigidBody>;
    fn get_interpolated_player_rigid_body(
        &self,
        player_id: &PlayerId,
        target_time: f32,
        close_enough_to_frame_time: f32,
    ) -> SubtrActorResult<boxcars::RigidBody>;

    fn get_player_name(&self, player_id: &PlayerId) -> SubtrActorResult<String>;
    fn get_player_team_key(&self, player_id: &PlayerId) -> SubtrActorResult<String>;
    fn get_player_is_team_0(&self, player_id: &PlayerId) -> SubtrActorResult<bool>;
}
