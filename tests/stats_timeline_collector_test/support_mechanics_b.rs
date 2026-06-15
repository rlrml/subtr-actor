fn assert_pass_team_stats_match(scope: &str, actual: &PassTeamStats, expected: &PassTeamStats) {
    assert_eq!(
        actual.completed_pass_count, expected.completed_pass_count,
        "{scope} pass.completed_pass_count"
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
}

fn assert_pass_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline_payloads_by_stream(timeline, "pass", |payload| match payload { EventPayload::Pass(event) => Some(event), _ => None });
    events.sort_by(|left, right| {
        left.sample_frame
            .cmp(&right.sample_frame)
            .then_with(|| left.sample_time.total_cmp(&right.sample_time))
    });
    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DerivedPassPlayerStats> = HashMap::new();
    let mut team_zero = PassTeamStats::default();
    let mut team_one = PassTeamStats::default();
    let mut last_completed_pass_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        for (player_id, stats) in players.iter_mut() {
            stats.advance_frame(
                frame,
                frame.is_live_play && last_completed_pass_player.as_ref() == Some(player_id),
            );
        }

        if !frame.is_live_play {
            last_completed_pass_player = None;
        } else {
            let mut processed_event = false;
            while event_index < events.len()
                && events[event_index].sample_frame <= frame.frame_number
            {
                let event = &events[event_index];
                players
                    .entry(event.passer.clone())
                    .or_default()
                    .record_completed_pass(event, frame);
                players
                    .entry(event.receiver.clone())
                    .or_default()
                    .record_received_pass();

                let team_stats = if event.is_team_0 {
                    &mut team_zero
                } else {
                    &mut team_one
                };
                team_stats.completed_pass_count += 1;
                team_stats.total_pass_distance += event.ball_travel_distance;
                team_stats.total_pass_advance += event.ball_advance_distance;
                team_stats.longest_pass_distance = team_stats
                    .longest_pass_distance
                    .max(event.ball_travel_distance);

                last_completed_pass_player = Some(event.passer.clone());
                event_index += 1;
                processed_event = true;
            }

            if processed_event {
                for stats in players.values_mut() {
                    stats.stats.is_last_completed_pass = false;
                }
            }

            if let Some(player_id) = last_completed_pass_player.as_ref() {
                if let Some(stats) = players.get_mut(player_id) {
                    stats.stats.is_last_completed_pass = true;
                }
            }
        }

        assert_pass_team_stats_match(
            &format!("{replay_path} team_zero frame {}", frame.frame_number),
            &frame.team_zero.pass,
            &team_zero,
        );
        assert_pass_team_stats_match(
            &format!("{replay_path} team_one frame {}", frame.frame_number),
            &frame.team_one.pass,
            &team_one,
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_pass_derived_player_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.pass,
                &expected.stats,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed pass events"
    );
}

fn apply_rush_event(stats: &mut RushTeamStats, event: &RushEvent) {
    stats.count += 1;
    match (event.attackers, event.defenders) {
        (2, 1) => stats.two_v_one_count += 1,
        (2, 2) => stats.two_v_two_count += 1,
        (2, 3) => stats.two_v_three_count += 1,
        (3, 1) => stats.three_v_one_count += 1,
        (3, 2) => stats.three_v_two_count += 1,
        (3, 3) => stats.three_v_three_count += 1,
        _ => {}
    }
}

fn assert_rush_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline_payloads_by_stream(timeline, "rush", |payload| match payload { EventPayload::Rush(event) => Some(event), _ => None });
    events.sort_by(|left, right| {
        left.start_frame
            .cmp(&right.start_frame)
            .then_with(|| left.start_time.total_cmp(&right.start_time))
            .then_with(|| left.end_frame.cmp(&right.end_frame))
    });

    let mut event_index = 0;
    let mut team_zero = RushTeamStats::default();
    let mut team_one = RushTeamStats::default();
    let min_retained_seconds = timeline.config.rush_min_possession_retained_seconds;

    for frame in &timeline.frames {
        while event_index < events.len()
            && frame.frame_number >= events[event_index].start_frame
            && frame.time - events[event_index].start_time >= min_retained_seconds
        {
            let event = &events[event_index];
            apply_rush_event(
                if event.is_team_0 {
                    &mut team_zero
                } else {
                    &mut team_one
                },
                event,
            );
            event_index += 1;
        }

        assert_eq!(
            frame.team_zero.rush, team_zero,
            "{replay_path} team_zero rush frame {}",
            frame.frame_number,
        );
        assert_eq!(
            frame.team_one.rush, team_one,
            "{replay_path} team_one rush frame {}",
            frame.frame_number,
        );
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed rush events"
    );
}

#[derive(Clone, Default)]
struct DerivedBumpPlayerStats {
    stats: BumpPlayerStats,
}

impl DerivedBumpPlayerStats {
    fn record_inflicted(&mut self, event: &BumpEvent) {
        self.stats.bumps_inflicted += 1;
        if event.is_team_bump {
            self.stats.team_bumps_inflicted += 1;
        }
        self.stats.last_bump_time = Some(event.time);
        self.stats.last_bump_frame = Some(event.frame);
        self.stats.last_bump_strength = Some(event.strength);
        self.stats.max_bump_strength = self.stats.max_bump_strength.max(event.strength);
        self.stats.cumulative_bump_strength += event.strength;
    }

    fn record_taken(&mut self, event: &BumpEvent) {
        self.stats.bumps_taken += 1;
        if event.is_team_bump {
            self.stats.team_bumps_taken += 1;
        }
    }
}

fn apply_bump_team_event(stats: &mut BumpTeamStats, event: &BumpEvent) {
    stats.bumps_inflicted += 1;
    if event.is_team_bump {
        stats.team_bumps_inflicted += 1;
    }
}

fn assert_bump_player_stats_match(
    scope: &str,
    actual: &BumpPlayerStats,
    expected: &BumpPlayerStats,
) {
    assert_eq!(
        actual.bumps_inflicted, expected.bumps_inflicted,
        "{scope} bump.bumps_inflicted"
    );
    assert_eq!(
        actual.bumps_taken, expected.bumps_taken,
        "{scope} bump.bumps_taken"
    );
    assert_eq!(
        actual.team_bumps_inflicted, expected.team_bumps_inflicted,
        "{scope} bump.team_bumps_inflicted"
    );
    assert_eq!(
        actual.team_bumps_taken, expected.team_bumps_taken,
        "{scope} bump.team_bumps_taken"
    );
    assert_eq!(
        actual.last_bump_frame, expected.last_bump_frame,
        "{scope} bump.last_bump_frame"
    );
    assert!(
        match (actual.last_bump_time, expected.last_bump_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} bump.last_bump_time: actual {:?} expected {:?}",
        actual.last_bump_time,
        expected.last_bump_time,
    );
    assert!(
        match (actual.last_bump_strength, expected.last_bump_strength) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} bump.last_bump_strength: actual {:?} expected {:?}",
        actual.last_bump_strength,
        expected.last_bump_strength,
    );
    assert!(
        (actual.max_bump_strength - expected.max_bump_strength).abs() < 0.001,
        "{scope} bump.max_bump_strength: actual {:.3} expected {:.3}",
        actual.max_bump_strength,
        expected.max_bump_strength,
    );
    assert!(
        (actual.cumulative_bump_strength - expected.cumulative_bump_strength).abs() < 0.001,
        "{scope} bump.cumulative_bump_strength: actual {:.3} expected {:.3}",
        actual.cumulative_bump_strength,
        expected.cumulative_bump_strength,
    );
}

fn assert_bump_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline_payloads_by_stream(timeline, "bump", |payload| match payload { EventPayload::Bump(event) => Some(event), _ => None });
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DerivedBumpPlayerStats> = HashMap::new();
    let mut team_zero = BumpTeamStats::default();
    let mut team_one = BumpTeamStats::default();

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].frame <= frame.frame_number {
            let event = &events[event_index];
            players
                .entry(event.initiator.clone())
                .or_default()
                .record_inflicted(event);
            players
                .entry(event.victim.clone())
                .or_default()
                .record_taken(event);
            apply_bump_team_event(
                if event.initiator_is_team_0 {
                    &mut team_zero
                } else {
                    &mut team_one
                },
                event,
            );
            event_index += 1;
        }

        assert_eq!(
            frame.team_zero.bump, team_zero,
            "{replay_path} team_zero bump frame {}",
            frame.frame_number,
        );
        assert_eq!(
            frame.team_one.bump, team_one,
            "{replay_path} team_one bump frame {}",
            frame.frame_number,
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_bump_player_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.bump,
                &expected.stats,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed bump events"
    );
}

fn assert_demo_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events: Vec<_> =
        timeline_payloads_by_stream(timeline, "demolition", |payload| match payload {
            EventPayload::Demolition(event) => Some(event),
            _ => None,
        })
        .into_iter()
        .collect();
    events.sort_by(|left, right| left.time.total_cmp(&right.time));

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DemoPlayerStats> = HashMap::new();
    let mut team_zero = DemoTeamStats::default();
    let mut team_one = DemoTeamStats::default();

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].time <= frame.time {
            let event = &events[event_index];
            players
                .entry(event.attacker.clone())
                .or_default()
                .demos_inflicted += 1;
            match event.attacker_is_team_0 {
                Some(true) => team_zero.demos_inflicted += 1,
                Some(false) => team_one.demos_inflicted += 1,
                None => {}
            }
            players
                .entry(event.victim.clone())
                .or_default()
                .demos_taken += 1;
            event_index += 1;
        }

        assert_eq!(
            frame.team_zero.demo, team_zero,
            "{replay_path} team_zero demo frame {}",
            frame.frame_number,
        );
        assert_eq!(
            frame.team_one.demo, team_one,
            "{replay_path} team_one demo frame {}",
            frame.frame_number,
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_eq!(
                player.demo, expected,
                "{replay_path} player {} demo frame {}",
                player.name, frame.frame_number,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed demo events"
    );
}
