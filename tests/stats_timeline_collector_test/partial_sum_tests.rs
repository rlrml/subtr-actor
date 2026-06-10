#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_mechanic_events_reconstruct_serialized_partial_sums() {
    let replay_paths = [
        "assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay",
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
    ];
    let mut saw_half_flip_event = false;
    let mut saw_wavedash_event = false;

    for replay_path in replay_paths {
        let replay = parse_replay(replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));

        if !timeline_has_stream(&timeline, "half_flip") && !timeline_has_stream(&timeline, "wavedash") {
            continue;
        }

        assert_quality_mechanic_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
        saw_half_flip_event |= timeline_has_stream(&timeline, "half_flip");
        saw_wavedash_event |= timeline_has_stream(&timeline, "wavedash");

        if saw_half_flip_event && saw_wavedash_event {
            break;
        }
    }

    assert!(
        saw_half_flip_event,
        "expected at least one fixture to contain a half-flip event"
    );
    assert!(
        saw_wavedash_event,
        "expected at least one fixture to contain a wavedash event"
    );
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_speed_flip_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/colonelpanic8-double-tap-third-goal-2026-05-24.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        timeline_has_stream(&timeline, "speed_flip"),
        "expected speed-flip fixture to contain speed-flip events"
    );
    assert_speed_flip_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_whiff_events_reconstruct_serialized_partial_sums() {
    let replay_paths = [
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
        "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g5.replay",
        "assets/recent-ranked-standard-2026-03-10-a.replay",
    ];
    let mut found_timeline = None;

    for replay_path in replay_paths {
        let replay = parse_replay(replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));
        if timeline_has_stream(&timeline, "whiff") {
            found_timeline = Some((replay_path, timeline));
            break;
        }
    }

    let (replay_path, timeline) =
        found_timeline.expect("expected at least one fixture to contain whiff events");
    assert_whiff_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_backboard_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        timeline_has_stream(&timeline, "backboard"),
        "expected backboard fixture to contain backboard events"
    );
    assert_backboard_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_double_tap_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/colonelpanic8-double-tap-third-goal-2026-05-24.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        timeline_has_stream(&timeline, "double_tap"),
        "expected double-tap fixture to contain double-tap events"
    );
    assert_double_tap_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_one_timer_events_reconstruct_serialized_partial_sums() {
    let replay_paths = [
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
        "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g5.replay",
        "assets/recent-ranked-standard-2026-03-10-a.replay",
        "assets/recent-ranked-standard-2026-03-10-b.replay",
    ];
    let mut found_timeline = None;

    for replay_path in replay_paths {
        let replay = parse_replay(replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));
        if timeline_has_stream(&timeline, "one_timer") {
            found_timeline = Some((replay_path, timeline));
            break;
        }
    }

    let (replay_path, timeline) =
        found_timeline.expect("expected at least one fixture to contain one-timer events");
    assert_one_timer_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_pass_events_reconstruct_serialized_partial_sums() {
    let replay_paths = [
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
        "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g5.replay",
        "assets/recent-ranked-standard-2026-03-10-a.replay",
        "assets/recent-ranked-standard-2026-03-10-b.replay",
    ];
    let mut found_timeline = None;

    for replay_path in replay_paths {
        let replay = parse_replay(replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));
        if timeline_has_stream(&timeline, "pass") {
            found_timeline = Some((replay_path, timeline));
            break;
        }
    }

    let (replay_path, timeline) =
        found_timeline.expect("expected at least one fixture to contain pass events");
    assert_pass_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_rush_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        timeline_has_stream(&timeline, "rush"),
        "expected rush fixture to contain rush events"
    );
    assert_rush_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_bump_events_reconstruct_serialized_partial_sums() {
    let replay_paths = [
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
        "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g5.replay",
        "assets/recent-ranked-standard-2026-03-10-a.replay",
        "assets/recent-ranked-standard-2026-03-10-b.replay",
        "assets/post-eac-ranked-standard-2026-04-28.replay",
    ];
    let mut found_timeline = None;

    for replay_path in replay_paths {
        let replay = parse_replay(replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));
        if timeline_has_stream(&timeline, "bump") {
            found_timeline = Some((replay_path, timeline));
            break;
        }
    }

    let (replay_path, timeline) =
        found_timeline.expect("expected at least one fixture to contain bump events");
    assert_bump_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_demo_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2026-01-14-v868-32-net10-demolish-extended.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        timeline_payloads_by_stream(&timeline, "timeline", |payload| match payload {
            EventPayload::Timeline(event) => Some(event),
            _ => None,
        })
        .iter()
        .any(|event| matches!(event.kind, TimelineEventKind::Kill | TimelineEventKind::Death)),
        "expected demo fixture to contain kill/death timeline events"
    );
    assert_demo_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_fifty_fifty_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        timeline_has_stream(&timeline, "fifty_fifty"),
        "expected fifty-fifty fixture to contain fifty-fifty events"
    );
    assert_fifty_fifty_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_half_volley_events_reconstruct_serialized_partial_sums() {
    let replay_paths = [
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
        "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g5.replay",
        "assets/recent-ranked-standard-2026-03-10-a.replay",
        "assets/recent-ranked-standard-2026-03-10-b.replay",
        "assets/air-dribble-goal-mouth-2026-05-24.replay",
    ];
    let mut found_timeline = None;

    for replay_path in replay_paths {
        let replay = parse_replay(replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));
        if timeline_has_stream(&timeline, "half_volley") {
            found_timeline = Some((replay_path, timeline));
            break;
        }
    }

    let (replay_path, timeline) =
        found_timeline.expect("expected at least one fixture to contain half-volley events");
    assert_half_volley_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_ball_carry_events_reconstruct_serialized_partial_sums() {
    let replay_paths = [
        "assets/air-dribble-goal-mouth-2026-05-24.replay",
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
    ];
    let mut found_timeline = None;

    for replay_path in replay_paths {
        let replay = parse_replay(replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));
        if timeline_has_stream(&timeline, "ball_carry") {
            found_timeline = Some((replay_path, timeline));
            break;
        }
    }

    let (replay_path, timeline) =
        found_timeline.expect("expected at least one fixture to contain ball-carry events");
    assert_ball_carry_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_wall_aerial_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/air-dribble-goal-mouth-2026-05-24.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        timeline_has_stream(&timeline, "wall_aerial"),
        "expected wall-aerial fixture to contain wall-aerial events"
    );
    assert_wall_aerial_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_wall_aerial_shot_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/air-dribble-goal-mouth-2026-05-24.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        timeline_has_stream(&timeline, "wall_aerial_shot"),
        "expected wall-aerial fixture to contain wall-aerial-shot events"
    );
    assert_wall_aerial_shot_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_flick_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        timeline_has_stream(&timeline, "flick"),
        "expected flick fixture to contain flick events"
    );
    assert_flick_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_musty_flick_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        timeline_has_stream(&timeline, "musty_flick"),
        "expected musty-flick fixture to contain musty-flick events"
    );
    assert_musty_flick_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_dodge_reset_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        timeline_has_stream(&timeline, "dodge_reset"),
        "expected dodge-reset fixture to contain dodge-reset events"
    );
    assert!(
        timeline_payloads_by_stream(&timeline, "dodge_reset", |payload| match payload {
            EventPayload::DodgeReset(event) => Some(event),
            _ => None,
        })
        .iter()
        .any(|event| event.on_ball),
        "expected dodge-reset fixture to contain on-ball dodge-reset events"
    );
    assert_dodge_reset_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_powerslide_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        timeline_has_stream(&timeline, "powerslide"),
        "expected powerslide fixture to contain powerslide events"
    );
    assert_powerslide_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_touch_events_reconstruct_final_serialized_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        timeline_has_stream(&timeline, "touch"),
        "expected touch fixture to contain touch events"
    );
    assert!(
        timeline_payloads_by_stream(&timeline, "touch", |payload| match payload {
            EventPayload::Touch(event) => Some(event),
            _ => None,
        })
        .iter()
        .any(|event| event.ball_movement.is_some()),
        "expected touch fixture to contain ball movement credits"
    );
    assert_touch_events_reconstruct_final_serialized_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_core_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        timeline_has_stream(&timeline, "core_player"),
        "expected core fixture to contain player scoreboard events"
    );
    assert!(
        timeline_has_stream(&timeline, "core_player_goal_context"),
        "expected core fixture to contain player goal-context events"
    );
    assert_core_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_possession_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        timeline_has_stream(&timeline, "possession"),
        "expected possession fixture to contain possession events"
    );
    assert_possession_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_pressure_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        timeline_has_stream(&timeline, "ball_half"),
        "expected ball_half fixture to contain ball_half events"
    );
    assert_pressure_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_movement_events_reconstruct_final_serialized_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        timeline_has_stream(&timeline, "movement"),
        "expected movement fixture to contain movement events"
    );
    assert_movement_events_reconstruct_final_serialized_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_positioning_events_reconstruct_final_serialized_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        timeline_has_stream(&timeline, "positioning_activity"),
        "expected positioning fixture to contain positioning events"
    );
    assert_positioning_events_reconstruct_final_serialized_sums(replay_path, &timeline);
}

#[test]
#[ignore = "replay-backed partial-sum reconstruction is slow; run explicitly when changing timeline derivation"]
fn test_rotation_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        timeline_has_stream(&timeline, "rotation_player"),
        "expected rotation fixture to contain rotation player events"
    );
    assert_rotation_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

fn assert_converted_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    assert_core_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_possession_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_pressure_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_movement_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_positioning_events_reconstruct_final_serialized_sums(replay_path, timeline);
    assert_rotation_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_quality_mechanic_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_speed_flip_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_whiff_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_backboard_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_double_tap_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_demo_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_fifty_fifty_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_bump_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_rush_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_pass_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_one_timer_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_ball_carry_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_wall_aerial_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_wall_aerial_shot_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_flick_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_ceiling_shot_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_musty_flick_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_dodge_reset_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_powerslide_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_touch_events_reconstruct_final_serialized_sums(replay_path, timeline);
    assert_half_volley_events_reconstruct_serialized_partial_sums(replay_path, timeline);
}

fn assert_replay_paths_reconstruct_serialized_partial_sums(replay_paths: Vec<String>) {
    for replay_path in replay_paths {
        eprintln!("checking {replay_path}");
        let replay = parse_replay(&replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));
        assert_converted_events_reconstruct_serialized_partial_sums(&replay_path, &timeline);
    }
}

#[test]
#[ignore = "wide replay-format parity is slow; run explicitly when changing compact timeline derivation"]
fn replay_format_fixture_events_reconstruct_serialized_partial_sums() {
    let replay_paths = replay_format_fixture_paths();
    assert!(
        !replay_paths.is_empty(),
        "expected replay-format docs to list checked-in fixtures"
    );
    assert!(
        std::env::var("SUBTR_ACTOR_REPLAY_FORMAT_FIXTURE").is_ok() || replay_paths.len() >= 10,
        "expected replay-format docs to list checked-in fixtures"
    );

    assert_replay_paths_reconstruct_serialized_partial_sums(replay_paths);
}

#[test]
#[ignore = "all replay asset event parity is slow; run explicitly before removing transferred partial sums"]
fn all_asset_fixture_events_reconstruct_serialized_partial_sums() {
    let replay_paths = asset_replay_fixture_paths();
    assert!(
        !replay_paths.is_empty(),
        "expected checked-in replay asset fixtures"
    );
    assert!(
        std::env::var("SUBTR_ACTOR_REPLAY_FIXTURE").is_ok() || replay_paths.len() >= 20,
        "expected broad replay fixture coverage"
    );

    assert_replay_paths_reconstruct_serialized_partial_sums(replay_paths);
}

#[test]
fn test_ceiling_shot_events_reconstruct_serialized_partial_sums() {
    let blue_player = PlayerId::Steam(1001);
    let orange_player = PlayerId::Steam(2002);
    let blue_event = CeilingShotEvent {
        time: 2.0,
        frame: 20,
        player: blue_player.clone(),
        player_position: None,
        is_team_0: true,
        ceiling_contact_time: 1.2,
        ceiling_contact_frame: 12,
        time_since_ceiling_contact: 0.8,
        ceiling_contact_position: [0.0, 0.0, 2040.0],
        touch_position: [500.0, 100.0, 520.0],
        local_ball_position: [120.0, 10.0, 30.0],
        separation_from_ceiling: 460.0,
        roof_alignment: 0.9,
        forward_alignment: 0.8,
        forward_approach_speed: 700.0,
        ball_speed_change: 500.0,
        confidence: 0.82,
    };
    let orange_event = CeilingShotEvent {
        time: 3.0,
        frame: 30,
        player: orange_player.clone(),
        player_position: None,
        is_team_0: false,
        ceiling_contact_time: 2.4,
        ceiling_contact_frame: 24,
        time_since_ceiling_contact: 0.6,
        ceiling_contact_position: [0.0, 0.0, 2040.0],
        touch_position: [-400.0, -200.0, 480.0],
        local_ball_position: [100.0, -20.0, 20.0],
        separation_from_ceiling: 520.0,
        roof_alignment: 0.85,
        forward_alignment: 0.7,
        forward_approach_speed: 650.0,
        ball_speed_change: 350.0,
        confidence: 0.7,
    };

    let mut blue_after_event = CeilingShotStats::default();
    blue_after_event
        .labeled_event_counts
        .increment([confidence_band_label_for_derivation(true)]);
    blue_after_event.count = 1;
    blue_after_event.high_confidence_count = 1;
    blue_after_event.is_last_ceiling_shot = true;
    blue_after_event.last_ceiling_shot_time = Some(2.0);
    blue_after_event.last_ceiling_shot_frame = Some(20);
    blue_after_event.time_since_last_ceiling_shot = Some(0.0);
    blue_after_event.frames_since_last_ceiling_shot = Some(0);
    blue_after_event.last_confidence = Some(0.82);
    blue_after_event.best_confidence = 0.82;
    blue_after_event.cumulative_confidence = 0.82;

    let mut blue_after_reset = blue_after_event.clone();
    blue_after_reset.is_last_ceiling_shot = false;
    blue_after_reset.time_since_last_ceiling_shot = Some(1.0);
    blue_after_reset.frames_since_last_ceiling_shot = Some(10);

    let mut orange_after_event = CeilingShotStats::default();
    orange_after_event
        .labeled_event_counts
        .increment([confidence_band_label_for_derivation(false)]);
    orange_after_event.count = 1;
    orange_after_event.high_confidence_count = 0;
    orange_after_event.is_last_ceiling_shot = true;
    orange_after_event.last_ceiling_shot_time = Some(3.0);
    orange_after_event.last_ceiling_shot_frame = Some(30);
    orange_after_event.time_since_last_ceiling_shot = Some(0.0);
    orange_after_event.frames_since_last_ceiling_shot = Some(0);
    orange_after_event.last_confidence = Some(0.7);
    orange_after_event.best_confidence = 0.7;
    orange_after_event.cumulative_confidence = 0.7;

    let frame = |frame_number: usize,
                 time: f32,
                 is_live_play: bool,
                 blue_stats: CeilingShotStats,
                 orange_stats: CeilingShotStats| {
        let mut blue = default_player_stats_snapshot(blue_player.clone(), "Blue ceiling", true);
        blue.ceiling_shot = blue_stats;
        let mut orange =
            default_player_stats_snapshot(orange_player.clone(), "Orange ceiling", false);
        orange.ceiling_shot = orange_stats;
        ReplayStatsFrame {
            frame_number,
            time,
            dt: 0.5,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: None,
            kickoff_countdown_time: None,
            gameplay_phase: if is_live_play {
                GameplayPhase::ActivePlay
            } else {
                GameplayPhase::PostGoal
            },
            is_live_play,
            team_zero: default_team_stats_snapshot(),
            team_one: default_team_stats_snapshot(),
            players: vec![blue, orange],
        }
    };

    let timeline = ReplayStatsTimeline {
        config: empty_stats_timeline_config(),
        replay_meta: ReplayMeta {
            team_zero: Vec::new(),
            team_one: Vec::new(),
            game_type: Default::default(),
            season: None,
            all_headers: Vec::new(),
        },
        events: ReplayStatsTimelineEvents {
            events: vec![
                test_event_envelope("ceiling_shot", 0, EventPayload::CeilingShot(blue_event)),
                test_event_envelope("ceiling_shot", 1, EventPayload::CeilingShot(orange_event)),
            ],
        },
        frames: vec![
            frame(
                20,
                2.0,
                true,
                blue_after_event.clone(),
                CeilingShotStats::default(),
            ),
            frame(
                25,
                2.5,
                false,
                blue_after_event.clone(),
                CeilingShotStats::default(),
            ),
            frame(30, 3.0, true, blue_after_reset, orange_after_event),
        ],
    };

    assert_ceiling_shot_events_reconstruct_serialized_partial_sums("synthetic", &timeline);
}
