use crate::*;

use super::{
    BallFrameState, BallSample, DemoEventSample, FrameEventsState, FrameInfo, GameplayState,
    PlayerFrameState, PlayerSample,
};

#[derive(Debug, Clone)]
pub struct FrameInput {
    frame_info: FrameInfo,
    gameplay_state: GameplayState,
    ball_frame_state: BallFrameState,
    player_frame_state: PlayerFrameState,
    frame_events_state: FrameEventsState,
}

impl FrameInput {
    /// Builds a frame input from already-materialized frame component states.
    ///
    /// Replay callers should usually use [`FrameInput::timeline`] or
    /// [`FrameInput::aggregate`]. Live callers can construct these same
    /// component states directly from their sampled game state.
    pub fn from_parts(
        frame_info: FrameInfo,
        gameplay_state: GameplayState,
        ball_frame_state: BallFrameState,
        player_frame_state: PlayerFrameState,
        frame_events_state: FrameEventsState,
    ) -> Self {
        Self {
            frame_info,
            gameplay_state,
            ball_frame_state,
            player_frame_state,
            frame_events_state,
        }
    }

    pub fn timeline(
        processor: &dyn ProcessorView,
        frame_number: usize,
        current_time: f32,
        dt: f32,
    ) -> Self {
        Self {
            frame_info: Self::build_frame_info(processor, frame_number, current_time, dt),
            gameplay_state: Self::build_gameplay_state(processor),
            ball_frame_state: Self::build_ball_frame_state(processor, current_time),
            player_frame_state: Self::build_player_frame_state(processor, current_time),
            frame_events_state: Self::build_current_frame_events_state(processor),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn aggregate(
        processor: &dyn ProcessorView,
        frame_number: usize,
        current_time: f32,
        dt: f32,
        last_demolish_count: usize,
        last_boost_pad_event_count: usize,
        last_touch_event_count: usize,
        last_player_stat_event_count: usize,
        last_goal_event_count: usize,
    ) -> Self {
        Self {
            frame_info: Self::build_frame_info(processor, frame_number, current_time, dt),
            gameplay_state: Self::build_gameplay_state(processor),
            ball_frame_state: Self::build_ball_frame_state(processor, current_time),
            player_frame_state: Self::build_player_frame_state(processor, current_time),
            frame_events_state: Self::build_events_since_last_sample(
                processor,
                last_demolish_count,
                last_boost_pad_event_count,
                last_touch_event_count,
                last_player_stat_event_count,
                last_goal_event_count,
            ),
        }
    }

    fn build_frame_info(
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

    fn build_gameplay_state(processor: &dyn ProcessorView) -> GameplayState {
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

    fn build_ball_frame_state(processor: &dyn ProcessorView, current_time: f32) -> BallFrameState {
        processor
            .get_interpolated_ball_rigid_body(current_time, 0.0)
            .ok()
            .map(|rigid_body| BallSample { rigid_body })
            .into()
    }

    fn build_player_frame_state(
        processor: &dyn ProcessorView,
        current_time: f32,
    ) -> PlayerFrameState {
        let mut players = Vec::new();
        for player_id in processor.iter_player_ids_in_order() {
            let Ok(is_team_0) = processor.get_player_is_team_0(player_id) else {
                continue;
            };
            players.push(PlayerSample {
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
            });
        }
        PlayerFrameState { players }
    }

    fn build_current_frame_events_state(processor: &dyn ProcessorView) -> FrameEventsState {
        let active_demos = if let Ok(demos) = processor.get_active_demos() {
            demos
                .into_iter()
                .filter_map(|demo| {
                    let attacker = processor
                        .get_player_id_from_car_id(&demo.attacker_actor_id())
                        .ok()?;
                    let victim = processor
                        .get_player_id_from_car_id(&demo.victim_actor_id())
                        .ok()?;
                    Some(DemoEventSample { attacker, victim })
                })
                .collect()
        } else {
            Vec::new()
        };
        FrameEventsState {
            active_demos,
            demo_events: Vec::new(),
            boost_pad_events: processor.current_frame_boost_pad_events().to_vec(),
            touch_events: processor.current_frame_touch_events().to_vec(),
            dodge_refreshed_events: processor.current_frame_dodge_refreshed_events().to_vec(),
            player_stat_events: processor.current_frame_player_stat_events().to_vec(),
            goal_events: processor.current_frame_goal_events().to_vec(),
        }
    }

    fn build_events_since_last_sample(
        processor: &dyn ProcessorView,
        last_demolish_count: usize,
        last_boost_pad_event_count: usize,
        last_touch_event_count: usize,
        last_player_stat_event_count: usize,
        last_goal_event_count: usize,
    ) -> FrameEventsState {
        FrameEventsState {
            active_demos: Vec::new(),
            demo_events: processor.demolishes()[last_demolish_count..].to_vec(),
            boost_pad_events: processor.boost_pad_events()[last_boost_pad_event_count..].to_vec(),
            touch_events: processor.touch_events()[last_touch_event_count..].to_vec(),
            dodge_refreshed_events: processor.current_frame_dodge_refreshed_events().to_vec(),
            player_stat_events: processor.player_stat_events()[last_player_stat_event_count..]
                .to_vec(),
            goal_events: processor.goal_events()[last_goal_event_count..].to_vec(),
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
}
