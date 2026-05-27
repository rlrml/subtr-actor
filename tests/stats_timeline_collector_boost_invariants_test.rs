mod common;

use common::parse_replay;
use subtr_actor::*;

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
    fn assert_stats(
        scope: &str,
        frame_number: usize,
        time: f32,
        stats: &BoostStats,
        is_live_play: bool,
    ) {
        let violations = boost_invariant_violations(stats, None);
        let violations = if is_live_play {
            violations
        } else {
            violations
                .into_iter()
                .filter(|violation| violation.kind != BoostInvariantKind::UsedSplitAmounts)
                .collect()
        };
        assert!(
            violations.is_empty(),
            "{scope} boost invariant violations at frame {frame_number} (t={time:.3}, is_live_play={is_live_play}): {violations:?}"
        );
    }

    for frame in &timeline.frames {
        assert_stats(
            "team_zero",
            frame.frame_number,
            frame.time,
            &frame.team_zero.boost,
            frame.is_live_play,
        );
        assert_stats(
            "team_one",
            frame.frame_number,
            frame.time,
            &frame.team_one.boost,
            frame.is_live_play,
        );
        for player in &frame.players {
            assert_stats(
                &format!("player {}", player.name),
                frame.frame_number,
                frame.time,
                &player.boost,
                frame.is_live_play,
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

#[test]
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
