//! Differential fidelity test for replay clipping.
//!
//! Processes a full replay while snapshotting per-frame `ProcessorView` state,
//! then clips a window out of the same replay, processes the clip, and asserts
//! that the clip reproduces the full replay's state frame-for-frame across the
//! window (after a short lead-in). This is the proof that a synthetic keyframe
//! faithfully seeds the processor, so detection logic tested on a clip behaves
//! identically to the same logic run over the whole replay.

use std::collections::BTreeMap;
use subtr_actor::{
    Collector, PlayerId, ProcessorView, ReplayProcessor, SubtrActorResult, TimeAdvance,
    clip_replay_around,
};

const REPLAY: &str = "assets/post-eac-ranked-duel-2026-04-28-a.replay";

fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).unwrap_or_else(|_| panic!("failed to read replay: {path}"));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("failed to parse replay: {path}"))
}

#[derive(Clone, Debug, PartialEq)]
struct PlayerSnap {
    location: Option<[f32; 3]>,
    boost: Option<i32>,
}

#[derive(Clone, Debug, PartialEq)]
struct FrameSnap {
    ball: Option<[f32; 3]>,
    players: BTreeMap<String, PlayerSnap>,
}

/// Records a `FrameSnap` per processed frame, keyed by the raw frame time bits so
/// full-replay and clip frames can be aligned exactly (clip frames preserve the
/// source frame times).
#[derive(Default)]
struct SnapshotCollector {
    snaps: Vec<(u32, FrameSnap)>,
}

fn location_of(rb: &boxcars::RigidBody) -> [f32; 3] {
    [rb.location.x, rb.location.y, rb.location.z]
}

impl Collector for SnapshotCollector {
    fn process_frame(
        &mut self,
        processor: &dyn ProcessorView,
        frame: &boxcars::Frame,
        _frame_number: usize,
        _current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        let ball = processor
            .get_normalized_ball_rigid_body()
            .ok()
            .map(|rb| location_of(&rb));

        let mut players = BTreeMap::new();
        let ids: Vec<PlayerId> = processor.iter_player_ids_in_order().cloned().collect();
        for id in ids {
            let name = processor
                .get_player_name(&id)
                .unwrap_or_else(|_| format!("{id:?}"));
            let location = processor
                .get_normalized_player_rigid_body(&id)
                .ok()
                .map(|rb| location_of(&rb));
            // Quantize boost so floating point jitter doesn't dominate the diff.
            let boost = processor
                .get_player_boost_level(&id)
                .ok()
                .map(|b| (b * 100.0).round() as i32);
            players.insert(name, PlayerSnap { location, boost });
        }

        self.snaps
            .push((frame.time.to_bits(), FrameSnap { ball, players }));
        Ok(TimeAdvance::NextFrame)
    }

    fn finish_replay(&mut self, _processor: &dyn ProcessorView) -> SubtrActorResult<()> {
        Ok(())
    }
}

fn collect_snapshots(replay: &boxcars::Replay) -> Vec<(u32, FrameSnap)> {
    let mut processor = ReplayProcessor::new(replay).expect("processor");
    let mut collector = SnapshotCollector::default();
    processor.process(&mut collector).expect("process");
    collector.snaps
}

/// Process `replay` to completion and return all touch events the processor
/// detected. Touch detection is exactly the kind of delta-based logic we want to
/// be able to test on a clip.
fn collect_touch_events(replay: &boxcars::Replay) -> Vec<subtr_actor::TouchEvent> {
    let mut processor = ReplayProcessor::new(replay).expect("processor");
    let mut collector = SnapshotCollector::default();
    processor.process(&mut collector).expect("process");
    processor.touch_events().to_vec()
}

/// Identity used to compare touches across full vs clip processing. Excludes
/// `frame` (shifted by the clip offset) and `touch_id` (a per-run monotonic
/// counter), both of which legitimately differ between runs.
fn touch_key(t: &subtr_actor::TouchEvent) -> (i64, bool, String, bool) {
    (
        (t.time * 1000.0).round() as i64,
        t.team_is_team_0,
        format!("{:?}", t.player),
        t.dodge_contact,
    )
}

#[test]
fn clip_reproduces_full_replay_processor_state_across_window() {
    let replay = parse_replay(REPLAY);
    let frame_count = replay
        .network_frames
        .as_ref()
        .expect("network frames")
        .frames
        .len();
    assert!(frame_count > 400, "fixture should be a full game");

    // A window in the thick of play, with lead-in warm-up frames before the
    // region we actually assert on.
    let lead_in = 60;
    let tail = 5;
    let region_start = frame_count / 2;
    let region_end = region_start + 120;

    let full = collect_snapshots(&replay);
    let full_by_time: BTreeMap<u32, &FrameSnap> = full.iter().map(|(t, s)| (*t, s)).collect();

    let clip =
        clip_replay_around(&replay, region_start, region_end, lead_in, tail).expect("build clip");
    // Clips are small and self-contained.
    assert!(
        clip.frames.len() < frame_count / 2,
        "clip should be much smaller than the source"
    );
    // Round-trip through JSON, exactly as a fixture would.
    let json = clip.to_json().expect("serialize clip");
    let restored = subtr_actor::ReplayClip::from_json(&json).expect("deserialize clip");
    assert_eq!(clip, restored, "clip should round-trip through JSON");

    let clip_replay = restored.to_replay();
    let clip_snaps = collect_snapshots(&clip_replay);

    // Skip the synthetic keyframe; align remaining clip frames to the full
    // replay by frame time and compare. Track how many leading frames diverge so
    // we can confirm the region of interest (after lead-in) is clean.
    let mut compared = 0;
    let mut first_clean_offset: Option<usize> = None;
    let mut mismatches_in_region = Vec::new();

    let real_clip_snaps = &clip_snaps[restored.provenance.synthetic_frame_count..];
    for (offset, (time_bits, clip_snap)) in real_clip_snaps.iter().enumerate() {
        let Some(full_snap) = full_by_time.get(time_bits) else {
            // Full collector may bootstrap past very early frames; skip any clip
            // frame that has no full counterpart.
            continue;
        };
        compared += 1;
        let matches = *full_snap == clip_snap;
        if matches && first_clean_offset.is_none() {
            first_clean_offset = Some(offset);
        }
        // `offset` counts real clip frames; the region of interest begins after
        // `lead_in` of them.
        if offset >= restored.provenance.lead_in_frames && !matches {
            mismatches_in_region.push((offset, (*full_snap).clone(), clip_snap.clone()));
        }
    }

    assert!(compared > 100, "should have compared a meaningful window");
    eprintln!(
        "clip fidelity: compared {compared} frames, first clean at offset {:?}, lead_in {}",
        first_clean_offset, restored.provenance.lead_in_frames
    );

    if let Some((offset, full_snap, clip_snap)) = mismatches_in_region.first() {
        panic!(
            "clip diverged from full replay inside region of interest at real-frame offset {offset} \
             (lead_in={}):\n full: {full_snap:?}\n clip: {clip_snap:?}\n total mismatches in region: {}",
            restored.provenance.lead_in_frames,
            mismatches_in_region.len(),
        );
    }
}

#[test]
fn clip_reproduces_touch_event_detection_in_window() {
    let replay = parse_replay(REPLAY);
    let frames = &replay
        .network_frames
        .as_ref()
        .expect("network frames")
        .frames;
    let frame_count = frames.len();

    let lead_in = 60;
    let tail = 30;
    let region_start = frame_count / 2;
    let region_end = region_start + 240;

    // Time bounds of the region of interest in the source replay.
    let region_t_start = frames[region_start].time;
    let region_t_end = frames[region_end].time;

    let in_region =
        |t: &subtr_actor::TouchEvent| t.time >= region_t_start && t.time <= region_t_end;

    let mut full_touches: Vec<_> = collect_touch_events(&replay)
        .into_iter()
        .filter(in_region)
        .map(|t| touch_key(&t))
        .collect();
    full_touches.sort();

    let clip =
        clip_replay_around(&replay, region_start, region_end, lead_in, tail).expect("build clip");
    let mut clip_touches: Vec<_> = collect_touch_events(&clip.to_replay())
        .into_iter()
        .filter(in_region)
        .map(|t| touch_key(&t))
        .collect();
    clip_touches.sort();

    assert!(
        !full_touches.is_empty(),
        "the chosen window should contain touches; widen it if this fixture is quiet here"
    );
    assert_eq!(
        full_touches, clip_touches,
        "touch detection on the clip should match the full replay within the region of interest"
    );
}
