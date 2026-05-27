use super::*;

impl FrameInput {
    pub(super) fn build_player_sample(
        processor: &dyn ProcessorView,
        player_id: &PlayerId,
        is_team_0: bool,
        current_time: f32,
    ) -> PlayerSample {
        PlayerSample {
            player_id: player_id.clone(),
            is_team_0,
            rigid_body: processor
                .get_interpolated_player_rigid_body(player_id, current_time, 0.0)
                .ok()
                .filter(|rigid_body| !rigid_body.sleeping),
            boost_amount: processor.get_player_boost_level(player_id).ok(),
            last_boost_amount: processor.get_player_last_boost_level(player_id).ok(),
            boost_active: processor.get_boost_active(player_id).unwrap_or(0) % 2 == 1,
            dodge_active: processor.get_dodge_active(player_id).unwrap_or(0) % 2 == 1,
            powerslide_active: processor.get_powerslide_active(player_id).unwrap_or(false),
            match_goals: processor.get_player_match_goals(player_id).ok(),
            match_assists: processor.get_player_match_assists(player_id).ok(),
            match_saves: processor.get_player_match_saves(player_id).ok(),
            match_shots: processor.get_player_match_shots(player_id).ok(),
            match_score: processor.get_player_match_score(player_id).ok(),
        }
    }

    pub fn frame_info(&self) -> FrameInfo {
        self.frame_info.clone()
    }

    pub fn gameplay_state(&self) -> GameplayState {
        self.gameplay_state.clone()
    }

    pub fn ball_frame_state(&self) -> BallFrameState {
        self.ball_frame_state.clone()
    }

    pub fn player_frame_state(&self) -> PlayerFrameState {
        self.player_frame_state.clone()
    }

    pub fn frame_events_state(&self) -> FrameEventsState {
        self.frame_events_state.clone()
    }

    pub fn live_play_state(&self) -> Option<LivePlayState> {
        self.live_play_state.clone()
    }
}
