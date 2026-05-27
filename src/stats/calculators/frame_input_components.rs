use super::*;

impl FrameInput {
    pub(super) fn build_frame_info(
        processor: &dyn ProcessorView,
        frame_number: usize,
        current_time: f32,
        dt: f32,
    ) -> FrameInfo {
        FrameInfo {
            frame_number,
            time: current_time,
            dt,
            seconds_remaining: processor.get_seconds_remaining().ok(),
        }
    }

    pub(super) fn build_gameplay_state(processor: &dyn ProcessorView) -> GameplayState {
        let team_scores = processor.get_team_scores().ok();
        let possession_team_is_team_0 =
            processor
                .get_ball_hit_team_num()
                .ok()
                .and_then(|team_num| match team_num {
                    0 => Some(true),
                    1 => Some(false),
                    _ => None,
                });
        let scored_on_team_is_team_0 =
            processor
                .get_scored_on_team_num()
                .ok()
                .and_then(|team_num| match team_num {
                    0 => Some(true),
                    1 => Some(false),
                    _ => None,
                });
        GameplayState {
            game_state: processor.get_replicated_state_name().ok(),
            ball_has_been_hit: processor.get_ball_has_been_hit().ok(),
            kickoff_countdown_time: processor.get_replicated_game_state_time_remaining().ok(),
            team_zero_score: team_scores.map(|scores| scores.0),
            team_one_score: team_scores.map(|scores| scores.1),
            possession_team_is_team_0,
            scored_on_team_is_team_0,
            current_in_game_team_player_counts: processor.current_in_game_team_player_counts(),
        }
    }

    pub(super) fn build_ball_frame_state(
        processor: &dyn ProcessorView,
        current_time: f32,
    ) -> BallFrameState {
        processor
            .get_interpolated_ball_rigid_body(current_time, 0.0)
            .ok()
            .map(|rigid_body| BallSample { rigid_body })
            .into()
    }

    pub(super) fn build_player_frame_state(
        processor: &dyn ProcessorView,
        current_time: f32,
    ) -> PlayerFrameState {
        let mut players = Vec::new();
        for player_id in processor.iter_player_ids_in_order() {
            let Ok(is_team_0) = processor.get_player_is_team_0(player_id) else {
                continue;
            };
            players.push(Self::build_player_sample(
                processor,
                player_id,
                is_team_0,
                current_time,
            ));
        }
        PlayerFrameState { players }
    }
}
