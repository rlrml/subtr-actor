//! Differential event-fidelity test for replay clips.
//!
//! For a window of a real replay, this collects every discrete *event* the full
//! pipeline reports (demolishes, goal events, and player stat events), then
//! builds a clip around that window and asserts the clip reports exactly the
//! same events — no more (phantom events manufactured by the synthetic
//! keyframe) and no fewer (events lost to clipping).
//!
//! This is the systematic counterpart to `clip_fidelity_test.rs` (which diffs
//! continuous rigid-body/boost state): it targets the delta-based event
//! detectors that key off `UpdatedAttribute`s and persisted actor state, which
//! are exactly the detectors at risk of firing on stale keyframe data. Windows
//! are centered on demolishes so both demolish detection paths
//! (`updated_actors` and `get_active_demos`) are exercised.

mod common;

use std::collections::BTreeMap;

use subtr_actor::*;

/// One discrete event, keyed for comparison by kind + source time (frame
/// indices shift inside a clip, but event times are preserved from the source
/// replay). Times are bucketed to the millisecond to absorb f32 noise.
type EventKey = (String, i64);

fn event_key(kind: &str, time: f32) -> EventKey {
    (kind.to_string(), (time * 1000.0).round() as i64)
}

#[derive(Default)]
struct EventCollector {
    /// kind -> count, for events seen during per-frame processing.
    per_frame: BTreeMap<EventKey, usize>,
    /// The clip frame (or source frame) each event landed on, for the
    /// phantom-at-keyframe check.
    frames: Vec<(EventKey, usize)>,
}

impl Collector for EventCollector {
    fn process_frame(
        &mut self,
        processor: &dyn ProcessorView,
        _frame: &boxcars::Frame,
        frame_number: usize,
        _current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        for event in processor.current_frame_goal_events() {
            let key = event_key("goal", event.time);
            *self.per_frame.entry(key.clone()).or_default() += 1;
            self.frames.push((key, frame_number));
        }
        for event in processor.current_frame_player_stat_events() {
            let key = event_key(&format!("stat:{:?}", event.kind), event.time);
            *self.per_frame.entry(key.clone()).or_default() += 1;
            self.frames.push((key, frame_number));
        }
        Ok(TimeAdvance::NextFrame)
    }

    fn finish_replay(&mut self, _processor: &dyn ProcessorView) -> SubtrActorResult<()> {
        Ok(())
    }
}

/// Run the full event collector plus pull accumulated demolishes off the
/// processor (demolishes are only exposed in aggregate, not per frame).
fn collect_events(replay: &boxcars::Replay) -> (BTreeMap<EventKey, usize>, Vec<(EventKey, usize)>) {
    let collector = EventCollector::default()
        .process_replay(replay)
        .expect("event collector should process replay");
    let mut per_frame = collector.per_frame;
    let mut frames = collector.frames;

    // Demolishes via a second pass that exposes the accumulated list.
    let processor_data = ReplayDataCollector::new()
        .get_replay_data(replay)
        .expect("replay data should collect");
    for demo in &processor_data.demolish_infos {
        let key = event_key("demolish", demo.time);
        *per_frame.entry(key.clone()).or_default() += 1;
        frames.push((key, demo.frame));
    }

    (per_frame, frames)
}

struct DifferentialCase {
    replay_path: &'static str,
    /// Source-replay window of interest [start, end].
    region: (usize, usize),
    /// What the window is built around, for assertion messages.
    description: &'static str,
}

const CASES: &[DifferentialCase] = &[
    DifferentialCase {
        // Demolish at frame 2974 with no goal nearby.
        replay_path: "assets/post-eac-ranked-standard-2026-04-28.replay",
        region: (2900, 3050),
        description: "standalone demolish",
    },
    DifferentialCase {
        // Goal at frame 1703 plus the shot/save/assist stat events around it.
        replay_path: "assets/post-eac-ranked-standard-2026-04-28.replay",
        region: (1650, 1750),
        description: "goal with surrounding stat events",
    },
    DifferentialCase {
        // Two demolishes close together (frames 5425, plus stats) in an RLCS game.
        replay_path: "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g5.replay",
        region: (5380, 5470),
        description: "RLCS demolish window",
    },
    DifferentialCase {
        // Lone demolish in the doubles replay.
        replay_path: "assets/post-eac-ranked-doubles-2026-04-28.replay",
        region: (5820, 5930),
        description: "doubles demolish",
    },
];

#[test]
fn clip_reproduces_full_replay_events_in_window() {
    for case in CASES {
        let replay = common::parse_replay(case.replay_path);
        let (region_start, region_end) = case.region;

        // Reference: every event the full replay reports with source frame in
        // the region of interest.
        let (_full_counts, full_frames) = collect_events(&replay);
        let mut expected: BTreeMap<EventKey, usize> = BTreeMap::new();
        for (key, source_frame) in &full_frames {
            if *source_frame >= region_start && *source_frame <= region_end {
                *expected.entry(key.clone()).or_default() += 1;
            }
        }

        let lead_in = 120;
        let tail = 60;
        let clip = clip_replay_around(&replay, region_start, region_end, lead_in, tail)
            .expect("clip should build");
        let (_clip_counts, clip_frames) = collect_events(&clip.to_replay());

        // No event may land on the synthetic keyframe (clip frame < the count of
        // synthetic frames): that would be a phantom event from stale state.
        let phantom_keyframe_events: Vec<_> = clip_frames
            .iter()
            .filter(|(_, clip_frame)| *clip_frame < clip.provenance.synthetic_frame_count)
            .collect();
        assert!(
            phantom_keyframe_events.is_empty(),
            "[{}] clip emitted events on the synthetic keyframe: {phantom_keyframe_events:?}",
            case.description
        );

        // Restrict the clip's events to the same region of interest, mapped back
        // to source frames, and compare against the full-replay reference.
        let mut clip_in_region: BTreeMap<EventKey, usize> = BTreeMap::new();
        for (key, clip_frame) in &clip_frames {
            // Map clip frame -> source frame.
            if *clip_frame < clip.provenance.synthetic_frame_count {
                continue;
            }
            let source_frame = clip.provenance.source_first_real_frame
                + (clip_frame - clip.provenance.synthetic_frame_count);
            if source_frame >= region_start && source_frame <= region_end {
                *clip_in_region.entry(key.clone()).or_default() += 1;
            }
        }

        assert_eq!(
            clip_in_region, expected,
            "[{}] clip events in region {region_start}..={region_end} differ from full replay\n\
             expected (full replay): {expected:?}\n\
             got (clip):             {clip_in_region:?}",
            case.description
        );

        // Sanity: the cases were chosen to contain events; a silently empty
        // comparison would pass vacuously.
        assert!(
            !expected.is_empty(),
            "[{}] expected the window to contain at least one event",
            case.description
        );
    }
}
