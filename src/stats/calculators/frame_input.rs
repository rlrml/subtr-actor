use crate::*;

use super::{
    BallFrameState, BallSample, DemoEventSample, FrameEventsState, FrameInfo, GameplayState,
    LivePlayState, PlayerFrameState, PlayerSample,
};

#[derive(Debug, Clone, Default)]
pub struct ReplayFrameInputBuilder {
    aggregate_events: FrameEventsSampler,
}

#[derive(Debug, Clone, Default)]
struct FrameEventsSampler {
    cursors: ProcessorEventCursors,
}

impl ReplayFrameInputBuilder {
    pub fn timeline(
        &mut self,
        processor: &dyn ProcessorView,
        frame_number: usize,
        current_time: f32,
        dt: f32,
    ) -> FrameInput {
        FrameInput::timeline(processor, frame_number, current_time, dt)
    }

    pub fn aggregate(
        &mut self,
        processor: &dyn ProcessorView,
        frame_number: usize,
        current_time: f32,
        dt: f32,
    ) -> FrameInput {
        let frame_events_state = self.aggregate_events.events_since_last_sample(processor);
        FrameInput::from_processor_and_events(
            processor,
            frame_number,
            current_time,
            dt,
            frame_events_state,
        )
    }
}

impl FrameEventsSampler {
    fn events_since_last_sample(&mut self, processor: &dyn ProcessorView) -> FrameEventsState {
        let events = FrameInput::build_events_since_last_sample(processor, &self.cursors);
        self.cursors = ProcessorEventCursors::from_processor(processor);
        events
    }
}

#[derive(Debug, Clone, Default)]
struct ProcessorEventCursors {
    demolish_count: usize,
    boost_pad_event_count: usize,
    touch_event_count: usize,
    dodge_refreshed_event_count: usize,
    player_stat_event_count: usize,
    goal_event_count: usize,
}

impl ProcessorEventCursors {
    fn from_processor(processor: &dyn ProcessorView) -> Self {
        Self {
            demolish_count: processor.demolishes().len(),
            boost_pad_event_count: processor.boost_pad_events().len(),
            touch_event_count: processor.touch_events().len(),
            dodge_refreshed_event_count: processor.dodge_refreshed_events().len(),
            player_stat_event_count: processor.player_stat_events().len(),
            goal_event_count: processor.goal_events().len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrameInput {
    frame_info: FrameInfo,
    gameplay_state: GameplayState,
    ball_frame_state: BallFrameState,
    player_frame_state: PlayerFrameState,
    frame_events_state: FrameEventsState,
    live_play_state: Option<LivePlayState>,
}

impl FrameInput {
    /// Builds a frame input from already-materialized frame component states.
    ///
    /// Replay callers should usually use [`ReplayFrameInputBuilder`]. Live
    /// callers can construct these same component states directly from their
    /// sampled game state.
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
            live_play_state: None,
        }
    }

    /// Builds a frame input with an explicitly sampled live-play state.
    ///
    /// Replay processing should let the graph derive live play from replicated
    /// gameplay fields. Live callers can use this when the host integration has
    /// a stronger source of truth for whether analysis should run on a frame.
    pub fn from_parts_with_live_play_state(
        frame_info: FrameInfo,
        gameplay_state: GameplayState,
        ball_frame_state: BallFrameState,
        player_frame_state: PlayerFrameState,
        frame_events_state: FrameEventsState,
        live_play_state: LivePlayState,
    ) -> Self {
        Self {
            frame_info,
            gameplay_state,
            ball_frame_state,
            player_frame_state,
            frame_events_state,
            live_play_state: Some(live_play_state),
        }
    }

    pub fn timeline(
        processor: &dyn ProcessorView,
        frame_number: usize,
        current_time: f32,
        dt: f32,
    ) -> Self {
        let frame_events_state = Self::build_current_frame_events_state(processor);
        Self::from_processor_and_events(
            processor,
            frame_number,
            current_time,
            dt,
            frame_events_state,
        )
    }

    fn from_processor_and_events(
        processor: &dyn ProcessorView,
        frame_number: usize,
        current_time: f32,
        dt: f32,
        frame_events_state: FrameEventsState,
    ) -> Self {
        Self {
            frame_info: Self::build_frame_info(processor, frame_number, current_time, dt),
            gameplay_state: Self::build_gameplay_state(processor),
            ball_frame_state: Self::build_ball_frame_state(processor, current_time),
            player_frame_state: Self::build_player_frame_state(processor, current_time),
            frame_events_state,
            live_play_state: None,
        }
    }

    pub fn timeline_with_live_play_state(
        processor: &dyn ProcessorView,
        frame_number: usize,
        current_time: f32,
        dt: f32,
        live_play_state: LivePlayState,
    ) -> Self {
        let mut input = Self::timeline(processor, frame_number, current_time, dt);
        input.live_play_state = Some(live_play_state);
        input
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
                hitbox: processor.get_player_car_hitbox(player_id),
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

    fn build_active_demo_events(processor: &dyn ProcessorView) -> Vec<DemoEventSample> {
        let active_demo_events = processor.current_frame_active_demo_events();
        if !active_demo_events.is_empty() {
            active_demo_events.to_vec()
        } else if let Ok(demos) = processor.get_active_demos() {
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
        }
    }

    fn build_current_frame_events_state(processor: &dyn ProcessorView) -> FrameEventsState {
        FrameEventsState {
            active_demos: Self::build_active_demo_events(processor),
            demo_events: processor.current_frame_demolish_events().to_vec(),
            boost_pad_events: processor.current_frame_boost_pad_events().to_vec(),
            touch_events: processor.current_frame_touch_events().to_vec(),
            dodge_refreshed_events: processor.current_frame_dodge_refreshed_events().to_vec(),
            player_stat_events: processor.current_frame_player_stat_events().to_vec(),
            goal_events: processor.current_frame_goal_events().to_vec(),
        }
    }

    fn build_events_since_last_sample(
        processor: &dyn ProcessorView,
        event_cursors: &ProcessorEventCursors,
    ) -> FrameEventsState {
        FrameEventsState {
            active_demos: Self::build_active_demo_events(processor),
            demo_events: processor.demolishes()[event_cursors.demolish_count..].to_vec(),
            boost_pad_events: processor.boost_pad_events()[event_cursors.boost_pad_event_count..]
                .to_vec(),
            touch_events: processor.touch_events()[event_cursors.touch_event_count..].to_vec(),
            dodge_refreshed_events: processor.dodge_refreshed_events()
                [event_cursors.dodge_refreshed_event_count..]
                .to_vec(),
            player_stat_events: processor.player_stat_events()
                [event_cursors.player_stat_event_count..]
                .to_vec(),
            goal_events: processor.goal_events()[event_cursors.goal_event_count..].to_vec(),
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
