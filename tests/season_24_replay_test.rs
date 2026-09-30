mod common;

use subtr_actor::{ReplayDataCollector, ReplayProcessor, StatsTimelineEventCollector};

fn assert_season_24_replay(
    path: &str,
    player_count: usize,
    frame_count: usize,
    honor_duels: usize,
) {
    let replay = common::parse_replay(path);
    assert_eq!(
        (
            replay.major_version,
            replay.minor_version,
            replay.net_version
        ),
        (868, 34, Some(12))
    );
    let frames = &replay.network_frames.as_ref().unwrap().frames;
    assert_eq!(frames.len(), frame_count);
    assert_eq!(
        replay
            .properties
            .iter()
            .find(|(key, _)| key == "NumFrames")
            .unwrap()
            .1
            .as_i32(),
        Some(frame_count as i32),
    );
    assert_eq!(
        frames
            .iter()
            .flat_map(|frame| &frame.updated_actors)
            .filter(|update| matches!(&update.attribute, boxcars::Attribute::HonorDuelChallenge(_)))
            .count(),
        honor_duels,
    );
    let camera = frames
        .iter()
        .flat_map(|frame| &frame.updated_actors)
        .find_map(|update| match &update.attribute {
            boxcars::Attribute::CamSettings(camera) => Some(camera),
            _ => None,
        })
        .expect("Season 24 camera profile should decode");
    assert!(camera.camera_accel_rate.is_some_and(|value| value > 0.0));
    assert!(camera.camera_decel_rate.is_some_and(|value| value > 0.0));
    assert!(camera.free_look_speed.is_some_and(|value| value > 0.0));
    assert!(camera.unconstrain_rotation.is_some());
    assert!(camera.free_look_smoothing.is_some());

    let mut processor = ReplayProcessor::new(&replay).expect("processor should initialize");
    let mut replay_data_collector = ReplayDataCollector::new();
    let mut stats_collector = StatsTimelineEventCollector::new();
    processor
        .process_all(&mut [&mut replay_data_collector, &mut stats_collector])
        .expect("Season 24 replay and stats should process");
    let replay_data = replay_data_collector
        .into_replay_data(processor)
        .expect("replay data should assemble");
    let stats = stats_collector
        .into_replay_stats_timeline_scaffold()
        .expect("stats timeline should assemble");
    assert_eq!(replay_data.frame_data.frame_count(), frame_count);
    assert_eq!(replay_data.frame_data.players.len(), player_count);
    assert_eq!(stats.frames.len(), frame_count);
    assert_eq!(stats.replay_meta.player_order().count(), player_count);
    assert!(!replay_data.touch_events.is_empty());
    assert!(!replay_data.goal_events.is_empty());
    assert!(!stats.events.events.is_empty());
    for player in replay_data.meta.player_order() {
        let camera = player
            .camera_settings
            .expect("player camera settings should resolve");
        assert!((60.0..=120.0).contains(&camera.fov));
        assert!((100.0..=400.0).contains(&camera.distance));
    }
}

#[test]
fn season_24_standard_replay_produces_replay_and_stats_data() {
    assert_season_24_replay(
        "assets/replay-format-2026-09-18-v868-34-net12-season24-standard.replay",
        6,
        7501,
        0,
    );
}

#[test]
fn season_24_accepted_honor_duel_replay_produces_replay_and_stats_data() {
    assert_season_24_replay(
        "assets/replay-format-2026-09-18-v868-34-net12-season24-honor-duel.replay",
        4,
        10123,
        4,
    );
}
