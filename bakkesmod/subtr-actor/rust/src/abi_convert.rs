use super::*;

pub(crate) fn vec3(value: SaVec3) -> Vector3f {
    Vector3f {
        x: value.x,
        y: value.y,
        z: value.z,
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

pub(crate) fn player_car_body_id(player: &SaPlayerFrame) -> Option<u32> {
    if player.has_car_body_id == 0 {
        return None;
    }
    u32::try_from(player.car_body_id).ok()
}

fn live_event_timing(timing: SaEventTiming) -> LiveEventTiming {
    LiveEventTiming {
        frame_and_time: (timing.has_timing != 0).then_some((timing.frame_number, timing.time)),
        seconds_remaining: (timing.has_seconds_remaining != 0).then_some(timing.seconds_remaining),
    }
}

pub(crate) fn live_player_frame(player: &SaPlayerFrame) -> LivePlayerFrame {
    LivePlayerFrame {
        player_index: player.player_index,
        name: player_name(player),
        // The BakkesMod ABI does not replicate platform ids, so identity stays
        // RemoteId::SplitScreen(player_index).
        remote_id: None,
        is_team_0: player.is_team_0 != 0,
        rigid_body: (player.has_rigid_body != 0).then(|| rigid_body(player.rigid_body)),
        boost_amount: player.boost_amount,
        last_boost_amount: player.last_boost_amount,
        boost_active: player.boost_active,
        jump_active: player.jump_active,
        double_jump_active: player.double_jump_active,
        dodge_active: player.dodge_active,
        powerslide_active: player.powerslide_active != 0,
        input: None,
        camera: None,
        dodge_impulse: None,
        dodge_torque: None,
        car_body_id: player_car_body_id(player),
        match_stats: (player.has_match_stats != 0).then_some(LiveMatchStats {
            goals: player.match_goals,
            assists: player.match_assists,
            saves: player.match_saves,
            shots: player.match_shots,
            score: player.match_score,
        }),
    }
}

pub(crate) fn live_player_frames(players: &[SaPlayerFrame]) -> Vec<LivePlayerFrame> {
    players.iter().map(live_player_frame).collect()
}

fn live_touch_event(event: &SaTouchEvent) -> LiveTouchEvent {
    LiveTouchEvent {
        timing: live_event_timing(event.timing),
        player: (event.has_player != 0).then(|| player_id(event.player_index)),
        is_team_0: event.is_team_0 != 0,
        closest_approach_distance: (event.has_closest_approach_distance != 0)
            .then_some(event.closest_approach_distance),
    }
}

fn live_dodge_refreshed_event(event: &SaDodgeRefreshedEvent) -> LiveDodgeRefreshedEvent {
    LiveDodgeRefreshedEvent {
        timing: live_event_timing(event.timing),
        player: player_id(event.player_index),
        is_team_0: event.is_team_0 != 0,
        counter_value: event.counter_value,
    }
}

fn live_boost_pad_event(event: &SaBoostPadEvent) -> LiveBoostPadEvent {
    LiveBoostPadEvent {
        timing: live_event_timing(event.timing),
        pad_id: event.pad_id.to_string(),
        kind: match event.kind {
            SaBoostPadEventKind::PickedUp => LiveBoostPadEventKind::PickedUp,
            SaBoostPadEventKind::Available => LiveBoostPadEventKind::Available,
        },
        sequence: event.sequence,
        player: (event.has_player != 0).then(|| player_id(event.player_index)),
    }
}

fn live_goal_event(event: &SaGoalEvent) -> LiveGoalEvent {
    LiveGoalEvent {
        timing: live_event_timing(event.timing),
        scoring_team_is_team_0: event.scoring_team_is_team_0 != 0,
        player: (event.has_player != 0).then(|| player_id(event.player_index)),
        team_zero_score: (event.has_team_zero_score != 0).then_some(event.team_zero_score),
        team_one_score: (event.has_team_one_score != 0).then_some(event.team_one_score),
    }
}

fn live_player_stat_event(event: &SaPlayerStatEvent) -> LivePlayerStatEvent {
    LivePlayerStatEvent {
        timing: live_event_timing(event.timing),
        player: player_id(event.player_index),
        is_team_0: event.is_team_0 != 0,
        kind: match event.kind {
            SaPlayerStatEventKind::Shot => LivePlayerStatEventKind::Shot,
            SaPlayerStatEventKind::Save => LivePlayerStatEventKind::Save,
            SaPlayerStatEventKind::Assist => LivePlayerStatEventKind::Assist,
        },
        shot_ball: (event.has_shot_ball != 0).then(|| rigid_body(event.shot_ball)),
        shot_player: (event.has_shot_player != 0).then(|| rigid_body(event.shot_player)),
    }
}

pub(crate) fn live_demolish_event(event: &SaDemolishEvent) -> LiveDemolishEvent {
    LiveDemolishEvent {
        timing: live_event_timing(event.timing),
        attacker: player_id(event.attacker_index),
        victim: player_id(event.victim_index),
        attacker_velocity: vec3(event.attacker_velocity),
        victim_velocity: vec3(event.victim_velocity),
        victim_location: vec3(event.victim_location),
        active_duration_seconds: event.active_duration_seconds,
    }
}

pub(crate) fn live_explicit_events(events: &SaFrameEventSlices<'_>) -> LiveExplicitEvents {
    LiveExplicitEvents {
        touches: events.touches.iter().map(live_touch_event).collect(),
        dodge_refreshes: events
            .dodge_refreshes
            .iter()
            .map(live_dodge_refreshed_event)
            .collect(),
        boost_pad_events: events
            .boost_pad_events
            .iter()
            .map(live_boost_pad_event)
            .collect(),
        goals: events.goals.iter().map(live_goal_event).collect(),
        player_stat_events: events
            .player_stat_events
            .iter()
            .map(live_player_stat_event)
            .collect(),
        demolishes: events.demolishes.iter().map(live_demolish_event).collect(),
    }
}

pub(crate) fn live_frame_data(frame: &SaLiveFrame) -> LiveFrame {
    LiveFrame {
        frame_number: frame.frame_number,
        time: frame.time,
        dt: frame.dt,
        seconds_remaining: (frame.has_seconds_remaining != 0).then_some(frame.seconds_remaining),
        game_state: (frame.has_game_state != 0).then_some(frame.game_state),
        kickoff_countdown_time: (frame.has_kickoff_countdown_time != 0)
            .then_some(frame.kickoff_countdown_time),
        ball_has_been_hit: (frame.has_ball_has_been_hit != 0)
            .then_some(frame.ball_has_been_hit != 0),
        team_zero_score: (frame.has_team_zero_score != 0).then_some(frame.team_zero_score),
        team_one_score: (frame.has_team_one_score != 0).then_some(frame.team_one_score),
        possession_team_is_team_0: (frame.has_possession_team != 0)
            .then_some(frame.possession_team_is_team_0 != 0),
        scored_on_team_is_team_0: (frame.has_scored_on_team != 0)
            .then_some(frame.scored_on_team_is_team_0 != 0),
        live_play: (frame.has_live_play != 0).then_some(frame.live_play != 0),
        ball: (frame.has_ball != 0).then(|| rigid_body(frame.ball)),
        players: Vec::new(),
        events: LiveExplicitEvents::default(),
    }
}

pub(crate) fn live_frame_from_abi(
    frame: &SaLiveFrame,
    players: &[SaPlayerFrame],
    explicit_events: &SaFrameEventSlices<'_>,
) -> LiveFrame {
    LiveFrame {
        players: live_player_frames(players),
        events: live_explicit_events(explicit_events),
        ..live_frame_data(frame)
    }
}
