#[test]
fn rust_event_abi_layout_matches_plugin_header_expectations() {
    assert_layout!(SaBoostPadEventKind, size = 4, align = 4);
    assert_layout!(SaPlayerStatEventKind, size = 4, align = 4);
    assert_layout!(SaMechanicKind, size = 4, align = 4);
    assert_layout!(SaTeamEventKind, size = 4, align = 4);
    assert_layout!(SaGoalBuildupKind, size = 4, align = 4);

    assert_layout!(SaVec3, size = 12, align = 4);
    assert_offset!(SaVec3, x, 0);
    assert_offset!(SaVec3, y, 4);
    assert_offset!(SaVec3, z, 8);

    assert_layout!(SaQuat, size = 16, align = 4);
    assert_offset!(SaQuat, x, 0);
    assert_offset!(SaQuat, y, 4);
    assert_offset!(SaQuat, z, 8);
    assert_offset!(SaQuat, w, 12);

    assert_layout!(SaRigidBody, size = 56, align = 4);
    assert_offset!(SaRigidBody, location, 0);
    assert_offset!(SaRigidBody, rotation, 12);
    assert_offset!(SaRigidBody, linear_velocity, 28);
    assert_offset!(SaRigidBody, angular_velocity, 40);
    assert_offset!(SaRigidBody, has_linear_velocity, 52);
    assert_offset!(SaRigidBody, has_angular_velocity, 53);
    assert_offset!(SaRigidBody, sleeping, 54);

    assert_layout!(SaPlayerFrame, size = 120, align = 8);
    assert_offset!(SaPlayerFrame, player_index, 0);
    assert_offset!(SaPlayerFrame, player_name, 8);
    assert_offset!(SaPlayerFrame, is_team_0, 16);
    assert_offset!(SaPlayerFrame, has_rigid_body, 17);
    assert_offset!(SaPlayerFrame, rigid_body, 20);
    assert_offset!(SaPlayerFrame, boost_amount, 76);
    assert_offset!(SaPlayerFrame, last_boost_amount, 80);
    assert_offset!(SaPlayerFrame, boost_active, 84);
    assert_offset!(SaPlayerFrame, jump_active, 85);
    assert_offset!(SaPlayerFrame, double_jump_active, 86);
    assert_offset!(SaPlayerFrame, dodge_active, 87);
    assert_offset!(SaPlayerFrame, powerslide_active, 88);
    assert_offset!(SaPlayerFrame, car_body_id, 92);
    assert_offset!(SaPlayerFrame, has_car_body_id, 96);
    assert_offset!(SaPlayerFrame, has_match_stats, 97);
    assert_offset!(SaPlayerFrame, match_goals, 100);
    assert_offset!(SaPlayerFrame, match_assists, 104);
    assert_offset!(SaPlayerFrame, match_saves, 108);
    assert_offset!(SaPlayerFrame, match_shots, 112);
    assert_offset!(SaPlayerFrame, match_score, 116);

    assert_layout!(SaEventTiming, size = 24, align = 8);
    assert_offset!(SaEventTiming, frame_number, 0);
    assert_offset!(SaEventTiming, time, 8);
    assert_offset!(SaEventTiming, seconds_remaining, 12);
    assert_offset!(SaEventTiming, has_timing, 16);
    assert_offset!(SaEventTiming, has_seconds_remaining, 17);

    assert_layout!(SaTouchEvent, size = 40, align = 8);
    assert_offset!(SaTouchEvent, timing, 0);
    assert_offset!(SaTouchEvent, player_index, 24);
    assert_offset!(SaTouchEvent, has_player, 28);
    assert_offset!(SaTouchEvent, is_team_0, 29);
    assert_offset!(SaTouchEvent, closest_approach_distance, 32);
    assert_offset!(SaTouchEvent, has_closest_approach_distance, 36);

    assert_layout!(SaDodgeRefreshedEvent, size = 40, align = 8);
    assert_offset!(SaDodgeRefreshedEvent, timing, 0);
    assert_offset!(SaDodgeRefreshedEvent, player_index, 24);
    assert_offset!(SaDodgeRefreshedEvent, is_team_0, 28);
    assert_offset!(SaDodgeRefreshedEvent, counter_value, 32);

    assert_layout!(SaBoostPadEvent, size = 48, align = 8);
    assert_offset!(SaBoostPadEvent, timing, 0);
    assert_offset!(SaBoostPadEvent, pad_id, 24);
    assert_offset!(SaBoostPadEvent, kind, 28);
    assert_offset!(SaBoostPadEvent, sequence, 32);
    assert_offset!(SaBoostPadEvent, player_index, 36);
    assert_offset!(SaBoostPadEvent, has_player, 40);

    assert_layout!(SaGoalEvent, size = 56, align = 8);
    assert_offset!(SaGoalEvent, timing, 0);
    assert_offset!(SaGoalEvent, scoring_team_is_team_0, 24);
    assert_offset!(SaGoalEvent, player_index, 28);
    assert_offset!(SaGoalEvent, has_player, 32);
    assert_offset!(SaGoalEvent, team_zero_score, 36);
    assert_offset!(SaGoalEvent, has_team_zero_score, 40);
    assert_offset!(SaGoalEvent, team_one_score, 44);
    assert_offset!(SaGoalEvent, has_team_one_score, 48);

    assert_layout!(SaPlayerStatEvent, size = 160, align = 8);
    assert_offset!(SaPlayerStatEvent, timing, 0);
    assert_offset!(SaPlayerStatEvent, player_index, 24);
    assert_offset!(SaPlayerStatEvent, is_team_0, 28);
    assert_offset!(SaPlayerStatEvent, kind, 32);
    assert_offset!(SaPlayerStatEvent, has_shot_ball, 36);
    assert_offset!(SaPlayerStatEvent, shot_ball, 40);
    assert_offset!(SaPlayerStatEvent, has_shot_player, 96);
    assert_offset!(SaPlayerStatEvent, shot_player, 100);

    assert_layout!(SaDemolishEvent, size = 72, align = 8);
    assert_offset!(SaDemolishEvent, timing, 0);
    assert_offset!(SaDemolishEvent, attacker_index, 24);
    assert_offset!(SaDemolishEvent, victim_index, 28);
    assert_offset!(SaDemolishEvent, attacker_velocity, 32);
    assert_offset!(SaDemolishEvent, victim_velocity, 44);
    assert_offset!(SaDemolishEvent, victim_location, 56);
    assert_offset!(SaDemolishEvent, active_duration_seconds, 68);

    assert_layout!(SaLiveFrame, size = 232, align = 8);
    assert_offset!(SaLiveFrame, frame_number, 0);
    assert_offset!(SaLiveFrame, time, 8);
    assert_offset!(SaLiveFrame, dt, 12);
    assert_offset!(SaLiveFrame, seconds_remaining, 16);
    assert_offset!(SaLiveFrame, has_seconds_remaining, 20);
    assert_offset!(SaLiveFrame, game_state, 24);
    assert_offset!(SaLiveFrame, has_game_state, 28);
    assert_offset!(SaLiveFrame, kickoff_countdown_time, 32);
    assert_offset!(SaLiveFrame, has_kickoff_countdown_time, 36);
    assert_offset!(SaLiveFrame, ball_has_been_hit, 37);
    assert_offset!(SaLiveFrame, has_ball_has_been_hit, 38);
    assert_offset!(SaLiveFrame, team_zero_score, 40);
    assert_offset!(SaLiveFrame, has_team_zero_score, 44);
    assert_offset!(SaLiveFrame, team_one_score, 48);
    assert_offset!(SaLiveFrame, has_team_one_score, 52);
    assert_offset!(SaLiveFrame, possession_team_is_team_0, 53);
    assert_offset!(SaLiveFrame, has_possession_team, 54);
    assert_offset!(SaLiveFrame, scored_on_team_is_team_0, 55);
    assert_offset!(SaLiveFrame, has_scored_on_team, 56);
    assert_offset!(SaLiveFrame, live_play, 57);
    assert_offset!(SaLiveFrame, has_live_play, 58);
    assert_offset!(SaLiveFrame, has_ball, 59);
    assert_offset!(SaLiveFrame, ball, 60);
    assert_offset!(SaLiveFrame, players, 120);
    assert_offset!(SaLiveFrame, player_count, 128);
    assert_offset!(SaLiveFrame, touches, 136);
    assert_offset!(SaLiveFrame, touch_count, 144);
    assert_offset!(SaLiveFrame, dodge_refreshes, 152);
    assert_offset!(SaLiveFrame, dodge_refresh_count, 160);
    assert_offset!(SaLiveFrame, boost_pad_events, 168);
    assert_offset!(SaLiveFrame, boost_pad_event_count, 176);
    assert_offset!(SaLiveFrame, goals, 184);
    assert_offset!(SaLiveFrame, goal_count, 192);
    assert_offset!(SaLiveFrame, player_stat_events, 200);
    assert_offset!(SaLiveFrame, player_stat_event_count, 208);
    assert_offset!(SaLiveFrame, demolishes, 216);
    assert_offset!(SaLiveFrame, demolish_count, 224);

    assert_layout!(SaReplayScore, size = 16, align = 4);
    assert_offset!(SaReplayScore, team_zero_score, 0);
    assert_offset!(SaReplayScore, has_team_zero_score, 4);
    assert_offset!(SaReplayScore, team_one_score, 8);
    assert_offset!(SaReplayScore, has_team_one_score, 12);

    assert_layout!(SaMechanicEvent, size = 32, align = 8);
    assert_offset!(SaMechanicEvent, kind, 0);
    assert_offset!(SaMechanicEvent, player_index, 4);
    assert_offset!(SaMechanicEvent, is_team_0, 8);
    assert_offset!(SaMechanicEvent, frame_number, 16);
    assert_offset!(SaMechanicEvent, time, 24);
    assert_offset!(SaMechanicEvent, confidence, 28);

    assert_layout!(SaReplayPlayerInfo, size = 16, align = 8);
    assert_offset!(SaReplayPlayerInfo, player_index, 0);
    assert_offset!(SaReplayPlayerInfo, is_team_0, 4);
    assert_offset!(SaReplayPlayerInfo, name, 8);

    assert_layout!(SaTeamEvent, size = 48, align = 8);
    assert_offset!(SaTeamEvent, kind, 0);
    assert_offset!(SaTeamEvent, is_team_0, 4);
    assert_offset!(SaTeamEvent, start_frame, 8);
    assert_offset!(SaTeamEvent, end_frame, 16);
    assert_offset!(SaTeamEvent, start_time, 24);
    assert_offset!(SaTeamEvent, end_time, 28);
    assert_offset!(SaTeamEvent, attackers, 32);
    assert_offset!(SaTeamEvent, defenders, 36);
    assert_offset!(SaTeamEvent, confidence, 40);

    assert_layout!(SaGoalContextEvent, size = 64, align = 8);
    assert_offset!(SaGoalContextEvent, frame_number, 0);
    assert_offset!(SaGoalContextEvent, time, 8);
    assert_offset!(SaGoalContextEvent, scoring_team_is_team_0, 12);
    assert_offset!(SaGoalContextEvent, has_scorer, 13);
    assert_offset!(SaGoalContextEvent, scorer_index, 16);
    assert_offset!(SaGoalContextEvent, has_scoring_team_most_back_player, 20);
    assert_offset!(SaGoalContextEvent, scoring_team_most_back_player_index, 24);
    assert_offset!(SaGoalContextEvent, has_defending_team_most_back_player, 28);
    assert_offset!(
        SaGoalContextEvent,
        defending_team_most_back_player_index,
        32
    );
    assert_offset!(SaGoalContextEvent, has_ball_position, 36);
    assert_offset!(SaGoalContextEvent, ball_position, 40);
    assert_offset!(SaGoalContextEvent, has_ball_air_time_before_goal, 52);
    assert_offset!(SaGoalContextEvent, ball_air_time_before_goal, 56);
    assert_offset!(SaGoalContextEvent, goal_buildup, 60);
}

fn rigid_body(location: SaVec3, linear_velocity: SaVec3) -> SaRigidBody {
    SaRigidBody {
        location,
        rotation: SaQuat::default(),
        linear_velocity,
        angular_velocity: SaVec3::default(),
        has_linear_velocity: 1,
        has_angular_velocity: 1,
        sleeping: 0,
    }
}

fn live_frame(frame_number: u64, ball: SaRigidBody, players: &[SaPlayerFrame]) -> SaLiveFrame {
    SaLiveFrame {
        frame_number,
        time: frame_number as f32 * 0.1,
        dt: 0.1,
        seconds_remaining: 299,
        has_seconds_remaining: 1,
        game_state: 0,
        has_game_state: 0,
        kickoff_countdown_time: 0,
        has_kickoff_countdown_time: 0,
        ball_has_been_hit: 1,
        has_ball_has_been_hit: 1,
        live_play: 1,
        has_ball: 1,
        ball,
        players: players.as_ptr(),
        player_count: players.len(),
        ..SaLiveFrame::default()
    }
}

fn player_at_index(player_index: u32, is_team_0: bool, location: SaVec3) -> SaPlayerFrame {
    SaPlayerFrame {
        player_index,
        player_name: ptr::null(),
        is_team_0: is_team_0 as u8,
        has_rigid_body: 1,
        rigid_body: rigid_body(location, SaVec3::default()),
        boost_amount: 33.0,
        last_boost_amount: 33.0,
        boost_active: 0,
        jump_active: 0,
        double_jump_active: 0,
        dodge_active: 0,
        powerslide_active: 0,
        car_body_id: 0,
        has_car_body_id: 0,
        has_match_stats: 1,
        match_goals: player_index as i32,
        match_assists: player_index as i32 + 1,
        match_saves: player_index as i32 + 2,
        match_shots: player_index as i32 + 3,
        match_score: player_index as i32 + 100,
    }
}

fn player_at(location: SaVec3) -> SaPlayerFrame {
    player_at_index(0, true, location)
}

fn event_envelope(
    id: &str,
    stream: &str,
    frame: usize,
    time: f32,
    primary_player: Option<RemoteId>,
    team_is_team_0: Option<bool>,
    payload: EventPayload,
) -> Event {
    let scope = payload.scope();
    Event {
        meta: EventMeta {
            id: id.to_owned(),
            stream: stream.to_owned(),
            label: stream.replace('_', " "),
            scope,
            lifecycle: EventLifecycle::Finalized,
            timing: EventTiming::Moment { frame, time },
            primary_player,
            secondary_player: None,
            player_position: None,
            ball_position: None,
            team_is_team_0,
            confidence: None,
            properties: Vec::new(),
        },
        payload,
    }
}

fn normalized_mechanic(id: &str, kind: &str, frame: usize, time: f32) -> Event {
    event_envelope(
        id,
        kind,
        frame,
        time,
        Some(RemoteId::SplitScreen(0)),
        Some(true),
        EventPayload::HalfFlip(HalfFlipEvent {
            time,
            frame,
            player: RemoteId::SplitScreen(0),
            is_team_0: true,
            start_position: [0.0, 0.0, 0.0],
            end_position: [0.0, 0.0, 0.0],
            start_speed: 0.0,
            end_speed: 0.0,
            start_backward_alignment: 0.0,
            best_reorientation_alignment: 0.0,
            best_forward_reversal: 0.0,
            max_forward_vertical: 0.0,
            confidence: 1.0,
        }),
    )
}

fn timeline_event_envelope(event: TimelineEvent) -> Event {
    let frame = event.frame.unwrap_or(0);
    let time = event.time;
    let primary_player = event.player_id.clone();
    let team_is_team_0 = event.is_team_0;
    event_envelope(
        &format!("timeline:{frame}:{time}"),
        "timeline",
        frame,
        time,
        primary_player,
        team_is_team_0,
        EventPayload::Timeline(event),
    )
}

fn goal_context_event_envelope(event: GoalContextEvent) -> Event {
    event_envelope(
        &format!("goal_context:{}:{}", event.frame, event.time),
        "goal_context",
        event.frame,
        event.time,
        event.scorer.clone(),
        Some(event.scoring_team_is_team_0),
        EventPayload::GoalContext(event),
    )
}

fn payload_event_envelope(
    stream: &str,
    frame: usize,
    time: f32,
    player: RemoteId,
    is_team_0: bool,
    payload: EventPayload,
) -> Event {
    event_envelope(
        &format!("{stream}:{frame}:{time}"),
        stream,
        frame,
        time,
        Some(player),
        Some(is_team_0),
        payload,
    )
}

fn team_event_envelope(stream: &str, frame: usize, time: f32, payload: EventPayload) -> Event {
    event_envelope(
        &format!("{stream}:{frame}:{time}"),
        stream,
        frame,
        time,
        None,
        None,
        payload,
    )
}

fn backboard_event_envelope(event: BackboardBounceEvent) -> Event {
    payload_event_envelope(
        "backboard",
        event.frame,
        event.time,
        event.player.clone(),
        event.is_team_0,
        EventPayload::Backboard(event),
    )
}

fn whiff_event_envelope(event: WhiffEvent) -> Event {
    payload_event_envelope(
        "whiff",
        event.frame,
        event.time,
        event.player.clone(),
        event.is_team_0,
        EventPayload::Whiff(event),
    )
}

fn boost_pickup_event_envelope(event: BoostPickupEvent) -> Event {
    payload_event_envelope(
        "boost_pickups",
        event.frame,
        event.time,
        event.player_id.clone(),
        event.is_team_0,
        EventPayload::BoostPickup(event),
    )
}

fn bump_event_envelope(event: BumpEvent) -> Event {
    payload_event_envelope(
        "bump",
        event.frame,
        event.time,
        event.initiator.clone(),
        event.initiator_is_team_0,
        EventPayload::Bump(event),
    )
}

fn demolition_event_envelope(event: DemolitionEvent) -> Event {
    payload_event_envelope(
        "demolition",
        event.frame,
        event.time,
        event.attacker.clone(),
        event.attacker_is_team_0.unwrap_or(false),
        EventPayload::Demolition(event),
    )
}

fn fifty_fifty_event_envelope(event: FiftyFiftyEvent) -> Event {
    team_event_envelope(
        "fifty_fifty",
        event.resolve_frame,
        event.resolve_time,
        EventPayload::FiftyFifty(event),
    )
}

fn rush_event_envelope(event: RushEvent) -> Event {
    team_event_envelope("rush", event.end_frame, event.end_time, EventPayload::Rush(event))
}

fn whiff_event(frame: usize, time: f32, player_index: u32) -> WhiffEvent {
    WhiffEvent {
        kind: WhiffEventKind::Whiff,
        start_time: time,
        start_frame: frame,
        time,
        frame,
        resolved_time: time,
        resolved_frame: frame,
        resolution_reason: WhiffResolutionReason::SeparatedFromBall,
        player: RemoteId::SplitScreen(player_index),
        is_team_0: player_index == 0,
        closest_approach_distance: 42.0,
        forward_alignment: 0.7,
        approach_speed: 900.0,
        closing_speed_at_closest: Some(850.0),
        velocity_alignment_at_closest: Some(0.9),
        local_ball_position_at_closest: Some([100.0, 0.0, 75.0]),
        resolved_distance: Some(400.0),
        dodge_active: false,
        aerial: false,
        player_position: None,
    }
}

fn bump_event(frame: usize, time: f32, confidence: f32) -> BumpEvent {
    BumpEvent {
        time,
        frame,
        initiator: RemoteId::SplitScreen(0),
        victim: RemoteId::SplitScreen(1),
        initiator_is_team_0: true,
        victim_is_team_0: false,
        is_team_bump: false,
        strength: 800.0,
        confidence,
        contact_distance: 120.0,
        closing_speed: 500.0,
        victim_impulse: 220.0,
        initiator_position: [0.0, 0.0, 0.0],
        victim_position: [100.0, 0.0, 0.0],
    }
}

fn demolition_event(frame: usize, time: f32) -> DemolitionEvent {
    DemolitionEvent {
        time,
        frame,
        attacker: RemoteId::SplitScreen(0),
        victim: RemoteId::SplitScreen(1),
        attacker_is_team_0: Some(true),
        victim_is_team_0: Some(false),
        attacker_position: None,
        victim_position: None,
    }
}

fn backboard_event(frame: usize, time: f32) -> BackboardBounceEvent {
    BackboardBounceEvent {
        time,
        frame,
        player: RemoteId::SplitScreen(0),
        player_position: None,
        is_team_0: true,
    }
}

fn boost_pickup_event(frame: usize, time: f32) -> BoostPickupEvent {
    BoostPickupEvent {
        frame,
        time,
        player_id: RemoteId::SplitScreen(0),
        player_position: None,
        is_team_0: true,
        pad_type: BoostPickupPadType::Big,
        field_half: BoostPickupFieldHalf::Opponent,
        activity: BoostPickupActivity::Active,
        detection: BoostPickupDetection::Both,
        pad_zone: Some(BoostPickupPadZone::Offensive),
        is_steal: true,
        collected_amount: 80.0,
        overfill_amount: 0.0,
        boost_before: Some(20.0),
        boost_after: Some(100.0),
    }
}

fn fifty_fifty_event(
    start_frame: usize,
    resolve_frame: usize,
    resolve_time: f32,
) -> FiftyFiftyEvent {
    FiftyFiftyEvent {
        start_time: 1.0,
        start_frame,
        resolve_time,
        resolve_frame,
        is_kickoff: false,
        team_zero_player: Some(RemoteId::SplitScreen(0)),
        team_one_player: Some(RemoteId::SplitScreen(1)),
        team_zero_touch_time: None,
        team_zero_touch_frame: None,
        team_zero_dodge_contact: false,
        team_one_touch_time: None,
        team_one_touch_frame: None,
        team_one_dodge_contact: false,
        team_zero_position: [0.0, 0.0, 0.0],
        team_one_position: [100.0, 0.0, 0.0],
        midpoint: [50.0, 0.0, 0.0],
        plane_normal: [1.0, 0.0, 0.0],
        winning_team_is_team_0: Some(false),
        possession_team_is_team_0: Some(false),
    }
}

fn goal_tag(kind: GoalTagKind) -> GoalTag {
    GoalTag::from_parts(
        kind,
        GoalTagMetadata {
            confidence: 0.72,
            performer: None,
            modifiers: Vec::new(),
            related_events: Vec::new(),
            details: Vec::new(),
            evidence: Vec::new(),
        },
    )
}

fn rush_event(start_frame: usize, end_frame: usize, end_time: f32, is_team_0: bool) -> RushEvent {
    RushEvent {
        start_time: 1.0,
        start_frame,
        end_time,
        end_frame,
        is_team_0,
        attackers: 3,
        defenders: 2,
    }
}

fn goal_context_event(frame: usize, time: f32) -> GoalContextEvent {
    GoalContextEvent {
        time,
        frame,
        scoring_team_is_team_0: false,
        scorer: Some(RemoteId::SplitScreen(1)),
        scoring_team_most_back_player: Some(RemoteId::SplitScreen(1)),
        defending_team_most_back_player: Some(RemoteId::SplitScreen(0)),
        ball_position: Some(subtr_actor::GoalContextPosition {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        }),
        ball_air_time_before_goal: Some(1.25),
        ball_speed_at_goal: None,
        pressure_duration_before_goal: None,
        time_after_kickoff: None,
        goal_buildup: GoalBuildupKind::CounterAttack,
        scorer_last_touch: None,
        players: Vec::new(),
        tags: Vec::new(),
    }
}

fn live_events_json_value(engine: *const SaEngine) -> serde_json::Value {
    let json_len = unsafe { subtr_actor_bakkesmod_events_json_len(engine) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written =
        unsafe { subtr_actor_bakkesmod_write_events_json(engine, bytes.as_mut_ptr(), bytes.len()) };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("events json should be valid")
}

fn timeline_payload_matches(
    event: &serde_json::Value,
    kind: &str,
    frame: usize,
    is_team_0: Option<bool>,
    require_player: bool,
) -> bool {
    let payload = &event["payload"];
    let timeline = &payload["payload"];
    payload["kind"] == serde_json::json!("timeline")
        && timeline["kind"] == serde_json::json!(kind)
        && timeline["frame"] == serde_json::json!(frame)
        && is_team_0.is_none_or(|team| timeline["is_team_0"] == serde_json::json!(team))
        && (!require_player || !timeline["player_id"].is_null())
}

fn live_timeline_json_value(engine: *const SaEngine) -> serde_json::Value {
    let json_len = unsafe { subtr_actor_bakkesmod_timeline_json_len(engine) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written = unsafe {
        subtr_actor_bakkesmod_write_timeline_json(engine, bytes.as_mut_ptr(), bytes.len())
    };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("timeline json should be valid")
}

fn live_frame_json_value(engine: *const SaEngine) -> serde_json::Value {
    let json_len = unsafe { subtr_actor_bakkesmod_frame_json_len(engine) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written =
        unsafe { subtr_actor_bakkesmod_write_frame_json(engine, bytes.as_mut_ptr(), bytes.len()) };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("frame json should be valid")
}

fn live_stats_json_value(engine: *const SaEngine) -> serde_json::Value {
    let json_len = unsafe { subtr_actor_bakkesmod_stats_json_len(engine) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written =
        unsafe { subtr_actor_bakkesmod_write_stats_json(engine, bytes.as_mut_ptr(), bytes.len()) };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("stats json should be valid")
}

fn live_graph_info_json_value(engine: *const SaEngine) -> serde_json::Value {
    let json_len = unsafe { subtr_actor_bakkesmod_graph_info_json_len(engine) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written = unsafe {
        subtr_actor_bakkesmod_write_graph_info_json(engine, bytes.as_mut_ptr(), bytes.len())
    };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("graph info json should be valid")
}

fn live_stats_module_json_value(engine: *const SaEngine, module_name: &str) -> serde_json::Value {
    let module_name =
        std::ffi::CString::new(module_name).expect("module name should not contain nul bytes");
    let json_len =
        unsafe { subtr_actor_bakkesmod_stats_module_json_len(engine, module_name.as_ptr()) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written = unsafe {
        subtr_actor_bakkesmod_write_stats_module_json(
            engine,
            module_name.as_ptr(),
            bytes.as_mut_ptr(),
            bytes.len(),
        )
    };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("stats module json should be valid")
}

fn live_stats_module_frame_json_value(
    engine: *const SaEngine,
    module_name: &str,
) -> serde_json::Value {
    let module_name =
        std::ffi::CString::new(module_name).expect("module name should not contain nul bytes");
    let json_len =
        unsafe { subtr_actor_bakkesmod_stats_module_frame_json_len(engine, module_name.as_ptr()) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written = unsafe {
        subtr_actor_bakkesmod_write_stats_module_frame_json(
            engine,
            module_name.as_ptr(),
            bytes.as_mut_ptr(),
            bytes.len(),
        )
    };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("stats module frame json should be valid")
}

fn live_stats_module_config_json_value(
    engine: *const SaEngine,
    module_name: &str,
) -> serde_json::Value {
    let module_name =
        std::ffi::CString::new(module_name).expect("module name should not contain nul bytes");
    let json_len =
        unsafe { subtr_actor_bakkesmod_stats_module_config_json_len(engine, module_name.as_ptr()) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written = unsafe {
        subtr_actor_bakkesmod_write_stats_module_config_json(
            engine,
            module_name.as_ptr(),
            bytes.as_mut_ptr(),
            bytes.len(),
        )
    };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("stats module config json should be valid")
}

fn live_graph_output_json_value(engine: *const SaEngine, output_name: &str) -> serde_json::Value {
    let output_name =
        std::ffi::CString::new(output_name).expect("output name should not contain nul bytes");
    let json_len =
        unsafe { subtr_actor_bakkesmod_graph_output_json_len(engine, output_name.as_ptr()) };
    assert!(json_len > 0, "graph output {output_name:?} should have JSON");
    let mut bytes = vec![0; json_len];
    let written = unsafe {
        subtr_actor_bakkesmod_write_graph_output_json(
            engine,
            output_name.as_ptr(),
            bytes.as_mut_ptr(),
            bytes.len(),
        )
    };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("graph output json should be valid")
}

fn live_analysis_node_json_value(engine: *const SaEngine, node_name: &str) -> serde_json::Value {
    let node_name =
        std::ffi::CString::new(node_name).expect("node name should not contain nul bytes");
    let json_len =
        unsafe { subtr_actor_bakkesmod_analysis_node_json_len(engine, node_name.as_ptr()) };
    assert!(json_len > 0, "analysis node {node_name:?} should have JSON");
    let mut bytes = vec![0; json_len];
    let written = unsafe {
        subtr_actor_bakkesmod_write_analysis_node_json(
            engine,
            node_name.as_ptr(),
            bytes.as_mut_ptr(),
            bytes.len(),
        )
    };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("analysis node json should be valid")
}

fn live_analysis_node_names_json_value(engine: *const SaEngine) -> serde_json::Value {
    let json_len = unsafe { subtr_actor_bakkesmod_analysis_node_names_json_len(engine) };
    assert!(json_len > 0);
    let mut bytes = vec![0; json_len];
    let written = unsafe {
        subtr_actor_bakkesmod_write_analysis_node_names_json(
            engine,
            bytes.as_mut_ptr(),
            bytes.len(),
        )
    };
    assert_eq!(written, json_len);
    serde_json::from_slice(&bytes).expect("analysis node names json should be valid")
}
