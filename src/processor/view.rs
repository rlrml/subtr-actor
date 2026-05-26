use super::*;

/// Read-only processor surface consumed by collectors and stat calculators.
///
/// `ReplayProcessor` still owns replay traversal and actor-state mutation, but
/// collectors should depend on this trait so the same collection pipeline can
/// later be driven by non-replay state sources.
pub trait ProcessorView {
    fn get_replay_meta(&self) -> SubtrActorResult<ReplayMeta>;
    fn player_count(&self) -> usize;
    fn iter_player_ids_in_order(&self) -> Box<dyn Iterator<Item = &PlayerId> + '_>;
    fn current_in_game_team_player_counts(&self) -> [usize; 2];

    fn get_seconds_remaining(&self) -> SubtrActorResult<i32>;
    fn get_replicated_state_name(&self) -> SubtrActorResult<i32>;
    fn get_replicated_game_state_time_remaining(&self) -> SubtrActorResult<i32>;
    fn get_ball_has_been_hit(&self) -> SubtrActorResult<bool>;
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
    fn current_frame_active_demo_events(&self) -> &[DemoEventSample] {
        &[]
    }
    fn current_frame_demolish_events(&self) -> &[DemolishInfo] {
        &[]
    }
    fn current_frame_boost_pad_events(&self) -> &[BoostPadEvent];
    fn current_frame_touch_events(&self) -> &[TouchEvent];
    fn current_frame_dodge_refreshed_events(&self) -> &[DodgeRefreshedEvent];
    fn current_frame_player_stat_events(&self) -> &[PlayerStatEvent];
    fn current_frame_goal_events(&self) -> &[GoalEvent];
}

impl ProcessorView for ReplayProcessor<'_> {
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

    fn current_frame_boost_pad_events(&self) -> &[BoostPadEvent] {
        ReplayProcessor::current_frame_boost_pad_events(self)
    }

    fn current_frame_touch_events(&self) -> &[TouchEvent] {
        ReplayProcessor::current_frame_touch_events(self)
    }

    fn current_frame_dodge_refreshed_events(&self) -> &[DodgeRefreshedEvent] {
        ReplayProcessor::current_frame_dodge_refreshed_events(self)
    }

    fn current_frame_player_stat_events(&self) -> &[PlayerStatEvent] {
        ReplayProcessor::current_frame_player_stat_events(self)
    }

    fn current_frame_goal_events(&self) -> &[GoalEvent] {
        ReplayProcessor::current_frame_goal_events(self)
    }
}
