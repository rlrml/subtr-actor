use std::collections::{HashMap, HashSet};

use boxcars::Vector3f;
use subtr_actor::{
    BallFrameState, BallSample, BoostPadEvent, BoostPadEventKind, CarHitbox, DemoEventSample,
    DemolishInfo, DodgeRefreshedEvent, FrameEventsState, FrameInfo, GameplayState, GoalEvent,
    LivePlayState, LivePlayTracker, PlayerFrameState, PlayerId, PlayerSample, PlayerStatEvent,
    PlayerStatEventKind, ShotEventMetadata, TouchEvent, TouchStateCalculator,
    car_hitbox_for_body_id, default_car_hitbox,
};

use crate::model::{
    LiveBoostPadEvent, LiveBoostPadEventKind, LiveDemolishEvent, LiveDodgeRefreshedEvent,
    LiveEventTiming, LiveFrame, LiveGoalEvent, LivePlayerFrame, LivePlayerStatEvent,
    LivePlayerStatEventKind, LiveTouchEvent,
};

/// Derives per-frame graph events from live frames, porting the BakkesMod
/// live-event heuristics (dedupe windows, respawn suppression, touch
/// synthesis, live-play inference) onto the owned model.
#[derive(Clone, Default)]
pub struct LiveEventGenerator {
    touch_state: TouchStateCalculator,
    live_play_tracker: LivePlayTracker,
    pub dodge_refresh_counters: Vec<(PlayerId, i32)>,
    pub active_demos: Vec<LiveActiveDemo>,
    pub known_demolishes: Vec<(DemoEventSample, usize)>,
    pub boost_pad_pickup_sequence_times: HashMap<(String, u8), f32>,
    pub last_goal_event: Option<GoalEvent>,
}

#[derive(Clone, Default)]
pub struct LiveEventHistory {
    pub demo_events: Vec<DemolishInfo>,
    pub boost_pad_events: Vec<BoostPadEvent>,
    pub touch_events: Vec<TouchEvent>,
    pub dodge_refreshed_events: Vec<DodgeRefreshedEvent>,
    pub player_stat_events: Vec<PlayerStatEvent>,
    pub goal_events: Vec<GoalEvent>,
}

impl LiveEventHistory {
    pub fn append_frame_events(&mut self, events: &FrameEventsState) {
        self.demo_events.extend(events.demo_events.iter().cloned());
        self.boost_pad_events
            .extend(events.boost_pad_events.iter().cloned());
        self.touch_events
            .extend(events.touch_events.iter().cloned());
        self.dodge_refreshed_events
            .extend(events.dodge_refreshed_events.iter().cloned());
        self.player_stat_events
            .extend(events.player_stat_events.iter().cloned());
        self.goal_events.extend(events.goal_events.iter().cloned());
    }
}

#[derive(Debug, Clone)]
pub struct LiveActiveDemo {
    pub sample: DemoEventSample,
    expires_at: f32,
}

pub fn zero_vec3() -> Vector3f {
    Vector3f {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    }
}

pub fn player_car_hitbox(player: &LivePlayerFrame) -> CarHitbox {
    player
        .car_body_id
        .and_then(car_hitbox_for_body_id)
        .unwrap_or_else(default_car_hitbox)
}

pub fn player_frame_position(players: &PlayerFrameState, player_id: &PlayerId) -> Option<Vector3f> {
    players
        .player_position(player_id)
        .map(|[x, y, z]| Vector3f { x, y, z })
}

pub fn player_frame_position_array(
    players: &PlayerFrameState,
    player_id: &PlayerId,
) -> Option<[f32; 3]> {
    players.player_position(player_id)
}

pub fn frame_info(frame: &LiveFrame) -> FrameInfo {
    FrameInfo {
        frame_number: frame.frame_number as usize,
        time: frame.time,
        dt: frame.dt,
        seconds_remaining: frame.seconds_remaining,
    }
}

pub fn gameplay_state(frame: &LiveFrame) -> GameplayState {
    let mut counts = [0, 0];
    for player in &frame.players {
        counts[usize::from(!player.is_team_0)] += 1;
    }

    GameplayState {
        game_state: frame.game_state,
        ball_has_been_hit: frame.ball_has_been_hit,
        kickoff_countdown_time: frame.kickoff_countdown_time,
        team_zero_score: frame.team_zero_score,
        team_one_score: frame.team_one_score,
        possession_team_is_team_0: frame.possession_team_is_team_0,
        scored_on_team_is_team_0: frame.scored_on_team_is_team_0,
        current_in_game_team_player_counts: counts,
    }
}

pub fn ball_state(frame: &LiveFrame) -> BallFrameState {
    match frame.ball {
        None => BallFrameState::Missing,
        Some(rigid_body) => BallFrameState::Present(BallSample { rigid_body }),
    }
}

pub fn player_state(players: &[LivePlayerFrame]) -> PlayerFrameState {
    PlayerFrameState {
        players: players
            .iter()
            .map(|player| PlayerSample {
                player_id: player.canonical_player_id(),
                is_team_0: player.is_team_0,
                hitbox: player_car_hitbox(player),
                rigid_body: player.rigid_body,
                boost_amount: Some(player.boost_amount),
                last_boost_amount: Some(player.last_boost_amount),
                boost_active: player.boost_active != 0,
                dodge_active: player.dodge_active != 0,
                dodge_torque: player
                    .dodge_torque
                    .map(|[x, y, z]| glam::Vec3::new(x, y, z)),
                powerslide_active: player.powerslide_active,
                match_goals: player.match_stats.map(|stats| stats.goals),
                match_assists: player.match_stats.map(|stats| stats.assists),
                match_saves: player.match_stats.map(|stats| stats.saves),
                match_shots: player.match_stats.map(|stats| stats.shots),
                match_score: player.match_stats.map(|stats| stats.score),
            })
            .collect(),
    }
}

pub fn explicit_live_play_state(frame: &LiveFrame) -> Option<LivePlayState> {
    let is_live_play = frame.live_play?;
    Some(LivePlayState {
        gameplay_phase: if is_live_play {
            subtr_actor::GameplayPhase::ActivePlay
        } else {
            subtr_actor::GameplayPhase::Unknown
        },
        is_live_play,
    })
}

pub fn event_frame_and_time(frame: &FrameInfo, timing: &LiveEventTiming) -> (usize, f32) {
    match timing.frame_and_time {
        Some((frame_number, time)) => (frame_number as usize, time),
        None => (frame.frame_number, frame.time),
    }
}

pub fn event_seconds_remaining(frame: &FrameInfo, timing: &LiveEventTiming) -> i32 {
    timing
        .seconds_remaining
        .unwrap_or_else(|| frame.seconds_remaining.unwrap_or_default())
}

pub fn explicit_touch_events(
    frame: &FrameInfo,
    players: &PlayerFrameState,
    events: &[LiveTouchEvent],
) -> Vec<TouchEvent> {
    let mut accepted = Vec::new();
    let mut seen = HashSet::new();
    for event in events {
        let (frame_number, time) = event_frame_and_time(frame, &event.timing);
        let player = event.player.clone();
        let team_is_team_0 = event.is_team_0;
        if !seen.insert((frame_number, player.clone(), team_is_team_0)) {
            continue;
        }
        accepted.push(TouchEvent {
            touch_id: None,
            time,
            frame: frame_number,
            team_is_team_0,
            player_position: player
                .as_ref()
                .and_then(|player_id| player_frame_position(players, player_id)),
            player,
            closest_approach_distance: event.closest_approach_distance,
            contact_local_ball_position: None,
            contact_local_hitbox_point: None,
            contact_world_hitbox_point: None,
            dodge_contact: false,
        });
    }
    accepted
}

pub fn explicit_dodge_refresh_keys(
    frame: &FrameInfo,
    events: &[LiveDodgeRefreshedEvent],
) -> HashSet<(PlayerId, usize)> {
    events
        .iter()
        .map(|event| {
            let (frame_number, _) = event_frame_and_time(frame, &event.timing);
            (event.player.clone(), frame_number)
        })
        .collect()
}

pub const MIN_BOOST_PAD_RESPAWN_SECONDS: f32 = 4.0;
pub const GOAL_EVENT_DEDUPE_WINDOW_SECONDS: f32 = 3.0;
pub const MAX_DEMOLISH_KNOWN_FRAMES_PASSED: usize = 150;

pub fn boost_pad_pickup_sequence_is_recent(
    sequence_times: &HashMap<(String, u8), f32>,
    pad_id: &str,
    sequence: u8,
    event_time: f32,
) -> bool {
    sequence_times
        .get(&(pad_id.to_owned(), sequence))
        .is_some_and(|last_time| {
            let elapsed = event_time - *last_time;
            (0.0..MIN_BOOST_PAD_RESPAWN_SECONDS).contains(&elapsed)
        })
}

pub fn demolish_is_known(
    known_demolishes: &[(DemoEventSample, usize)],
    sample: &DemoEventSample,
    frame_number: usize,
) -> bool {
    known_demolishes.iter().any(|(existing, existing_frame)| {
        existing.attacker == sample.attacker
            && existing.victim == sample.victim
            && frame_number.abs_diff(*existing_frame) < MAX_DEMOLISH_KNOWN_FRAMES_PASSED
    })
}

pub fn goal_event_is_duplicate(previous: &GoalEvent, candidate: &GoalEvent) -> bool {
    match (
        candidate.team_zero_score,
        candidate.team_one_score,
        previous.team_zero_score,
        previous.team_one_score,
    ) {
        (Some(team_zero), Some(team_one), Some(prev_team_zero), Some(prev_team_one)) => {
            team_zero == prev_team_zero && team_one == prev_team_one
        }
        _ => {
            previous.scoring_team_is_team_0 == candidate.scoring_team_is_team_0
                && (candidate.time - previous.time).abs() <= GOAL_EVENT_DEDUPE_WINDOW_SECONDS
        }
    }
}

pub fn explicit_player_stat_events(
    frame: &FrameInfo,
    players: &PlayerFrameState,
    events: &[LivePlayerStatEvent],
) -> Vec<PlayerStatEvent> {
    events
        .iter()
        .map(|event| {
            let (frame_number, time) = event_frame_and_time(frame, &event.timing);
            let player = event.player.clone();
            let shot = shot_event_metadata(event);
            PlayerStatEvent {
                time,
                frame: frame_number,
                player_position: shot
                    .as_ref()
                    .and_then(|shot| shot.player_position)
                    .or_else(|| player_frame_position(players, &player)),
                player,
                is_team_0: event.is_team_0,
                kind: match event.kind {
                    LivePlayerStatEventKind::Shot => PlayerStatEventKind::Shot,
                    LivePlayerStatEventKind::Save => PlayerStatEventKind::Save,
                    LivePlayerStatEventKind::Assist => PlayerStatEventKind::Assist,
                },
                shot,
            }
        })
        .collect()
}

pub fn shot_event_metadata(event: &LivePlayerStatEvent) -> Option<ShotEventMetadata> {
    if event.kind != LivePlayerStatEventKind::Shot {
        return None;
    }
    let ball_body = event.shot_ball?;

    Some(ShotEventMetadata::from_rigid_bodies(
        event.is_team_0,
        &ball_body,
        event.shot_player.as_ref(),
    ))
}

pub fn explicit_demolish_events(
    frame: &FrameInfo,
    players: &PlayerFrameState,
    events: &[LiveDemolishEvent],
) -> Vec<DemolishInfo> {
    events
        .iter()
        .map(|event| {
            let (frame_number, time) = event_frame_and_time(frame, &event.timing);
            let attacker = event.attacker.clone();
            DemolishInfo {
                time,
                seconds_remaining: event_seconds_remaining(frame, &event.timing),
                frame: frame_number,
                attacker_location: player_frame_position(players, &attacker),
                attacker,
                victim: event.victim.clone(),
                attacker_velocity: event.attacker_velocity,
                victim_velocity: event.victim_velocity,
                victim_location: event.victim_location,
            }
        })
        .collect()
}

impl LiveEventGenerator {
    fn explicit_dodge_refreshed_events(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        events: &[LiveDodgeRefreshedEvent],
    ) -> Vec<DodgeRefreshedEvent> {
        let mut dodge_refreshed_events = Vec::new();
        for event in events {
            let player = event.player.clone();
            if find_counter(&self.dodge_refresh_counters, &player)
                .is_some_and(|previous| event.counter_value <= previous)
            {
                continue;
            }
            set_counter(
                &mut self.dodge_refresh_counters,
                player.clone(),
                event.counter_value,
            );
            let (frame_number, time) = event_frame_and_time(frame, &event.timing);
            dodge_refreshed_events.push(DodgeRefreshedEvent {
                time,
                frame: frame_number,
                player_position: player_frame_position_array(players, &player),
                player,
                is_team_0: event.is_team_0,
                counter_value: event.counter_value,
            });
        }
        dodge_refreshed_events
    }

    fn explicit_demolish_events(
        &mut self,
        frame: &FrameInfo,
        events: &[LiveDemolishEvent],
    ) -> Vec<LiveDemolishEvent> {
        let mut accepted_events = Vec::new();
        for event in events {
            let (frame_number, _) = event_frame_and_time(frame, &event.timing);
            self.known_demolishes.retain(|(_, known_frame)| {
                frame_number.abs_diff(*known_frame) < MAX_DEMOLISH_KNOWN_FRAMES_PASSED
            });
            let sample = DemoEventSample {
                attacker: event.attacker.clone(),
                victim: event.victim.clone(),
            };
            if demolish_is_known(&self.known_demolishes, &sample, frame_number) {
                continue;
            }
            self.known_demolishes.push((sample, frame_number));
            accepted_events.push(event.clone());
        }
        accepted_events
    }

    fn sync_active_demos(
        &mut self,
        frame: &FrameInfo,
        events: &[LiveDemolishEvent],
    ) -> Vec<DemoEventSample> {
        self.active_demos
            .retain(|demo| demo.expires_at + f32::EPSILON >= frame.time);

        for event in events {
            let sample = DemoEventSample {
                attacker: event.attacker.clone(),
                victim: event.victim.clone(),
            };
            let active_duration_seconds = if event.active_duration_seconds.is_finite()
                && event.active_duration_seconds > 0.0
            {
                event.active_duration_seconds
            } else {
                0.0
            };
            let (_, event_time) = event_frame_and_time(frame, &event.timing);
            let expires_at = event_time + active_duration_seconds;
            if expires_at + f32::EPSILON < frame.time {
                continue;
            }
            if let Some(active_demo) = self.active_demos.iter_mut().find(|active_demo| {
                active_demo.sample.attacker == sample.attacker
                    && active_demo.sample.victim == sample.victim
            }) {
                active_demo.expires_at = expires_at;
            } else {
                self.active_demos
                    .push(LiveActiveDemo { sample, expires_at });
            }
        }

        self.active_demos
            .iter()
            .map(|active_demo| active_demo.sample.clone())
            .collect()
    }

    fn explicit_boost_pad_events(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        events: &[LiveBoostPadEvent],
    ) -> Vec<BoostPadEvent> {
        let mut boost_pad_events = Vec::new();
        for event in events {
            let (frame_number, time) = event_frame_and_time(frame, &event.timing);
            let pad_id = event.pad_id.clone();
            let kind = match event.kind {
                LiveBoostPadEventKind::PickedUp => {
                    if boost_pad_pickup_sequence_is_recent(
                        &self.boost_pad_pickup_sequence_times,
                        &pad_id,
                        event.sequence,
                        time,
                    ) {
                        continue;
                    }
                    self.boost_pad_pickup_sequence_times
                        .insert((pad_id.clone(), event.sequence), time);
                    BoostPadEventKind::PickedUp {
                        sequence: event.sequence,
                    }
                }
                LiveBoostPadEventKind::Available => BoostPadEventKind::Available,
            };
            let player = event.player.clone();
            boost_pad_events.push(BoostPadEvent {
                time,
                frame: frame_number,
                pad_id,
                player_position: player
                    .as_ref()
                    .and_then(|player_id| player_frame_position(players, player_id)),
                player,
                kind,
            });
        }
        boost_pad_events
    }

    fn explicit_goal_events(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        events: &[LiveGoalEvent],
    ) -> Vec<GoalEvent> {
        let mut goal_events = Vec::new();
        for event in events {
            let (frame_number, time) = event_frame_and_time(frame, &event.timing);
            let player = event.player.clone();
            let goal_event = GoalEvent {
                time,
                frame: frame_number,
                scoring_team_is_team_0: event.scoring_team_is_team_0,
                player_position: player
                    .as_ref()
                    .and_then(|player_id| player_frame_position(players, player_id)),
                player,
                team_zero_score: event.team_zero_score,
                team_one_score: event.team_one_score,
            };
            if self
                .last_goal_event
                .as_ref()
                .is_some_and(|previous| goal_event_is_duplicate(previous, &goal_event))
            {
                continue;
            }
            self.last_goal_event = Some(goal_event.clone());
            goal_events.push(goal_event);
        }
        goal_events
    }

    pub fn frame_events(&mut self, frame: &LiveFrame) -> (FrameEventsState, LivePlayState) {
        let frame_info = frame_info(frame);
        let ball = ball_state(frame);
        let players = player_state(&frame.players);
        let gameplay = gameplay_state(frame);
        let explicit_live_play = explicit_live_play_state(frame);
        let explicit_events = &frame.events;

        let explicit_touch_events =
            explicit_touch_events(&frame_info, &players, &explicit_events.touches);
        let has_explicit_touch_events = !explicit_touch_events.is_empty();
        let explicit_dodge_refresh_keys =
            explicit_dodge_refresh_keys(&frame_info, &explicit_events.dodge_refreshes);
        let has_explicit_dodge_refreshed_events = !explicit_dodge_refresh_keys.is_empty();
        let explicit_dodge_refreshed_events = self.explicit_dodge_refreshed_events(
            &frame_info,
            &players,
            &explicit_events.dodge_refreshes,
        );
        let explicit_demolishes =
            self.explicit_demolish_events(&frame_info, &explicit_events.demolishes);
        let demo_events = explicit_demolish_events(&frame_info, &players, &explicit_demolishes);
        let active_demos = self.sync_active_demos(&frame_info, &explicit_demolishes);
        let boost_pad_events = self.explicit_boost_pad_events(
            &frame_info,
            &players,
            &explicit_events.boost_pad_events,
        );
        let player_stat_events =
            explicit_player_stat_events(&frame_info, &players, &explicit_events.player_stat_events);
        let goal_events = self.explicit_goal_events(&frame_info, &players, &explicit_events.goals);
        let base_events = FrameEventsState {
            active_demos,
            demo_events,
            boost_pad_events,
            player_stat_events,
            goal_events,
            ..FrameEventsState::default()
        };
        let live_play = explicit_live_play.unwrap_or_else(|| {
            let mut gameplay = gameplay.clone();
            if has_explicit_touch_events || has_explicit_dodge_refreshed_events {
                if gameplay.ball_has_been_hit == Some(false) {
                    gameplay.ball_has_been_hit = Some(true);
                }
                if gameplay.kickoff_countdown_time.is_some_and(|time| time > 0) {
                    gameplay.kickoff_countdown_time = Some(0);
                    gameplay.game_state = None;
                }
            }
            self.live_play_tracker.state_parts(&gameplay, &base_events)
        });
        let touch_tracker_events = FrameEventsState {
            touch_events: explicit_touch_events,
            dodge_refreshed_events: explicit_dodge_refreshed_events.clone(),
            ..FrameEventsState::default()
        };
        let touch_state = self.touch_state.update(
            &frame_info,
            &ball,
            &players,
            &touch_tracker_events,
            &live_play,
        );
        let mut touch_events = touch_state.touch_events;
        if touch_events.is_empty() && has_explicit_touch_events {
            touch_events = touch_tracker_events.touch_events.clone();
        }
        let mut dodge_refreshed_events = explicit_dodge_refreshed_events;
        if touch_events.is_empty() && has_explicit_dodge_refreshed_events {
            touch_events = dodge_refreshed_events
                .iter()
                .map(|event| TouchEvent {
                    touch_id: None,
                    time: event.time,
                    frame: event.frame,
                    team_is_team_0: event.is_team_0,
                    player: Some(event.player.clone()),
                    player_position: event.player_position.map(|[x, y, z]| Vector3f { x, y, z }),
                    closest_approach_distance: None,
                    contact_local_ball_position: None,
                    contact_local_hitbox_point: None,
                    contact_world_hitbox_point: None,
                    dodge_contact: true,
                })
                .collect();
        }
        dodge_refreshed_events.sort_by_key(|event| event.counter_value);

        (
            FrameEventsState {
                touch_events,
                dodge_refreshed_events,
                ..base_events
            },
            live_play,
        )
    }
}

pub fn find_counter(counters: &[(PlayerId, i32)], player_id: &PlayerId) -> Option<i32> {
    counters
        .iter()
        .find_map(|(id, value)| (id == player_id).then_some(*value))
}

pub fn set_counter(counters: &mut Vec<(PlayerId, i32)>, player_id: PlayerId, value: i32) {
    if let Some((_, counter)) = counters.iter_mut().find(|(id, _)| id == &player_id) {
        *counter = value;
    } else {
        counters.push((player_id, value));
    }
}

pub fn has_duplicate_player_indices(players: &[LivePlayerFrame]) -> bool {
    let mut seen = HashSet::new();
    players
        .iter()
        .any(|player| !seen.insert(player.player_index))
}

#[cfg(test)]
#[path = "generator_tests.rs"]
mod tests;
