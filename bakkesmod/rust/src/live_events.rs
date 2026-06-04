use super::*;

#[derive(Clone, Default)]
pub(crate) struct SaLiveEventGenerator {
    touch_state: TouchStateCalculator,
    live_play_tracker: subtr_actor::LivePlayTracker,
    pub(crate) dodge_refresh_counters: Vec<(RemoteId, i32)>,
    pub(crate) active_demos: Vec<SaActiveDemo>,
    pub(crate) known_demolishes: Vec<(DemoEventSample, usize)>,
    pub(crate) boost_pad_pickup_sequence_times: HashMap<(String, u8), f32>,
    pub(crate) last_goal_event: Option<GoalEvent>,
}

#[derive(Clone, Default)]
pub(crate) struct SaLiveEventHistory {
    pub(crate) demo_events: Vec<DemolishInfo>,
    pub(crate) boost_pad_events: Vec<BoostPadEvent>,
    pub(crate) touch_events: Vec<TouchEvent>,
    pub(crate) dodge_refreshed_events: Vec<DodgeRefreshedEvent>,
    pub(crate) player_stat_events: Vec<PlayerStatEvent>,
    pub(crate) goal_events: Vec<GoalEvent>,
}

impl SaLiveEventHistory {
    pub(crate) fn append_frame_events(&mut self, events: &FrameEventsState) {
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
pub(crate) struct SaActiveDemo {
    pub(crate) sample: DemoEventSample,
    expires_at: f32,
}

pub(crate) fn vec3(value: SaVec3) -> Vector3f {
    Vector3f {
        x: value.x,
        y: value.y,
        z: value.z,
    }
}

pub(crate) fn zero_vec3() -> Vector3f {
    Vector3f {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    }
}

pub(crate) fn quat(value: SaQuat) -> Quaternion {
    Quaternion {
        x: value.x,
        y: value.y,
        z: value.z,
        w: value.w,
    }
}

pub(crate) fn rigid_body(value: SaRigidBody) -> RigidBody {
    RigidBody {
        location: vec3(value.location),
        rotation: quat(value.rotation),
        sleeping: value.sleeping != 0,
        linear_velocity: (value.has_linear_velocity != 0).then_some(vec3(value.linear_velocity)),
        angular_velocity: (value.has_angular_velocity != 0).then_some(vec3(value.angular_velocity)),
    }
}

pub(crate) fn player_id(index: u32) -> RemoteId {
    RemoteId::SplitScreen(index)
}

pub(crate) fn player_index(id: &RemoteId) -> u32 {
    match id {
        RemoteId::SplitScreen(index) => *index,
        _ => 0,
    }
}

pub(crate) fn player_car_body_id(player: &SaPlayerFrame) -> Option<u32> {
    if player.has_car_body_id == 0 {
        return None;
    }
    u32::try_from(player.car_body_id).ok()
}

pub(crate) fn player_car_hitbox(player: &SaPlayerFrame) -> CarHitbox {
    player_car_body_id(player)
        .and_then(car_hitbox_for_body_id)
        .unwrap_or_else(default_car_hitbox)
}

pub(crate) fn player_frame_position(
    players: &PlayerFrameState,
    player_id: &PlayerId,
) -> Option<Vector3f> {
    players
        .player_position(player_id)
        .map(|[x, y, z]| Vector3f { x, y, z })
}

pub(crate) fn player_frame_position_array(
    players: &PlayerFrameState,
    player_id: &PlayerId,
) -> Option<[f32; 3]> {
    players.player_position(player_id)
}

pub(crate) fn live_car_actor_id(id: &PlayerId) -> SubtrActorResult<boxcars::ActorId> {
    let Some(index) = SaLiveProcessorView::player_index(id) else {
        return SubtrActorError::new_result(SubtrActorErrorVariant::PropertyNotFoundInState {
            property: "live player id",
        });
    };
    let Ok(index) = i32::try_from(index) else {
        return SubtrActorError::new_result(SubtrActorErrorVariant::PropertyNotFoundInState {
            property: "live player id",
        });
    };
    Ok(boxcars::ActorId(index))
}

pub(crate) fn live_demolish_attribute(
    attacker: &PlayerId,
    victim: &PlayerId,
    demolish: Option<&DemolishInfo>,
) -> SubtrActorResult<DemolishAttribute> {
    Ok(DemolishAttribute::Fx(boxcars::DemolishFx {
        custom_demo_flag: false,
        custom_demo_id: 0,
        attacker_flag: true,
        attacker: live_car_actor_id(attacker)?,
        victim_flag: true,
        victim: live_car_actor_id(victim)?,
        attack_velocity: demolish
            .map(|demolish| demolish.attacker_velocity)
            .unwrap_or_else(zero_vec3),
        victim_velocity: demolish
            .map(|demolish| demolish.victim_velocity)
            .unwrap_or_else(zero_vec3),
    }))
}

pub(crate) fn frame_info(frame: &SaLiveFrame) -> FrameInfo {
    FrameInfo {
        frame_number: frame.frame_number as usize,
        time: frame.time,
        dt: frame.dt,
        seconds_remaining: (frame.has_seconds_remaining != 0).then_some(frame.seconds_remaining),
    }
}

pub(crate) fn gameplay_state(frame: &SaLiveFrame, players: &[SaPlayerFrame]) -> GameplayState {
    let mut counts = [0, 0];
    for player in players {
        counts[usize::from(player.is_team_0 == 0)] += 1;
    }

    GameplayState {
        game_state: (frame.has_game_state != 0).then_some(frame.game_state),
        ball_has_been_hit: (frame.has_ball_has_been_hit != 0)
            .then_some(frame.ball_has_been_hit != 0),
        kickoff_countdown_time: (frame.has_kickoff_countdown_time != 0)
            .then_some(frame.kickoff_countdown_time),
        team_zero_score: (frame.has_team_zero_score != 0).then_some(frame.team_zero_score),
        team_one_score: (frame.has_team_one_score != 0).then_some(frame.team_one_score),
        possession_team_is_team_0: (frame.has_possession_team != 0)
            .then_some(frame.possession_team_is_team_0 != 0),
        scored_on_team_is_team_0: (frame.has_scored_on_team != 0)
            .then_some(frame.scored_on_team_is_team_0 != 0),
        current_in_game_team_player_counts: counts,
    }
}

pub(crate) fn ball_state(frame: &SaLiveFrame) -> BallFrameState {
    if frame.has_ball == 0 {
        BallFrameState::Missing
    } else {
        BallFrameState::Present(BallSample {
            rigid_body: rigid_body(frame.ball),
        })
    }
}

pub(crate) fn player_state(players: &[SaPlayerFrame]) -> PlayerFrameState {
    PlayerFrameState {
        players: players
            .iter()
            .map(|player| PlayerSample {
                player_id: player_id(player.player_index),
                is_team_0: player.is_team_0 != 0,
                hitbox: player_car_hitbox(player),
                rigid_body: (player.has_rigid_body != 0).then_some(rigid_body(player.rigid_body)),
                boost_amount: Some(player.boost_amount),
                last_boost_amount: Some(player.last_boost_amount),
                boost_active: player.boost_active != 0,
                dodge_active: player.dodge_active != 0,
                powerslide_active: player.powerslide_active != 0,
                match_goals: (player.has_match_stats != 0).then_some(player.match_goals),
                match_assists: (player.has_match_stats != 0).then_some(player.match_assists),
                match_saves: (player.has_match_stats != 0).then_some(player.match_saves),
                match_shots: (player.has_match_stats != 0).then_some(player.match_shots),
                match_score: (player.has_match_stats != 0).then_some(player.match_score),
            })
            .collect(),
    }
}

pub(crate) fn explicit_live_play_state(frame: &SaLiveFrame) -> Option<LivePlayState> {
    if frame.has_live_play == 0 {
        return None;
    }

    let is_live_play = frame.live_play != 0;
    Some(LivePlayState {
        gameplay_phase: if is_live_play {
            GameplayPhase::ActivePlay
        } else {
            GameplayPhase::Unknown
        },
        is_live_play,
    })
}

pub(crate) fn event_frame_and_time(frame: &FrameInfo, timing: SaEventTiming) -> (usize, f32) {
    if timing.has_timing != 0 {
        (timing.frame_number as usize, timing.time)
    } else {
        (frame.frame_number, frame.time)
    }
}

pub(crate) fn event_seconds_remaining(frame: &FrameInfo, timing: SaEventTiming) -> i32 {
    if timing.has_seconds_remaining != 0 {
        timing.seconds_remaining
    } else {
        frame.seconds_remaining.unwrap_or_default()
    }
}

pub(crate) fn explicit_touch_events(
    frame: &FrameInfo,
    players: &PlayerFrameState,
    events: &[SaTouchEvent],
) -> Vec<TouchEvent> {
    let mut accepted = Vec::new();
    let mut seen = HashSet::new();
    for event in events {
        let (frame_number, time) = event_frame_and_time(frame, event.timing);
        let player = (event.has_player != 0).then_some(player_id(event.player_index));
        let team_is_team_0 = event.is_team_0 != 0;
        if !seen.insert((frame_number, player.clone(), team_is_team_0)) {
            continue;
        }
        accepted.push(TouchEvent {
            time,
            frame: frame_number,
            team_is_team_0,
            player_position: player
                .as_ref()
                .and_then(|player_id| player_frame_position(players, player_id)),
            player,
            closest_approach_distance: (event.has_closest_approach_distance != 0)
                .then_some(event.closest_approach_distance),
            dodge_contact: false,
        });
    }
    accepted
}

pub(crate) fn explicit_dodge_refresh_keys(
    frame: &FrameInfo,
    events: &[SaDodgeRefreshedEvent],
) -> HashSet<(RemoteId, usize)> {
    events
        .iter()
        .map(|event| {
            let (frame_number, _) = event_frame_and_time(frame, event.timing);
            (player_id(event.player_index), frame_number)
        })
        .collect()
}

pub(crate) const MIN_BOOST_PAD_RESPAWN_SECONDS: f32 = 4.0;
pub(crate) const GOAL_EVENT_DEDUPE_WINDOW_SECONDS: f32 = 3.0;
pub(crate) const MAX_DEMOLISH_KNOWN_FRAMES_PASSED: usize = 150;

pub(crate) fn boost_pad_pickup_sequence_is_recent(
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

pub(crate) fn demolish_is_known(
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

pub(crate) fn goal_event_is_duplicate(previous: &GoalEvent, candidate: &GoalEvent) -> bool {
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

pub(crate) fn explicit_player_stat_events(
    frame: &FrameInfo,
    players: &PlayerFrameState,
    events: &[SaPlayerStatEvent],
) -> Vec<PlayerStatEvent> {
    events
        .iter()
        .map(|event| {
            let (frame_number, time) = event_frame_and_time(frame, event.timing);
            let player = player_id(event.player_index);
            let shot = shot_event_metadata(event);
            PlayerStatEvent {
                time,
                frame: frame_number,
                player_position: shot
                    .as_ref()
                    .and_then(|shot| shot.player_position)
                    .or_else(|| player_frame_position(players, &player)),
                player,
                is_team_0: event.is_team_0 != 0,
                kind: match event.kind {
                    SaPlayerStatEventKind::Shot => PlayerStatEventKind::Shot,
                    SaPlayerStatEventKind::Save => PlayerStatEventKind::Save,
                    SaPlayerStatEventKind::Assist => PlayerStatEventKind::Assist,
                },
                shot,
            }
        })
        .collect()
}

pub(crate) fn shot_event_metadata(event: &SaPlayerStatEvent) -> Option<ShotEventMetadata> {
    if event.kind != SaPlayerStatEventKind::Shot || event.has_shot_ball == 0 {
        return None;
    }

    let ball_body = rigid_body(event.shot_ball);
    let player_body = (event.has_shot_player != 0).then(|| rigid_body(event.shot_player));
    Some(ShotEventMetadata::from_rigid_bodies(
        event.is_team_0 != 0,
        &ball_body,
        player_body.as_ref(),
    ))
}

pub(crate) fn explicit_demolish_events(
    frame: &FrameInfo,
    players: &PlayerFrameState,
    events: &[SaDemolishEvent],
) -> Vec<DemolishInfo> {
    events
        .iter()
        .map(|event| {
            let (frame_number, time) = event_frame_and_time(frame, event.timing);
            let attacker = player_id(event.attacker_index);
            let victim = player_id(event.victim_index);
            DemolishInfo {
                time,
                seconds_remaining: event_seconds_remaining(frame, event.timing),
                frame: frame_number,
                attacker_location: player_frame_position(players, &attacker),
                attacker,
                victim,
                attacker_velocity: vec3(event.attacker_velocity),
                victim_velocity: vec3(event.victim_velocity),
                victim_location: vec3(event.victim_location),
            }
        })
        .collect()
}

impl SaLiveEventGenerator {
    fn explicit_dodge_refreshed_events(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        events: &[SaDodgeRefreshedEvent],
    ) -> Vec<DodgeRefreshedEvent> {
        let mut dodge_refreshed_events = Vec::new();
        for event in events {
            let player = player_id(event.player_index);
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
            let (frame_number, time) = event_frame_and_time(frame, event.timing);
            dodge_refreshed_events.push(DodgeRefreshedEvent {
                time,
                frame: frame_number,
                player_position: player_frame_position_array(players, &player),
                player,
                is_team_0: event.is_team_0 != 0,
                counter_value: event.counter_value,
            });
        }
        dodge_refreshed_events
    }

    fn explicit_demolish_events(
        &mut self,
        frame: &FrameInfo,
        events: &[SaDemolishEvent],
    ) -> Vec<SaDemolishEvent> {
        let mut accepted_events = Vec::new();
        for event in events {
            let (frame_number, _) = event_frame_and_time(frame, event.timing);
            self.known_demolishes.retain(|(_, known_frame)| {
                frame_number.abs_diff(*known_frame) < MAX_DEMOLISH_KNOWN_FRAMES_PASSED
            });
            let sample = DemoEventSample {
                attacker: player_id(event.attacker_index),
                victim: player_id(event.victim_index),
            };
            if demolish_is_known(&self.known_demolishes, &sample, frame_number) {
                continue;
            }
            self.known_demolishes.push((sample, frame_number));
            accepted_events.push(*event);
        }
        accepted_events
    }

    fn sync_active_demos(
        &mut self,
        frame: &FrameInfo,
        events: &[SaDemolishEvent],
    ) -> Vec<DemoEventSample> {
        self.active_demos
            .retain(|demo| demo.expires_at + f32::EPSILON >= frame.time);

        for event in events {
            let sample = DemoEventSample {
                attacker: player_id(event.attacker_index),
                victim: player_id(event.victim_index),
            };
            let active_duration_seconds = if event.active_duration_seconds.is_finite()
                && event.active_duration_seconds > 0.0
            {
                event.active_duration_seconds
            } else {
                0.0
            };
            let (_, event_time) = event_frame_and_time(frame, event.timing);
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
                self.active_demos.push(SaActiveDemo { sample, expires_at });
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
        events: &[SaBoostPadEvent],
    ) -> Vec<BoostPadEvent> {
        let mut boost_pad_events = Vec::new();
        for event in events {
            let (frame_number, time) = event_frame_and_time(frame, event.timing);
            let pad_id = event.pad_id.to_string();
            let kind = match event.kind {
                SaBoostPadEventKind::PickedUp => {
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
                SaBoostPadEventKind::Available => BoostPadEventKind::Available,
            };
            let player = (event.has_player != 0).then_some(player_id(event.player_index));
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
        events: &[SaGoalEvent],
    ) -> Vec<GoalEvent> {
        let mut goal_events = Vec::new();
        for event in events {
            let (frame_number, time) = event_frame_and_time(frame, event.timing);
            let player = (event.has_player != 0).then_some(player_id(event.player_index));
            let goal_event = GoalEvent {
                time,
                frame: frame_number,
                scoring_team_is_team_0: event.scoring_team_is_team_0 != 0,
                player_position: player
                    .as_ref()
                    .and_then(|player_id| player_frame_position(players, player_id)),
                player,
                team_zero_score: (event.has_team_zero_score != 0).then_some(event.team_zero_score),
                team_one_score: (event.has_team_one_score != 0).then_some(event.team_one_score),
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

    fn frame_events(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        gameplay: &GameplayState,
        explicit_live_play: Option<LivePlayState>,
        explicit_events: &SaFrameEventSlices<'_>,
    ) -> (FrameEventsState, LivePlayState) {
        let explicit_touch_events = explicit_touch_events(frame, players, explicit_events.touches);
        let has_explicit_touch_events = !explicit_touch_events.is_empty();
        let explicit_dodge_refresh_keys =
            explicit_dodge_refresh_keys(frame, explicit_events.dodge_refreshes);
        let has_explicit_dodge_refreshed_events = !explicit_dodge_refresh_keys.is_empty();
        let explicit_dodge_refreshed_events =
            self.explicit_dodge_refreshed_events(frame, players, explicit_events.dodge_refreshes);
        let explicit_demolishes = self.explicit_demolish_events(frame, explicit_events.demolishes);
        let demo_events = explicit_demolish_events(frame, players, &explicit_demolishes);
        let active_demos = self.sync_active_demos(frame, &explicit_demolishes);
        let boost_pad_events =
            self.explicit_boost_pad_events(frame, players, explicit_events.boost_pad_events);
        let player_stat_events =
            explicit_player_stat_events(frame, players, explicit_events.player_stat_events);
        let goal_events = self.explicit_goal_events(frame, players, explicit_events.goals);
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
        let touch_state =
            self.touch_state
                .update(frame, ball, players, &touch_tracker_events, &live_play);
        let mut touch_events = touch_state.touch_events;
        if touch_events.is_empty() && has_explicit_touch_events {
            touch_events = touch_tracker_events.touch_events.clone();
        }
        let mut dodge_refreshed_events = explicit_dodge_refreshed_events;
        if touch_events.is_empty() && has_explicit_dodge_refreshed_events {
            touch_events = dodge_refreshed_events
                .iter()
                .map(|event| TouchEvent {
                    time: event.time,
                    frame: event.frame,
                    team_is_team_0: event.is_team_0,
                    player: Some(event.player.clone()),
                    player_position: event.player_position.map(|[x, y, z]| Vector3f { x, y, z }),
                    closest_approach_distance: None,
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

pub(crate) fn frame_input_from_live_state(
    live_events: &mut SaLiveEventGenerator,
    live_event_history: &mut SaLiveEventHistory,
    replay_meta: Option<&ReplayMeta>,
    frame: &SaLiveFrame,
    sampled_players: &[SaPlayerFrame],
    explicit_events: &SaFrameEventSlices<'_>,
) -> FrameInput {
    let frame_info = frame_info(frame);
    let ball = ball_state(frame);
    let players = player_state(sampled_players);
    let gameplay = gameplay_state(frame, sampled_players);
    let explicit_live_play = explicit_live_play_state(frame);
    let (frame_events, live_play) = live_events.frame_events(
        &frame_info,
        &ball,
        &players,
        &gameplay,
        explicit_live_play,
        explicit_events,
    );
    live_event_history.append_frame_events(&frame_events);
    let processor = SaLiveProcessorView::new(
        replay_meta,
        frame,
        sampled_players,
        frame_events,
        live_event_history,
    );
    FrameInput::timeline_with_live_play_state(
        &processor,
        frame.frame_number as usize,
        frame.time,
        frame.dt,
        live_play,
    )
}

#[cfg(test)]
pub(crate) fn frame_input(
    engine: &mut SaEngine,
    frame: &SaLiveFrame,
    sampled_players: &[SaPlayerFrame],
    explicit_events: &SaFrameEventSlices<'_>,
) -> FrameInput {
    frame_input_from_live_state(
        &mut engine.live_events,
        &mut engine.live_event_history,
        engine.live_replay_meta.as_ref(),
        frame,
        sampled_players,
        explicit_events,
    )
}

pub(crate) fn player_name(player: &SaPlayerFrame) -> Option<String> {
    if player.player_name.is_null() {
        return None;
    }
    let name = unsafe { CStr::from_ptr(player.player_name) }
        .to_string_lossy()
        .trim()
        .to_owned();
    (!name.is_empty()).then_some(name)
}

pub(crate) fn default_live_player_name(player_id: &RemoteId) -> String {
    match player_id {
        RemoteId::SplitScreen(index) => format!("Player {index}"),
        _ => format!("{player_id:?}"),
    }
}

pub(crate) fn live_replay_meta_signature(
    players: &[SaPlayerFrame],
) -> Vec<(RemoteId, bool, Option<String>)> {
    players
        .iter()
        .map(|player| {
            (
                player_id(player.player_index),
                player.is_team_0 != 0,
                player_name(player),
            )
        })
        .collect()
}

pub(crate) fn live_replay_meta(players: &[SaPlayerFrame]) -> ReplayMeta {
    let mut team_zero = Vec::new();
    let mut team_one = Vec::new();
    for player in players {
        let player_id = player_id(player.player_index);
        let car_body_id = player_car_body_id(player);
        let info = PlayerInfo {
            remote_id: player_id.clone(),
            stats: None,
            name: player_name(player).unwrap_or_else(|| default_live_player_name(&player_id)),
            car_body_id,
            car_hitbox_family: car_body_id
                .and_then(hitbox_family_for_body_id)
                .map(|family| format!("{family:?}"))
                .or_else(|| Some("Octane".to_owned())),
        };
        if player.is_team_0 != 0 {
            team_zero.push(info);
        } else {
            team_one.push(info);
        }
    }
    ReplayMeta {
        team_zero,
        team_one,
        all_headers: Vec::new(),
    }
}

pub(crate) fn sync_live_replay_meta(
    engine: &mut SaEngine,
    players: &[SaPlayerFrame],
) -> subtr_actor::SubtrActorResult<()> {
    let signature = live_replay_meta_signature(players);
    if engine.live_replay_meta_initialized && engine.live_replay_meta_signature == signature {
        return Ok(());
    }

    let replay_meta = live_replay_meta(players);
    engine.graph.on_replay_meta(&replay_meta)?;
    engine.live_replay_meta_initialized = true;
    engine.live_replay_meta = Some(replay_meta);
    engine.live_replay_meta_signature = signature;
    Ok(())
}

pub(crate) fn has_duplicate_player_indices(players: &[SaPlayerFrame]) -> bool {
    let mut seen = HashSet::new();
    players
        .iter()
        .any(|player| !seen.insert(player.player_index))
}
