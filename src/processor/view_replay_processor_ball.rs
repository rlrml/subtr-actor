use super::*;

impl ProcessorBallView for ReplayProcessor<'_> {
    fn get_ignore_ball_syncing(&self) -> SubtrActorResult<bool> {
        ReplayProcessor::get_ignore_ball_syncing(self)
    }

    fn get_team_scores(&self) -> SubtrActorResult<(i32, i32)> {
        ReplayProcessor::get_team_scores(self)
    }

    fn get_ball_hit_team_num(&self) -> SubtrActorResult<u8> {
        ReplayProcessor::get_ball_hit_team_num(self)
    }

    fn get_scored_on_team_num(&self) -> SubtrActorResult<u8> {
        ReplayProcessor::get_scored_on_team_num(self)
    }

    fn get_normalized_ball_rigid_body(&self) -> SubtrActorResult<boxcars::RigidBody> {
        ReplayProcessor::get_normalized_ball_rigid_body(self)
    }

    fn get_velocity_applied_ball_rigid_body(
        &self,
        target_time: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        ReplayProcessor::get_velocity_applied_ball_rigid_body(self, target_time)
    }

    fn get_interpolated_ball_rigid_body(
        &self,
        target_time: f32,
        close_enough_to_frame_time: f32,
    ) -> SubtrActorResult<boxcars::RigidBody> {
        ReplayProcessor::get_interpolated_ball_rigid_body(
            self,
            target_time,
            close_enough_to_frame_time,
        )
    }
}
