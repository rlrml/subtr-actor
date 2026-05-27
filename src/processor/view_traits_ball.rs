use super::*;

pub trait ProcessorBallView {
    fn get_ignore_ball_syncing(&self) -> SubtrActorResult<bool>;
    fn get_team_scores(&self) -> SubtrActorResult<(i32, i32)>;
    fn get_ball_hit_team_num(&self) -> SubtrActorResult<u8>;
    fn get_scored_on_team_num(&self) -> SubtrActorResult<u8>;
    fn get_normalized_ball_rigid_body(&self) -> SubtrActorResult<boxcars::RigidBody>;
    fn get_velocity_applied_ball_rigid_body(
        &self,
        target_time: f32,
    ) -> SubtrActorResult<boxcars::RigidBody>;
    fn get_interpolated_ball_rigid_body(
        &self,
        target_time: f32,
        close_enough_to_frame_time: f32,
    ) -> SubtrActorResult<boxcars::RigidBody>;
}
