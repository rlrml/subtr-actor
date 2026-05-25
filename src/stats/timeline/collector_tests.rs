use super::*;
use std::collections::BTreeMap;

const STATS_TIMELINE_FIXTURE: &str =
    "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";

fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).unwrap_or_else(|_| panic!("Failed to read replay file: {path}"));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay: {path}"))
}

fn event_set_counts(events: &ReplayStatsTimelineEvents) -> Vec<(&'static str, usize)> {
    vec![
        ("timeline", events.timeline.len()),
        ("core_player", events.core_player.len()),
        ("core_team", events.core_team.len()),
        ("possession", events.possession.len()),
        ("pressure", events.pressure.len()),
        ("movement", events.movement.len()),
        ("positioning", events.positioning.len()),
        ("rotation_player", events.rotation_player.len()),
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
        ("goal_tags", events.goal_tags.len()),
        ("rush", events.rush.len()),
        ("speed_flip", events.speed_flip.len()),
        ("half_flip", events.half_flip.len()),
        ("half_volley", events.half_volley.len()),
        ("wavedash", events.wavedash.len()),
        ("whiff", events.whiff.len()),
        ("powerslide", events.powerslide.len()),
        ("touch", events.touch.len()),
        ("touch_ball_movement", events.touch_ball_movement.len()),
        ("touch_last_touch", events.touch_last_touch.len()),
        ("boost_pickups", events.boost_pickups.len()),
        ("boost_ledger", events.boost_ledger.len()),
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
        panic!(
            "event set {name} differs: left_count={}, right_count={}, first_mismatch={first_mismatch:?}",
            left_entries.len(),
            right_entries.len()
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
}

#[test]
fn event_timeline_scaffold_matches_full_timeline_without_stat_snapshots() {
    let replay = parse_replay(STATS_TIMELINE_FIXTURE);
    let full_timeline = StatsTimelineCollector::new()
        .get_replay_data(&replay)
        .expect("full stats timeline should collect");
    let scaffold_timeline = StatsTimelineEventCollector::new()
        .get_replay_data(&replay)
        .expect("event stats timeline scaffold should collect");

    assert_eq!(scaffold_timeline.config, full_timeline.config);
    assert_eq!(scaffold_timeline.replay_meta, full_timeline.replay_meta);
    assert_eq!(
        event_set_counts(&scaffold_timeline.events),
        event_set_counts(&full_timeline.events)
    );
    assert_canonical_event_sets_match(&scaffold_timeline.events, &full_timeline.events);
    assert_eq!(scaffold_timeline.frames.len(), full_timeline.frames.len());

    for (scaffold_frame, full_frame) in scaffold_timeline.frames.iter().zip(&full_timeline.frames) {
        assert_eq!(scaffold_frame.frame_number, full_frame.frame_number);
        assert_eq!(scaffold_frame.time, full_frame.time);
        assert_eq!(scaffold_frame.dt, full_frame.dt);
        assert_eq!(
            scaffold_frame.seconds_remaining,
            full_frame.seconds_remaining
        );
        assert_eq!(scaffold_frame.game_state, full_frame.game_state);
        assert_eq!(scaffold_frame.gameplay_phase, full_frame.gameplay_phase);
        assert_eq!(scaffold_frame.is_live_play, full_frame.is_live_play);

        assert!(
            scaffold_frame.team_zero.is_empty(),
            "event scaffold should not carry team-zero stat modules"
        );
        assert!(
            scaffold_frame.team_one.is_empty(),
            "event scaffold should not carry team-one stat modules"
        );
        assert_eq!(scaffold_frame.players.len(), full_frame.players.len());
        for (scaffold_player, full_player) in scaffold_frame.players.iter().zip(&full_frame.players)
        {
            assert_eq!(scaffold_player.player_id, full_player.player_id);
            assert_eq!(scaffold_player.name, full_player.name);
            assert_eq!(scaffold_player.is_team_0, full_player.is_team_0);
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
        ["is_team_0", "name", "player_id"]
    );
}
