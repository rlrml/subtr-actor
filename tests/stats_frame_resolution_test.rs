mod common;

use subtr_actor::{
    PlayerStatsSnapshot, ReplayStatsTimeline, StatsCollector, StatsFrameResolution,
    StatsTimelineCollector, clip_replay_around_times,
};

const SMALL_STATS_FIXTURE: &str = "assets/replay-format-2016-07-21-v868-12-net-none-lan.replay";

#[test]
#[ignore = "full replay-backed stats frame parity is slow; frame persistence is covered by collector unit tests"]
fn stats_collector_default_resolution_matches_every_frame() {
    let replay = common::parse_replay(SMALL_STATS_FIXTURE);

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
#[ignore = "full replay event and frame parity is slow; clip parity runs in CI"]
fn stats_collector_and_timeline_collector_match_at_sampled_resolution() {
    let replay = common::parse_replay(SMALL_STATS_FIXTURE);
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

#[test]
fn clip_collectors_preserve_canonical_events_and_sampled_frames() {
    let replay = common::parse_replay("assets/post-eac-ranked-doubles-2026-04-28.replay");
    let clip = clip_replay_around_times(&replay, 79.0, 88.0, 90, 300)
        .expect("clip should build")
        .to_replay();
    for resolution in [
        StatsFrameResolution::EveryFrame,
        StatsFrameResolution::TimeStep { seconds: 0.5 },
    ] {
        let mut expected = StatsTimelineCollector::new()
            .with_frame_resolution(resolution)
            .get_legacy_replay_stats_timeline(&clip)
            .expect("timeline should build");
        complete_sparse_player_breakdowns(&mut expected);
        let actual = StatsCollector::new()
            .with_frame_resolution(resolution)
            .get_legacy_replay_stats_timeline(&clip)
            .expect("captured timeline should build");
        assert!(
            expected
                .events
                .events
                .iter()
                .any(|event| event.meta.stream == "loose_possession")
        );
        common::assert_replay_stats_timeline_eq(&actual, &expected);
    }
}

#[test]
#[allow(clippy::result_large_err)]
fn captured_events_survive_snapshot_conversions_and_include_dependencies() {
    let replay = common::parse_replay("assets/post-eac-ranked-doubles-2026-04-28.replay");
    let clip = clip_replay_around_times(&replay, 79.0, 88.0, 90, 300)
        .expect("clip should build")
        .to_replay();
    let snapshot = StatsCollector::with_builtin_module_names(["possession"])
        .expect("module selection should build")
        .with_frame_resolution(StatsFrameResolution::TimeStep { seconds: 0.5 })
        .get_snapshot_data(&clip)
        .expect("snapshot should build");
    assert_eq!(snapshot.modules.len(), 1);
    for stream in ["possession", "touch"] {
        assert!(
            snapshot
                .events
                .events
                .iter()
                .any(|event| event.meta.stream == stream),
            "expected selected or dependency stream {stream}"
        );
    }
    assert!(
        snapshot
            .events
            .events
            .iter()
            .all(|event| event.meta.lifecycle == subtr_actor::EventLifecycle::Finalized)
    );
    let typed = snapshot
        .to_legacy_replay_stats_timeline()
        .expect("typed conversion");
    assert_eq!(typed.events, snapshot.events);
    let json = snapshot
        .to_legacy_stats_timeline_value()
        .expect("JSON conversion");
    assert_eq!(
        json["events"],
        serde_json::to_value(&snapshot.events).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&snapshot).unwrap()["events"],
        json["events"]
    );
    let with_progress = snapshot
        .into_legacy_replay_stats_timeline_with_progress(10, |_, _| Ok(()))
        .expect("progress conversion");
    common::assert_replay_stats_timeline_eq(&typed, &with_progress);
}

struct InterimStatsCollector(StatsCollector);

impl subtr_actor::Collector for InterimStatsCollector {
    fn process_frame(
        &mut self,
        processor: &dyn subtr_actor::ProcessorView,
        frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> subtr_actor::SubtrActorResult<subtr_actor::TimeAdvance> {
        self.0
            .process_frame(processor, frame, frame_number, current_time)
    }
}

#[test]
fn capture_before_finish_projects_interim_events() {
    let replay = common::parse_replay("assets/post-eac-ranked-doubles-2026-04-28.replay");
    let clip = clip_replay_around_times(&replay, 79.0, 88.0, 90, 300)
        .expect("clip should build")
        .to_replay();
    let mut collector = InterimStatsCollector(StatsCollector::only_modules(["possession"]));
    subtr_actor::ReplayProcessor::new(&clip)
        .unwrap()
        .process(&mut collector)
        .unwrap();
    // The wrapper's default finish hook leaves the inner collector unfinished.
    let captured = collector.0.into_captured_data().expect("interim capture");
    assert!(!captured.events.events.is_empty());
    assert!(
        captured
            .events
            .events
            .iter()
            .any(|event| event.meta.lifecycle != subtr_actor::EventLifecycle::Finalized)
    );
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
