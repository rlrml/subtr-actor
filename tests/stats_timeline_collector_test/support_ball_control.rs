fn ball_carry_kind_label_for_derivation(kind: BallCarryKind) -> StatLabel {
    match kind {
        BallCarryKind::Carry => StatLabel::new("kind", "carry"),
        BallCarryKind::AirDribble => StatLabel::new("kind", "air_dribble"),
    }
}

fn air_dribble_origin_label_for_derivation(origin: AirDribbleOrigin) -> StatLabel {
    StatLabel::new("origin", origin.as_label_value())
}

fn apply_ball_carry_event(stats: &mut BallCarryStats, event: &BallCarryEvent) {
    stats
        .labeled_event_counts
        .increment([ball_carry_kind_label_for_derivation(event.kind)]);
    stats.carry_count = stats.labeled_event_counts.total();
    stats.total_carry_time += event.duration;
    stats.total_straight_line_distance += event.straight_line_distance;
    stats.total_path_distance += event.path_distance;
    stats.longest_carry_time = stats.longest_carry_time.max(event.duration);
    stats.furthest_carry_distance = stats
        .furthest_carry_distance
        .max(event.straight_line_distance);
    stats.fastest_carry_speed = stats.fastest_carry_speed.max(event.average_speed);
    stats.carry_speed_sum += event.average_speed;
    stats.average_horizontal_gap_sum += event.average_horizontal_gap;
    stats.average_vertical_gap_sum += event.average_vertical_gap;
}

fn apply_air_dribble_event(stats: &mut AirDribbleStats, event: &BallCarryEvent) {
    if let Some(origin) = event.air_dribble_origin {
        stats
            .labeled_event_counts
            .increment([air_dribble_origin_label_for_derivation(origin)]);
        match origin {
            AirDribbleOrigin::GroundToAir => stats.ground_to_air_count += 1,
            AirDribbleOrigin::WallToAir => stats.wall_to_air_count += 1,
        }
    }
    stats.count = stats.labeled_event_counts.total();
    stats.total_time += event.duration;
    stats.total_straight_line_distance += event.straight_line_distance;
    stats.total_path_distance += event.path_distance;
    stats.longest_time = stats.longest_time.max(event.duration);
    stats.furthest_distance = stats.furthest_distance.max(event.straight_line_distance);
    stats.fastest_speed = stats.fastest_speed.max(event.average_speed);
    stats.speed_sum += event.average_speed;
    stats.average_horizontal_gap_sum += event.average_horizontal_gap;
    stats.average_vertical_gap_sum += event.average_vertical_gap;
    stats.total_touch_count += event.touch_count;
    stats.max_touch_count = stats.max_touch_count.max(event.touch_count);
}

fn assert_ball_carry_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline_payloads_by_stream(timeline, "ball_carry", |payload| match payload { EventPayload::BallCarry(event) => Some(event), _ => None });
    events.sort_by(|left, right| {
        left.end_frame
            .cmp(&right.end_frame)
            .then_with(|| left.end_time.total_cmp(&right.end_time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, BallCarryStats> = HashMap::new();
    let mut player_air_dribbles: HashMap<PlayerId, AirDribbleStats> = HashMap::new();
    let mut team_zero = BallCarryStats::default();
    let mut team_one = BallCarryStats::default();
    let mut team_zero_air_dribble = AirDribbleStats::default();
    let mut team_one_air_dribble = AirDribbleStats::default();

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].end_frame < frame.frame_number {
            let event = &events[event_index];
            match event.kind {
                BallCarryKind::Carry => {
                    apply_ball_carry_event(
                        players.entry(event.player_id.clone()).or_default(),
                        event,
                    );
                    apply_ball_carry_event(
                        if event.is_team_0 {
                            &mut team_zero
                        } else {
                            &mut team_one
                        },
                        event,
                    );
                }
                BallCarryKind::AirDribble => {
                    apply_air_dribble_event(
                        player_air_dribbles
                            .entry(event.player_id.clone())
                            .or_default(),
                        event,
                    );
                    apply_air_dribble_event(
                        if event.is_team_0 {
                            &mut team_zero_air_dribble
                        } else {
                            &mut team_one_air_dribble
                        },
                        event,
                    );
                }
            }
            event_index += 1;
        }

        assert_eq!(
            frame.team_zero.ball_carry, team_zero,
            "{replay_path} team_zero ball_carry frame {}",
            frame.frame_number,
        );
        assert_eq!(
            frame.team_one.ball_carry, team_one,
            "{replay_path} team_one ball_carry frame {}",
            frame.frame_number,
        );
        assert_eq!(
            frame.team_zero.air_dribble, team_zero_air_dribble,
            "{replay_path} team_zero air_dribble frame {}",
            frame.frame_number,
        );
        assert_eq!(
            frame.team_one.air_dribble, team_one_air_dribble,
            "{replay_path} team_one air_dribble frame {}",
            frame.frame_number,
        );
        for player in &frame.players {
            assert_eq!(
                player.ball_carry,
                players.get(&player.player_id).cloned().unwrap_or_default(),
                "{replay_path} player {} ball_carry frame {}",
                player.name,
                frame.frame_number,
            );
            assert_eq!(
                player.air_dribble,
                player_air_dribbles
                    .get(&player.player_id)
                    .cloned()
                    .unwrap_or_default(),
                "{replay_path} player {} air_dribble frame {}",
                player.name,
                frame.frame_number,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed ball-carry events"
    );
}

fn apply_wall_aerial_event(
    stats: &mut WallAerialStats,
    event: &WallAerialEvent,
    frame: &ReplayStatsFrame,
) {
    const WALL_AERIAL_HIGH_CONFIDENCE: f32 = 0.78;

    stats.count += 1;
    if event.confidence >= WALL_AERIAL_HIGH_CONFIDENCE {
        stats.high_confidence_count += 1;
    }
    stats.is_last_wall_aerial = true;
    stats.last_wall_aerial_time = Some(event.time);
    stats.last_wall_aerial_frame = Some(event.frame);
    stats.time_since_last_wall_aerial = Some((frame.time - event.time).max(0.0));
    stats.frames_since_last_wall_aerial = Some(frame.frame_number.saturating_sub(event.frame));
    stats.last_confidence = Some(event.confidence);
    stats.best_confidence = stats.best_confidence.max(event.confidence);
    stats.cumulative_confidence += event.confidence;
    stats.cumulative_setup_duration += event.setup_duration;
    stats.cumulative_takeoff_to_touch_time += event.time_since_takeoff;
    stats.cumulative_touch_height += event.player_position[2];
}

fn advance_wall_aerial_stats(
    stats: &mut WallAerialStats,
    frame: &ReplayStatsFrame,
    is_last_wall_aerial_player: bool,
) {
    stats.is_last_wall_aerial = is_last_wall_aerial_player;
    stats.time_since_last_wall_aerial = stats
        .last_wall_aerial_time
        .map(|time| (frame.time - time).max(0.0));
    stats.frames_since_last_wall_aerial = stats
        .last_wall_aerial_frame
        .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
}

fn assert_wall_aerial_stats_match(
    scope: &str,
    actual: &WallAerialStats,
    expected: &WallAerialStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} wall_aerial.count");
    assert_eq!(
        actual.high_confidence_count, expected.high_confidence_count,
        "{scope} wall_aerial.high_confidence_count"
    );
    assert_eq!(
        actual.is_last_wall_aerial, expected.is_last_wall_aerial,
        "{scope} wall_aerial.is_last_wall_aerial"
    );
    assert_eq!(
        actual.last_wall_aerial_frame, expected.last_wall_aerial_frame,
        "{scope} wall_aerial.last_wall_aerial_frame"
    );
    assert!(
        match (actual.last_wall_aerial_time, expected.last_wall_aerial_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} wall_aerial.last_wall_aerial_time: actual {:?} expected {:?}",
        actual.last_wall_aerial_time,
        expected.last_wall_aerial_time
    );
    assert_eq!(
        actual.frames_since_last_wall_aerial, expected.frames_since_last_wall_aerial,
        "{scope} wall_aerial.frames_since_last_wall_aerial"
    );
    assert!(
        match (
            actual.time_since_last_wall_aerial,
            expected.time_since_last_wall_aerial,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} wall_aerial.time_since_last_wall_aerial: actual {:?} expected {:?}",
        actual.time_since_last_wall_aerial,
        expected.time_since_last_wall_aerial
    );
    assert!(
        match (actual.last_confidence, expected.last_confidence) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} wall_aerial.last_confidence: actual {:?} expected {:?}",
        actual.last_confidence,
        expected.last_confidence
    );
    assert!(
        (actual.best_confidence - expected.best_confidence).abs() < 0.001,
        "{scope} wall_aerial.best_confidence: actual {:.3} expected {:.3}",
        actual.best_confidence,
        expected.best_confidence
    );
    assert!(
        (actual.cumulative_confidence - expected.cumulative_confidence).abs() < 0.001,
        "{scope} wall_aerial.cumulative_confidence: actual {:.3} expected {:.3}",
        actual.cumulative_confidence,
        expected.cumulative_confidence
    );
    assert!(
        (actual.cumulative_setup_duration - expected.cumulative_setup_duration).abs() < 0.001,
        "{scope} wall_aerial.cumulative_setup_duration: actual {:.3} expected {:.3}",
        actual.cumulative_setup_duration,
        expected.cumulative_setup_duration
    );
    assert!(
        (actual.cumulative_takeoff_to_touch_time - expected.cumulative_takeoff_to_touch_time).abs()
            < 0.001,
        "{scope} wall_aerial.cumulative_takeoff_to_touch_time: actual {:.3} expected {:.3}",
        actual.cumulative_takeoff_to_touch_time,
        expected.cumulative_takeoff_to_touch_time
    );
    assert!(
        (actual.cumulative_touch_height - expected.cumulative_touch_height).abs() < 0.001,
        "{scope} wall_aerial.cumulative_touch_height: actual {:.3} expected {:.3}",
        actual.cumulative_touch_height,
        expected.cumulative_touch_height
    );
}

fn assert_wall_aerial_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline_payloads_by_stream(timeline, "wall_aerial", |payload| match payload { EventPayload::WallAerial(event) => Some(event), _ => None });
    events.sort_by(|left, right| {
        left.sample_frame
            .cmp(&right.sample_frame)
            .then_with(|| left.sample_time.total_cmp(&right.sample_time))
            .then_with(|| left.frame.cmp(&right.frame))
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, WallAerialStats> = HashMap::new();
    let mut last_wall_aerial_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        for (player_id, stats) in players.iter_mut() {
            advance_wall_aerial_stats(
                stats,
                frame,
                frame.is_live_play && last_wall_aerial_player.as_ref() == Some(player_id),
            );
        }

        if frame.is_live_play {
            let mut processed_event = false;
            while event_index < events.len()
                && events[event_index].sample_frame <= frame.frame_number
            {
                let event = &events[event_index];
                let stats = players.entry(event.player.clone()).or_default();
                apply_wall_aerial_event(stats, event, frame);
                last_wall_aerial_player = Some(event.player.clone());
                processed_event = true;
                event_index += 1;
            }

            if processed_event {
                for stats in players.values_mut() {
                    stats.is_last_wall_aerial = false;
                }
            }
            if let Some(player_id) = last_wall_aerial_player.as_ref() {
                if let Some(stats) = players.get_mut(player_id) {
                    stats.is_last_wall_aerial = true;
                }
            }
        } else {
            last_wall_aerial_player = None;
        }

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_wall_aerial_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.wall_aerial,
                &expected,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed wall-aerial events"
    );
}

fn apply_wall_aerial_shot_event(stats: &mut WallAerialShotStats, event: &WallAerialShotEvent) {
    const WALL_AERIAL_HIGH_CONFIDENCE: f32 = 0.78;

    stats.count += 1;
    if event.confidence >= WALL_AERIAL_HIGH_CONFIDENCE {
        stats.high_confidence_count += 1;
    }
    stats.is_last_wall_aerial_shot = true;
    stats.last_wall_aerial_shot_time = Some(event.time);
    stats.last_wall_aerial_shot_frame = Some(event.frame);
    stats.time_since_last_wall_aerial_shot = Some(0.0);
    stats.frames_since_last_wall_aerial_shot = Some(0);
    stats.last_confidence = Some(event.confidence);
    stats.best_confidence = stats.best_confidence.max(event.confidence);
    stats.cumulative_confidence += event.confidence;
    stats.cumulative_takeoff_to_shot_time += event.time_since_takeoff;
    stats.cumulative_shot_height += event.player_position[2];
}

fn advance_wall_aerial_shot_stats(
    stats: &mut WallAerialShotStats,
    frame: &ReplayStatsFrame,
    is_last_wall_aerial_shot_player: bool,
) {
    stats.is_last_wall_aerial_shot = is_last_wall_aerial_shot_player;
    stats.time_since_last_wall_aerial_shot = stats
        .last_wall_aerial_shot_time
        .map(|time| (frame.time - time).max(0.0));
    stats.frames_since_last_wall_aerial_shot = stats
        .last_wall_aerial_shot_frame
        .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
}

fn assert_wall_aerial_shot_stats_match(
    scope: &str,
    actual: &WallAerialShotStats,
    expected: &WallAerialShotStats,
) {
    assert_eq!(
        actual.count, expected.count,
        "{scope} wall_aerial_shot.count"
    );
    assert_eq!(
        actual.high_confidence_count, expected.high_confidence_count,
        "{scope} wall_aerial_shot.high_confidence_count"
    );
    assert_eq!(
        actual.is_last_wall_aerial_shot, expected.is_last_wall_aerial_shot,
        "{scope} wall_aerial_shot.is_last_wall_aerial_shot"
    );
    assert_eq!(
        actual.last_wall_aerial_shot_frame, expected.last_wall_aerial_shot_frame,
        "{scope} wall_aerial_shot.last_wall_aerial_shot_frame"
    );
    assert!(
        match (
            actual.last_wall_aerial_shot_time,
            expected.last_wall_aerial_shot_time,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} wall_aerial_shot.last_wall_aerial_shot_time: actual {:?} expected {:?}",
        actual.last_wall_aerial_shot_time,
        expected.last_wall_aerial_shot_time
    );
    assert_eq!(
        actual.frames_since_last_wall_aerial_shot, expected.frames_since_last_wall_aerial_shot,
        "{scope} wall_aerial_shot.frames_since_last_wall_aerial_shot"
    );
    assert!(
        match (
            actual.time_since_last_wall_aerial_shot,
            expected.time_since_last_wall_aerial_shot,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} wall_aerial_shot.time_since_last_wall_aerial_shot: actual {:?} expected {:?}",
        actual.time_since_last_wall_aerial_shot,
        expected.time_since_last_wall_aerial_shot
    );
    assert!(
        match (actual.last_confidence, expected.last_confidence) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} wall_aerial_shot.last_confidence: actual {:?} expected {:?}",
        actual.last_confidence,
        expected.last_confidence
    );
    assert!(
        (actual.best_confidence - expected.best_confidence).abs() < 0.001,
        "{scope} wall_aerial_shot.best_confidence: actual {:.3} expected {:.3}",
        actual.best_confidence,
        expected.best_confidence
    );
    assert!(
        (actual.cumulative_confidence - expected.cumulative_confidence).abs() < 0.001,
        "{scope} wall_aerial_shot.cumulative_confidence: actual {:.3} expected {:.3}",
        actual.cumulative_confidence,
        expected.cumulative_confidence
    );
    assert!(
        (actual.cumulative_takeoff_to_shot_time - expected.cumulative_takeoff_to_shot_time).abs()
            < 0.001,
        "{scope} wall_aerial_shot.cumulative_takeoff_to_shot_time: actual {:.3} expected {:.3}",
        actual.cumulative_takeoff_to_shot_time,
        expected.cumulative_takeoff_to_shot_time
    );
    assert!(
        (actual.cumulative_shot_height - expected.cumulative_shot_height).abs() < 0.001,
        "{scope} wall_aerial_shot.cumulative_shot_height: actual {:.3} expected {:.3}",
        actual.cumulative_shot_height,
        expected.cumulative_shot_height
    );
}

fn assert_wall_aerial_shot_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline_payloads_by_stream(timeline, "wall_aerial_shot", |payload| match payload { EventPayload::WallAerialShot(event) => Some(event), _ => None });
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, WallAerialShotStats> = HashMap::new();
    let mut last_wall_aerial_shot_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        for (player_id, stats) in players.iter_mut() {
            advance_wall_aerial_shot_stats(
                stats,
                frame,
                frame.is_live_play && last_wall_aerial_shot_player.as_ref() == Some(player_id),
            );
        }

        if frame.is_live_play {
            let mut processed_event = false;
            while event_index < events.len() && events[event_index].frame <= frame.frame_number {
                let event = &events[event_index];
                let stats = players.entry(event.player.clone()).or_default();
                apply_wall_aerial_shot_event(stats, event);
                last_wall_aerial_shot_player = Some(event.player.clone());
                processed_event = true;
                event_index += 1;
            }

            if processed_event {
                for stats in players.values_mut() {
                    stats.is_last_wall_aerial_shot = false;
                }
            }
            if let Some(player_id) = last_wall_aerial_shot_player.as_ref() {
                if let Some(stats) = players.get_mut(player_id) {
                    stats.is_last_wall_aerial_shot = true;
                }
            }
        } else {
            last_wall_aerial_shot_player = None;
        }

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_wall_aerial_shot_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.wall_aerial_shot,
                &expected,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed wall-aerial-shot events"
    );
}

fn apply_ceiling_shot_event(stats: &mut CeilingShotStats, event: &CeilingShotEvent) {
    const CEILING_SHOT_HIGH_CONFIDENCE: f32 = 0.78;

    stats
        .labeled_event_counts
        .increment([confidence_band_label_for_derivation(
            event.confidence >= CEILING_SHOT_HIGH_CONFIDENCE,
        )]);
    stats.count = stats.labeled_event_counts.total();
    stats.high_confidence_count = stats
        .labeled_event_counts
        .count_matching(&[confidence_band_label_for_derivation(true)]);
    stats.is_last_ceiling_shot = true;
    stats.last_ceiling_shot_time = Some(event.time);
    stats.last_ceiling_shot_frame = Some(event.frame);
    stats.time_since_last_ceiling_shot = Some(0.0);
    stats.frames_since_last_ceiling_shot = Some(0);
    stats.last_confidence = Some(event.confidence);
    stats.best_confidence = stats.best_confidence.max(event.confidence);
    stats.cumulative_confidence += event.confidence;
}

fn advance_ceiling_shot_stats(
    stats: &mut CeilingShotStats,
    frame: &ReplayStatsFrame,
    is_last_ceiling_shot_player: bool,
) {
    stats.is_last_ceiling_shot = is_last_ceiling_shot_player;
    stats.time_since_last_ceiling_shot = stats
        .last_ceiling_shot_time
        .map(|time| (frame.time - time).max(0.0));
    stats.frames_since_last_ceiling_shot = stats
        .last_ceiling_shot_frame
        .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
}

fn assert_ceiling_shot_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline_payloads_by_stream(timeline, "ceiling_shot", |payload| match payload { EventPayload::CeilingShot(event) => Some(event), _ => None });
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, CeilingShotStats> = HashMap::new();
    let mut last_ceiling_shot_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        if frame.is_live_play {
            for (player_id, stats) in players.iter_mut() {
                advance_ceiling_shot_stats(
                    stats,
                    frame,
                    last_ceiling_shot_player.as_ref() == Some(player_id),
                );
            }

            while event_index < events.len() && events[event_index].frame <= frame.frame_number {
                let event = &events[event_index];
                let stats = players.entry(event.player.clone()).or_default();
                apply_ceiling_shot_event(stats, event);
                last_ceiling_shot_player = Some(event.player.clone());
                event_index += 1;
            }
        } else {
            last_ceiling_shot_player = None;
        }

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_eq!(
                player.ceiling_shot, expected,
                "{replay_path} player {} ceiling_shot frame {}",
                player.name, frame.frame_number,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed ceiling-shot events"
    );
}

fn confidence_band_label_for_derivation(high_confidence: bool) -> StatLabel {
    if high_confidence {
        StatLabel::new("confidence_band", "high")
    } else {
        StatLabel::new("confidence_band", "standard")
    }
}

fn vertical_state_label_for_derivation(aerial: bool) -> StatLabel {
    if aerial {
        StatLabel::new("vertical_state", "aerial")
    } else {
        StatLabel::new("vertical_state", "grounded")
    }
}

fn flick_kind_label_for_derivation(value: &str) -> StatLabel {
    match value {
        "reverse" => StatLabel::new("kind", "reverse"),
        _ => StatLabel::new("kind", "other"),
    }
}

fn apply_flick_event(stats: &mut FlickStats, event: &FlickEvent) {
    const FLICK_HIGH_CONFIDENCE: f32 = 0.80;

    stats.labeled_event_counts.increment([
        confidence_band_label_for_derivation(
            event.confidence >= FLICK_HIGH_CONFIDENCE,
        ),
        flick_kind_label_for_derivation(&event.kind),
    ]);
    stats.count = stats.labeled_event_counts.total();
    stats.high_confidence_count = stats
        .labeled_event_counts
        .count_matching(&[confidence_band_label_for_derivation(true)]);
    stats.is_last_flick = true;
    stats.last_flick_time = Some(event.time);
    stats.last_flick_frame = Some(event.frame);
    stats.time_since_last_flick = Some(0.0);
    stats.frames_since_last_flick = Some(0);
    stats.last_confidence = Some(event.confidence);
    stats.best_confidence = stats.best_confidence.max(event.confidence);
    stats.cumulative_confidence += event.confidence;
    stats.cumulative_setup_duration += event.setup_duration;
    stats.cumulative_ball_speed_change += event.ball_speed_change;
}

fn advance_flick_stats(
    stats: &mut FlickStats,
    frame: &ReplayStatsFrame,
    is_last_flick_player: bool,
) {
    stats.is_last_flick = is_last_flick_player;
    stats.time_since_last_flick = stats
        .last_flick_time
        .map(|time| (frame.time - time).max(0.0));
    stats.frames_since_last_flick = stats
        .last_flick_frame
        .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
}

fn assert_flick_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline_payloads_by_stream(timeline, "flick", |payload| match payload { EventPayload::Flick(event) => Some(event), _ => None });
    events.sort_by(|left, right| {
        left.sample_frame
            .cmp(&right.sample_frame)
            .then_with(|| left.sample_time.total_cmp(&right.sample_time))
            .then_with(|| left.frame.cmp(&right.frame))
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, FlickStats> = HashMap::new();
    let mut last_flick_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        if frame.is_live_play {
            for (player_id, stats) in players.iter_mut() {
                advance_flick_stats(stats, frame, last_flick_player.as_ref() == Some(player_id));
            }

            while event_index < events.len()
                && events[event_index].sample_frame <= frame.frame_number
            {
                let event = &events[event_index];
                let stats = players.entry(event.player.clone()).or_default();
                apply_flick_event(stats, event);
                last_flick_player = Some(event.player.clone());
                event_index += 1;
            }

            if let Some(player_id) = last_flick_player.as_ref() {
                if let Some(stats) = players.get_mut(player_id) {
                    stats.is_last_flick = true;
                }
            }
        } else {
            last_flick_player = None;
        }

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_eq!(
                player.flick, expected,
                "{replay_path} player {} flick frame {} live={} phase={:?}",
                player.name, frame.frame_number, frame.is_live_play, frame.gameplay_phase,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed flick events"
    );
}

fn apply_musty_flick_event(stats: &mut MustyFlickStats, event: &MustyFlickEvent) {
    const MUSTY_HIGH_CONFIDENCE: f32 = 0.80;

    stats.labeled_event_counts.increment([
        vertical_state_label_for_derivation(event.aerial),
        confidence_band_label_for_derivation(event.confidence >= MUSTY_HIGH_CONFIDENCE),
    ]);
    stats.count = stats.labeled_event_counts.total();
    stats.aerial_count = stats
        .labeled_event_counts
        .count_matching(&[vertical_state_label_for_derivation(true)]);
    stats.high_confidence_count = stats
        .labeled_event_counts
        .count_matching(&[confidence_band_label_for_derivation(true)]);
    stats.is_last_musty = true;
    stats.last_musty_time = Some(event.time);
    stats.last_musty_frame = Some(event.frame);
    stats.time_since_last_musty = Some(0.0);
    stats.frames_since_last_musty = Some(0);
    stats.last_confidence = Some(event.confidence);
    stats.best_confidence = stats.best_confidence.max(event.confidence);
    stats.cumulative_confidence += event.confidence;
}

fn advance_musty_flick_stats(
    stats: &mut MustyFlickStats,
    frame: &ReplayStatsFrame,
    is_last_musty_player: bool,
) {
    stats.is_last_musty = is_last_musty_player;
    stats.time_since_last_musty = stats
        .last_musty_time
        .map(|time| (frame.time - time).max(0.0));
    stats.frames_since_last_musty = stats
        .last_musty_frame
        .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
}

fn assert_musty_flick_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline_payloads_by_stream(timeline, "musty_flick", |payload| match payload { EventPayload::MustyFlick(event) => Some(event), _ => None });
    events.sort_by(|left, right| {
        left.sample_frame
            .cmp(&right.sample_frame)
            .then_with(|| left.sample_time.total_cmp(&right.sample_time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, MustyFlickStats> = HashMap::new();
    let mut last_musty_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        if frame.is_live_play {
            for (player_id, stats) in players.iter_mut() {
                advance_musty_flick_stats(
                    stats,
                    frame,
                    last_musty_player.as_ref() == Some(player_id),
                );
            }

            let mut processed_event = false;
            while event_index < events.len()
                && events[event_index].sample_frame <= frame.frame_number
            {
                let event = &events[event_index];
                let stats = players.entry(event.player.clone()).or_default();
                apply_musty_flick_event(stats, event);
                last_musty_player = Some(event.player.clone());
                event_index += 1;
                processed_event = true;
            }

            if processed_event {
                for stats in players.values_mut() {
                    stats.is_last_musty = false;
                }
            }

            if let Some(player_id) = last_musty_player.as_ref() {
                if let Some(stats) = players.get_mut(player_id) {
                    stats.is_last_musty = true;
                }
            }
        } else {
            last_musty_player = None;
        }

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_eq!(
                player.musty_flick, expected,
                "{replay_path} player {} musty_flick frame {}",
                player.name, frame.frame_number,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed musty-flick events"
    );
}

fn assert_dodge_reset_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline_payloads_by_stream(timeline, "dodge_reset", |payload| match payload { EventPayload::DodgeReset(event) => Some(event), _ => None });
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DodgeResetStats> = HashMap::new();

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].frame <= frame.frame_number {
            let event = &events[event_index];
            let stats = players.entry(event.player.clone()).or_default();
            stats.count += 1;
            if event.on_ball {
                stats.on_ball_count += 1;
            }
            event_index += 1;
        }

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_eq!(
                player.dodge_reset, expected,
                "{replay_path} player {} dodge_reset frame {}",
                player.name, frame.frame_number,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed dodge-reset events"
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DerivedPowerslideState {
    active: bool,
    is_team_0: bool,
}
