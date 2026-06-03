fn assert_fifty_fifty_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.fifty_fifty.clone();
    events.sort_by(|left, right| {
        left.resolve_frame
            .cmp(&right.resolve_frame)
            .then_with(|| left.resolve_time.total_cmp(&right.resolve_time))
    });

    let mut event_index = 0;
    let mut team_zero = FiftyFiftyTeamStats::default();
    let mut team_one = FiftyFiftyTeamStats::default();
    let mut players: HashMap<PlayerId, FiftyFiftyPlayerStats> = HashMap::new();

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].resolve_frame <= frame.frame_number
        {
            let event = &events[event_index];
            apply_fifty_fifty_team_event(&mut team_zero, true, event);
            apply_fifty_fifty_team_event(&mut team_one, false, event);
            if let Some(player_id) = event.team_zero_player.as_ref() {
                apply_fifty_fifty_player_event(
                    players.entry(player_id.clone()).or_default(),
                    true,
                    event,
                );
            }
            if let Some(player_id) = event.team_one_player.as_ref() {
                apply_fifty_fifty_player_event(
                    players.entry(player_id.clone()).or_default(),
                    false,
                    event,
                );
            }
            event_index += 1;
        }

        assert_eq!(
            frame.team_zero.fifty_fifty, team_zero,
            "{replay_path} team_zero fifty_fifty frame {}",
            frame.frame_number,
        );
        assert_eq!(
            frame.team_one.fifty_fifty, team_one,
            "{replay_path} team_one fifty_fifty frame {}",
            frame.frame_number,
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_eq!(
                player.fifty_fifty, expected,
                "{replay_path} player {} fifty_fifty frame {}",
                player.name, frame.frame_number,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed fifty-fifty events"
    );
}

#[derive(Clone, Default)]
struct DerivedOneTimerPlayerStats {
    stats: OneTimerPlayerStats,
}

impl DerivedOneTimerPlayerStats {
    fn advance_frame(&mut self, frame: &ReplayStatsFrame, is_last_one_timer_player: bool) {
        self.stats.is_last_one_timer = is_last_one_timer_player;
        self.stats.time_since_last_one_timer = self
            .stats
            .last_one_timer_time
            .map(|time| (frame.time - time).max(0.0));
        self.stats.frames_since_last_one_timer = self
            .stats
            .last_one_timer_frame
            .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
    }

    fn record_event(&mut self, event: &OneTimerEvent, frame: &ReplayStatsFrame) {
        self.stats.count += 1;
        self.stats.total_ball_speed += event.ball_speed;
        self.stats.fastest_ball_speed = self.stats.fastest_ball_speed.max(event.ball_speed);
        self.stats.total_pass_distance += event.pass_travel_distance;
        self.stats.last_one_timer_time = Some(event.time);
        self.stats.last_one_timer_frame = Some(event.frame);
        self.stats.time_since_last_one_timer = Some((frame.time - event.time).max(0.0));
        self.stats.frames_since_last_one_timer =
            Some(frame.frame_number.saturating_sub(event.frame));
    }
}

fn assert_one_timer_derived_player_stats_match(
    scope: &str,
    actual: &OneTimerPlayerStats,
    expected: &OneTimerPlayerStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} one_timer.count");
    assert!(
        (actual.total_ball_speed - expected.total_ball_speed).abs() < 0.001,
        "{scope} one_timer.total_ball_speed: actual {:.3} expected {:.3}",
        actual.total_ball_speed,
        expected.total_ball_speed,
    );
    assert!(
        (actual.fastest_ball_speed - expected.fastest_ball_speed).abs() < 0.001,
        "{scope} one_timer.fastest_ball_speed: actual {:.3} expected {:.3}",
        actual.fastest_ball_speed,
        expected.fastest_ball_speed,
    );
    assert!(
        (actual.total_pass_distance - expected.total_pass_distance).abs() < 0.001,
        "{scope} one_timer.total_pass_distance: actual {:.3} expected {:.3}",
        actual.total_pass_distance,
        expected.total_pass_distance,
    );
    assert_eq!(
        actual.is_last_one_timer, expected.is_last_one_timer,
        "{scope} one_timer.is_last_one_timer"
    );
    assert_eq!(
        actual.last_one_timer_frame, expected.last_one_timer_frame,
        "{scope} one_timer.last_one_timer_frame"
    );
    assert!(
        match (actual.last_one_timer_time, expected.last_one_timer_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} one_timer.last_one_timer_time: actual {:?} expected {:?}",
        actual.last_one_timer_time,
        expected.last_one_timer_time,
    );
    assert_eq!(
        actual.frames_since_last_one_timer, expected.frames_since_last_one_timer,
        "{scope} one_timer.frames_since_last_one_timer"
    );
    assert!(
        match (
            actual.time_since_last_one_timer,
            expected.time_since_last_one_timer,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} one_timer.time_since_last_one_timer: actual {:?} expected {:?}",
        actual.time_since_last_one_timer,
        expected.time_since_last_one_timer,
    );
}

fn assert_one_timer_team_stats_match(
    scope: &str,
    actual: &OneTimerTeamStats,
    expected: &OneTimerTeamStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} one_timer.count");
    assert!(
        (actual.total_ball_speed - expected.total_ball_speed).abs() < 0.001,
        "{scope} one_timer.total_ball_speed: actual {:.3} expected {:.3}",
        actual.total_ball_speed,
        expected.total_ball_speed,
    );
    assert!(
        (actual.fastest_ball_speed - expected.fastest_ball_speed).abs() < 0.001,
        "{scope} one_timer.fastest_ball_speed: actual {:.3} expected {:.3}",
        actual.fastest_ball_speed,
        expected.fastest_ball_speed,
    );
}

fn assert_one_timer_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.one_timer.clone();
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DerivedOneTimerPlayerStats> = HashMap::new();
    let mut team_zero = OneTimerTeamStats::default();
    let mut team_one = OneTimerTeamStats::default();
    let mut last_one_timer_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        for (player_id, stats) in players.iter_mut() {
            stats.advance_frame(
                frame,
                frame.is_live_play && last_one_timer_player.as_ref() == Some(player_id),
            );
        }

        if !frame.is_live_play {
            last_one_timer_player = None;
        } else {
            let mut processed_event = false;
            while event_index < events.len() && events[event_index].frame <= frame.frame_number {
                let event = &events[event_index];
                players
                    .entry(event.player.clone())
                    .or_default()
                    .record_event(event, frame);
                let team_stats = if event.is_team_0 {
                    &mut team_zero
                } else {
                    &mut team_one
                };
                team_stats.count += 1;
                team_stats.total_ball_speed += event.ball_speed;
                team_stats.fastest_ball_speed = team_stats.fastest_ball_speed.max(event.ball_speed);
                last_one_timer_player = Some(event.player.clone());
                event_index += 1;
                processed_event = true;
            }

            if processed_event {
                for stats in players.values_mut() {
                    stats.stats.is_last_one_timer = false;
                }
            }

            if let Some(player_id) = last_one_timer_player.as_ref() {
                if let Some(stats) = players.get_mut(player_id) {
                    stats.stats.is_last_one_timer = true;
                }
            }
        }

        assert_one_timer_team_stats_match(
            &format!("{replay_path} team_zero frame {}", frame.frame_number),
            &frame.team_zero.one_timer,
            &team_zero,
        );
        assert_one_timer_team_stats_match(
            &format!("{replay_path} team_one frame {}", frame.frame_number),
            &frame.team_one.one_timer,
            &team_one,
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_one_timer_derived_player_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.one_timer,
                &expected.stats,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed one-timer events"
    );
}

#[derive(Clone, Default)]
struct DerivedHalfVolleyPlayerStats {
    stats: HalfVolleyPlayerStats,
}

impl DerivedHalfVolleyPlayerStats {
    fn advance_frame(&mut self, frame: &ReplayStatsFrame, is_last_half_volley_player: bool) {
        self.stats.is_last_half_volley = is_last_half_volley_player;
        self.stats.time_since_last_half_volley = self
            .stats
            .last_half_volley_time
            .map(|time| (frame.time - time).max(0.0));
        self.stats.frames_since_last_half_volley = self
            .stats
            .last_half_volley_frame
            .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
    }

    fn record_event(&mut self, event: &HalfVolleyEvent, frame: &ReplayStatsFrame) {
        self.stats.count += 1;
        self.stats.total_ball_speed += event.ball_speed;
        self.stats.fastest_ball_speed = self.stats.fastest_ball_speed.max(event.ball_speed);
        self.stats.last_half_volley_time = Some(event.time);
        self.stats.last_half_volley_frame = Some(event.frame);
        self.stats.time_since_last_half_volley = Some((frame.time - event.time).max(0.0));
        self.stats.frames_since_last_half_volley =
            Some(frame.frame_number.saturating_sub(event.frame));
    }
}

fn assert_half_volley_derived_player_stats_match(
    scope: &str,
    actual: &HalfVolleyPlayerStats,
    expected: &HalfVolleyPlayerStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} half_volley.count");
    assert!(
        (actual.total_ball_speed - expected.total_ball_speed).abs() < 0.001,
        "{scope} half_volley.total_ball_speed: actual {:.3} expected {:.3}",
        actual.total_ball_speed,
        expected.total_ball_speed,
    );
    assert!(
        (actual.fastest_ball_speed - expected.fastest_ball_speed).abs() < 0.001,
        "{scope} half_volley.fastest_ball_speed: actual {:.3} expected {:.3}",
        actual.fastest_ball_speed,
        expected.fastest_ball_speed,
    );
    assert_eq!(
        actual.is_last_half_volley, expected.is_last_half_volley,
        "{scope} half_volley.is_last_half_volley"
    );
    assert_eq!(
        actual.last_half_volley_frame, expected.last_half_volley_frame,
        "{scope} half_volley.last_half_volley_frame"
    );
    assert!(
        match (actual.last_half_volley_time, expected.last_half_volley_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} half_volley.last_half_volley_time: actual {:?} expected {:?}",
        actual.last_half_volley_time,
        expected.last_half_volley_time,
    );
    assert_eq!(
        actual.frames_since_last_half_volley, expected.frames_since_last_half_volley,
        "{scope} half_volley.frames_since_last_half_volley"
    );
    assert!(
        match (
            actual.time_since_last_half_volley,
            expected.time_since_last_half_volley,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} half_volley.time_since_last_half_volley: actual {:?} expected {:?}",
        actual.time_since_last_half_volley,
        expected.time_since_last_half_volley,
    );
}

fn assert_half_volley_team_stats_match(
    scope: &str,
    actual: &HalfVolleyTeamStats,
    expected: &HalfVolleyTeamStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} half_volley.count");
    assert!(
        (actual.total_ball_speed - expected.total_ball_speed).abs() < 0.001,
        "{scope} half_volley.total_ball_speed: actual {:.3} expected {:.3}",
        actual.total_ball_speed,
        expected.total_ball_speed,
    );
    assert!(
        (actual.fastest_ball_speed - expected.fastest_ball_speed).abs() < 0.001,
        "{scope} half_volley.fastest_ball_speed: actual {:.3} expected {:.3}",
        actual.fastest_ball_speed,
        expected.fastest_ball_speed,
    );
}

fn assert_half_volley_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.half_volley.clone();
    events.sort_by(|left, right| {
        left.sample_frame
            .cmp(&right.sample_frame)
            .then_with(|| left.sample_time.total_cmp(&right.sample_time))
            .then_with(|| left.frame.cmp(&right.frame))
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DerivedHalfVolleyPlayerStats> = HashMap::new();
    let mut team_zero = HalfVolleyTeamStats::default();
    let mut team_one = HalfVolleyTeamStats::default();
    let mut last_half_volley_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        for (player_id, stats) in players.iter_mut() {
            stats.advance_frame(
                frame,
                frame.is_live_play && last_half_volley_player.as_ref() == Some(player_id),
            );
        }

        if !frame.is_live_play {
            last_half_volley_player = None;
        } else {
            let mut processed_event = false;
            while event_index < events.len()
                && events[event_index].sample_frame <= frame.frame_number
            {
                let event = &events[event_index];
                players
                    .entry(event.player.clone())
                    .or_default()
                    .record_event(event, frame);
                let team_stats = if event.is_team_0 {
                    &mut team_zero
                } else {
                    &mut team_one
                };
                team_stats.count += 1;
                team_stats.total_ball_speed += event.ball_speed;
                team_stats.fastest_ball_speed = team_stats.fastest_ball_speed.max(event.ball_speed);
                last_half_volley_player = Some(event.player.clone());
                event_index += 1;
                processed_event = true;
            }

            if processed_event {
                for stats in players.values_mut() {
                    stats.stats.is_last_half_volley = false;
                }
            }

            if let Some(player_id) = last_half_volley_player.as_ref() {
                if let Some(stats) = players.get_mut(player_id) {
                    stats.stats.is_last_half_volley = true;
                }
            }
        }

        assert_half_volley_team_stats_match(
            &format!("{replay_path} team_zero frame {}", frame.frame_number),
            &frame.team_zero.half_volley,
            &team_zero,
        );
        assert_half_volley_team_stats_match(
            &format!("{replay_path} team_one frame {}", frame.frame_number),
            &frame.team_one.half_volley,
            &team_one,
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_half_volley_derived_player_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.half_volley,
                &expected.stats,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed half-volley events"
    );
}

