mod common;
mod stats_timeline_collector_mechanic_shots;

use common::{
    default_player_stats_snapshot, default_team_stats_snapshot, empty_stats_timeline_config,
    parse_replay,
};
use stats_timeline_collector_mechanic_shots::{
    assert_ceiling_shot_events_reconstruct_serialized_partial_sums,
    assert_dodge_reset_events_reconstruct_serialized_partial_sums,
    assert_flick_events_reconstruct_serialized_partial_sums,
    assert_musty_flick_events_reconstruct_serialized_partial_sums,
    assert_wall_aerial_events_reconstruct_serialized_partial_sums,
    assert_wall_aerial_shot_events_reconstruct_serialized_partial_sums,
};
use subtr_actor::*;

#[test]
fn test_wall_aerial_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/air-dribble-goal-mouth-2026-05-24.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.wall_aerial.is_empty(),
        "expected wall-aerial fixture to contain wall-aerial events"
    );
    assert_wall_aerial_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_wall_aerial_shot_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/air-dribble-goal-mouth-2026-05-24.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.wall_aerial_shot.is_empty(),
        "expected wall-aerial fixture to contain wall-aerial-shot events"
    );
    assert_wall_aerial_shot_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_flick_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.flick.is_empty(),
        "expected flick fixture to contain flick events"
    );
    assert_flick_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_musty_flick_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.musty_flick.is_empty(),
        "expected musty-flick fixture to contain musty-flick events"
    );
    assert_musty_flick_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_dodge_reset_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.dodge_reset.is_empty(),
        "expected dodge-reset fixture to contain dodge-reset events"
    );
    assert!(
        timeline
            .events
            .dodge_reset
            .iter()
            .any(|event| event.on_ball),
        "expected dodge-reset fixture to contain on-ball dodge-reset events"
    );
    assert_dodge_reset_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_ceiling_shot_events_reconstruct_serialized_partial_sums() {
    let blue_player = PlayerId::Steam(1001);
    let orange_player = PlayerId::Steam(2002);
    let blue_event = CeilingShotEvent {
        time: 2.0,
        frame: 20,
        player: blue_player.clone(),
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
        .increment([StatLabel::new("confidence_band", "high")]);
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
        .increment([StatLabel::new("confidence_band", "standard")]);
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
            all_headers: Vec::new(),
        },
        events: ReplayStatsTimelineEvents {
            ceiling_shot: vec![blue_event, orange_event],
            ..Default::default()
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
