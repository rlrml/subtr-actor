use subtr_actor::*;

fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).unwrap_or_else(|_| panic!("Failed to read replay file: {path}"));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay: {path}"))
}

/// Check that a cumulative stat field never decreases between consecutive frames
/// for any player in the timeline.
fn assert_player_boost_field_monotonic(
    timeline: &ReplayStatsTimeline,
    field_name: &str,
    getter: fn(&BoostStats) -> f64,
) {
    for window in timeline.frames.windows(2) {
        let prev = &window[0];
        let curr = &window[1];
        for prev_player in &prev.players {
            let Some(curr_player) = curr
                .players
                .iter()
                .find(|p| p.player_id == prev_player.player_id)
            else {
                continue;
            };
            let prev_val = getter(&prev_player.boost);
            let curr_val = getter(&curr_player.boost);
            assert!(
                curr_val >= prev_val - 1e-4,
                "Player {} {field_name} decreased from {prev_val:.4} to {curr_val:.4} \
                 between frames {} (t={:.3}) and {} (t={:.3})",
                prev_player.name,
                prev.frame_number,
                prev.time,
                curr.frame_number,
                curr.time,
            );
        }
    }
}

/// Check that amount_collected_big + amount_collected_small ≈ amount_collected
/// for every player on every frame.
fn assert_boost_bucket_sums_consistent(timeline: &ReplayStatsTimeline) {
    for frame in &timeline.frames {
        for player in &frame.players {
            let bucket_sum =
                player.boost.amount_collected_big + player.boost.amount_collected_small;
            let diff = (bucket_sum - player.boost.amount_collected).abs();
            assert!(
                diff < 1.0,
                "Player {} bucket mismatch at frame {} (t={:.3}): \
                 big({:.1}) + small({:.1}) = {:.1} vs amount_collected({:.1}), diff={:.1}",
                player.name,
                frame.frame_number,
                frame.time,
                player.boost.amount_collected_big,
                player.boost.amount_collected_small,
                bucket_sum,
                player.boost.amount_collected,
                diff,
            );
        }
    }
}

/// Check that the boost accounting identity holds on every frame:
/// amount_used = max(0, amount_obtained - current_boost), so the
/// implied current boost = amount_obtained - amount_used must be in
/// [0, 255].  If a boost source was missed (e.g. a kickoff respawn),
/// amount_obtained would be too low and current_boost would go negative.
fn assert_boost_accounting_consistent(timeline: &ReplayStatsTimeline) {
    for frame in &timeline.frames {
        for player in &frame.players {
            let obtained = player.boost.amount_obtained();
            let implied_current = obtained - player.boost.amount_used;
            assert!(
                implied_current >= -1.0,
                "Player {} has negative implied boost {:.1} at frame {} (t={:.3}): \
                 obtained({:.1}) - used({:.1}) = {:.1}  [missing boost source?]",
                player.name,
                implied_current,
                frame.frame_number,
                frame.time,
                obtained,
                player.boost.amount_used,
                implied_current,
            );
            assert!(
                implied_current <= 256.0,
                "Player {} has impossible implied boost {:.1} at frame {} (t={:.3}): \
                 obtained({:.1}) - used({:.1}) = {:.1}  [over-counted boost source?]",
                player.name,
                implied_current,
                frame.frame_number,
                frame.time,
                obtained,
                player.boost.amount_used,
                implied_current,
            );
        }
    }
}

/// Check that pad counts imply the same nominal boost total as
/// collected boost plus tracked overfill.
fn assert_boost_pickup_nominal_amounts_consistent(timeline: &ReplayStatsTimeline) {
    fn assert_stats(scope: &str, frame_number: usize, time: f32, stats: &BoostStats) {
        let violations = boost_invariant_violations(stats, None);
        assert!(
            violations.is_empty(),
            "{scope} boost invariant violations at frame {frame_number} (t={time:.3}): {:?}",
            violations
        );
    }

    for frame in &timeline.frames {
        assert_stats(
            "team_zero",
            frame.frame_number,
            frame.time,
            &frame.team_zero.boost,
        );
        assert_stats(
            "team_one",
            frame.frame_number,
            frame.time,
            &frame.team_one.boost,
        );
        for player in &frame.players {
            assert_stats(
                &format!("player {}", player.name),
                frame.frame_number,
                frame.time,
                &player.boost,
            );
        }
    }
}

/// Check that amount_respawned is within reasonable bounds.
/// Each kickoff/demo grants ~85 raw.  A 7-min game with 15 kickoffs + 10 demos ≈ 2125.
fn assert_boost_respawns_reasonable(timeline: &ReplayStatsTimeline, max_raw: f32) {
    let last_frame = timeline.frames.last().expect("non-empty frames");
    for player in &last_frame.players {
        assert!(
            player.boost.amount_respawned <= max_raw,
            "Player {} has unreasonable amount_respawned: {:.1} (max expected {max_raw:.0})",
            player.name,
            player.boost.amount_respawned,
        );
    }
}

/// Dump final boost stats for every player (diagnostics).
fn dump_final_boost_stats(timeline: &ReplayStatsTimeline) {
    let last_frame = timeline.frames.last().expect("non-empty frames");
    for p in &last_frame.players {
        eprintln!(
            "FINAL {} | collected:{:.0} big_amt:{:.0} small_amt:{:.0} \
             respawn:{:.0} used:{:.0} overfill:{:.0} | \
             big:{} small:{} stolen_big:{} stolen_small:{}",
            p.name,
            p.boost.amount_collected,
            p.boost.amount_collected_big,
            p.boost.amount_collected_small,
            p.boost.amount_respawned,
            p.boost.amount_used,
            p.boost.overfill_total,
            p.boost.big_pads_collected,
            p.boost.small_pads_collected,
            p.boost.big_pads_stolen,
            p.boost.small_pads_stolen,
        );
    }
}

fn find_field<'a>(fields: &'a [ExportedStat], domain: &str, name: &str) -> &'a ExportedStat {
    fields
        .iter()
        .find(|field| field.descriptor.domain == domain && field.descriptor.name == name)
        .unwrap_or_else(|| panic!("Missing field {domain}.{name}"))
}

#[test]
fn test_stats_timeline_frame_lookup_uses_frame_number() {
    let timeline = ReplayStatsTimeline {
        config: StatsTimelineConfig {
            most_back_forward_threshold_y: PositioningReducerConfig::default()
                .most_back_forward_threshold_y,
        },
        replay_meta: ReplayMeta {
            team_zero: Vec::new(),
            team_one: Vec::new(),
            all_headers: Vec::new(),
        },
        timeline_events: Vec::new(),
        frames: vec![
            ReplayStatsFrame {
                frame_number: 10,
                time: 0.0,
                dt: 0.0,
                seconds_remaining: None,
                game_state: None,
                is_live_play: true,
                possession: PossessionStats::default(),
                team_zero: TeamStatsSnapshot {
                    core: CoreTeamStats::default(),
                    ball_carry: BallCarryStats::default(),
                    boost: BoostStats::default(),
                    movement: MovementStats::default(),
                    powerslide: PowerslideStats::default(),
                    demo: DemoTeamStats::default(),
                },
                team_one: TeamStatsSnapshot {
                    core: CoreTeamStats::default(),
                    ball_carry: BallCarryStats::default(),
                    boost: BoostStats::default(),
                    movement: MovementStats::default(),
                    powerslide: PowerslideStats::default(),
                    demo: DemoTeamStats::default(),
                },
                players: Vec::new(),
            },
            ReplayStatsFrame {
                frame_number: 11,
                time: 0.1,
                dt: 0.1,
                seconds_remaining: None,
                game_state: None,
                is_live_play: true,
                possession: PossessionStats::default(),
                team_zero: TeamStatsSnapshot {
                    core: CoreTeamStats::default(),
                    ball_carry: BallCarryStats::default(),
                    boost: BoostStats::default(),
                    movement: MovementStats::default(),
                    powerslide: PowerslideStats::default(),
                    demo: DemoTeamStats::default(),
                },
                team_one: TeamStatsSnapshot {
                    core: CoreTeamStats::default(),
                    ball_carry: BallCarryStats::default(),
                    boost: BoostStats::default(),
                    movement: MovementStats::default(),
                    powerslide: PowerslideStats::default(),
                    demo: DemoTeamStats::default(),
                },
                players: Vec::new(),
            },
            ReplayStatsFrame {
                frame_number: 15,
                time: 0.2,
                dt: 0.1,
                seconds_remaining: None,
                game_state: None,
                is_live_play: true,
                possession: PossessionStats::default(),
                team_zero: TeamStatsSnapshot {
                    core: CoreTeamStats::default(),
                    ball_carry: BallCarryStats::default(),
                    boost: BoostStats::default(),
                    movement: MovementStats::default(),
                    powerslide: PowerslideStats::default(),
                    demo: DemoTeamStats::default(),
                },
                team_one: TeamStatsSnapshot {
                    core: CoreTeamStats::default(),
                    ball_carry: BallCarryStats::default(),
                    boost: BoostStats::default(),
                    movement: MovementStats::default(),
                    powerslide: PowerslideStats::default(),
                    demo: DemoTeamStats::default(),
                },
                players: Vec::new(),
            },
        ],
    };

    assert_eq!(timeline.frames[2].frame_number, 15);
    assert_eq!(timeline.frame_by_number(2), None);
    assert_eq!(
        timeline
            .frame_by_number(15)
            .expect("Expected frame lookup by frame number")
            .frame_number,
        15
    );
}

#[test]
fn test_stats_timeline_collector_final_frame_matches_reducers() {
    let replay = parse_replay("assets/replays/rlcs.replay");
    let timeline = StatsTimelineCollector::new()
        .get_replay_data(&replay)
        .expect("Expected stats timeline data");
    let final_frame = timeline.frames.last().expect("Expected at least one frame");

    let mut possession_collector = ReducerCollector::new(PossessionReducer::new());
    let mut match_collector = ReducerCollector::new(MatchStatsReducer::new());
    let mut ball_carry_collector = ReducerCollector::new(BallCarryReducer::new());
    let mut boost_collector = ReducerCollector::new(BoostReducer::new());
    let mut movement_collector = ReducerCollector::new(MovementReducer::new());
    let mut positioning_collector = ReducerCollector::new(PositioningReducer::new());
    let mut powerslide_collector = ReducerCollector::new(PowerslideReducer::new());
    let mut demo_collector = ReducerCollector::new(DemoReducer::new());

    let mut processor = ReplayProcessor::new(&replay).expect("Expected replay processor");
    let mut collectors: [&mut dyn Collector; 8] = [
        &mut possession_collector,
        &mut match_collector,
        &mut ball_carry_collector,
        &mut boost_collector,
        &mut movement_collector,
        &mut positioning_collector,
        &mut powerslide_collector,
        &mut demo_collector,
    ];
    processor
        .process_all(&mut collectors)
        .expect("Expected reducers to process replay");

    let possession = possession_collector.into_inner();
    let match_stats = match_collector.into_inner();
    let ball_carry = ball_carry_collector.into_inner();
    let boost = boost_collector.into_inner();
    let movement = movement_collector.into_inner();
    let positioning = positioning_collector.into_inner();
    let powerslide = powerslide_collector.into_inner();
    let demo = demo_collector.into_inner();

    assert_eq!(final_frame.possession, possession.stats().clone());
    assert_eq!(final_frame.team_zero.core, match_stats.team_zero_stats());
    assert_eq!(final_frame.team_one.core, match_stats.team_one_stats());
    assert_eq!(
        final_frame.team_zero.ball_carry,
        ball_carry.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.ball_carry,
        ball_carry.team_one_stats().clone()
    );
    assert_eq!(final_frame.team_zero.boost, boost.team_zero_stats().clone());
    assert_eq!(final_frame.team_one.boost, boost.team_one_stats().clone());
    assert_eq!(
        final_frame.team_zero.movement,
        movement.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.movement,
        movement.team_one_stats().clone()
    );
    assert_eq!(
        final_frame.team_zero.powerslide,
        powerslide.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.powerslide,
        powerslide.team_one_stats().clone()
    );
    assert_eq!(final_frame.team_zero.demo, demo.team_zero_stats().clone());
    assert_eq!(final_frame.team_one.demo, demo.team_one_stats().clone());

    assert_eq!(
        final_frame.players.len(),
        timeline.replay_meta.player_count()
    );
    for player in &final_frame.players {
        assert_eq!(
            player.core,
            match_stats
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.ball_carry,
            ball_carry
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.boost,
            boost
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.movement,
            movement
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.positioning,
            positioning
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.powerslide,
            powerslide
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.demo,
            demo.player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
    }
}

#[test]
fn test_stats_timeline_collector_frames_are_sorted_and_cumulative() {
    let replay = parse_replay("assets/replays/rlcs.replay");
    let timeline = StatsTimelineCollector::new()
        .get_replay_data(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.frames.is_empty(),
        "Expected stats timeline frames"
    );
    assert!(
        timeline
            .frames
            .windows(2)
            .all(|frames| frames[1].time >= frames[0].time),
        "Expected frame times to be nondecreasing"
    );
    assert!(
        timeline
            .frames
            .windows(2)
            .all(|frames| frames[1].team_zero.core.goals >= frames[0].team_zero.core.goals),
        "Expected team-zero goals to accumulate monotonically"
    );
    assert!(
        timeline.frames.windows(2).all(|frames| {
            frames[1].team_zero.boost.amount_collected >= frames[0].team_zero.boost.amount_collected
        }),
        "Expected collected boost to accumulate monotonically"
    );
    assert!(
        timeline
            .timeline_events
            .windows(2)
            .all(|events| events[1].time >= events[0].time),
        "Expected emitted timeline events to be time sorted"
    );
    assert_boost_pickup_nominal_amounts_consistent(&timeline);
}

#[test]
fn test_stats_timeline_boost_monotonic_dodges_replay() {
    let replay = parse_replay("assets/replays/dodges_refreshed_counter.replay");
    let timeline = StatsTimelineCollector::new()
        .get_replay_data(&replay)
        .expect("Expected stats timeline data");

    dump_final_boost_stats(&timeline);

    // Invariant 1: All cumulative boost stats must be monotonically non-decreasing
    type BoostStatGetter = fn(&BoostStats) -> f64;
    let monotonic_checks: &[(&str, BoostStatGetter)] = &[
        ("amount_collected", |b| b.amount_collected as f64),
        ("amount_collected_big", |b| b.amount_collected_big as f64),
        ("amount_collected_small", |b| {
            b.amount_collected_small as f64
        }),
        ("amount_respawned", |b| b.amount_respawned as f64),
        ("amount_stolen", |b| b.amount_stolen as f64),
        ("amount_stolen_big", |b| b.amount_stolen_big as f64),
        ("amount_stolen_small", |b| b.amount_stolen_small as f64),
        ("overfill_total", |b| b.overfill_total as f64),
        ("big_pads_collected", |b| b.big_pads_collected as f64),
        ("small_pads_collected", |b| b.small_pads_collected as f64),
        ("big_pads_stolen", |b| b.big_pads_stolen as f64),
        ("small_pads_stolen", |b| b.small_pads_stolen as f64),
        // NOTE: amount_used is NOT monotonic because kickoff resets set
        // current_boost to 85 without it being "used", lowering amount_used.
    ];
    for (name, getter) in monotonic_checks {
        assert_player_boost_field_monotonic(&timeline, name, *getter);
    }

    // Invariant 2: Bucket sums consistent (every frame)
    assert_boost_bucket_sums_consistent(&timeline);

    // Invariant 3: Respawns reasonable
    assert_boost_respawns_reasonable(&timeline, 3000.0);

    // Invariant 4: Pad counts match collected boost plus overfill
    assert_boost_pickup_nominal_amounts_consistent(&timeline);

    // Invariant 5: Boost accounting — implied current boost in [0, 255]
    assert_boost_accounting_consistent(&timeline);

    // Invariant 6: Every player got at least one kickoff respawn
    let last_frame = timeline.frames.last().unwrap();
    for player in &last_frame.players {
        assert!(
            player.boost.amount_respawned >= BOOST_KICKOFF_START_AMOUNT - 1.0,
            "Player {} has amount_respawned={:.1}, expected at least one kickoff ({:.0})",
            player.name,
            player.boost.amount_respawned,
            BOOST_KICKOFF_START_AMOUNT,
        );
    }

    // Diagnostic: count goals to verify kickoff count
    let goal_count = timeline
        .timeline_events
        .iter()
        .filter(|e| matches!(e.kind, TimelineEventKind::Goal))
        .count();
    let expected_kickoffs = goal_count + 1; // +1 for initial kickoff
    eprintln!("Goals: {goal_count}, expected kickoffs: {expected_kickoffs}");
    // Each player should have expected_kickoffs * 85 in respawns
    let expected_respawn = expected_kickoffs as f32 * 85.0;
    eprintln!("Expected respawn per player: {expected_respawn:.0}");
    // Check first frame's game state
    if let Some(first) = timeline.frames.first() {
        eprintln!(
            "First frame: number={}, time={:.3}, game_state={:?}, is_live={}",
            first.frame_number, first.time, first.game_state, first.is_live_play
        );
    }
}

#[test]
fn test_stats_timeline_collector_can_export_dynamic_stats() {
    let replay = parse_replay("assets/replays/rlcs.replay");
    let typed_timeline = StatsTimelineCollector::new()
        .get_replay_data(&replay)
        .expect("Expected typed stats timeline data");
    let dynamic_timeline = StatsTimelineCollector::new()
        .get_dynamic_replay_data(&replay)
        .expect("Expected dynamic stats timeline data");

    let typed_frame = typed_timeline
        .frames
        .last()
        .expect("Expected typed final frame");
    let dynamic_frame = dynamic_timeline
        .frames
        .last()
        .expect("Expected dynamic final frame");

    assert_eq!(
        find_field(&dynamic_frame.possession, "possession", "team_zero_time").value,
        StatValue::Float(typed_frame.possession.team_zero_time)
    );
    assert_eq!(
        find_field(&dynamic_frame.team_zero.stats, "core", "goals").value,
        StatValue::Signed(typed_frame.team_zero.core.goals)
    );
    assert_eq!(
        find_field(&dynamic_frame.team_zero.stats, "ball_carry", "count").value,
        StatValue::Unsigned(typed_frame.team_zero.ball_carry.carry_count)
    );

    let typed_player = typed_frame
        .players
        .first()
        .expect("Expected at least one player");
    let dynamic_player = dynamic_frame
        .players
        .iter()
        .find(|player| player.player_id == typed_player.player_id)
        .expect("Expected matching dynamic player");

    assert_eq!(
        find_field(&dynamic_player.stats, "positioning", "percent_behind_ball").value,
        StatValue::Float(typed_player.positioning.behind_ball_pct())
    );
    assert_eq!(
        find_field(&dynamic_player.stats, "movement", "total_distance").value,
        StatValue::Float(typed_player.movement.total_distance)
    );
    assert_eq!(
        find_field(&dynamic_player.stats, "ball_carry", "count").value,
        StatValue::Unsigned(typed_player.ball_carry.carry_count)
    );
}
