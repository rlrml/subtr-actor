//! Clip-based variant of `boost_pickup_duplicate_investigation_test.rs`.
//!
//! Frame numbers shift inside a clip, so the source-replay frame anchors
//! (1960/1996) are mapped through [`ClipProvenance::clip_index_of`]; pickup
//! times are preserved from the source replay and asserted exactly. This also
//! exercises boost pad actor state (including the pickup `sequence` counter)
//! being seeded through the synthetic keyframe.

mod common;

use std::collections::HashMap;

use subtr_actor::stats::analysis_graph::{
    AnalysisNodeCollector, graph_with_builtin_analysis_nodes,
};
use subtr_actor::*;

const PROBLEMATIC_DUEL_REPLAY: &str = "assets/problematic-private-duel-2026-03-20.replay";

/// Source-replay frame of IcedSpace's original small pickup.
const ORIGINAL_PICKUP_SOURCE_FRAME: usize = 1960;
/// Source-replay frame carrying both the real pickup and the duplicate report.
const DUPLICATE_PICKUP_SOURCE_FRAME: usize = 1996;

#[derive(Clone, Debug)]
struct ReportedPickup {
    frame: usize,
    time: f32,
    pad_id: String,
    player_id: PlayerId,
    sequence: u8,
}

struct BoostPickupInvestigationCollector {
    analysis: AnalysisNodeCollector,
    player_names: HashMap<PlayerId, String>,
    reported_pickups: Vec<ReportedPickup>,
}

impl BoostPickupInvestigationCollector {
    fn new() -> Self {
        Self {
            analysis: AnalysisNodeCollector::new(
                graph_with_builtin_analysis_nodes(["boost"]).expect("boost graph should build"),
            ),
            player_names: HashMap::new(),
            reported_pickups: Vec::new(),
        }
    }
}

impl Collector for BoostPickupInvestigationCollector {
    fn process_frame(
        &mut self,
        processor: &dyn ProcessorView,
        frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        if self.player_names.is_empty() {
            let replay_meta = processor.get_replay_meta()?;
            self.player_names = replay_meta
                .team_zero
                .iter()
                .chain(replay_meta.team_one.iter())
                .map(|player| (player.remote_id.clone(), player.name.clone()))
                .collect();
        }

        let advance = self
            .analysis
            .process_frame(processor, frame, frame_number, current_time)?;
        self.reported_pickups.extend(
            processor
                .current_frame_boost_pad_events()
                .iter()
                .filter_map(|event| {
                    let BoostPadEventKind::PickedUp { sequence } = event.kind else {
                        return None;
                    };
                    Some(ReportedPickup {
                        frame: event.frame,
                        time: event.time,
                        pad_id: event.pad_id.clone(),
                        player_id: event.player.clone()?,
                        sequence,
                    })
                }),
        );
        Ok(advance)
    }

    fn finish_replay(&mut self, processor: &dyn ProcessorView) -> SubtrActorResult<()> {
        self.analysis.finish_replay(processor)
    }
}

#[test]
fn clip_suppresses_iced_space_duplicate_reported_small_boost_pickup() {
    let replay = common::parse_replay(PROBLEMATIC_DUEL_REPLAY);
    let clip = clip_replay_around(
        &replay,
        ORIGINAL_PICKUP_SOURCE_FRAME,
        DUPLICATE_PICKUP_SOURCE_FRAME,
        90,
        60,
    )
    .expect("clip should build");
    let original_clip_frame = clip
        .provenance
        .clip_index_of(ORIGINAL_PICKUP_SOURCE_FRAME)
        .expect("original pickup frame should be inside the clip");
    let duplicate_clip_frame = clip
        .provenance
        .clip_index_of(DUPLICATE_PICKUP_SOURCE_FRAME)
        .expect("duplicate pickup frame should be inside the clip");

    let collector = BoostPickupInvestigationCollector::new()
        .process_replay(&clip.to_replay())
        .expect("boost graph should process the clip");
    let boost = collector
        .analysis
        .graph()
        .state::<BoostCalculator>()
        .expect("boost calculator should be present");
    let iced_space = collector
        .player_names
        .iter()
        .find_map(|(player_id, name)| (name == "IcedSpace").then(|| player_id.clone()))
        .expect("IcedSpace should be present in clip metadata");

    // The synthetic keyframe re-emits persistent actor state but must not
    // replay transient event attributes: a stale `PickupNew` would otherwise
    // manufacture a pickup that never happened inside the clip window.
    assert!(
        !collector
            .reported_pickups
            .iter()
            .any(|pickup| pickup.frame < clip.provenance.synthetic_frame_count),
        "the synthetic keyframe must not emit phantom pickups; got {:?}",
        collector.reported_pickups
    );

    let accepted_for_iced_space = collector
        .reported_pickups
        .iter()
        .filter(|pickup| pickup.player_id == iced_space)
        .collect::<Vec<_>>();
    let graph_events_for_iced_space = boost
        .pickup_events()
        .iter()
        .filter(|event| event.player_id == iced_space)
        .collect::<Vec<_>>();

    let duplicate_pad_id = "VehiclePickup_Boost_TA_38";
    assert!(accepted_for_iced_space.iter().any(|pickup| {
        pickup.frame == original_clip_frame
            && (pickup.time - 79.389_73).abs() < 0.00001
            && pickup.pad_id == duplicate_pad_id
            && pickup.sequence == 5
    }));
    let duplicate_frame_pickups = accepted_for_iced_space
        .iter()
        .filter(|pickup| pickup.frame == duplicate_clip_frame)
        .collect::<Vec<_>>();
    assert_eq!(
        duplicate_frame_pickups.len(),
        1,
        "the duplicate frame should retain the real pickup and suppress the duplicate reported pad"
    );
    assert!(duplicate_frame_pickups.iter().any(|pickup| {
        (pickup.time - 80.636_2).abs() < 0.00001
            && pickup.pad_id == "VehiclePickup_Boost_TA_23"
            && pickup.sequence == 5
    }));
    assert!(
        !duplicate_frame_pickups
            .iter()
            .any(|pickup| pickup.pad_id == duplicate_pad_id && pickup.sequence == 5)
    );

    let graph_events_at_duplicate_frame = graph_events_for_iced_space
        .iter()
        .filter(|event| event.frame == duplicate_clip_frame)
        .collect::<Vec<_>>();
    assert_eq!(
        graph_events_at_duplicate_frame.len(),
        1,
        "the boost graph should not count both same-frame reported pickups"
    );
    let counted_pickup = graph_events_at_duplicate_frame[0];
    assert_eq!(counted_pickup.detection, BoostPickupDetection::Both);
    assert_eq!(counted_pickup.pad_type, BoostPickupPadType::Small);
    assert_eq!(counted_pickup.boost_before, Some(0.0));
    assert_eq!(counted_pickup.boost_after, Some(31.0));

    let small_graph_events = graph_events_for_iced_space
        .iter()
        .filter(|event| event.pad_type == BoostPickupPadType::Small)
        .collect::<Vec<_>>();
    let original_small_ordinal = small_graph_events
        .iter()
        .position(|event| event.frame == original_clip_frame)
        .map(|index| index + 1);
    let counted_small_ordinal = small_graph_events
        .iter()
        .position(|event| event.frame == duplicate_clip_frame)
        .map(|index| index + 1);
    assert!(
        original_small_ordinal.is_some(),
        "the original reported small pickup should be counted"
    );
    assert_eq!(
        counted_small_ordinal,
        original_small_ordinal.map(|ordinal| ordinal + 1),
        "the retained pickup should immediately follow the original small pickup"
    );
}
