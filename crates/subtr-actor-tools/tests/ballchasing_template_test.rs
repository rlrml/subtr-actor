#![allow(unused_macros)]

use std::collections::{HashMap, HashSet};

use serde_json::Value;
use subtr_actor::{
    boost_amount_to_percent, standard_soccar_boost_pad_layout, stats, BoostCalculator,
    BoostPadEventKind, BoostPadSize, Collector, FrameInput, LivePlayTracker, PlayerId, PlayerInfo,
    ReplayProcessor, StatsTimelineCollector, TimeAdvance, BOOST_KICKOFF_START_AMOUNT,
};

macro_rules! ballchasing_fixture_test {
    ($test_name:ident, $fixture_dir:literal) => {
        #[test]
        #[ignore = "Ballchasing fixtures are opt-in and should be enabled fixture-by-fixture"]
        fn $test_name() {
            let report = subtr_actor_tools::ballchasing::compare_fixture_directory(
                &asset_path($fixture_dir),
                &subtr_actor_tools::ballchasing::recommended_match_config(),
            )
            .expect("Failed to compare Ballchasing fixture");
            report.assert_matches();
        }
    };
}

ballchasing_fixture_test!(
    compare_recent_ranked_doubles_2026_03_10,
    "recent-ranked-doubles-2026-03-10"
);

ballchasing_fixture_test!(
    compare_recent_ranked_standard_2026_03_10_a,
    "recent-ranked-standard-2026-03-10-a"
);

ballchasing_fixture_test!(
    compare_recent_ranked_standard_2026_03_10_b,
    "recent-ranked-standard-2026-03-10-b"
);

fn asset_path(path: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets")
        .join(path)
}

fn parse_replay(path: &str) -> boxcars::Replay {
    let path = asset_path(path);
    let data = std::fs::read(&path)
        .unwrap_or_else(|_| panic!("Failed to read replay file: {}", path.display()));
    boxcars::ParserBuilder::new(&data)
        .must_parse_network_data()
        .always_check_crc()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay file: {}", path.display()))
}

fn json_u32(value: &Value, path: &str) -> u32 {
    value
        .pointer(path)
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or_else(|| panic!("Expected u32 at JSON pointer {path}"))
}

fn expected_player_big_pad_count(ballchasing: &Value, team_key: &str, player_name: &str) -> u32 {
    let players = ballchasing
        .pointer(&format!("/{team_key}/players"))
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("Expected players array for team {team_key}"));
    let player = players
        .iter()
        .find(|player| player.get("name").and_then(Value::as_str) == Some(player_name))
        .unwrap_or_else(|| panic!("Expected Ballchasing player {team_key}.{player_name}"));
    json_u32(player, "/stats/boost/count_collected_big")
}

fn expected_team_pad_count(ballchasing: &Value, team_key: &str, field: &str) -> u32 {
    json_u32(ballchasing, &format!("/{team_key}/stats/boost/{field}"))
}

fn describe_inactive_inclusive_team_pad_counts_against_ballchasing(
    boost: &BoostCalculator,
    ballchasing: &Value,
    team_key: &str,
    fixture_name: &str,
) {
    let team_stats = match team_key {
        "blue" => boost.team_zero_stats(),
        "orange" => boost.team_one_stats(),
        _ => panic!("Unexpected team key {team_key}"),
    };
    let actual_big = team_stats.big_pads_collected + team_stats.big_pads_collected_inactive;
    let actual_small = team_stats.small_pads_collected + team_stats.small_pads_collected_inactive;
    let expected_big = expected_team_pad_count(ballchasing, team_key, "count_collected_big");
    let expected_small = expected_team_pad_count(ballchasing, team_key, "count_collected_small");

    eprintln!(
        "{fixture_name} {team_key}: \
         big actual={} expected={} delta={} (active={} inactive={}); \
         small actual={} expected={} delta={} (active={} inactive={})",
        actual_big,
        expected_big,
        i64::from(actual_big) - i64::from(expected_big),
        team_stats.big_pads_collected,
        team_stats.big_pads_collected_inactive,
        actual_small,
        expected_small,
        i64::from(actual_small) - i64::from(expected_small),
        team_stats.small_pads_collected,
        team_stats.small_pads_collected_inactive,
    );
}

fn assert_inactive_inclusive_big_pad_counts_match_ballchasing(
    boost: &BoostCalculator,
    players: &[PlayerInfo],
    ballchasing: &Value,
    team_key: &str,
) {
    let mut actual_team_count = 0;
    let mut expected_team_count = 0;

    for player in players {
        let player_stats = boost
            .player_stats()
            .get(&player.remote_id)
            .unwrap_or_else(|| panic!("Expected boost stats for {}", player.name));
        let actual = player_stats.big_pads_collected + player_stats.big_pads_collected_inactive;
        let expected = expected_player_big_pad_count(ballchasing, team_key, &player.name);
        assert_eq!(
            actual, expected,
            "inactive-inclusive big pad count mismatch for {team_key}.{}: \
             active={} inactive={}",
            player.name, player_stats.big_pads_collected, player_stats.big_pads_collected_inactive,
        );
        actual_team_count += actual;
        expected_team_count += expected;
    }

    assert_eq!(
        actual_team_count,
        json_u32(
            ballchasing,
            &format!("/{team_key}/stats/boost/count_collected_big")
        ),
        "inactive-inclusive team big pad count should match Ballchasing team stat for {team_key}"
    );
    assert_eq!(
        actual_team_count, expected_team_count,
        "inactive-inclusive team big pad count should equal player total for {team_key}"
    );
}

#[test]
fn problematic_private_duel_big_pad_counts_match_ballchasing_with_inactive_pickups() {
    let replay = parse_replay("problematic-private-duel-2026-03-20.replay");
    let ballchasing: Value = serde_json::from_slice(
        &std::fs::read(asset_path(
            "problematic-private-duel-2026-03-20.ballchasing.json",
        ))
        .expect("Failed to read Ballchasing JSON fixture"),
    )
    .expect("Failed to parse Ballchasing JSON fixture");
    let replay_meta = ReplayProcessor::new(&replay)
        .expect("Expected replay processor")
        .get_replay_meta()
        .expect("Expected replay metadata");
    let graph =
        stats::analysis_graph::collect_builtin_analysis_graph_for_replay(&replay, ["boost"])
            .expect("Expected boost analysis graph to process replay");
    let boost = graph
        .state::<BoostCalculator>()
        .expect("Expected boost calculator state");

    assert_inactive_inclusive_big_pad_counts_match_ballchasing(
        boost,
        &replay_meta.team_zero,
        &ballchasing,
        "blue",
    );
    assert_inactive_inclusive_big_pad_counts_match_ballchasing(
        boost,
        &replay_meta.team_one,
        &ballchasing,
        "orange",
    );
}

#[test]
#[ignore = "Diagnostic output for inactive-inclusive team pad count deltas across Ballchasing fixtures."]
fn describe_ballchasing_fixture_team_pad_count_deltas_with_inactive_pickups() {
    for fixture_name in [
        "problematic-private-duel-2026-03-20",
        "recent-ranked-doubles-2026-03-10",
        "recent-ranked-standard-2026-03-10-a",
        "recent-ranked-standard-2026-03-10-b",
    ] {
        let replay = parse_replay(&format!("{fixture_name}.replay"));
        let ballchasing: Value = serde_json::from_slice(
            &std::fs::read(asset_path(&format!("{fixture_name}.ballchasing.json")))
                .unwrap_or_else(|_| panic!("Failed to read Ballchasing JSON for {fixture_name}")),
        )
        .unwrap_or_else(|_| panic!("Failed to parse Ballchasing JSON for {fixture_name}"));
        let graph =
            stats::analysis_graph::collect_builtin_analysis_graph_for_replay(&replay, ["boost"])
                .unwrap_or_else(|_| panic!("Expected boost analysis graph for {fixture_name}"));
        let boost = graph
            .state::<BoostCalculator>()
            .unwrap_or_else(|| panic!("Expected boost calculator state for {fixture_name}"));

        describe_inactive_inclusive_team_pad_counts_against_ballchasing(
            boost,
            &ballchasing,
            "blue",
            fixture_name,
        );
        describe_inactive_inclusive_team_pad_counts_against_ballchasing(
            boost,
            &ballchasing,
            "orange",
            fixture_name,
        );
    }
}

#[derive(Clone)]
struct PickupSequenceObservation {
    time: f32,
    frame: usize,
    player: String,
    phase: String,
    pad_size: Option<BoostPadSize>,
}

#[derive(Default)]
struct PickupSequenceReportCollector {
    live_play_tracker: LivePlayTracker,
    previous_time: Option<f32>,
    last_by_key: HashMap<(String, u8), PickupSequenceObservation>,
    repeated_sequences: Vec<(
        (String, u8),
        PickupSequenceObservation,
        PickupSequenceObservation,
    )>,
}

fn nearest_standard_pad_size(position: glam::Vec3) -> Option<BoostPadSize> {
    standard_soccar_boost_pad_layout()
        .iter()
        .min_by(|(left_position, _), (right_position, _)| {
            position
                .distance_squared(*left_position)
                .partial_cmp(&position.distance_squared(*right_position))
                .unwrap()
        })
        .map(|(_, size)| *size)
}

impl Collector for PickupSequenceReportCollector {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        _frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> subtr_actor::SubtrActorResult<TimeAdvance> {
        let dt = self
            .previous_time
            .map(|previous_time| current_time - previous_time)
            .unwrap_or(0.0);
        self.previous_time = Some(current_time);
        let input = FrameInput::timeline(processor, frame_number, current_time, dt);
        let gameplay = input.gameplay_state();
        let events = input.frame_events_state();
        let players = input.player_frame_state();
        let live_play = self.live_play_tracker.state_parts(&gameplay, &events);

        for event in &events.boost_pad_events {
            let BoostPadEventKind::PickedUp { sequence } = event.kind else {
                continue;
            };
            let player = event
                .player
                .as_ref()
                .and_then(|player_id| players.players.iter().find(|p| &p.player_id == player_id));
            let player_name = event
                .player
                .as_ref()
                .and_then(|player_id| processor.get_player_name(player_id).ok())
                .unwrap_or_else(|| "<unknown>".to_string());
            let observation = PickupSequenceObservation {
                time: event.time,
                frame: event.frame,
                player: player_name,
                phase: format!("{:?}", live_play.gameplay_phase),
                pad_size: player
                    .and_then(|player| player.position())
                    .and_then(nearest_standard_pad_size),
            };
            let key = (event.pad_id.clone(), sequence);
            if let Some(previous) = self.last_by_key.insert(key.clone(), observation.clone()) {
                self.repeated_sequences.push((key, previous, observation));
            }
        }

        Ok(TimeAdvance::NextFrame)
    }
}

#[test]
#[ignore = "Diagnostic output for repeated boost pickup (pad_id, sequence) keys in the problematic replay."]
fn describe_problematic_private_duel_repeated_boost_pickup_sequences() {
    let replay = parse_replay("problematic-private-duel-2026-03-20.replay");
    let report = PickupSequenceReportCollector::default()
        .process_replay(&replay)
        .expect("Expected replay processing to produce sequence report");

    for ((pad_id, sequence), previous, current) in report.repeated_sequences {
        let gap = current.time - previous.time;
        if gap < 10.0 {
            continue;
        }
        eprintln!(
            "pad_id={pad_id} sequence={sequence} gap={:.3}s | \
             previous: t={:.3} frame={} player={} phase={} size={:?} | \
             current: t={:.3} frame={} player={} phase={} size={:?}",
            gap,
            previous.time,
            previous.frame,
            previous.player,
            previous.phase,
            previous.pad_size,
            current.time,
            current.frame,
            current.player,
            current.phase,
            current.pad_size,
        );
    }
}

#[derive(Clone)]
struct BoostIncreaseObservation {
    time: f32,
    frame: usize,
    player: String,
    player_id: PlayerId,
    previous_boost: f32,
    boost: f32,
    phase: String,
    nearest_pad_size: Option<BoostPadSize>,
    pickup_events: Vec<String>,
    respawn_notes: Vec<String>,
}

#[derive(Clone)]
struct BoostPickupEventObservation {
    time: f32,
    frame: usize,
    player_id: PlayerId,
    pad_id: String,
    sequence: u8,
    nearest_pad_size: Option<BoostPadSize>,
}

#[derive(Default)]
struct BoostIncreaseReportCollector {
    live_play_tracker: LivePlayTracker,
    previous_time: Option<f32>,
    previous_boost_by_player: HashMap<PlayerId, f32>,
    pending_demo_respawns: HashSet<PlayerId>,
    observations: Vec<BoostIncreaseObservation>,
    pickup_events: Vec<BoostPickupEventObservation>,
}

impl Collector for BoostIncreaseReportCollector {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        _frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> subtr_actor::SubtrActorResult<TimeAdvance> {
        let dt = self
            .previous_time
            .map(|previous_time| current_time - previous_time)
            .unwrap_or(0.0);
        self.previous_time = Some(current_time);
        let input = FrameInput::timeline(processor, frame_number, current_time, dt);
        let gameplay = input.gameplay_state();
        let events = input.frame_events_state();
        let players = input.player_frame_state();
        let live_play = self.live_play_tracker.state_parts(&gameplay, &events);

        for demo in &events.demo_events {
            self.pending_demo_respawns.insert(demo.victim.clone());
        }

        for event in &events.boost_pad_events {
            let BoostPadEventKind::PickedUp { sequence } = event.kind else {
                continue;
            };
            let Some(player_id) = &event.player else {
                continue;
            };
            let player = players
                .players
                .iter()
                .find(|player| &player.player_id == player_id);
            self.pickup_events.push(BoostPickupEventObservation {
                time: event.time,
                frame: event.frame,
                player_id: player_id.clone(),
                pad_id: event.pad_id.clone(),
                sequence,
                nearest_pad_size: player
                    .and_then(|player| player.position())
                    .and_then(nearest_standard_pad_size),
            });
        }

        for player in &players.players {
            let Some(boost) = player.boost_amount else {
                continue;
            };
            let Some(previous_boost) = self
                .previous_boost_by_player
                .insert(player.player_id.clone(), boost)
            else {
                continue;
            };
            let boost_delta = boost - previous_boost;
            if boost_delta <= 1.0 {
                continue;
            }
            let player_name = processor.get_player_name(&player.player_id)?;
            let pickup_events = events
                .boost_pad_events
                .iter()
                .filter(|event| event.player.as_ref() == Some(&player.player_id))
                .map(|event| match event.kind {
                    BoostPadEventKind::PickedUp { sequence } => {
                        format!("{}#{}", event.pad_id, sequence)
                    }
                    BoostPadEventKind::Available => format!("{} available", event.pad_id),
                })
                .collect();
            let mut respawn_notes = Vec::new();
            if gameplay.kickoff_phase_active() && (boost - BOOST_KICKOFF_START_AMOUNT).abs() <= 1.0
            {
                respawn_notes.push("kickoff_respawn_candidate".to_string());
            }
            if self.pending_demo_respawns.contains(&player.player_id) && player.rigid_body.is_some()
            {
                respawn_notes.push("demo_respawn_candidate".to_string());
                self.pending_demo_respawns.remove(&player.player_id);
            }
            self.observations.push(BoostIncreaseObservation {
                time: current_time,
                frame: frame_number,
                player: player_name,
                player_id: player.player_id.clone(),
                previous_boost,
                boost,
                phase: format!("{:?}", live_play.gameplay_phase),
                nearest_pad_size: player.position().and_then(nearest_standard_pad_size),
                pickup_events,
                respawn_notes,
            });
        }

        Ok(TimeAdvance::NextFrame)
    }
}

#[test]
#[ignore = "Diagnostic output for boost-level increases in the problematic replay."]
fn describe_problematic_private_duel_boost_increase_candidates() {
    let replay = parse_replay("problematic-private-duel-2026-03-20.replay");
    let report = BoostIncreaseReportCollector::default()
        .process_replay(&replay)
        .expect("Expected replay processing to produce boost increase report");
    let timeline = StatsTimelineCollector::new()
        .get_replay_data(&replay)
        .expect("Expected stats timeline to process replay");
    let mut previous_big_pad_count = None;
    let mut big_pad_count_increments = Vec::new();
    for frame in &timeline.frames {
        let Some(player) = frame
            .players
            .iter()
            .find(|player| player.name == "IcedSpace")
        else {
            continue;
        };
        let big_pad_count =
            player.boost.big_pads_collected + player.boost.big_pads_collected_inactive;
        let Some(previous_count) = previous_big_pad_count.replace(big_pad_count) else {
            continue;
        };
        if big_pad_count > previous_count {
            big_pad_count_increments.push((
                frame.time,
                frame.frame_number,
                previous_count,
                big_pad_count,
            ));
        }
    }
    eprintln!("IcedSpace inactive-inclusive big-pad count increments:");
    for (time, frame, previous_count, current_count) in &big_pad_count_increments {
        eprintln!("  t={time:.3} frame={frame} {previous_count}->{current_count}");
    }

    for observation in report
        .observations
        .iter()
        .filter(|observation| observation.player == "IcedSpace")
    {
        let delta = observation.boost - observation.previous_boost;
        if observation.nearest_pad_size != Some(BoostPadSize::Big)
            || delta < 20.0
            || !observation.respawn_notes.is_empty()
        {
            continue;
        }
        let pickup_events = if observation.pickup_events.is_empty() {
            "<none>".to_string()
        } else {
            observation.pickup_events.join(", ")
        };
        let nearby_pickup_events = report
            .pickup_events
            .iter()
            .filter(|event| {
                event.player_id == observation.player_id
                    && (event.time - observation.time).abs() <= 0.25
            })
            .map(|event| {
                format!(
                    "dt={:+.3}s frame={} {}#{} nearest={:?}",
                    event.time - observation.time,
                    event.frame,
                    event.pad_id,
                    event.sequence,
                    event.nearest_pad_size,
                )
            })
            .collect::<Vec<_>>();
        let nearby_pickup_events = if nearby_pickup_events.is_empty() {
            "<none>".to_string()
        } else {
            nearby_pickup_events.join(", ")
        };
        let nearby_count_increments = big_pad_count_increments
            .iter()
            .filter(|(time, _, _, _)| (*time - observation.time).abs() <= 0.25)
            .map(|(time, frame, previous_count, current_count)| {
                format!(
                    "dt={:+.3}s frame={} {}->{}",
                    *time - observation.time,
                    frame,
                    previous_count,
                    current_count
                )
            })
            .collect::<Vec<_>>();
        let nearby_count_increments = if nearby_count_increments.is_empty() {
            "<none>".to_string()
        } else {
            nearby_count_increments.join(", ")
        };
        let respawn_notes = if observation.respawn_notes.is_empty() {
            "<none>".to_string()
        } else {
            observation.respawn_notes.join(", ")
        };
        eprintln!(
            "t={:.3} frame={} player={} phase={} boost={:.1}->{:.1} \
             delta={:.1} ({:.1}pp) nearest_size={:?} pickup_events=[{}] \
             nearby_pickup_events=[{}] nearby_count_increments=[{}] respawn_notes=[{}]",
            observation.time,
            observation.frame,
            observation.player,
            observation.phase,
            observation.previous_boost,
            observation.boost,
            delta,
            boost_amount_to_percent(delta),
            observation.nearest_pad_size,
            pickup_events,
            nearby_pickup_events,
            nearby_count_increments,
            respawn_notes,
        );
    }
}
