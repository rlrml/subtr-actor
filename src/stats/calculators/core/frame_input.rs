use crate::*;

use super::{
    BallFrameState, BallSample, DemoEventSample, FrameEventsState, FrameInfo, FrameState,
    GameplayState, PlayerFrameState, PlayerSample,
};

#[derive(Debug, Clone, Copy)]
enum EventWindow {
    CurrentFrame,
    SinceLastSample {
        last_demolish_count: usize,
        last_boost_pad_event_count: usize,
        last_touch_event_count: usize,
        last_player_stat_event_count: usize,
        last_goal_event_count: usize,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct FrameInput {
    processor: *const (),
    frame_number: usize,
    current_time: f32,
    dt: f32,
    event_window: EventWindow,
}

impl FrameInput {
    pub fn timeline(
        processor: &ReplayProcessor,
        frame_number: usize,
        current_time: f32,
        dt: f32,
    ) -> Self {
        Self {
            processor: processor as *const ReplayProcessor<'_> as *const (),
            frame_number,
            current_time,
            dt,
            event_window: EventWindow::CurrentFrame,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn aggregate(
        processor: &ReplayProcessor,
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
            processor: processor as *const ReplayProcessor<'_> as *const (),
            frame_number,
            current_time,
            dt,
            event_window: EventWindow::SinceLastSample {
                last_demolish_count,
                last_boost_pad_event_count,
                last_touch_event_count,
                last_player_stat_event_count,
                last_goal_event_count,
            },
        }
    }

    fn processor(&self) -> &ReplayProcessor<'_> {
        // `FrameInput` is only used while evaluating the graph, so the replay
        // processor borrowed by the collector outlives this ephemeral wrapper.
        unsafe { &*(self.processor as *const ReplayProcessor<'_>) }
    }

    pub fn frame_state(&self) -> SubtrActorResult<FrameState> {
        Ok(FrameState::from_parts(
            self.frame_info(),
            self.gameplay_state(),
            self.ball_frame_state(),
            self.player_frame_state(),
            self.frame_events_state(),
        ))
    }

    pub fn frame_info(&self) -> FrameInfo {
        FrameInfo {
            frame_number: self.frame_number,
            time: self.current_time,
            dt: self.dt,
            seconds_remaining: self.processor().get_seconds_remaining().ok(),
        }
    }

    pub fn gameplay_state(&self) -> GameplayState {
        let processor = self.processor();
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

    pub fn ball_frame_state(&self) -> BallFrameState {
        let ball = self
            .processor()
            .get_interpolated_ball_rigid_body(self.current_time, 0.0)
            .ok()
            .filter(|rigid_body| !rigid_body.sleeping)
            .map(|rigid_body| BallSample { rigid_body });
        BallFrameState { ball }
    }

    pub fn player_frame_state(&self) -> PlayerFrameState {
        let processor = self.processor();
        let mut players = Vec::new();
        for player_id in processor.iter_player_ids_in_order() {
            let Ok(is_team_0) = processor.get_player_is_team_0(player_id) else {
                continue;
            };
            players.push(PlayerSample {
                player_id: player_id.clone(),
                is_team_0,
                rigid_body: processor
                    .get_interpolated_player_rigid_body(player_id, self.current_time, 0.0)
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

    pub fn frame_events_state(&self) -> FrameEventsState {
        let processor = self.processor();
        let active_demos = if let Ok(demos) = processor.get_active_demos() {
            demos
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
        let mut events = FrameEventsState {
            active_demos,
            demo_events: Vec::new(),
            boost_pad_events: processor.current_frame_boost_pad_events().to_vec(),
            touch_events: processor.current_frame_touch_events().to_vec(),
            dodge_refreshed_events: processor.current_frame_dodge_refreshed_events().to_vec(),
            player_stat_events: processor.current_frame_player_stat_events().to_vec(),
            goal_events: processor.current_frame_goal_events().to_vec(),
        };
        if let EventWindow::SinceLastSample {
            last_demolish_count,
            last_boost_pad_event_count,
            last_touch_event_count,
            last_player_stat_event_count,
            last_goal_event_count,
        } = self.event_window
        {
            events.active_demos.clear();
            events.demo_events = processor.demolishes[last_demolish_count..].to_vec();
            events.boost_pad_events =
                processor.boost_pad_events[last_boost_pad_event_count..].to_vec();
            events.touch_events = processor.touch_events[last_touch_event_count..].to_vec();
            events.player_stat_events =
                processor.player_stat_events[last_player_stat_event_count..].to_vec();
            events.goal_events = processor.goal_events[last_goal_event_count..].to_vec();
        }
        events
    }
}
