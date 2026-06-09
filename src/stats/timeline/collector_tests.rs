use super::*;
use std::collections::BTreeMap;

const STATS_TIMELINE_FIXTURE: &str = "assets/post-eac-ranked-duel-2026-04-28-a.replay";

const REPLAY_FORMAT_EVOLUTION_DOC: &str = include_str!("../../../docs/replay-format-evolution.md");

fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).unwrap_or_else(|_| panic!("Failed to read replay file: {path}"));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay: {path}"))
}

fn replay_format_fixture_paths() -> Vec<String> {
    REPLAY_FORMAT_EVOLUTION_DOC
        .lines()
        .filter_map(|line| {
            let start = line.find("| `")? + 3;
            let rest = &line[start..];
            let end = rest.find("` |")?;
            let fixture = &rest[..end];
            fixture
                .ends_with(".replay")
                .then(|| format!("assets/{fixture}"))
        })
        .collect()
}

fn asset_replay_fixture_paths() -> Vec<String> {
    let fixture_filter = std::env::var("SUBTR_ACTOR_REPLAY_FIXTURE").ok();
    let mut replay_paths = std::fs::read_dir("assets")
        .expect("expected checked-in replay asset directory")
        .filter_map(|entry| {
            let entry = entry.expect("expected replay asset directory entry");
            let path = entry.path();
            (path
                .extension()
                .is_some_and(|extension| extension == "replay"))
            .then(|| {
                path.to_str()
                    .expect("expected replay fixture path to be valid UTF-8")
                    .to_owned()
            })
        })
        .filter(|path| {
            fixture_filter
                .as_ref()
                .map(|filter| path.contains(filter))
                .unwrap_or(true)
        })
        .collect::<Vec<_>>();
    replay_paths.sort();
    replay_paths
}

fn event_set_counts(events: &ReplayStatsTimelineEvents) -> Vec<(&'static str, usize)> {
    vec![
        ("timeline", events.timeline.len()),
        ("core_player", events.core_player.len()),
        (
            "core_player_goal_context",
            events.core_player_goal_context.len(),
        ),
        ("possession", events.possession.len()),
        ("pressure", events.pressure.len()),
        ("movement", events.movement.len()),
        ("positioning_activity", events.positioning_activity.len()),
        (
            "positioning_possession",
            events.positioning_possession.len(),
        ),
        (
            "positioning_field_zone",
            events.positioning_field_zone.len(),
        ),
        (
            "positioning_ball_depth",
            events.positioning_ball_depth.len(),
        ),
        (
            "positioning_teammate_role",
            events.positioning_teammate_role.len(),
        ),
        (
            "positioning_ball_proximity",
            events.positioning_ball_proximity.len(),
        ),
        (
            "positioning_goal_context",
            events.positioning_goal_context.len(),
        ),
        ("rotation_player", events.rotation_player.len()),
        ("rotation_role_span", events.rotation_role_span.len()),
        ("rotation_depth_span", events.rotation_depth_span.len()),
        (
            "rotation_first_man_stint",
            events.rotation_first_man_stint.len(),
        ),
        ("rotation_team", events.rotation_team.len()),
        ("mechanics", events.mechanics.len()),
        ("goal_context", events.goal_context.len()),
        ("backboard", events.backboard.len()),
        ("ceiling_shot", events.ceiling_shot.len()),
        ("wall_aerial", events.wall_aerial.len()),
        ("wall_aerial_shot", events.wall_aerial_shot.len()),
        ("center", events.center.len()),
        ("flick", events.flick.len()),
        ("musty_flick", events.musty_flick.len()),
        ("dodge_reset", events.dodge_reset.len()),
        ("double_tap", events.double_tap.len()),
        ("fifty_fifty", events.fifty_fifty.len()),
        ("one_timer", events.one_timer.len()),
        ("pass", events.pass.len()),
        ("ball_carry", events.ball_carry.len()),
        ("rush", events.rush.len()),
        ("speed_flip", events.speed_flip.len()),
        ("half_flip", events.half_flip.len()),
        ("half_volley", events.half_volley.len()),
        ("wavedash", events.wavedash.len()),
        ("whiff", events.whiff.len()),
        ("powerslide", events.powerslide.len()),
        ("touch", events.touch.len()),
        ("boost_pickups", events.boost_pickups.len()),
        ("boost_ledger", events.boost_ledger.len()),
        ("boost_bucket", events.boost_bucket.len()),
        ("boost_state", events.boost_state.len()),
        ("bump", events.bump.len()),
    ]
}

fn canonical_event_sets(events: &ReplayStatsTimelineEvents) -> BTreeMap<String, Vec<String>> {
    let value = serde_json::to_value(events).expect("events should serialize");
    value
        .as_object()
        .expect("events should serialize as an object")
        .iter()
        .map(|(name, events)| {
            let mut entries = events
                .as_array()
                .unwrap_or_else(|| panic!("event set {name} should serialize as an array"))
                .iter()
                .map(|event| serde_json::to_string(event).expect("event should serialize"))
                .collect::<Vec<_>>();
            entries.sort();
            (name.clone(), entries)
        })
        .collect()
}

fn assert_canonical_event_sets_match(
    context: &str,
    left: &ReplayStatsTimelineEvents,
    right: &ReplayStatsTimelineEvents,
) {
    let left_sets = canonical_event_sets(left);
    let right_sets = canonical_event_sets(right);
    assert_eq!(
        left_sets.keys().collect::<Vec<_>>(),
        right_sets.keys().collect::<Vec<_>>()
    );
    for (name, left_entries) in left_sets {
        let right_entries = right_sets
            .get(&name)
            .unwrap_or_else(|| panic!("missing right event set {name}"));
        if &left_entries == right_entries {
            continue;
        }
        let first_mismatch = left_entries
            .iter()
            .zip(right_entries)
            .position(|(left, right)| left != right);
        let mismatch_detail = first_mismatch
            .map(|index| {
                format!(
                    ", left={}, right={}",
                    left_entries[index], right_entries[index]
                )
            })
            .unwrap_or_default();
        panic!(
            "{context} event set {name} differs: left_count={}, right_count={}, first_mismatch={first_mismatch:?}{mismatch_detail}",
            left_entries.len(),
            right_entries.len(),
        );
    }
}

#[test]
fn event_timeline_graph_does_not_build_full_stats_frame_snapshots() {
    let mut graph = build_timeline_event_graph();
    graph
        .resolve()
        .expect("event timeline graph should resolve");
    let node_names = graph.node_names().collect::<Vec<_>>();

    assert!(node_names.contains(&"stats_timeline_events"));
    assert!(
        !node_names.contains(&"stats_timeline_frame"),
        "event timeline transfer should not evaluate the full partial-sum frame node"
    );
    assert!(
        !node_names.contains(&"stats_projection"),
        "event timeline transfer should not evaluate full partial-sum projections"
    );
}

fn assert_event_timeline_scaffold_matches_full_timeline_without_stat_snapshots(replay_path: &str) {
    let replay = parse_replay(replay_path);
    let mut processor = ReplayProcessor::new(&replay).expect("replay processor should initialize");
    let mut full_collector = StatsTimelineCollector::new();
    let mut scaffold_collector = StatsTimelineEventCollector::new();
    processor
        .process_all(&mut [&mut full_collector, &mut scaffold_collector])
        .expect("full and event stats timelines should collect from the same processor");
    let full_timeline = full_collector
        .into_legacy_replay_stats_timeline()
        .expect("full stats timeline should assemble");
    let scaffold_timeline = scaffold_collector
        .into_replay_stats_timeline_scaffold()
        .expect("event stats timeline scaffold should assemble");

    assert_eq!(scaffold_timeline.config, full_timeline.config);
    assert_eq!(scaffold_timeline.replay_meta, full_timeline.replay_meta);
    assert_eq!(
        event_set_counts(&scaffold_timeline.events),
        event_set_counts(&full_timeline.events),
        "{replay_path} event set counts should match"
    );
    assert_canonical_event_sets_match(
        replay_path,
        &scaffold_timeline.events,
        &full_timeline.events,
    );
    assert_eq!(
        scaffold_timeline.frames.len(),
        full_timeline.frames.len(),
        "{replay_path} frame count should match"
    );

    for (scaffold_frame, full_frame) in scaffold_timeline.frames.iter().zip(&full_timeline.frames) {
        assert_eq!(
            scaffold_frame.frame_number, full_frame.frame_number,
            "{replay_path} scaffold frame number should match"
        );
        assert_eq!(
            scaffold_frame.time, full_frame.time,
            "{replay_path} scaffold frame time should match"
        );
        assert_eq!(
            scaffold_frame.dt, full_frame.dt,
            "{replay_path} scaffold frame dt should match"
        );
        assert_eq!(
            scaffold_frame.seconds_remaining, full_frame.seconds_remaining,
            "{replay_path} scaffold seconds_remaining should match"
        );
        assert_eq!(
            scaffold_frame.game_state, full_frame.game_state,
            "{replay_path} scaffold game_state should match"
        );
        assert_eq!(
            scaffold_frame.ball_has_been_hit, full_frame.ball_has_been_hit,
            "{replay_path} scaffold ball_has_been_hit should match"
        );
        assert_eq!(
            scaffold_frame.kickoff_countdown_time, full_frame.kickoff_countdown_time,
            "{replay_path} scaffold kickoff_countdown_time should match"
        );
        assert_eq!(
            scaffold_frame.gameplay_phase, full_frame.gameplay_phase,
            "{replay_path} scaffold gameplay_phase should match"
        );
        assert_eq!(
            scaffold_frame.is_live_play, full_frame.is_live_play,
            "{replay_path} scaffold live-play flag should match"
        );

        assert!(
            scaffold_frame.team_zero.is_empty(),
            "{replay_path} event scaffold should not carry team-zero stat modules"
        );
        assert!(
            scaffold_frame.team_one.is_empty(),
            "{replay_path} event scaffold should not carry team-one stat modules"
        );
        assert_eq!(
            scaffold_frame.players.len(),
            full_frame.players.len(),
            "{replay_path} scaffold player count should match"
        );
        for (scaffold_player, full_player) in scaffold_frame.players.iter().zip(&full_frame.players)
        {
            assert_eq!(
                scaffold_player.player_id, full_player.player_id,
                "{replay_path} scaffold player id should match"
            );
            assert_eq!(
                scaffold_player.name, full_player.name,
                "{replay_path} scaffold player name should match"
            );
            assert_eq!(
                scaffold_player.is_team_0, full_player.is_team_0,
                "{replay_path} scaffold player team should match"
            );
        }
    }

    // Distance is a continuous magnitude shipped once as a whole-match summary (not per
    // frame), so it must match the full timeline's final accumulated distance for each player.
    let final_full_frame = full_timeline
        .frames
        .last()
        .expect("full timeline should have at least one frame");
    for summary in &scaffold_timeline.positioning_summary {
        let Some(full_player) = final_full_frame
            .players
            .iter()
            .find(|player| player.player_id == summary.player_id)
        else {
            continue;
        };
        let distance = &summary.distance;
        let positioning = &full_player.positioning;
        for (label, summary_value, stat_value) in [
            (
                "sum_distance_to_ball",
                distance.sum_distance_to_ball,
                positioning.sum_distance_to_ball,
            ),
            (
                "sum_distance_to_teammates",
                distance.sum_distance_to_teammates,
                positioning.sum_distance_to_teammates,
            ),
            (
                "sum_distance_to_ball_has_possession",
                distance.sum_distance_to_ball_has_possession,
                positioning.sum_distance_to_ball_has_possession,
            ),
            (
                "sum_distance_to_ball_no_possession",
                distance.sum_distance_to_ball_no_possession,
                positioning.sum_distance_to_ball_no_possession,
            ),
        ] {
            assert_eq!(
                summary_value, stat_value,
                "{replay_path} positioning_summary.{label} should match full timeline final frame"
            );
        }
    }

    let first_scaffold_frame = scaffold_timeline
        .frames
        .iter()
        .find(|frame| !frame.players.is_empty())
        .expect("fixture should produce at least one player frame");
    let serialized_frame =
        serde_json::to_value(first_scaffold_frame).expect("scaffold frame should serialize");
    assert_eq!(
        serialized_frame
            .pointer("/team_zero")
            .and_then(serde_json::Value::as_object)
            .map(serde_json::Map::len),
        Some(0)
    );
    assert_eq!(
        serialized_frame
            .pointer("/team_one")
            .and_then(serde_json::Value::as_object)
            .map(serde_json::Map::len),
        Some(0)
    );
    let player = serialized_frame
        .pointer("/players/0")
        .and_then(serde_json::Value::as_object)
        .expect("scaffold player should serialize as an object");
    assert_eq!(
        player.keys().cloned().collect::<Vec<_>>(),
        ["is_team_0", "name", "player_id"],
        "{replay_path} scaffold player should serialize identity fields only"
    );
}

#[test]
#[ignore = "compact/full timeline scaffold replay parity is slow; run explicitly when changing timeline transfer"]
fn event_timeline_scaffold_matches_full_timeline_without_stat_snapshots() {
    assert_event_timeline_scaffold_matches_full_timeline_without_stat_snapshots(
        STATS_TIMELINE_FIXTURE,
    );
}

#[test]
#[ignore = "wide replay-format parity is slow; run explicitly when changing compact timeline transfer"]
fn replay_format_fixture_event_timeline_scaffolds_match_full_timelines() {
    let fixture_paths = replay_format_fixture_paths();
    assert!(
        fixture_paths.len() >= 10,
        "expected replay-format docs to list checked-in fixtures"
    );
    for replay_path in fixture_paths {
        println!("checking {replay_path}");
        assert_event_timeline_scaffold_matches_full_timeline_without_stat_snapshots(&replay_path);
    }
}

#[test]
#[ignore = "all replay asset scaffold parity is slow; run explicitly before removing transferred partial sums"]
fn all_asset_fixture_event_timeline_scaffolds_match_full_timelines_without_stat_snapshots() {
    let fixture_paths = asset_replay_fixture_paths();
    assert!(
        !fixture_paths.is_empty(),
        "expected checked-in replay asset fixtures"
    );
    assert!(
        std::env::var("SUBTR_ACTOR_REPLAY_FIXTURE").is_ok() || fixture_paths.len() >= 20,
        "expected broad replay fixture coverage"
    );
    for replay_path in fixture_paths {
        println!("checking {replay_path}");
        assert_event_timeline_scaffold_matches_full_timeline_without_stat_snapshots(&replay_path);
    }
}
