//! Clip-based variant of the speed-flip regression coverage in
//! `speed_flip_replay_regression_test.rs`.
//!
//! Instead of running the stats timeline over the entire replay, this clips a
//! small window around the Rocket Sense reviewed events and asserts the same
//! detections on the clip. This is the intended workflow for event-level
//! regression tests: process the full replay once to *find* a case, then pin it
//! with a clip so the test only ever processes the frames that matter.

use subtr_actor::{
    clip_replay_around, EventPayload, PlayerId, ReplayMeta, SpeedFlipEvent,
    StatsTimelineEventCollector,
};

const ROCKET_SENSE_REVIEWED_DUEL_REPLAY: &str = "assets/post-eac-ranked-duel-2026-04-28-a.replay";

/// Source-replay frame of the Rocket Sense confirmed OSIDE_SMURF speed flip.
const CONFIRMED_SPEED_FLIP_FRAME: usize = 1848;
/// Source-replay time of the confirmed speed flip.
const CONFIRMED_SPEED_FLIP_TIME: f32 = 90.561_89;
/// Rocket Sense rejected Adamboi04 candidate that falls inside the clip window.
const REJECTED_CANDIDATE_FRAME: usize = 1850;

fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).unwrap_or_else(|_| panic!("Failed to read replay file: {path}"));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay: {path}"))
}

fn player_ids_by_name<'a>(replay_meta: &'a ReplayMeta, name: &str) -> Vec<&'a PlayerId> {
    replay_meta
        .team_zero
        .iter()
        .chain(replay_meta.team_one.iter())
        .filter(|player| player.name == name)
        .map(|player| &player.remote_id)
        .collect()
}

fn speed_flip_events(timeline: &subtr_actor::ReplayStatsTimelineScaffold) -> Vec<&SpeedFlipEvent> {
    timeline
        .events
        .events
        .iter()
        .filter_map(|event| match &event.payload {
            EventPayload::SpeedFlip(event) => Some(event),
            _ => None,
        })
        .collect()
}

#[test]
fn clip_reproduces_reviewed_speed_flip_detections() {
    let replay = parse_replay(ROCKET_SENSE_REVIEWED_DUEL_REPLAY);

    // Window around the reviewed events, with generous warm-up so the dodge and
    // movement trackers feeding speed-flip detection are fully seeded.
    let clip = clip_replay_around(
        &replay,
        CONFIRMED_SPEED_FLIP_FRAME - 60,
        REJECTED_CANDIDATE_FRAME + 60,
        90,
        30,
    )
    .expect("clip should build");
    let clip_replay = clip.to_replay();
    assert!(
        clip_replay.network_frames.as_ref().unwrap().frames.len() < 400,
        "clip should be a small fraction of the full replay"
    );

    let timeline = StatsTimelineEventCollector::new()
        .get_replay_stats_timeline_scaffold(&clip_replay)
        .expect("stats timeline should build from a clip");

    let oside_ids = player_ids_by_name(&timeline.replay_meta, "OSIDE_SMURF");
    let adamboi_ids = player_ids_by_name(&timeline.replay_meta, "Adamboi04");
    assert!(
        !oside_ids.is_empty(),
        "clip keyframe should preserve OSIDE_SMURF identity"
    );
    assert!(
        !adamboi_ids.is_empty(),
        "clip keyframe should preserve Adamboi04 identity"
    );

    let speed_flip_events = speed_flip_events(&timeline);

    // Frame indices shift inside a clip, so match the confirmed event by time
    // (preserved from the source replay) and by provenance-mapped frame.
    let expected_clip_frame = clip
        .provenance
        .clip_index_of(CONFIRMED_SPEED_FLIP_FRAME)
        .expect("confirmed frame should be inside the clip");
    let oside_confirmed = speed_flip_events.iter().any(|event| {
        oside_ids.contains(&&event.player)
            && ((event.time - CONFIRMED_SPEED_FLIP_TIME).abs() <= 0.15
                || event.frame.abs_diff(expected_clip_frame) <= 3)
    });
    assert!(
        oside_confirmed,
        "expected the Rocket Sense confirmed OSIDE_SMURF speed flip near source frame \
         {CONFIRMED_SPEED_FLIP_FRAME} (clip frame {expected_clip_frame}); got {speed_flip_events:#?}"
    );

    let rejected_clip_frame = clip
        .provenance
        .clip_index_of(REJECTED_CANDIDATE_FRAME)
        .expect("rejected frame should be inside the clip");
    let adamboi_false_positive = speed_flip_events.iter().find(|event| {
        adamboi_ids.contains(&&event.player) && event.frame.abs_diff(rejected_clip_frame) <= 3
    });
    assert!(
        adamboi_false_positive.is_none(),
        "unexpected speed flip for Rocket Sense rejected Adamboi04 candidate: \
         {adamboi_false_positive:#?}"
    );
}
