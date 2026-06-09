use super::*;

pub(crate) fn mechanic_kind(kind: &str) -> Option<SaMechanicKind> {
    match kind {
        "air_dribble" => Some(SaMechanicKind::AirDribble),
        "ball_carry" => Some(SaMechanicKind::BallCarry),
        "ceiling_shot" => Some(SaMechanicKind::CeilingShot),
        "center" => Some(SaMechanicKind::Center),
        "double_tap" => Some(SaMechanicKind::DoubleTap),
        "dodge_reset" => Some(SaMechanicKind::FlipReset),
        "flick" => Some(SaMechanicKind::Flick),
        "flip_reset" => Some(SaMechanicKind::FlipReset),
        "half_flip" => Some(SaMechanicKind::HalfFlip),
        "half_volley" => Some(SaMechanicKind::HalfVolley),
        "musty_flick" => Some(SaMechanicKind::MustyFlick),
        "one_timer" => Some(SaMechanicKind::OneTimer),
        "pass" => Some(SaMechanicKind::Pass),
        "speed_flip" => Some(SaMechanicKind::SpeedFlip),
        "wall_aerial" => Some(SaMechanicKind::WallAerial),
        "wall_aerial_shot" => Some(SaMechanicKind::WallAerialShot),
        "wavedash" => Some(SaMechanicKind::Wavedash),
        _ => None,
    }
}

pub(crate) fn mechanic_start(event: &Event) -> (usize, f32) {
    match event.meta.timing {
        EventTiming::Moment { frame, time } => (frame, time),
        EventTiming::Span {
            start_frame,
            start_time,
            ..
        } => (start_frame, start_time),
    }
}

pub(crate) struct PendingGraphEvent {
    id: String,
    kind: SaMechanicKind,
    player_id: RemoteId,
    is_team_0: bool,
    frame_number: usize,
    time: f32,
    confidence: f32,
}

pub(crate) fn push_pending_graph_event(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    event: PendingGraphEvent,
) {
    if !emitted_mechanic_ids.insert(event.id) {
        return;
    }
    pending_events.push(SaMechanicEvent {
        kind: event.kind,
        player_index: player_index(&event.player_id),
        is_team_0: event.is_team_0 as u8,
        frame_number: event.frame_number as u64,
        time: event.time,
        confidence: event.confidence,
    });
}

pub(crate) fn push_pending_team_event(
    pending_team_events: &mut Vec<SaTeamEvent>,
    emitted_team_event_ids: &mut HashSet<String>,
    id: String,
    event: SaTeamEvent,
) {
    if !emitted_team_event_ids.insert(id) {
        return;
    }
    pending_team_events.push(event);
}

pub(crate) fn push_pending_goal_context_event(
    pending_goal_context_events: &mut Vec<SaGoalContextEvent>,
    emitted_goal_context_ids: &mut HashSet<String>,
    id: String,
    event: SaGoalContextEvent,
) {
    if !emitted_goal_context_ids.insert(id) {
        return;
    }
    pending_goal_context_events.push(event);
}

pub(crate) fn push_mechanic_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    mechanics: impl IntoIterator<Item = impl std::borrow::Borrow<Event>>,
) {
    for event in mechanics {
        let event = event.borrow();
        let Some(kind) = mechanic_kind(&event.meta.stream) else {
            continue;
        };
        let (Some(player_id), Some(is_team_0)) = (
            event.meta.primary_player.as_ref(),
            event.meta.team_is_team_0,
        ) else {
            continue;
        };
        let (frame_number, time) = mechanic_start(event);
        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id: event.meta.id.clone(),
                kind,
                player_id: player_id.clone(),
                is_team_0,
                frame_number,
                time,
                confidence: event.meta.confidence.unwrap_or(1.0),
            },
        );
    }
}

pub(crate) fn push_whiff_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    whiffs: impl IntoIterator<Item = impl std::borrow::Borrow<WhiffEvent>>,
) {
    for (index, event) in whiffs.into_iter().enumerate() {
        let event = event.borrow();
        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id: format!(
                    "whiff:{}:{}:{index}",
                    event.frame,
                    player_index(&event.player)
                ),
                kind: SaMechanicKind::Whiff,
                player_id: event.player.clone(),
                is_team_0: event.is_team_0,
                frame_number: event.frame,
                time: event.time,
                confidence: 1.0,
            },
        );
    }
}

pub(crate) fn push_bump_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    bumps: impl IntoIterator<Item = impl std::borrow::Borrow<BumpEvent>>,
) {
    for (index, event) in bumps.into_iter().enumerate() {
        let event = event.borrow();
        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id: format!(
                    "bump:{}:{}:{}:{index}",
                    event.frame,
                    player_index(&event.initiator),
                    player_index(&event.victim)
                ),
                kind: SaMechanicKind::Bump,
                player_id: event.initiator.clone(),
                is_team_0: event.initiator_is_team_0,
                frame_number: event.frame,
                time: event.time,
                confidence: event.confidence,
            },
        );
    }
}

pub(crate) fn push_backboard_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    backboard: impl IntoIterator<Item = impl std::borrow::Borrow<BackboardBounceEvent>>,
) {
    for (index, event) in backboard.into_iter().enumerate() {
        let event = event.borrow();
        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id: format!(
                    "backboard:{}:{}:{index}",
                    event.frame,
                    player_index(&event.player)
                ),
                kind: SaMechanicKind::Backboard,
                player_id: event.player.clone(),
                is_team_0: event.is_team_0,
                frame_number: event.frame,
                time: event.time,
                confidence: 1.0,
            },
        );
    }
}

pub(crate) fn push_boost_pickup_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    boost_pickups: impl IntoIterator<Item = impl std::borrow::Borrow<BoostPickupComparisonEvent>>,
) {
    for (index, event) in boost_pickups.into_iter().enumerate() {
        let event = event.borrow();
        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id: format!(
                    "boost_pickup:{}:{}:{:?}:{:?}:{index}",
                    event.frame,
                    player_index(&event.player_id),
                    event.reported_frame,
                    event.inferred_frame
                ),
                kind: SaMechanicKind::BoostPickup,
                player_id: event.player_id.clone(),
                is_team_0: event.is_team_0,
                frame_number: event.frame,
                time: event.time,
                confidence: 1.0,
            },
        );
    }
}

pub(crate) fn timeline_event_kind(kind: TimelineEventKind) -> SaMechanicKind {
    match kind {
        TimelineEventKind::Goal => SaMechanicKind::Goal,
        TimelineEventKind::Shot => SaMechanicKind::Shot,
        TimelineEventKind::Save => SaMechanicKind::Save,
        TimelineEventKind::Assist => SaMechanicKind::Assist,
        TimelineEventKind::Kill => SaMechanicKind::Demo,
        TimelineEventKind::Death => SaMechanicKind::Death,
    }
}

pub(crate) fn push_timeline_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    timeline: impl IntoIterator<Item = impl std::borrow::Borrow<TimelineEvent>>,
) {
    let mut occurrence_by_key = HashMap::new();
    for event in timeline {
        let event = event.borrow();
        let (Some(player_id), Some(is_team_0)) = (&event.player_id, event.is_team_0) else {
            continue;
        };
        let frame_number = event.frame.unwrap_or(0);
        let event_key = format!(
            "{:?}:{}:{}:{}:{}",
            event.kind,
            event.time.to_bits(),
            frame_number,
            player_index(player_id),
            is_team_0 as u8
        );
        let occurrence = occurrence_by_key.entry(event_key.clone()).or_insert(0);
        let id = format!("timeline:{event_key}:{occurrence}");
        *occurrence += 1;
        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id,
                kind: timeline_event_kind(event.kind),
                player_id: player_id.clone(),
                is_team_0,
                frame_number,
                time: event.time,
                confidence: 1.0,
            },
        );
    }
}

pub(crate) fn push_repeated_core_player_stat_events(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    event: &CorePlayerScoreboardEvent,
    kind: SaMechanicKind,
    count: i32,
) {
    for index in 0..count.max(0) {
        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id: format!(
                    "core_player:{:?}:{}:{}:{}",
                    kind,
                    event.frame,
                    player_index(&event.player),
                    index
                ),
                kind,
                player_id: event.player.clone(),
                is_team_0: event.is_team_0,
                frame_number: event.frame,
                time: event.time,
                confidence: 1.0,
            },
        );
    }
}

pub(crate) fn push_core_player_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    core_player: impl IntoIterator<Item = impl std::borrow::Borrow<CorePlayerScoreboardEvent>>,
) {
    for event in core_player {
        let event = event.borrow();
        push_repeated_core_player_stat_events(
            pending_events,
            emitted_mechanic_ids,
            event,
            SaMechanicKind::Shot,
            event.shots_delta,
        );
        push_repeated_core_player_stat_events(
            pending_events,
            emitted_mechanic_ids,
            event,
            SaMechanicKind::Save,
            event.saves_delta,
        );
        push_repeated_core_player_stat_events(
            pending_events,
            emitted_mechanic_ids,
            event,
            SaMechanicKind::Assist,
            event.assists_delta,
        );
    }
}

pub(crate) fn push_fifty_fifty_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    fifty_fifty: impl IntoIterator<Item = impl std::borrow::Borrow<FiftyFiftyEvent>>,
) {
    for (index, event) in fifty_fifty.into_iter().enumerate() {
        let event = event.borrow();
        let Some(winning_team_is_team_0) = event.winning_team_is_team_0 else {
            continue;
        };
        let Some(player_id) = (if winning_team_is_team_0 {
            event.team_zero_player.as_ref()
        } else {
            event.team_one_player.as_ref()
        }) else {
            continue;
        };

        push_pending_graph_event(
            pending_events,
            emitted_mechanic_ids,
            PendingGraphEvent {
                id: format!(
                    "fifty_fifty:{}:{}:{}:{index}",
                    event.start_frame,
                    event.resolve_frame,
                    player_index(player_id)
                ),
                kind: SaMechanicKind::FiftyFifty,
                player_id: player_id.clone(),
                is_team_0: winning_team_is_team_0,
                frame_number: event.resolve_frame,
                time: event.resolve_time,
                confidence: 1.0,
            },
        );
    }
}

pub(crate) fn goal_tag_kind(kind: GoalTagKind) -> SaMechanicKind {
    match kind {
        GoalTagKind::AerialGoal => SaMechanicKind::AerialGoal,
        GoalTagKind::HighAerialGoal => SaMechanicKind::HighAerialGoal,
        GoalTagKind::LongDistanceGoal => SaMechanicKind::LongDistanceGoal,
        GoalTagKind::OwnHalfGoal => SaMechanicKind::OwnHalfGoal,
        GoalTagKind::EmptyNetGoal => SaMechanicKind::EmptyNetGoal,
        GoalTagKind::CounterAttackGoal => SaMechanicKind::CounterAttackGoal,
        GoalTagKind::SustainedPressureGoal => SaMechanicKind::SustainedPressureGoal,
        GoalTagKind::KickoffGoal => SaMechanicKind::KickoffGoal,
        GoalTagKind::FlickGoal => SaMechanicKind::FlickGoal,
        GoalTagKind::CeilingShotGoal => SaMechanicKind::CeilingShotGoal,
        GoalTagKind::DoubleTapGoal => SaMechanicKind::DoubleTapGoal,
        GoalTagKind::OneTimerGoal => SaMechanicKind::OneTimerGoal,
        GoalTagKind::PassingGoal => SaMechanicKind::PassingGoal,
        GoalTagKind::AirDribbleGoal => SaMechanicKind::AirDribbleGoal,
        GoalTagKind::FlipResetGoal => SaMechanicKind::FlipResetGoal,
        GoalTagKind::HalfVolleyGoal => SaMechanicKind::HalfVolleyGoal,
        GoalTagKind::BumpGoal => SaMechanicKind::BumpGoal,
        GoalTagKind::DemoGoal => SaMechanicKind::DemoGoal,
    }
}

pub(crate) fn push_goal_tag_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    goal_context: impl IntoIterator<Item = impl std::borrow::Borrow<GoalContextEvent>>,
) {
    for (goal_index, event) in goal_context.into_iter().enumerate() {
        let event = event.borrow();
        let Some(scorer) = event.scorer.as_ref() else {
            continue;
        };
        for tag in &event.tags {
            push_pending_graph_event(
                pending_events,
                emitted_mechanic_ids,
                PendingGraphEvent {
                    id: format!(
                        "goal_tag:{}:{}:{:?}:{}",
                        goal_index,
                        event.frame,
                        tag.kind(),
                        player_index(scorer)
                    ),
                    kind: goal_tag_kind(tag.kind()),
                    player_id: scorer.clone(),
                    is_team_0: event.scoring_team_is_team_0,
                    frame_number: event.frame,
                    time: event.time,
                    confidence: tag.metadata().confidence,
                },
            );
        }
    }
}

pub(crate) fn push_rush_events_from_timeline(
    pending_team_events: &mut Vec<SaTeamEvent>,
    emitted_team_event_ids: &mut HashSet<String>,
    rush: impl IntoIterator<Item = impl std::borrow::Borrow<RushEvent>>,
) {
    for event in rush {
        let event = event.borrow();
        push_pending_team_event(
            pending_team_events,
            emitted_team_event_ids,
            format!(
                "rush:{}:{}:{}",
                event.start_frame, event.end_frame, event.is_team_0
            ),
            SaTeamEvent {
                kind: SaTeamEventKind::Rush,
                is_team_0: event.is_team_0 as u8,
                start_frame: event.start_frame as u64,
                end_frame: event.end_frame as u64,
                start_time: event.start_time,
                end_time: event.end_time,
                attackers: event.attackers as u32,
                defenders: event.defenders as u32,
                confidence: 1.0,
            },
        );
    }
}

pub(crate) fn goal_buildup_kind(kind: GoalBuildupKind) -> SaGoalBuildupKind {
    match kind {
        GoalBuildupKind::CounterAttack => SaGoalBuildupKind::CounterAttack,
        GoalBuildupKind::SustainedPressure => SaGoalBuildupKind::SustainedPressure,
        GoalBuildupKind::Other => SaGoalBuildupKind::Other,
    }
}

pub(crate) fn goal_context_position(position: Option<subtr_actor::GoalContextPosition>) -> SaVec3 {
    position
        .map(|position| SaVec3 {
            x: position.x,
            y: position.y,
            z: position.z,
        })
        .unwrap_or_default()
}

pub(crate) fn push_goal_context_events_from_timeline(
    pending_goal_context_events: &mut Vec<SaGoalContextEvent>,
    emitted_goal_context_ids: &mut HashSet<String>,
    goal_context: impl IntoIterator<Item = impl std::borrow::Borrow<GoalContextEvent>>,
) {
    for (index, event) in goal_context.into_iter().enumerate() {
        let event = event.borrow();
        let scorer = event.scorer.as_ref();
        let scoring_team_most_back_player = event.scoring_team_most_back_player.as_ref();
        let defending_team_most_back_player = event.defending_team_most_back_player.as_ref();
        push_pending_goal_context_event(
            pending_goal_context_events,
            emitted_goal_context_ids,
            format!("goal_context:{}:{}:{index}", event.frame, event.time),
            SaGoalContextEvent {
                frame_number: event.frame as u64,
                time: event.time,
                scoring_team_is_team_0: event.scoring_team_is_team_0 as u8,
                has_scorer: scorer.is_some() as u8,
                scorer_index: scorer.map(player_index).unwrap_or(0),
                has_scoring_team_most_back_player: scoring_team_most_back_player.is_some() as u8,
                scoring_team_most_back_player_index: scoring_team_most_back_player
                    .map(player_index)
                    .unwrap_or(0),
                has_defending_team_most_back_player: defending_team_most_back_player.is_some()
                    as u8,
                defending_team_most_back_player_index: defending_team_most_back_player
                    .map(player_index)
                    .unwrap_or(0),
                has_ball_position: event.ball_position.is_some() as u8,
                ball_position: goal_context_position(event.ball_position),
                has_ball_air_time_before_goal: event.ball_air_time_before_goal.is_some() as u8,
                ball_air_time_before_goal: event.ball_air_time_before_goal.unwrap_or(0.0),
                goal_buildup: goal_buildup_kind(event.goal_buildup),
            },
        );
    }
}

pub(crate) fn replay_player_index_map(replay_meta: &ReplayMeta) -> HashMap<RemoteId, u32> {
    replay_meta
        .player_order()
        .enumerate()
        .map(|(index, player)| (player.remote_id.clone(), index as u32))
        .collect()
}

pub(crate) fn replay_annotation_players(
    replay_meta: &ReplayMeta,
) -> (Vec<CString>, Vec<SaReplayPlayerInfo>) {
    let mut names = Vec::new();
    let mut players = Vec::new();
    for (player_index, player) in replay_meta.player_order().enumerate() {
        names.push(CString::new(player.name.as_str()).unwrap_or_else(|_| {
            CString::new(player.name.replace('\0', "")).expect("nul bytes removed")
        }));
        players.push(SaReplayPlayerInfo {
            player_index: player_index as u32,
            is_team_0: (player_index < replay_meta.team_zero.len()) as u8,
            name: names.last().expect("player name was just pushed").as_ptr(),
        });
    }
    (names, players)
}

pub(crate) fn replay_player_index(index_map: &HashMap<RemoteId, u32>, id: &RemoteId) -> u32 {
    index_map
        .get(id)
        .copied()
        .unwrap_or_else(|| player_index(id))
}

pub(crate) fn push_replay_annotation(
    events: &mut Vec<SaMechanicEvent>,
    emitted_ids: &mut HashSet<String>,
    index_map: &HashMap<RemoteId, u32>,
    event: PendingGraphEvent,
) {
    if !emitted_ids.insert(event.id) {
        return;
    }
    events.push(SaMechanicEvent {
        kind: event.kind,
        player_index: replay_player_index(index_map, &event.player_id),
        is_team_0: event.is_team_0 as u8,
        frame_number: event.frame_number as u64,
        time: event.time,
        confidence: event.confidence,
    });
}

pub(crate) fn replay_annotations_from_timeline(
    replay_meta: &ReplayMeta,
    timeline: &ReplayStatsTimelineEvents,
) -> Vec<SaMechanicEvent> {
    let index_map = replay_player_index_map(replay_meta);
    let mut events = Vec::new();
    let mut emitted_ids = HashSet::new();

    let mut occurrence_by_key = HashMap::new();
    for (index, envelope) in timeline.events.iter().enumerate() {
        if let Some(kind) = mechanic_kind(&envelope.meta.stream) {
            if let (Some(player_id), Some(is_team_0)) = (
                envelope.meta.primary_player.as_ref(),
                envelope.meta.team_is_team_0,
            ) {
                let (frame_number, time) = mechanic_start(envelope);
                push_replay_annotation(
                    &mut events,
                    &mut emitted_ids,
                    &index_map,
                    PendingGraphEvent {
                        id: envelope.meta.id.clone(),
                        kind,
                        player_id: player_id.clone(),
                        is_team_0,
                        frame_number,
                        time,
                        confidence: envelope.meta.confidence.unwrap_or(1.0),
                    },
                );
            }
        }

        match &envelope.payload {
            EventPayload::Backboard(event) => {
                push_replay_annotation(
                    &mut events,
                    &mut emitted_ids,
                    &index_map,
                    PendingGraphEvent {
                        id: format!(
                            "replay_backboard:{}:{}:{index}",
                            event.frame,
                            replay_player_index(&index_map, &event.player)
                        ),
                        kind: SaMechanicKind::Backboard,
                        player_id: event.player.clone(),
                        is_team_0: event.is_team_0,
                        frame_number: event.frame,
                        time: event.time,
                        confidence: 1.0,
                    },
                );
            }
            EventPayload::Whiff(event) => {
                push_replay_annotation(
                    &mut events,
                    &mut emitted_ids,
                    &index_map,
                    PendingGraphEvent {
                        id: format!(
                            "replay_whiff:{}:{}:{index}",
                            event.frame,
                            replay_player_index(&index_map, &event.player)
                        ),
                        kind: SaMechanicKind::Whiff,
                        player_id: event.player.clone(),
                        is_team_0: event.is_team_0,
                        frame_number: event.frame,
                        time: event.time,
                        confidence: 1.0,
                    },
                );
            }
            EventPayload::BoostPickup(event) => {
                push_replay_annotation(
                    &mut events,
                    &mut emitted_ids,
                    &index_map,
                    PendingGraphEvent {
                        id: format!(
                            "replay_boost_pickup:{}:{}:{:?}:{:?}:{index}",
                            event.frame,
                            replay_player_index(&index_map, &event.player_id),
                            event.reported_frame,
                            event.inferred_frame
                        ),
                        kind: SaMechanicKind::BoostPickup,
                        player_id: event.player_id.clone(),
                        is_team_0: event.is_team_0,
                        frame_number: event.frame,
                        time: event.time,
                        confidence: 1.0,
                    },
                );
            }
            EventPayload::Bump(event) => {
                push_replay_annotation(
                    &mut events,
                    &mut emitted_ids,
                    &index_map,
                    PendingGraphEvent {
                        id: format!(
                            "replay_bump:{}:{}:{}:{index}",
                            event.frame,
                            replay_player_index(&index_map, &event.initiator),
                            replay_player_index(&index_map, &event.victim)
                        ),
                        kind: SaMechanicKind::Bump,
                        player_id: event.initiator.clone(),
                        is_team_0: event.initiator_is_team_0,
                        frame_number: event.frame,
                        time: event.time,
                        confidence: event.confidence,
                    },
                );
            }
            EventPayload::Timeline(event) => {
                let (Some(player_id), Some(is_team_0)) = (&event.player_id, event.is_team_0) else {
                    continue;
                };
                let frame_number = event.frame.unwrap_or(0);
                let event_key = format!(
                    "replay_timeline:{:?}:{}:{}:{}:{}",
                    event.kind,
                    event.time.to_bits(),
                    frame_number,
                    replay_player_index(&index_map, player_id),
                    is_team_0 as u8
                );
                let occurrence = occurrence_by_key.entry(event_key.clone()).or_insert(0);
                let id = format!("{event_key}:{occurrence}");
                *occurrence += 1;
                push_replay_annotation(
                    &mut events,
                    &mut emitted_ids,
                    &index_map,
                    PendingGraphEvent {
                        id,
                        kind: timeline_event_kind(event.kind),
                        player_id: player_id.clone(),
                        is_team_0,
                        frame_number,
                        time: event.time,
                        confidence: 1.0,
                    },
                );
            }
            EventPayload::CorePlayer(event) => {
                for (kind, count) in [
                    (SaMechanicKind::Shot, event.shots_delta),
                    (SaMechanicKind::Save, event.saves_delta),
                    (SaMechanicKind::Assist, event.assists_delta),
                ] {
                    for repeated_index in 0..count.max(0) {
                        push_replay_annotation(
                            &mut events,
                            &mut emitted_ids,
                            &index_map,
                            PendingGraphEvent {
                                id: format!(
                                    "replay_core_player:{:?}:{}:{}:{}",
                                    kind,
                                    event.frame,
                                    replay_player_index(&index_map, &event.player),
                                    repeated_index
                                ),
                                kind,
                                player_id: event.player.clone(),
                                is_team_0: event.is_team_0,
                                frame_number: event.frame,
                                time: event.time,
                                confidence: 1.0,
                            },
                        );
                    }
                }
            }
            EventPayload::FiftyFifty(event) => {
                let Some(winning_team_is_team_0) = event.winning_team_is_team_0 else {
                    continue;
                };
                let Some(player_id) = (if winning_team_is_team_0 {
                    event.team_zero_player.as_ref()
                } else {
                    event.team_one_player.as_ref()
                }) else {
                    continue;
                };
                push_replay_annotation(
                    &mut events,
                    &mut emitted_ids,
                    &index_map,
                    PendingGraphEvent {
                        id: format!(
                            "replay_fifty_fifty:{}:{}:{}:{index}",
                            event.start_frame,
                            event.resolve_frame,
                            replay_player_index(&index_map, player_id)
                        ),
                        kind: SaMechanicKind::FiftyFifty,
                        player_id: player_id.clone(),
                        is_team_0: winning_team_is_team_0,
                        frame_number: event.resolve_frame,
                        time: event.resolve_time,
                        confidence: 1.0,
                    },
                );
            }
            EventPayload::GoalContext(event) => {
                let Some(scorer) = event.scorer.as_ref() else {
                    continue;
                };
                for tag in &event.tags {
                    push_replay_annotation(
                        &mut events,
                        &mut emitted_ids,
                        &index_map,
                        PendingGraphEvent {
                            id: format!(
                                "replay_goal_tag:{}:{}:{:?}:{}",
                                index,
                                event.frame,
                                tag.kind(),
                                replay_player_index(&index_map, scorer)
                            ),
                            kind: goal_tag_kind(tag.kind()),
                            player_id: scorer.clone(),
                            is_team_0: event.scoring_team_is_team_0,
                            frame_number: event.frame,
                            time: event.time,
                            confidence: tag.metadata().confidence,
                        },
                    );
                }
            }
            _ => {}
        }
    }

    events.sort_by(|left, right| {
        left.time
            .total_cmp(&right.time)
            .then_with(|| left.frame_number.cmp(&right.frame_number))
            .then_with(|| (left.kind as u32).cmp(&(right.kind as u32)))
            .then_with(|| left.player_index.cmp(&right.player_index))
    });
    events
}

pub(crate) fn push_drainable_events_from_timeline(
    pending_events: &mut Vec<SaMechanicEvent>,
    emitted_mechanic_ids: &mut HashSet<String>,
    pending_team_events: &mut Vec<SaTeamEvent>,
    emitted_team_event_ids: &mut HashSet<String>,
    pending_goal_context_events: &mut Vec<SaGoalContextEvent>,
    emitted_goal_context_ids: &mut HashSet<String>,
    events: &ReplayStatsTimelineEvents,
) {
    push_mechanic_events_from_timeline(
        pending_events,
        emitted_mechanic_ids,
        events
            .events
            .iter()
            .filter(|event| mechanic_kind(&event.meta.stream).is_some()),
    );
    push_backboard_events_from_timeline(
        pending_events,
        emitted_mechanic_ids,
        events
            .events
            .iter()
            .filter_map(|event| match &event.payload {
                EventPayload::Backboard(payload) => Some(payload),
                _ => None,
            }),
    );
    push_whiff_events_from_timeline(
        pending_events,
        emitted_mechanic_ids,
        events
            .events
            .iter()
            .filter_map(|event| match &event.payload {
                EventPayload::Whiff(payload) => Some(payload),
                _ => None,
            }),
    );
    push_boost_pickup_events_from_timeline(
        pending_events,
        emitted_mechanic_ids,
        events
            .events
            .iter()
            .filter_map(|event| match &event.payload {
                EventPayload::BoostPickup(payload) => Some(payload),
                _ => None,
            }),
    );
    push_bump_events_from_timeline(
        pending_events,
        emitted_mechanic_ids,
        events
            .events
            .iter()
            .filter_map(|event| match &event.payload {
                EventPayload::Bump(payload) => Some(payload),
                _ => None,
            }),
    );
    push_core_player_events_from_timeline(
        pending_events,
        emitted_mechanic_ids,
        events
            .events
            .iter()
            .filter_map(|event| match &event.payload {
                EventPayload::CorePlayer(payload) => Some(payload),
                _ => None,
            }),
    );
    push_timeline_events_from_timeline(
        pending_events,
        emitted_mechanic_ids,
        events
            .events
            .iter()
            .filter_map(|event| match &event.payload {
                EventPayload::Timeline(payload) => Some(payload),
                _ => None,
            }),
    );
    push_fifty_fifty_events_from_timeline(
        pending_events,
        emitted_mechanic_ids,
        events
            .events
            .iter()
            .filter_map(|event| match &event.payload {
                EventPayload::FiftyFifty(payload) => Some(payload),
                _ => None,
            }),
    );
    push_goal_tag_events_from_timeline(
        pending_events,
        emitted_mechanic_ids,
        events
            .events
            .iter()
            .filter_map(|event| match &event.payload {
                EventPayload::GoalContext(payload) => Some(payload),
                _ => None,
            }),
    );
    push_rush_events_from_timeline(
        pending_team_events,
        emitted_team_event_ids,
        events
            .events
            .iter()
            .filter_map(|event| match &event.payload {
                EventPayload::Rush(payload) => Some(payload),
                _ => None,
            }),
    );
    push_goal_context_events_from_timeline(
        pending_goal_context_events,
        emitted_goal_context_ids,
        events
            .events
            .iter()
            .filter_map(|event| match &event.payload {
                EventPayload::GoalContext(payload) => Some(payload),
                _ => None,
            }),
    );
    pending_events.sort_by(|left, right| {
        left.time
            .total_cmp(&right.time)
            .then_with(|| left.frame_number.cmp(&right.frame_number))
            .then_with(|| (left.kind as u32).cmp(&(right.kind as u32)))
            .then_with(|| left.player_index.cmp(&right.player_index))
    });
    pending_team_events.sort_by(|left, right| {
        left.end_time
            .total_cmp(&right.end_time)
            .then_with(|| left.end_frame.cmp(&right.end_frame))
            .then_with(|| (left.kind as u32).cmp(&(right.kind as u32)))
            .then_with(|| left.is_team_0.cmp(&right.is_team_0))
    });
    pending_goal_context_events.sort_by(|left, right| {
        left.time
            .total_cmp(&right.time)
            .then_with(|| left.frame_number.cmp(&right.frame_number))
            .then_with(|| {
                left.scoring_team_is_team_0
                    .cmp(&right.scoring_team_is_team_0)
            })
    });
}
