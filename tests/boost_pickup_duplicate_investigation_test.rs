use std::collections::HashMap;

use subtr_actor::stats::analysis_graph::{
    graph_with_builtin_analysis_nodes, AnalysisNodeCollector,
};
use subtr_actor::*;

fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).unwrap_or_else(|_| panic!("Failed to read replay file: {path}"));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay: {path}"))
}

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
        processor: &ReplayProcessor,
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

    fn finish_replay(&mut self, processor: &ReplayProcessor) -> SubtrActorResult<()> {
        self.analysis.finish_replay(processor)
    }
}

#[test]
fn iced_space_duplicate_reported_small_boost_pickup_is_suppressed_by_processor() {
    let replay = parse_replay("assets/problematic-private-duel-2026-03-20.replay");
    let collector = BoostPickupInvestigationCollector::new()
        .process_replay(&replay)
        .expect("boost graph should process replay");
    let boost = collector
        .analysis
        .graph()
        .state::<BoostCalculator>()
        .expect("boost calculator should be present");
    let iced_space = collector
        .player_names
        .iter()
        .find_map(|(player_id, name)| (name == "IcedSpace").then(|| player_id.clone()))
        .expect("IcedSpace should be present in replay metadata");

    let accepted_for_iced_space = collector
        .reported_pickups
        .iter()
        .filter(|pickup| pickup.player_id == iced_space)
        .collect::<Vec<_>>();
    let graph_events_for_iced_space = boost
        .pickup_comparison_events()
        .iter()
        .filter(|event| event.player_id == iced_space)
        .collect::<Vec<_>>();

    let duplicate_pad_id = "VehiclePickup_Boost_TA_38";
    assert!(accepted_for_iced_space.iter().any(|pickup| {
        pickup.frame == 1960
            && (pickup.time - 79.389_73).abs() < 0.00001
            && pickup.pad_id == duplicate_pad_id
            && pickup.sequence == 5
    }));
    let frame_1996_pickups = accepted_for_iced_space
        .iter()
        .filter(|pickup| pickup.frame == 1996)
        .collect::<Vec<_>>();
    assert_eq!(
        frame_1996_pickups.len(),
        1,
        "frame 1996 should retain the real pickup and suppress the duplicate reported pad"
    );
    assert!(frame_1996_pickups.iter().any(|pickup| {
        (pickup.time - 80.636_2).abs() < 0.00001
            && pickup.pad_id == "VehiclePickup_Boost_TA_23"
            && pickup.sequence == 5
    }));
    assert!(!frame_1996_pickups
        .iter()
        .any(|pickup| pickup.pad_id == duplicate_pad_id && pickup.sequence == 5));

    let graph_events_at_frame_1996 = graph_events_for_iced_space
        .iter()
        .filter(|event| event.reported_frame == Some(1996))
        .collect::<Vec<_>>();
    assert_eq!(
        graph_events_at_frame_1996.len(),
        1,
        "the boost graph should not count both same-frame reported pickups"
    );
    let counted_pickup = graph_events_at_frame_1996[0];
    assert_eq!(counted_pickup.comparison, BoostPickupComparison::Both);
    assert_eq!(counted_pickup.pad_type, BoostPickupPadType::Small);
    assert_eq!(counted_pickup.boost_before, Some(0.0));
    assert_eq!(counted_pickup.boost_after, Some(31.0));

    let small_graph_events = graph_events_for_iced_space
        .iter()
        .filter(|event| event.pad_type == BoostPickupPadType::Small)
        .collect::<Vec<_>>();
    let original_small_ordinal = small_graph_events
        .iter()
        .position(|event| event.reported_frame == Some(1960))
        .map(|index| index + 1);
    let counted_small_ordinal = small_graph_events
        .iter()
        .position(|event| event.reported_frame == Some(1996))
        .map(|index| index + 1);
    assert_eq!(original_small_ordinal, Some(18));
    assert_eq!(counted_small_ordinal, Some(19));
}
