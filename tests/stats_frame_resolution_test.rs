mod common;

use subtr_actor::{
    PlayerStatsSnapshot, ReplayStatsTimeline, StatsCollector, StatsFrameResolution,
    StatsTimelineCollector,
};

#[test]
fn stats_collector_default_resolution_matches_every_frame() {
    let replay =
        common::parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");

    let default_stats_collector = StatsCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("default stats collector timeline should build");
    let explicit_every_frame_stats_collector = StatsCollector::new()
        .with_frame_resolution(StatsFrameResolution::EveryFrame)
        .get_legacy_replay_stats_timeline(&replay)
        .expect("explicit every-frame stats collector timeline should build");
    common::assert_replay_stats_timeline_eq(
        &default_stats_collector,
        &explicit_every_frame_stats_collector,
    );
}

#[test]
fn stats_collector_and_timeline_collector_match_at_sampled_resolution() {
    let replay =
        common::parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let resolution = StatsFrameResolution::TimeStep { seconds: 0.5 };

    let full_timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("full stats timeline should build");
    let sampled_collector_timeline = StatsCollector::new()
        .with_frame_resolution(resolution)
        .get_legacy_replay_stats_timeline(&replay)
        .expect("sampled stats collector timeline should build");
    let mut sampled_timeline_collector = StatsTimelineCollector::new()
        .with_frame_resolution(resolution)
        .get_legacy_replay_stats_timeline(&replay)
        .expect("sampled stats timeline collector should build");
    complete_sparse_player_breakdowns(&mut sampled_timeline_collector);

    common::assert_replay_stats_timeline_eq(
        &sampled_collector_timeline,
        &sampled_timeline_collector,
    );

    assert!(
        sampled_collector_timeline.frames.len() < full_timeline.frames.len(),
        "expected sampled output to persist fewer frames than full output"
    );
    assert_eq!(
        sampled_collector_timeline
            .frames
            .first()
            .map(|frame| frame.frame_number),
        full_timeline.frames.first().map(|frame| frame.frame_number),
        "expected sampled output to retain the first frame"
    );
    assert_eq!(
        sampled_collector_timeline
            .frames
            .last()
            .map(|frame| frame.frame_number),
        full_timeline.frames.last().map(|frame| frame.frame_number),
        "expected sampled output to retain the final frame"
    );

    let first_frame = sampled_collector_timeline
        .frames
        .first()
        .expect("sampled output should include at least one frame");
    assert!(
        first_frame.dt.abs() < 1e-6,
        "expected first sampled frame dt to be zero, got {}",
        first_frame.dt
    );

    for window in sampled_collector_timeline.frames.windows(2) {
        let previous = &window[0];
        let current = &window[1];
        let expected_dt = (current.time - previous.time).max(0.0);
        let diff = (current.dt - expected_dt).abs();
        assert!(
            diff < 1e-4,
            "expected sampled frame dt to match emitted spacing between frames {} and {}: got dt={}, expected {}",
            previous.frame_number,
            current.frame_number,
            current.dt,
            expected_dt,
        );
    }
}

fn complete_sparse_player_breakdowns(timeline: &mut ReplayStatsTimeline) {
    for frame in &mut timeline.frames {
        for player in &mut frame.players {
            complete_player_breakdowns(player);
        }
    }
}

fn complete_player_breakdowns(player: &mut PlayerStatsSnapshot) {
    player.touch = player.touch.clone().with_complete_labeled_touch_counts();
    player.movement = player.movement.clone().with_complete_labeled_tracked_time();
}
