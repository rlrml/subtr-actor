use subtr_actor::*;

fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).unwrap_or_else(|_| panic!("Failed to read replay file: {path}"));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay: {path}"))
}

fn frame_total_goals(frame: &ReplayStatsFrame) -> i32 {
    frame.team_zero.core.goals + frame.team_one.core.goals
}

fn player_snapshot_by_name<'a>(
    frame: &'a ReplayStatsFrame,
    player_name: &str,
) -> &'a PlayerStatsSnapshot {
    frame
        .players
        .iter()
        .find(|player| player.name == player_name)
        .unwrap_or_else(|| {
            panic!(
                "Missing player {player_name} in frame {} (t={:.3})",
                frame.frame_number, frame.time
            )
        })
}

fn normalized_team_stats_for_live_play_comparison(
    snapshot: &TeamStatsSnapshot,
) -> TeamStatsSnapshot {
    let mut normalized = snapshot.clone();
    normalized.core = CoreTeamStats::default();
    normalized.boost.amount_used = 0.0;
    normalized.demo = DemoTeamStats::default();
    normalized
}

fn default_team_stats_snapshot() -> TeamStatsSnapshot {
    TeamStatsSnapshot {
        fifty_fifty: FiftyFiftyTeamStats::default(),
        possession: PossessionTeamStats::default(),
        pressure: PressureTeamStats::default(),
        rush: RushTeamStats::default(),
        core: CoreTeamStats::default(),
        backboard: BackboardTeamStats::default(),
        double_tap: DoubleTapTeamStats::default(),
        ball_carry: BallCarryStats::default(),
        boost: BoostStats::default(),
        movement: MovementStats::default(),
        powerslide: PowerslideStats::default(),
        demo: DemoTeamStats::default(),
    }
}

fn normalized_player_stats_for_live_play_comparison(
    snapshot: &PlayerStatsSnapshot,
) -> PlayerStatsSnapshot {
    let mut normalized = snapshot.clone();
    normalized.core = CorePlayerStats::default();
    normalized.boost.amount_used = 0.0;
    normalized.demo = DemoPlayerStats::default();
    normalized
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
            "{scope} boost invariant violations at frame {frame_number} (t={time:.3}): {violations:?}"
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

#[test]
fn test_stats_timeline_frame_lookup_uses_frame_number() {
    let timeline = ReplayStatsTimeline {
        config: StatsTimelineConfig {
            most_back_forward_threshold_y: PositioningCalculatorConfig::default()
                .most_back_forward_threshold_y,
            level_ball_depth_margin: PositioningCalculatorConfig::default().level_ball_depth_margin,
            pressure_neutral_zone_half_width_y: PressureCalculatorConfig::default()
                .neutral_zone_half_width_y,
            rush_max_start_y: RushCalculatorConfig::default().max_start_y,
            rush_attack_support_distance_y: RushCalculatorConfig::default()
                .attack_support_distance_y,
            rush_defender_distance_y: RushCalculatorConfig::default().defender_distance_y,
            rush_min_possession_retained_seconds: RushCalculatorConfig::default()
                .min_possession_retained_seconds,
        },
        replay_meta: ReplayMeta {
            team_zero: Vec::new(),
            team_one: Vec::new(),
            all_headers: Vec::new(),
        },
        events: ReplayStatsTimelineEvents {
            timeline: Vec::new(),
            backboard: Vec::new(),
            ceiling_shot: Vec::new(),
            double_tap: Vec::new(),
            fifty_fifty: Vec::new(),
            rush: Vec::new(),
            speed_flip: Vec::new(),
        },
        frames: vec![
            ReplayStatsFrame {
                frame_number: 10,
                time: 0.0,
                dt: 0.0,
                seconds_remaining: None,
                game_state: None,
                gameplay_phase: GameplayPhase::ActivePlay,
                is_live_play: true,
                team_zero: default_team_stats_snapshot(),
                team_one: default_team_stats_snapshot(),
                players: Vec::new(),
            },
            ReplayStatsFrame {
                frame_number: 11,
                time: 0.1,
                dt: 0.1,
                seconds_remaining: None,
                game_state: None,
                gameplay_phase: GameplayPhase::ActivePlay,
                is_live_play: true,
                team_zero: default_team_stats_snapshot(),
                team_one: default_team_stats_snapshot(),
                players: Vec::new(),
            },
            ReplayStatsFrame {
                frame_number: 15,
                time: 0.2,
                dt: 0.1,
                seconds_remaining: None,
                game_state: None,
                gameplay_phase: GameplayPhase::ActivePlay,
                is_live_play: true,
                team_zero: default_team_stats_snapshot(),
                team_one: default_team_stats_snapshot(),
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
fn test_stats_timeline_collector_final_frame_matches_analysis_graph() {
    let replay = parse_replay("assets/replays/rlcs.replay");
    let timeline = StatsTimelineCollector::new()
        .get_replay_data(&replay)
        .expect("Expected stats timeline data");
    let final_frame = timeline.frames.last().expect("Expected at least one frame");

    let graph = stats::analysis_graph::collect_builtin_analysis_graph_for_replay(
        &replay,
        [
            "fifty_fifty",
            "possession",
            "pressure",
            "rush",
            "core",
            "backboard",
            "double_tap",
            "ball_carry",
            "boost",
            "movement",
            "positioning",
            "powerslide",
            "demo",
        ],
    )
    .expect("Expected analysis graph to process replay");

    let possession = graph
        .state::<PossessionCalculator>()
        .expect("missing possession calculator state");
    let fifty_fifty = graph
        .state::<FiftyFiftyCalculator>()
        .expect("missing fifty_fifty calculator state");
    let pressure = graph
        .state::<PressureCalculator>()
        .expect("missing pressure calculator state");
    let rush = graph
        .state::<RushCalculator>()
        .expect("missing rush calculator state");
    let match_stats = graph
        .state::<MatchStatsCalculator>()
        .expect("missing match stats calculator state");
    let backboard = graph
        .state::<BackboardCalculator>()
        .expect("missing backboard calculator state");
    let double_tap = graph
        .state::<DoubleTapCalculator>()
        .expect("missing double tap calculator state");
    let ball_carry = graph
        .state::<BallCarryCalculator>()
        .expect("missing ball carry calculator state");
    let boost = graph
        .state::<BoostCalculator>()
        .expect("missing boost calculator state");
    let movement = graph
        .state::<MovementCalculator>()
        .expect("missing movement calculator state");
    let positioning = graph
        .state::<PositioningCalculator>()
        .expect("missing positioning calculator state");
    let powerslide = graph
        .state::<PowerslideCalculator>()
        .expect("missing powerslide calculator state");
    let demo = graph
        .state::<DemoCalculator>()
        .expect("missing demo calculator state");

    let assert_core_team_stats_match =
        |actual: &CoreTeamStats, expected: &CoreTeamStats, team_label: &str| {
            assert_eq!(actual.score, expected.score, "{team_label} score");
            assert_eq!(actual.goals, expected.goals, "{team_label} goals");
            assert_eq!(actual.assists, expected.assists, "{team_label} assists");
            assert_eq!(actual.saves, expected.saves, "{team_label} saves");
            assert_eq!(actual.shots, expected.shots, "{team_label} shots");
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.kickoff_goal_count,
                expected
                    .scoring_context
                    .goal_after_kickoff
                    .kickoff_goal_count,
                "{team_label} kickoff-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.short_goal_count,
                expected.scoring_context.goal_after_kickoff.short_goal_count,
                "{team_label} short-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.medium_goal_count,
                expected
                    .scoring_context
                    .goal_after_kickoff
                    .medium_goal_count,
                "{team_label} medium-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.long_goal_count,
                expected.scoring_context.goal_after_kickoff.long_goal_count,
                "{team_label} long-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_buildup, expected.scoring_context.goal_buildup,
                "{team_label} goal buildup classifications",
            );
        };

    let assert_core_player_stats_match =
        |actual: &CorePlayerStats, expected: &CorePlayerStats, player_label: &str| {
            assert_eq!(actual.score, expected.score, "{player_label} score");
            assert_eq!(actual.goals, expected.goals, "{player_label} goals");
            assert_eq!(actual.assists, expected.assists, "{player_label} assists");
            assert_eq!(actual.saves, expected.saves, "{player_label} saves");
            assert_eq!(actual.shots, expected.shots, "{player_label} shots");
            assert_eq!(
                actual.scoring_context.goals_conceded_while_last_defender,
                expected.scoring_context.goals_conceded_while_last_defender,
                "{player_label} last-defender concessions",
            );
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.kickoff_goal_count,
                expected
                    .scoring_context
                    .goal_after_kickoff
                    .kickoff_goal_count,
                "{player_label} kickoff-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.short_goal_count,
                expected.scoring_context.goal_after_kickoff.short_goal_count,
                "{player_label} short-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.medium_goal_count,
                expected
                    .scoring_context
                    .goal_after_kickoff
                    .medium_goal_count,
                "{player_label} medium-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.long_goal_count,
                expected.scoring_context.goal_after_kickoff.long_goal_count,
                "{player_label} long-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_buildup, expected.scoring_context.goal_buildup,
                "{player_label} goal buildup classifications",
            );
        };

    assert_eq!(
        final_frame.team_zero.fifty_fifty,
        fifty_fifty.stats().for_team(true)
    );
    assert_eq!(
        final_frame.team_one.fifty_fifty,
        fifty_fifty.stats().for_team(false)
    );
    assert_eq!(
        final_frame.team_zero.possession,
        possession.stats().for_team(true)
    );
    assert_eq!(
        final_frame.team_one.possession,
        possession.stats().for_team(false)
    );
    assert_eq!(
        final_frame.team_zero.pressure,
        pressure.stats().for_team(true)
    );
    assert_eq!(
        final_frame.team_one.pressure,
        pressure.stats().for_team(false)
    );
    assert_eq!(final_frame.team_zero.rush, rush.stats().for_team(true));
    assert_eq!(final_frame.team_one.rush, rush.stats().for_team(false));
    assert_core_team_stats_match(
        &final_frame.team_zero.core,
        &match_stats.team_zero_stats(),
        "team zero",
    );
    assert_core_team_stats_match(
        &final_frame.team_one.core,
        &match_stats.team_one_stats(),
        "team one",
    );
    assert_eq!(
        final_frame.team_zero.ball_carry,
        ball_carry.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_zero.backboard,
        backboard.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.backboard,
        backboard.team_one_stats().clone()
    );
    assert_eq!(
        final_frame.team_zero.double_tap,
        double_tap.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.double_tap,
        double_tap.team_one_stats().clone()
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
        assert_core_player_stats_match(
            &player.core,
            &match_stats
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default(),
            &player.name,
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
            player.backboard,
            backboard
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.double_tap,
            double_tap
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
                .with_complete_labeled_tracked_time()
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
    assert_eq!(timeline.events.backboard, backboard.events());
    assert_eq!(timeline.events.double_tap, double_tap.events());
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
            .events
            .timeline
            .windows(2)
            .all(|events| events[1].time >= events[0].time),
        "Expected emitted timeline events to be time sorted"
    );
    assert_boost_pickup_nominal_amounts_consistent(&timeline);
}

#[test]
fn test_stats_timeline_value_serializes_for_rlcs_replay() {
    let replay = parse_replay("assets/replays/rlcs.replay");
    let captured = StatsCollector::new()
        .capture_frames()
        .get_captured_data(&replay)
        .expect("Expected captured stats data");

    captured
        .into_stats_timeline_value()
        .expect("Expected stats timeline value");
}

#[test]
fn test_stats_timeline_excludes_post_goal_reset_frames_from_cumulative_stats() {
    let replay = parse_replay("assets/replays/rlcs.replay");
    let replay_data = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .expect("Expected replay data");
    let timeline = StatsTimelineCollector::new()
        .get_replay_data(&replay)
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
fn test_stats_timeline_awards_touch_for_on_ball_reset_in_dodges_replay() {
    let replay = parse_replay("assets/replays/dodges_refreshed_counter.replay");
    let timeline = StatsTimelineCollector::new()
        .get_replay_data(&replay)
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
fn test_stats_timeline_first_kickoff_credits_both_players() {
    let replay = parse_replay("assets/replays/dodges_refreshed_counter.replay");
    let timeline = StatsTimelineCollector::new()
        .get_replay_data(&replay)
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
