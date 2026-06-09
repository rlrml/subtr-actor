fn assert_quality_mechanic_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    const HALF_FLIP_HIGH_CONFIDENCE: f32 = 0.78;
    const WAVEDASH_HIGH_CONFIDENCE: f32 = 0.75;

    let half_flip_events =
        timeline_payloads_by_stream(timeline, "half_flip", |payload| match payload {
            EventPayload::HalfFlip(event) => Some(event),
            _ => None,
        });
    let wavedash_events =
        timeline_payloads_by_stream(timeline, "wavedash", |payload| match payload {
            EventPayload::Wavedash(event) => Some(event),
            _ => None,
        });
    let mut half_flip_event_index = 0;
    let mut wavedash_event_index = 0;
    let mut half_flip_players: HashMap<PlayerId, DerivedQualityMechanicStats> = HashMap::new();
    let mut wavedash_players: HashMap<PlayerId, DerivedQualityMechanicStats> = HashMap::new();
    let mut half_flip_frame_stats: HashMap<PlayerId, DerivedQualityMechanicFrameStats> =
        HashMap::new();
    let mut wavedash_frame_stats: HashMap<PlayerId, DerivedQualityMechanicFrameStats> =
        HashMap::new();
    let mut last_half_flip_player: Option<PlayerId> = None;
    let mut last_wavedash_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        if frame.is_live_play {
            while half_flip_event_index < half_flip_events.len()
                && half_flip_events[half_flip_event_index].frame <= frame.frame_number
            {
                let event = &half_flip_events[half_flip_event_index];
                half_flip_players
                    .entry(event.player.clone())
                    .or_default()
                    .record(
                        event.frame,
                        event.time,
                        event.frame,
                        event.time,
                        event.confidence,
                        event.confidence >= HALF_FLIP_HIGH_CONFIDENCE,
                    );
                last_half_flip_player = Some(event.player.clone());
                half_flip_event_index += 1;
            }

            while wavedash_event_index < wavedash_events.len()
                && wavedash_events[wavedash_event_index].frame <= frame.frame_number
            {
                let event = &wavedash_events[wavedash_event_index];
                wavedash_players
                    .entry(event.player.clone())
                    .or_default()
                    .record(
                        event.frame,
                        event.time,
                        event.frame,
                        event.time,
                        event.confidence,
                        event.confidence >= WAVEDASH_HIGH_CONFIDENCE,
                    );
                last_wavedash_player = Some(event.player.clone());
                wavedash_event_index += 1;
            }

            for player in &frame.players {
                let half_flip_expected = DerivedQualityMechanicFrameStats::from_accumulator(
                    half_flip_players.get(&player.player_id),
                    frame,
                    last_half_flip_player.as_ref() == Some(&player.player_id),
                );
                half_flip_frame_stats.insert(player.player_id.clone(), half_flip_expected.clone());
                assert_half_flip_derived_stats_match(
                    &format!(
                        "{replay_path} player {} frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.half_flip,
                    &half_flip_expected,
                );

                let wavedash_expected = DerivedQualityMechanicFrameStats::from_accumulator(
                    wavedash_players.get(&player.player_id),
                    frame,
                    last_wavedash_player.as_ref() == Some(&player.player_id),
                );
                wavedash_frame_stats.insert(player.player_id.clone(), wavedash_expected.clone());
                assert_wavedash_derived_stats_match(
                    &format!(
                        "{replay_path} player {} frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.wavedash,
                    &wavedash_expected,
                );
            }
        } else {
            for player in &frame.players {
                let half_flip_expected = half_flip_frame_stats
                    .get(&player.player_id)
                    .cloned()
                    .unwrap_or_default();
                assert_half_flip_derived_stats_match(
                    &format!(
                        "{replay_path} player {} inactive frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.half_flip,
                    &half_flip_expected,
                );

                let wavedash_expected = wavedash_frame_stats
                    .get(&player.player_id)
                    .cloned()
                    .unwrap_or_default();
                assert_wavedash_derived_stats_match(
                    &format!(
                        "{replay_path} player {} inactive frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.wavedash,
                    &wavedash_expected,
                );
            }
            last_half_flip_player = None;
            last_wavedash_player = None;
        }
    }

    assert_eq!(
        half_flip_event_index,
        half_flip_events.len(),
        "{replay_path} unprocessed half-flip events"
    );
    assert_eq!(
        wavedash_event_index,
        wavedash_events.len(),
        "{replay_path} unprocessed wavedash events"
    );
}

fn assert_speed_flip_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    const SPEED_FLIP_HIGH_CONFIDENCE: f32 = 0.75;

    let speed_flip_events =
        timeline_payloads_by_stream(timeline, "speed_flip", |payload| match payload {
            EventPayload::SpeedFlip(event) => Some(event),
            _ => None,
        });
    let mut speed_flip_event_index = 0;
    let mut speed_flip_players: HashMap<PlayerId, DerivedQualityMechanicStats> = HashMap::new();
    let mut speed_flip_frame_stats: HashMap<PlayerId, DerivedQualityMechanicFrameStats> =
        HashMap::new();
    let mut last_speed_flip_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        let speed_flip_stats_advance = frame.is_live_play || frame.ball_has_been_hit == Some(false);
        if speed_flip_stats_advance {
            while speed_flip_event_index < speed_flip_events.len()
                && speed_flip_events[speed_flip_event_index].resolved_frame <= frame.frame_number
            {
                let event = &speed_flip_events[speed_flip_event_index];
                speed_flip_players
                    .entry(event.player.clone())
                    .or_default()
                    .record(
                        event.frame,
                        event.time,
                        event.resolved_frame,
                        event.resolved_time,
                        event.confidence,
                        event.confidence >= SPEED_FLIP_HIGH_CONFIDENCE,
                    );
                last_speed_flip_player = Some(event.player.clone());
                speed_flip_event_index += 1;
            }

            for player in &frame.players {
                let expected = DerivedQualityMechanicFrameStats::from_accumulator(
                    speed_flip_players.get(&player.player_id),
                    frame,
                    last_speed_flip_player.as_ref() == Some(&player.player_id),
                );
                speed_flip_frame_stats.insert(player.player_id.clone(), expected.clone());
                assert_speed_flip_derived_stats_match(
                    &format!(
                        "{replay_path} player {} frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.speed_flip,
                    &expected,
                );
            }
        } else {
            for player in &frame.players {
                let expected = speed_flip_frame_stats
                    .get(&player.player_id)
                    .cloned()
                    .unwrap_or_default();
                assert_speed_flip_derived_stats_match(
                    &format!(
                        "{replay_path} player {} frozen frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.speed_flip,
                    &expected,
                );
            }
        }
    }

    assert_eq!(
        speed_flip_event_index,
        speed_flip_events.len(),
        "{replay_path} unprocessed speed-flip events"
    );
}

#[derive(Clone, Default)]
struct DerivedWhiffFrameStats {
    stats: WhiffStats,
}

impl DerivedWhiffFrameStats {
    fn record_event(&mut self, event: &WhiffEvent, frame: &ReplayStatsFrame) {
        match event.kind {
            WhiffEventKind::Whiff => {
                self.stats.whiff_count += 1;
                if event.aerial {
                    self.stats.aerial_whiff_count += 1;
                } else {
                    self.stats.grounded_whiff_count += 1;
                }
                if event.dodge_active {
                    self.stats.dodge_whiff_count += 1;
                }
                self.stats.last_whiff_time = Some(event.time);
                self.stats.last_whiff_frame = Some(event.frame);
                self.stats.time_since_last_whiff = Some((frame.time - event.time).max(0.0));
                self.stats.frames_since_last_whiff =
                    Some(frame.frame_number.saturating_sub(event.frame));
                self.stats.last_closest_approach_distance = Some(event.closest_approach_distance);
                self.stats.best_closest_approach_distance = Some(
                    self.stats
                        .best_closest_approach_distance
                        .map(|distance| distance.min(event.closest_approach_distance))
                        .unwrap_or(event.closest_approach_distance),
                );
                self.stats.cumulative_closest_approach_distance += event.closest_approach_distance;
                self.stats.is_last_whiff = true;
            }
            WhiffEventKind::BeatenToBall => {
                self.stats.beaten_to_ball_count += 1;
            }
        }
    }

    fn advance_live_frame(&mut self, frame: &ReplayStatsFrame, is_last_whiff_player: bool) {
        self.stats.is_last_whiff = is_last_whiff_player;
        self.stats.time_since_last_whiff = self
            .stats
            .last_whiff_time
            .map(|time| (frame.time - time).max(0.0));
        self.stats.frames_since_last_whiff = self
            .stats
            .last_whiff_frame
            .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
    }
}

fn assert_whiff_derived_stats_match(scope: &str, actual: &WhiffStats, expected: &WhiffStats) {
    assert_eq!(
        actual.whiff_count, expected.whiff_count,
        "{scope} whiff_count"
    );
    assert_eq!(
        actual.beaten_to_ball_count, expected.beaten_to_ball_count,
        "{scope} beaten_to_ball_count"
    );
    assert_eq!(
        actual.grounded_whiff_count, expected.grounded_whiff_count,
        "{scope} grounded_whiff_count"
    );
    assert_eq!(
        actual.aerial_whiff_count, expected.aerial_whiff_count,
        "{scope} aerial_whiff_count"
    );
    assert_eq!(
        actual.dodge_whiff_count, expected.dodge_whiff_count,
        "{scope} dodge_whiff_count"
    );
    assert_eq!(
        actual.is_last_whiff, expected.is_last_whiff,
        "{scope} is_last_whiff"
    );
    assert_eq!(
        actual.last_whiff_frame, expected.last_whiff_frame,
        "{scope} last_whiff_frame"
    );
    assert!(
        match (actual.last_whiff_time, expected.last_whiff_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} last_whiff_time: actual {:?} expected {:?}",
        actual.last_whiff_time,
        expected.last_whiff_time,
    );
    assert_eq!(
        actual.frames_since_last_whiff, expected.frames_since_last_whiff,
        "{scope} frames_since_last_whiff"
    );
    assert!(
        match (actual.time_since_last_whiff, expected.time_since_last_whiff) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} time_since_last_whiff: actual {:?} expected {:?}",
        actual.time_since_last_whiff,
        expected.time_since_last_whiff,
    );
    assert!(
        match (
            actual.last_closest_approach_distance,
            expected.last_closest_approach_distance,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} last_closest_approach_distance: actual {:?} expected {:?}",
        actual.last_closest_approach_distance,
        expected.last_closest_approach_distance,
    );
    assert!(
        match (
            actual.best_closest_approach_distance,
            expected.best_closest_approach_distance,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} best_closest_approach_distance: actual {:?} expected {:?}",
        actual.best_closest_approach_distance,
        expected.best_closest_approach_distance,
    );
    assert!(
        (actual.cumulative_closest_approach_distance
            - expected.cumulative_closest_approach_distance)
            .abs()
            < 0.001,
        "{scope} cumulative_closest_approach_distance: actual {:.3} expected {:.3}",
        actual.cumulative_closest_approach_distance,
        expected.cumulative_closest_approach_distance,
    );
}

fn assert_whiff_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline_payloads_by_stream(timeline, "whiff", |payload| match payload { EventPayload::Whiff(event) => Some(event), _ => None });
    events.sort_by(|left, right| {
        left.resolved_frame
            .cmp(&right.resolved_frame)
            .then_with(|| left.resolved_time.total_cmp(&right.resolved_time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DerivedWhiffFrameStats> = HashMap::new();
    let mut frozen_players: HashMap<PlayerId, DerivedWhiffFrameStats> = HashMap::new();
    let mut last_whiff_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        if frame.is_live_play {
            for player in players.values_mut() {
                player.advance_live_frame(frame, false);
            }

            while event_index < events.len()
                && events[event_index].resolved_frame <= frame.frame_number
            {
                let event = &events[event_index];
                let player = players.entry(event.player.clone()).or_default();
                player.record_event(event, frame);
                if event.kind == WhiffEventKind::Whiff {
                    last_whiff_player = Some(event.player.clone());
                }
                event_index += 1;
            }

            if let Some(player_id) = last_whiff_player.as_ref() {
                if let Some(player) = players.get_mut(player_id) {
                    player.stats.is_last_whiff = true;
                }
            }

            for player in &frame.players {
                let expected = players.get(&player.player_id).cloned().unwrap_or_default();
                assert_whiff_derived_stats_match(
                    &format!(
                        "{replay_path} player {} frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.whiff,
                    &expected.stats,
                );
                frozen_players.insert(player.player_id.clone(), expected);
            }
        } else {
            for player in &frame.players {
                let expected = frozen_players
                    .get(&player.player_id)
                    .cloned()
                    .unwrap_or_default();
                assert_whiff_derived_stats_match(
                    &format!(
                        "{replay_path} player {} inactive frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.whiff,
                    &expected.stats,
                );
            }
            last_whiff_player = None;
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed whiff events"
    );
}

#[derive(Clone, Default)]
struct DerivedBackboardPlayerStats {
    stats: BackboardPlayerStats,
}

impl DerivedBackboardPlayerStats {
    fn advance_frame(&mut self, frame: &ReplayStatsFrame, is_last_backboard_player: bool) {
        self.stats.is_last_backboard = is_last_backboard_player;
        self.stats.time_since_last_backboard = self
            .stats
            .last_backboard_time
            .map(|time| (frame.time - time).max(0.0));
        self.stats.frames_since_last_backboard = self
            .stats
            .last_backboard_frame
            .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
    }

    fn record_event(&mut self, event: &BackboardBounceEvent, frame: &ReplayStatsFrame) {
        self.stats.count += 1;
        self.stats.last_backboard_time = Some(event.time);
        self.stats.last_backboard_frame = Some(event.frame);
        self.stats.time_since_last_backboard = Some((frame.time - event.time).max(0.0));
        self.stats.frames_since_last_backboard =
            Some(frame.frame_number.saturating_sub(event.frame));
    }
}

fn assert_backboard_derived_player_stats_match(
    scope: &str,
    actual: &BackboardPlayerStats,
    expected: &BackboardPlayerStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} backboard.count");
    assert_eq!(
        actual.is_last_backboard, expected.is_last_backboard,
        "{scope} backboard.is_last_backboard"
    );
    assert_eq!(
        actual.last_backboard_frame, expected.last_backboard_frame,
        "{scope} backboard.last_backboard_frame"
    );
    assert!(
        match (actual.last_backboard_time, expected.last_backboard_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} backboard.last_backboard_time: actual {:?} expected {:?}",
        actual.last_backboard_time,
        expected.last_backboard_time,
    );
    assert_eq!(
        actual.frames_since_last_backboard, expected.frames_since_last_backboard,
        "{scope} backboard.frames_since_last_backboard"
    );
    assert!(
        match (
            actual.time_since_last_backboard,
            expected.time_since_last_backboard,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} backboard.time_since_last_backboard: actual {:?} expected {:?}",
        actual.time_since_last_backboard,
        expected.time_since_last_backboard,
    );
}

fn assert_backboard_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline_payloads_by_stream(timeline, "backboard", |payload| match payload { EventPayload::Backboard(event) => Some(event), _ => None });
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DerivedBackboardPlayerStats> = HashMap::new();
    let mut team_zero = BackboardTeamStats::default();
    let mut team_one = BackboardTeamStats::default();
    let mut last_backboard_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        for (player_id, stats) in players.iter_mut() {
            stats.advance_frame(frame, last_backboard_player.as_ref() == Some(player_id));
        }

        let mut processed_event = false;
        while event_index < events.len() && events[event_index].frame <= frame.frame_number {
            let event = &events[event_index];
            players
                .entry(event.player.clone())
                .or_default()
                .record_event(event, frame);
            if event.is_team_0 {
                team_zero.count += 1;
            } else {
                team_one.count += 1;
            }
            last_backboard_player = Some(event.player.clone());
            event_index += 1;
            processed_event = true;
        }

        if processed_event {
            for stats in players.values_mut() {
                stats.stats.is_last_backboard = false;
            }
        }

        if let Some(player_id) = last_backboard_player.as_ref() {
            if let Some(stats) = players.get_mut(player_id) {
                stats.stats.is_last_backboard = true;
            }
        }

        assert_eq!(
            frame.team_zero.backboard, team_zero,
            "{replay_path} team_zero backboard frame {}",
            frame.frame_number
        );
        assert_eq!(
            frame.team_one.backboard, team_one,
            "{replay_path} team_one backboard frame {}",
            frame.frame_number
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_backboard_derived_player_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.backboard,
                &expected.stats,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed backboard events"
    );
}

#[derive(Clone, Default)]
struct DerivedDoubleTapPlayerStats {
    stats: DoubleTapPlayerStats,
}

impl DerivedDoubleTapPlayerStats {
    fn advance_frame(&mut self, frame: &ReplayStatsFrame, is_last_double_tap_player: bool) {
        self.stats.is_last_double_tap = is_last_double_tap_player;
        self.stats.time_since_last_double_tap = self
            .stats
            .last_double_tap_time
            .map(|time| (frame.time - time).max(0.0));
        self.stats.frames_since_last_double_tap = self
            .stats
            .last_double_tap_frame
            .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
    }

    fn record_event(&mut self, event: &DoubleTapEvent, frame: &ReplayStatsFrame) {
        self.stats.count += 1;
        self.stats.last_double_tap_time = Some(event.time);
        self.stats.last_double_tap_frame = Some(event.frame);
        self.stats.time_since_last_double_tap = Some((frame.time - event.time).max(0.0));
        self.stats.frames_since_last_double_tap =
            Some(frame.frame_number.saturating_sub(event.frame));
    }
}

fn assert_double_tap_derived_player_stats_match(
    scope: &str,
    actual: &DoubleTapPlayerStats,
    expected: &DoubleTapPlayerStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} double_tap.count");
    assert_eq!(
        actual.is_last_double_tap, expected.is_last_double_tap,
        "{scope} double_tap.is_last_double_tap"
    );
    assert_eq!(
        actual.last_double_tap_frame, expected.last_double_tap_frame,
        "{scope} double_tap.last_double_tap_frame"
    );
    assert!(
        match (actual.last_double_tap_time, expected.last_double_tap_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} double_tap.last_double_tap_time: actual {:?} expected {:?}",
        actual.last_double_tap_time,
        expected.last_double_tap_time,
    );
    assert_eq!(
        actual.frames_since_last_double_tap, expected.frames_since_last_double_tap,
        "{scope} double_tap.frames_since_last_double_tap"
    );
    assert!(
        match (
            actual.time_since_last_double_tap,
            expected.time_since_last_double_tap,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} double_tap.time_since_last_double_tap: actual {:?} expected {:?}",
        actual.time_since_last_double_tap,
        expected.time_since_last_double_tap,
    );
}

fn assert_double_tap_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline_payloads_by_stream(timeline, "double_tap", |payload| match payload { EventPayload::DoubleTap(event) => Some(event), _ => None });
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DerivedDoubleTapPlayerStats> = HashMap::new();
    let mut team_zero = DoubleTapTeamStats::default();
    let mut team_one = DoubleTapTeamStats::default();
    let mut last_double_tap_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        for (player_id, stats) in players.iter_mut() {
            stats.advance_frame(frame, last_double_tap_player.as_ref() == Some(player_id));
        }

        let mut processed_event = false;
        while event_index < events.len() && events[event_index].frame <= frame.frame_number {
            let event = &events[event_index];
            players
                .entry(event.player.clone())
                .or_default()
                .record_event(event, frame);
            if event.is_team_0 {
                team_zero.count += 1;
            } else {
                team_one.count += 1;
            }
            last_double_tap_player = Some(event.player.clone());
            event_index += 1;
            processed_event = true;
        }

        if processed_event {
            for stats in players.values_mut() {
                stats.stats.is_last_double_tap = false;
            }
        }

        if let Some(player_id) = last_double_tap_player.as_ref() {
            if let Some(stats) = players.get_mut(player_id) {
                stats.stats.is_last_double_tap = true;
            }
        }

        assert_eq!(
            frame.team_zero.double_tap, team_zero,
            "{replay_path} team_zero double_tap frame {}",
            frame.frame_number
        );
        assert_eq!(
            frame.team_one.double_tap, team_one,
            "{replay_path} team_one double_tap frame {}",
            frame.frame_number
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_double_tap_derived_player_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.double_tap,
                &expected.stats,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed double-tap events"
    );
}

#[derive(Clone, Default)]
struct DerivedPassPlayerStats {
    stats: PassPlayerStats,
}

impl DerivedPassPlayerStats {
    fn advance_frame(&mut self, frame: &ReplayStatsFrame, is_last_completed_pass_player: bool) {
        self.stats.is_last_completed_pass = is_last_completed_pass_player;
        self.stats.time_since_last_completed_pass = self
            .stats
            .last_completed_pass_time
            .map(|time| (frame.time - time).max(0.0));
        self.stats.frames_since_last_completed_pass = self
            .stats
            .last_completed_pass_frame
            .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
    }

    fn record_completed_pass(&mut self, event: &PassEvent, frame: &ReplayStatsFrame) {
        self.stats.completed_pass_count += 1;
        self.stats.total_pass_distance += event.ball_travel_distance;
        self.stats.total_pass_advance += event.ball_advance_distance;
        self.stats.longest_pass_distance = self
            .stats
            .longest_pass_distance
            .max(event.ball_travel_distance);
        self.stats.last_completed_pass_time = Some(event.time);
        self.stats.last_completed_pass_frame = Some(event.frame);
        self.stats.time_since_last_completed_pass = Some((frame.time - event.time).max(0.0));
        self.stats.frames_since_last_completed_pass =
            Some(frame.frame_number.saturating_sub(event.frame));
    }

    fn record_received_pass(&mut self) {
        self.stats.received_pass_count += 1;
    }
}

fn assert_pass_derived_player_stats_match(
    scope: &str,
    actual: &PassPlayerStats,
    expected: &PassPlayerStats,
) {
    assert_eq!(
        actual.completed_pass_count, expected.completed_pass_count,
        "{scope} pass.completed_pass_count"
    );
    assert_eq!(
        actual.received_pass_count, expected.received_pass_count,
        "{scope} pass.received_pass_count"
    );
    assert!(
        (actual.total_pass_distance - expected.total_pass_distance).abs() < 0.001,
        "{scope} pass.total_pass_distance: actual {:.3} expected {:.3}",
        actual.total_pass_distance,
        expected.total_pass_distance,
    );
    assert!(
        (actual.total_pass_advance - expected.total_pass_advance).abs() < 0.001,
        "{scope} pass.total_pass_advance: actual {:.3} expected {:.3}",
        actual.total_pass_advance,
        expected.total_pass_advance,
    );
    assert!(
        (actual.longest_pass_distance - expected.longest_pass_distance).abs() < 0.001,
        "{scope} pass.longest_pass_distance: actual {:.3} expected {:.3}",
        actual.longest_pass_distance,
        expected.longest_pass_distance,
    );
    assert_eq!(
        actual.is_last_completed_pass, expected.is_last_completed_pass,
        "{scope} pass.is_last_completed_pass"
    );
    assert_eq!(
        actual.last_completed_pass_frame, expected.last_completed_pass_frame,
        "{scope} pass.last_completed_pass_frame"
    );
    assert!(
        match (
            actual.last_completed_pass_time,
            expected.last_completed_pass_time,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} pass.last_completed_pass_time: actual {:?} expected {:?}",
        actual.last_completed_pass_time,
        expected.last_completed_pass_time,
    );
    assert_eq!(
        actual.frames_since_last_completed_pass, expected.frames_since_last_completed_pass,
        "{scope} pass.frames_since_last_completed_pass"
    );
    assert!(
        match (
            actual.time_since_last_completed_pass,
            expected.time_since_last_completed_pass,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} pass.time_since_last_completed_pass: actual {:?} expected {:?}",
        actual.time_since_last_completed_pass,
        expected.time_since_last_completed_pass,
    );
}
