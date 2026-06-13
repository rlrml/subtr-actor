//! Clip-based replacement for `speed_flip_replay_regression_test.rs`.
//!
//! The original scanned whole replays for speed-flip detections at scattered
//! frames; this reproduces every one of those assertions on small clips. Each
//! reviewed point is covered by a clip window wide enough to contain it, with
//! generous warm-up so the dodge/movement trackers feeding speed-flip detection
//! are fully seeded. This is the intended workflow for event-level regression
//! tests: process the full replay once to *find* a case, then pin it with a
//! clip so the test only ever processes the frames that matter.

use subtr_actor::{
    EventPayload, PlayerId, ReplayClip, ReplayMeta, ReplayStatsTimelineScaffold, SpeedFlipEvent,
    StatsTimelineEventCollector, clip_replay_around,
};

const COLONELPANIC_NO_SPEED_FLIP_REPLAY: &str =
    "assets/colonelpanic8-double-tap-third-goal-2026-05-24.replay";
const ROCKET_SENSE_REVIEWED_DUEL_REPLAY: &str = "assets/post-eac-ranked-duel-2026-04-28-a.replay";

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

fn speed_flip_events(timeline: &ReplayStatsTimelineScaffold) -> Vec<&SpeedFlipEvent> {
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

/// Build a clip around `[region_start, region_end]` and run the stats timeline
/// over it. Returns the clip (for provenance) and the resulting scaffold.
fn clip_timeline(
    replay: &boxcars::Replay,
    region_start: usize,
    region_end: usize,
) -> (ReplayClip, ReplayStatsTimelineScaffold) {
    let clip = clip_replay_around(replay, region_start, region_end, 120, 60).expect("clip builds");
    let timeline = StatsTimelineEventCollector::new()
        .get_replay_stats_timeline_scaffold(&clip.to_replay())
        .expect("stats timeline should build from a clip");
    (clip, timeline)
}

/// Assert no speed flip for `player_ids` lands within 3 frames of any
/// `source_frame` that falls inside the clip (mapped through provenance).
fn assert_no_speed_flip_at_source_frames(
    clip: &ReplayClip,
    timeline: &ReplayStatsTimelineScaffold,
    player_ids: &[&PlayerId],
    source_frames: &[usize],
    context: &str,
) {
    let events = speed_flip_events(timeline);
    for &source_frame in source_frames {
        let Some(clip_frame) = clip.provenance.clip_index_of(source_frame) else {
            continue; // outside this clip window; covered elsewhere
        };
        let false_positive = events.iter().find(|event| {
            player_ids.contains(&&event.player) && event.frame.abs_diff(clip_frame) <= 3
        });
        assert!(
            false_positive.is_none(),
            "[{context}] unexpected speed flip near rejected source frame {source_frame} \
             (clip frame {clip_frame}): {false_positive:#?}"
        );
    }
}

#[test]
fn clip_colonelpanic_replay_has_no_speed_flip_at_normalized_28_1_seconds() {
    // Rocket Sense rejected candidate: colonelpanic8 near raw frame 837 / 31.7s.
    let replay = parse_replay(COLONELPANIC_NO_SPEED_FLIP_REPLAY);
    let (clip, timeline) = clip_timeline(&replay, 837, 837);
    let colonelpanic_ids = player_ids_by_name(&timeline.replay_meta, "colonelpanic8");
    assert!(
        !colonelpanic_ids.is_empty(),
        "clip keyframe should preserve colonelpanic8 identity"
    );
    assert_no_speed_flip_at_source_frames(
        &clip,
        &timeline,
        &colonelpanic_ids,
        &[837],
        "colonelpanic8 frame 837",
    );
    // Also assert by preserved source time, independent of frame mapping.
    let by_time = speed_flip_events(&timeline).iter().any(|event| {
        colonelpanic_ids.contains(&&event.player) && (event.time - 31.695_719).abs() <= 0.15
    });
    assert!(
        !by_time,
        "unexpected colonelpanic8 speed flip near 31.7s in clip"
    );
}

#[test]
fn clip_reviewed_post_eac_duel_keeps_confirmed_speed_flip_and_rejects_nearby_false_positive() {
    let replay = parse_replay(ROCKET_SENSE_REVIEWED_DUEL_REPLAY);

    // The confirmed OSIDE_SMURF speed flip (source frame 1848) and the
    // adjacent rejected Adamboi04 candidates clustered around it.
    let (cluster_clip, cluster_timeline) = clip_timeline(&replay, 1540, 1900);
    let oside_ids = player_ids_by_name(&cluster_timeline.replay_meta, "OSIDE_SMURF");
    let adamboi_ids = player_ids_by_name(&cluster_timeline.replay_meta, "Adamboi04");
    assert!(!oside_ids.is_empty(), "clip should contain OSIDE_SMURF");
    assert!(!adamboi_ids.is_empty(), "clip should contain Adamboi04");

    let confirmed_clip_frame = cluster_clip
        .provenance
        .clip_index_of(1848)
        .expect("confirmed frame should be inside the cluster clip");
    let oside_confirmed = speed_flip_events(&cluster_timeline).iter().any(|event| {
        oside_ids.contains(&&event.player)
            && (event.frame.abs_diff(confirmed_clip_frame) <= 3
                || (event.time - 90.561_89).abs() <= 0.15)
    });
    assert!(
        oside_confirmed,
        "expected the Rocket Sense confirmed OSIDE_SMURF speed flip near source frame 1848; \
         got {:#?}",
        speed_flip_events(&cluster_timeline)
    );
    assert_no_speed_flip_at_source_frames(
        &cluster_clip,
        &cluster_timeline,
        &adamboi_ids,
        &[1557, 1644, 1850],
        "Adamboi04 cluster",
    );

    // The earlier rejected Adamboi04 candidates, covered by a clip taken from
    // the start of the replay (no synthetic keyframe).
    let (early_clip, early_timeline) = clip_timeline(&replay, 0, 1100);
    let early_adamboi_ids = player_ids_by_name(&early_timeline.replay_meta, "Adamboi04");
    assert!(
        !early_adamboi_ids.is_empty(),
        "early clip should contain Adamboi04"
    );
    assert_no_speed_flip_at_source_frames(
        &early_clip,
        &early_timeline,
        &early_adamboi_ids,
        &[110, 576, 1020],
        "Adamboi04 early",
    );
}
