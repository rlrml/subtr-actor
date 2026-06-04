#[test]
#[ignore = "replay-backed timeline fixture test is slow; run explicitly when changing timeline collection"]
fn test_stats_timeline_collector_frames_are_sorted_and_cumulative() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
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
            .events
            .timeline
            .windows(2)
            .all(|events| events[1].time >= events[0].time),
        "Expected emitted timeline events to be time sorted"
    );
    assert_boost_pickup_nominal_amounts_consistent(&timeline);
}

#[test]
#[ignore = "replay-backed timeline serialization test is slow; run explicitly when changing timeline export"]
fn test_stats_timeline_value_serializes_for_rlcs_replay() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let captured = StatsCollector::new()
        .capture_frames()
        .get_captured_data(&replay)
        .expect("Expected captured stats data");

    captured
        .into_legacy_stats_timeline_value()
        .expect("Expected stats timeline value");
}

#[test]
#[ignore = "replay-backed timeline fixture test is slow; run explicitly when changing timeline collection"]
fn test_stats_timeline_excludes_post_goal_reset_frames_from_cumulative_stats() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let replay_data = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .expect("Expected replay data");
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    let first_goal = replay_data
        .goal_events
        .first()
        .expect("Expected at least one goal event");
    let kickoff_countdown_start = replay_data
        .frame_data
        .metadata_frames
        .iter()
        .skip(first_goal.frame + 1)
        .find(|metadata| metadata.replicated_game_state_time_remaining > 0)
        .map(|metadata| metadata.time)
        .expect("Expected a kickoff countdown after the first goal");
    let post_goal_frames: Vec<_> = timeline
        .frames
        .iter()
        .filter(|frame| frame.time >= first_goal.time && frame.time < kickoff_countdown_start)
        .collect();

    assert!(
        post_goal_frames.len() > 1,
        "Expected multiple frames between the goal and the next kickoff countdown"
    );
    assert!(
        post_goal_frames.iter().all(|frame| !frame.is_live_play),
        "Expected all post-goal reset frames to be inactive"
    );

    let first_post_goal_frame = post_goal_frames
        .first()
        .expect("Expected a first post-goal frame");
    let last_post_goal_frame = post_goal_frames
        .last()
        .expect("Expected a last post-goal frame");

    assert_eq!(
        frame_total_goals(first_post_goal_frame),
        frame_total_goals(last_post_goal_frame),
        "Expected the score to stay fixed throughout the post-goal reset"
    );
    assert_eq!(
        last_post_goal_frame.team_zero.possession,
        first_post_goal_frame.team_zero.possession
    );
    assert_eq!(
        last_post_goal_frame.team_one.possession,
        first_post_goal_frame.team_one.possession
    );
    assert_eq!(
        normalized_team_stats_for_live_play_comparison(&last_post_goal_frame.team_zero),
        normalized_team_stats_for_live_play_comparison(&first_post_goal_frame.team_zero),
    );
    assert_eq!(
        normalized_team_stats_for_live_play_comparison(&last_post_goal_frame.team_one),
        normalized_team_stats_for_live_play_comparison(&first_post_goal_frame.team_one),
    );
    let normalized_last_players: Vec<_> = last_post_goal_frame
        .players
        .iter()
        .map(normalized_player_stats_for_live_play_comparison)
        .collect();
    let normalized_first_players: Vec<_> = first_post_goal_frame
        .players
        .iter()
        .map(normalized_player_stats_for_live_play_comparison)
        .collect();
    assert_eq!(normalized_last_players, normalized_first_players);
}

#[test]
#[ignore = "replay-backed timeline fixture test is slow; run explicitly when changing player discovery"]
fn test_stats_timeline_old_replay_with_substitutions_discovers_late_players() {
    let replay = parse_replay("assets/old-ballchasing-midfield-car.replay");
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");
    let final_frame = timeline.frames.last().expect("Expected timeline frames");
    let names = player_names(final_frame);

    for expected_name in [
        "CritRomney",
        "DatLilBabyG",
        "b_corner",
        "Raptor_Attacks_",
        "jboy42069",
        "remrocker29",
        "a093q262",
        "Q-money219",
    ] {
        assert!(
            names.contains(expected_name),
            "Expected final stats timeline frame to include late player {expected_name}, got {names:?}"
        );
    }
}

#[test]
#[ignore = "replay-backed timeline fixture test is slow; run explicitly when changing boost accounting"]
fn test_stats_timeline_boost_monotonic_dodges_replay() {
    let replay =
        parse_replay("assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay");
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
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
        ("amount_used_while_grounded", |b| {
            b.amount_used_while_grounded as f64
        }),
        ("amount_used_while_airborne", |b| {
            b.amount_used_while_airborne as f64
        }),
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
        .events
        .timeline
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
#[ignore = "replay-backed timeline fixture test is slow; run explicitly when changing dodge-reset touch attribution"]
fn test_stats_timeline_awards_touch_for_on_ball_reset_in_dodges_replay() {
    let replay =
        parse_replay("assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay");
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    let pre_reset_frame = timeline
        .frame_by_number(2114)
        .expect("Expected pre-reset frame in timeline");
    let reset_frame = timeline
        .frame_by_number(2117)
        .expect("Expected dodge-reset frame in timeline");
    let post_reset_window = (2115..=2118)
        .map(|frame_number| {
            timeline
                .frame_by_number(frame_number)
                .unwrap_or_else(|| panic!("Expected frame {frame_number} in timeline"))
        })
        .collect::<Vec<_>>();

    let pre_reset_player = player_snapshot_by_name(pre_reset_frame, "rayman ty");
    let reset_player = player_snapshot_by_name(reset_frame, "rayman ty");
    let max_touch_count_in_window = post_reset_window
        .iter()
        .map(|frame| {
            player_snapshot_by_name(frame, "rayman ty")
                .touch
                .touch_count
        })
        .max()
        .expect("Expected non-empty post-reset window");

    assert_eq!(
        reset_player.dodge_reset.on_ball_count,
        pre_reset_player.dodge_reset.on_ball_count + 1,
        "Expected rayman ty to get an on-ball reset by frame 2117"
    );
    assert!(
        max_touch_count_in_window > pre_reset_player.touch.touch_count,
        "Expected rayman ty's touch count to increase within frames 2115..=2118 after the on-ball reset, but it stayed at {}",
        pre_reset_player.touch.touch_count
    );
}

#[test]
#[ignore = "replay-backed timeline fixture test is slow; run explicitly when changing kickoff touch attribution"]
fn test_stats_timeline_first_kickoff_credits_both_players() {
    let replay =
        parse_replay("assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay");
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    let baseline_frame = timeline
        .frame_by_number(170)
        .expect("Expected baseline kickoff frame in timeline");
    let kickoff_resolution_frame = timeline
        .frame_by_number(205)
        .expect("Expected kickoff-resolution frame in timeline");

    let baseline_orange = player_snapshot_by_name(baseline_frame, "tykop");
    let baseline_blue = player_snapshot_by_name(baseline_frame, "mrtyzz.");
    let kickoff_resolution_orange = player_snapshot_by_name(kickoff_resolution_frame, "tykop");
    let kickoff_resolution_blue = player_snapshot_by_name(kickoff_resolution_frame, "mrtyzz.");

    assert!(
        kickoff_resolution_orange.touch.touch_count > baseline_orange.touch.touch_count,
        "Expected tykop to receive a credited touch during the first kickoff sequence by frame 205"
    );
    assert!(
        kickoff_resolution_blue.touch.touch_count > baseline_blue.touch.touch_count,
        "Expected mrtyzz. to receive a credited touch during the first kickoff sequence by frame 205"
    );
}
