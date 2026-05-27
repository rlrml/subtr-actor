mod common;

use std::collections::HashSet;

use common::parse_replay;
use subtr_actor::*;

fn max_abs_player_position_from_replay_data(replay_data: &ReplayData) -> f32 {
    replay_data
        .frame_data
        .players
        .iter()
        .flat_map(|(_, player_data)| player_data.frames().iter())
        .filter_map(|frame| match frame {
            PlayerFrame::Data { rigid_body, .. } => Some(rigid_body.location),
            PlayerFrame::Empty => None,
        })
        .flat_map(|location| [location.x.abs(), location.y.abs(), location.z.abs()])
        .fold(0.0f32, f32::max)
}

/// Test that all sample replays can be parsed and processed without errors
#[test]
fn test_all_replays_parse_successfully() {
    let replays = [
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
        "assets/replay-format-2020-09-25-v868-29-net10-tournament.replay",
        "assets/replay-format-2016-07-21-v868-12-net-none-lan.replay",
    ];

    for path in replays {
        let replay = parse_replay(path);
        assert!(
            replay.network_frames.is_some(),
            "Replay {path} should have network frames",
        );
    }
}

/// Test ReplayDataCollector works on known-good replays
#[test]
fn test_replay_data_collector_multiple_replays() {
    // Use replays that are known to work with the collector
    let replays = [
        "assets/replay-format-2022-09-29-v868-32-net10-legacy-boost.replay",
        "assets/replay-format-2025-06-10-v868-32-net10-replicated-boost.replay",
    ];

    for path in replays {
        let replay = parse_replay(path);
        let collector = ReplayDataCollector::new();

        let replay_data = collector
            .get_replay_data(&replay)
            .unwrap_or_else(|_| panic!("Failed to get replay data for {path}"));

        // Verify we got meaningful data
        assert!(
            replay_data.frame_data.frame_count() > 0,
            "Replay {path} should have frames",
        );
        assert!(
            replay_data.frame_data.duration() > 0.0,
            "Replay {path} should have positive duration",
        );

        // Verify JSON serialization works
        let json = replay_data
            .as_json()
            .expect("JSON serialization should work");
        assert!(!json.is_empty(), "JSON should not be empty");
    }
}

#[test]
fn test_replay_data_exposes_powerslide_activity() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let replay_data = ReplayDataCollector::new().get_replay_data(&replay).expect(
        "Failed to get replay data for replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
    );

    let mut powerslide_true_count = 0usize;
    let mut powerslide_false_count = 0usize;

    for (_, player_data) in &replay_data.frame_data.players {
        for frame in player_data.frames() {
            if let PlayerFrame::Data {
                powerslide_active, ..
            } = frame
            {
                if *powerslide_active {
                    powerslide_true_count += 1;
                } else {
                    powerslide_false_count += 1;
                }
            }
        }
    }

    assert!(
        powerslide_true_count > 0,
        "Expected replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay to contain at least one powerslide-active frame"
    );
    assert!(
        powerslide_false_count > 0,
        "Expected replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay to contain at least one non-powerslide frame"
    );
    assert!(
        !replay_data.touch_events.is_empty(),
        "Expected replay data to expose exact touch events"
    );
    assert!(
        !replay_data.goal_events.is_empty(),
        "Expected replay data to expose exact goal events"
    );
    assert!(
        !replay_data.player_stat_events.is_empty(),
        "Expected replay data to expose exact player stat events"
    );
}

#[test]
fn test_legacy_replays_use_spatial_normalization() {
    for path in [
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
        "assets/replay-format-2016-07-21-v868-12-net-none-lan.replay",
    ] {
        let replay = parse_replay(path);
        let processor = ReplayProcessor::new(&replay).expect("Failed to construct processor");
        assert_eq!(
            processor.spatial_normalization_factor(),
            100.0,
            "Expected legacy replay {path} to use 100x spatial normalization"
        );
        assert_eq!(
            processor.rigid_body_velocity_normalization_factor(),
            10.0,
            "Expected legacy replay {path} to use 10x rigid-body velocity normalization"
        );
    }
}

#[test]
fn test_modern_replays_keep_native_spatial_scale() {
    for path in [
        "assets/replay-format-2022-09-29-v868-32-net10-legacy-boost.replay",
        "assets/replay-format-2025-06-10-v868-32-net10-replicated-boost.replay",
        "assets/replay-format-2020-09-25-v868-29-net10-tournament.replay",
        "assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay",
    ] {
        let replay = parse_replay(path);
        let processor = ReplayProcessor::new(&replay).expect("Failed to construct processor");
        assert_eq!(
            processor.spatial_normalization_factor(),
            1.0,
            "Expected modern replay {path} to keep native spatial scale"
        );
        assert_eq!(
            processor.rigid_body_velocity_normalization_factor(),
            1.0,
            "Expected modern replay {path} to keep native rigid-body velocity scale"
        );
    }
}

#[test]
fn test_legacy_replay_player_positions_are_normalized_to_field_units() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let replay_data = ReplayDataCollector::new().get_replay_data(&replay).expect(
        "Failed to get replay data for replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
    );
    let max_abs_player_position = max_abs_player_position_from_replay_data(&replay_data);
    assert!(
        max_abs_player_position > 1000.0,
        "Expected normalized player positions for replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay to reach Rocket League field units, got {max_abs_player_position}"
    );
    assert!(
        max_abs_player_position < 10000.0,
        "Expected normalized player positions for replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay to stay within plausible Rocket League field bounds, got {max_abs_player_position}"
    );
}

#[test]
fn test_old_replay_with_substitutions_discovers_late_players() {
    let replay = parse_replay("assets/old-ballchasing-midfield-car.replay");
    let replay_data = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .expect("Failed to get replay data for old substitution replay");

    let names: HashSet<_> = replay_data
        .meta
        .player_order()
        .map(|player| player.name.as_str())
        .collect();
    for expected_name in [
        "CritRomney",
        "DatLilBabyG",
        "b_corner",
        "Raptor_Attacks_",
        "jboy42069",
        "remrocker29",
        "a093q262",
        "Q-money219",
    ] {
        assert!(
            names.contains(expected_name),
            "Expected replay metadata to include late player {expected_name}, got {names:?}"
        );
    }

    assert_eq!(
        replay_data.meta.player_count(),
        replay_data.frame_data.players.len(),
        "Expected frame data to include one player series per metadata player"
    );
    assert!(
        replay_data
            .frame_data
            .players
            .iter()
            .all(|(_, player_data)| player_data.frames().iter().any(|frame| {
                matches!(frame, PlayerFrame::Data { rigid_body, .. } if !rigid_body.sleeping)
            })),
        "Expected every discovered player to have at least one non-empty frame"
    );

    assert!(
        replay_data
            .frame_data
            .players
            .iter()
            .any(|(_, player_data)| player_data.frames().iter().any(|frame| {
                matches!(frame, PlayerFrame::Data { rigid_body, .. } if rigid_body.sleeping)
            })),
        "Expected replay data export to preserve sleeping player positions"
    );

    let early_positioned_players = replay_data
        .frame_data
        .players
        .iter()
        .filter(|(_, player_data)| {
            player_data
                .frames()
                .iter()
                .take(10)
                .any(|frame| matches!(frame, PlayerFrame::Data { .. }))
        })
        .count();
    assert!(
        early_positioned_players >= 6,
        "Expected bootstrap player/car mappings to expose the starting roster early, got {early_positioned_players}"
    );
}

#[test]
fn test_modern_replay_player_positions_are_not_overscaled() {
    let replay = parse_replay("assets/replay-format-2022-09-29-v868-32-net10-legacy-boost.replay");
    let replay_data = ReplayDataCollector::new().get_replay_data(&replay).expect(
        "Failed to get replay data for replay-format-2022-09-29-v868-32-net10-legacy-boost.replay",
    );
    let max_abs_player_position = max_abs_player_position_from_replay_data(&replay_data);
    assert!(
        max_abs_player_position > 1000.0,
        "Expected modern replay positions to remain in Rocket League field units, got {max_abs_player_position}"
    );
    assert!(
        max_abs_player_position < 10000.0,
        "Expected modern replay positions not to be multiplied again, got {max_abs_player_position}"
    );
}

#[test]
fn test_processor_extracts_exact_boost_pad_events() {
    let replay =
        parse_replay("assets/replay-format-2025-06-10-v868-32-net10-replicated-boost.replay");
    let mut processor = ReplayProcessor::new(&replay).expect("Failed to construct processor");
    let mut counter = FrameCounter::new();
    processor
        .process(&mut counter)
        .expect("Failed to process replay for boost pad extraction");

    assert!(
        processor
            .boost_pad_events
            .iter()
            .any(|event| matches!(event.kind, BoostPadEventKind::PickedUp { .. })),
        "Expected at least one exact boost pickup event"
    );
    assert!(
        processor
            .boost_pad_events
            .iter()
            .any(|event| matches!(event.kind, BoostPadEventKind::Available)),
        "Expected at least one boost pad availability event"
    );
    assert!(
        processor
            .boost_pad_events
            .iter()
            .any(|event| event.pad_id.starts_with("VehiclePickup_Boost_TA_")),
        "Expected boost pad events to keep stable per-pad instance identifiers"
    );
    assert!(
        processor
            .boost_pad_events
            .iter()
            .any(|event| event.player.is_some()),
        "Expected at least one boost pad event to resolve to a player"
    );
}

#[test]
fn test_replay_data_exposes_exact_dodge_refresh_events() {
    let replay =
        parse_replay("assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay");
    let replay_data = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .expect("Failed to get replay data for replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay");

    assert!(
        !replay_data.dodge_refreshed_events.is_empty(),
        "Expected replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay to expose at least one exact dodge refresh event"
    );
    assert!(
        replay_data.dodge_refreshed_events.len() == 12,
        "Expected replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay to expose the known 12 exact dodge refresh events"
    );
    assert!(
        replay_data
            .dodge_refreshed_events
            .iter()
            .all(|event| event.counter_value >= 1),
        "Expected dodge refresh counter values to be positive event counts"
    );
    assert!(
        replay_data.dodge_refreshed_events.iter().all(|event| {
            replay_data
                .meta
                .player_order()
                .any(|player| player.remote_id == event.player)
        }),
        "Expected dodge refresh events to resolve to known replay players"
    );
    let unique_counter_values = replay_data
        .dodge_refreshed_events
        .iter()
        .map(|event| event.counter_value)
        .collect::<HashSet<_>>();
    assert_eq!(
        unique_counter_values,
        HashSet::from([1, 2, 3]),
        "Expected replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay to expose the known counter increments"
    );
}

#[test]
fn test_processor_extracts_exact_goal_events() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let mut processor = ReplayProcessor::new(&replay).expect("Failed to construct processor");
    let mut counter = FrameCounter::new();
    processor
        .process(&mut counter)
        .expect("Failed to process replay for goal extraction");

    assert!(
        !processor.goal_events.is_empty(),
        "Expected at least one exact goal event"
    );

    let replay_meta = processor
        .get_replay_meta()
        .expect("Expected replay metadata after processing");
    let total_goals = replay_meta
        .player_order()
        .filter_map(|player| player.stats.as_ref())
        .filter_map(|stats| match stats.get("Goals") {
            Some(boxcars::HeaderProp::Int(value)) => Some(*value),
            _ => None,
        })
        .sum::<i32>();
    assert_eq!(
        processor.goal_events.len(),
        total_goals as usize,
        "Expected one deduplicated goal event per scored goal"
    );
    assert!(
        processor
            .goal_events
            .iter()
            .any(|event| event.player.is_some()),
        "Expected at least some exact goal events to resolve a scorer directly from frame updates"
    );
    let scorer_count = processor
        .goal_events
        .iter()
        .filter(|event| event.player.is_some())
        .count();
    assert!(
        scorer_count * 2 >= processor.goal_events.len(),
        "Expected scorer extraction to cover at least half of the goal events in replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay, found {scorer_count}/{}",
        processor.goal_events.len(),
    );
    for event in processor
        .goal_events
        .iter()
        .filter(|event| event.player.is_some())
    {
        let scorer = event
            .player
            .as_ref()
            .expect("Filtered to goal events with scorers");
        assert_eq!(
            processor.get_player_is_team_0(scorer).ok(),
            Some(event.scoring_team_is_team_0),
            "Expected resolved goal scorer to be on the scoring team"
        );
    }
    let goal_scores: Vec<(i32, i32)> = processor
        .goal_events
        .iter()
        .filter_map(|event| event.team_zero_score.zip(event.team_one_score))
        .collect();
    assert_eq!(
        goal_scores.len(),
        processor.goal_events.len(),
        "Expected exact goal events in replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay to carry score tuples for score-aware dedupe"
    );
    for window in goal_scores.windows(2) {
        let previous_total = window[0].0 + window[0].1;
        let current_total = window[1].0 + window[1].1;
        assert_eq!(
            current_total,
            previous_total + 1,
            "Expected deduplicated goal events to advance the total score by exactly one"
        );
    }
    let mut previous_score = (0, 0);
    for event in &processor.goal_events {
        let (team_zero_score, team_one_score) = event
            .team_zero_score
            .zip(event.team_one_score)
            .expect("Expected all goal events to carry score tuples");
        let expected_scoring_team = if team_zero_score == previous_score.0 + 1 {
            Some(true)
        } else if team_one_score == previous_score.1 + 1 {
            Some(false)
        } else {
            None
        };
        assert_eq!(
            expected_scoring_team,
            Some(event.scoring_team_is_team_0),
            "Expected goal side to agree with score tuple progression"
        );
        previous_score = (team_zero_score, team_one_score);
    }
}

#[test]
fn test_processor_extracts_touch_events() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let mut processor = ReplayProcessor::new(&replay).expect("Failed to construct processor");
    let mut counter = FrameCounter::new();
    processor
        .process(&mut counter)
        .expect("Failed to process replay for touch extraction");

    assert!(
        processor.touch_events.len() > 100,
        "Expected many touch events from HitTeamNum updates"
    );
    assert!(
        processor
            .touch_events
            .iter()
            .any(|event| event.player.is_some()),
        "Expected at least some touch events to resolve to a player"
    );
    assert!(
        processor
            .touch_events
            .iter()
            .filter_map(|event| event.closest_approach_distance)
            .all(|distance| distance <= 700.0),
        "Expected attributed touch distances to respect the heuristic threshold"
    );
    assert!(
        processor
            .touch_events
            .windows(2)
            .any(|window| window[0].team_is_team_0 == window[1].team_is_team_0),
        "Expected same-team consecutive touch events, not just team changes"
    );
}

#[test]
fn test_processor_extracts_flip_reset_events() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let mut processor = ReplayProcessor::new(&replay).expect("Failed to construct processor");
    let mut tracker = FlipResetTracker::new();
    processor
        .process(&mut tracker)
        .expect("Failed to process replay for flip-reset extraction");

    assert!(
        !tracker.flip_reset_events().is_empty(),
        "Expected the heuristic to find at least one flip-reset candidate in replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay"
    );
    assert!(
        tracker
            .flip_reset_events()
            .iter()
            .all(|event| (0.0..=1.0).contains(&event.confidence)),
        "Expected heuristic confidence to stay normalized"
    );
    assert!(
        tracker
            .flip_reset_events()
            .iter()
            .all(|event| event.closest_approach_distance <= 220.0),
        "Expected flip-reset candidates to be backed by close attributed touches in Rocket League units"
    );
    assert!(
        tracker.flip_reset_events().iter().all(|event| {
            processor.get_player_is_team_0(&event.player).ok() == Some(event.is_team_0)
        }),
        "Expected flip-reset candidate team labels to agree with the resolved player team"
    );
}

#[test]
fn test_processor_extracts_post_wall_dodge_events() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let mut processor = ReplayProcessor::new(&replay).expect("Failed to construct processor");
    let mut tracker = FlipResetTracker::new();
    processor
        .process(&mut tracker)
        .expect("Failed to process replay for post-wall dodge extraction");

    assert!(
        !tracker.post_wall_dodge_events().is_empty(),
        "Expected the heuristic to find at least one post-wall dodge in replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay"
    );
    assert!(
        tracker
            .post_wall_dodge_events()
            .iter()
            .all(|event| event.time_since_wall_contact >= 0.20),
        "Expected post-wall dodge events to occur after the minimum wall-contact delay"
    );
    assert!(
        tracker.post_wall_dodge_events().iter().all(|event| {
            processor.get_player_is_team_0(&event.player).ok() == Some(event.is_team_0)
        }),
        "Expected post-wall dodge team labels to agree with resolved player teams"
    );
}

#[test]
fn test_processor_extracts_flip_reset_followup_dodge_events() {
    let replay =
        parse_replay("assets/replay-format-2026-01-14-v868-32-net10-demolish-extended.replay");
    let mut processor = ReplayProcessor::new(&replay).expect("Failed to construct processor");
    let mut tracker = FlipResetTracker::new();
    processor
        .process(&mut tracker)
        .expect("Failed to process replay for flip-reset followup dodge extraction");

    assert!(
        !tracker.flip_reset_followup_dodge_events().is_empty(),
        "Expected the heuristic to find at least one followup dodge after a likely reset touch"
    );
    assert!(
        tracker
            .flip_reset_followup_dodge_events()
            .iter()
            .all(|event| (0.05..=1.75).contains(&event.time_since_candidate_touch)),
        "Expected followup dodges to occur within the candidate-touch timing window"
    );
    assert!(
        tracker
            .flip_reset_followup_dodge_events()
            .iter()
            .all(|event| (0.0..=1.0).contains(&event.candidate_touch_confidence)),
        "Expected candidate-touch confidence to remain normalized"
    );
    assert!(
        tracker
            .flip_reset_followup_dodge_events()
            .iter()
            .all(|event| {
                processor.get_player_is_team_0(&event.player).ok() == Some(event.is_team_0)
            }),
        "Expected followup dodge team labels to agree with resolved player teams"
    );
}

#[test]
fn test_processor_extracts_player_stat_events() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let mut processor = ReplayProcessor::new(&replay).expect("Failed to construct processor");
    let mut counter = FrameCounter::new();
    processor
        .process(&mut counter)
        .expect("Failed to process replay for player stat events");

    let replay_meta = processor
        .get_replay_meta()
        .expect("Expected replay metadata after processing");
    let total_shots = replay_meta
        .player_order()
        .filter_map(|player| player.stats.as_ref())
        .filter_map(|stats| match stats.get("Shots") {
            Some(boxcars::HeaderProp::Int(value)) => Some(*value),
            _ => None,
        })
        .sum::<i32>() as usize;
    let total_saves = replay_meta
        .player_order()
        .filter_map(|player| player.stats.as_ref())
        .filter_map(|stats| match stats.get("Saves") {
            Some(boxcars::HeaderProp::Int(value)) => Some(*value),
            _ => None,
        })
        .sum::<i32>() as usize;
    let total_assists = replay_meta
        .player_order()
        .filter_map(|player| player.stats.as_ref())
        .filter_map(|stats| match stats.get("Assists") {
            Some(boxcars::HeaderProp::Int(value)) => Some(*value),
            _ => None,
        })
        .sum::<i32>() as usize;

    assert_eq!(
        processor
            .player_stat_events
            .iter()
            .filter(|event| event.kind == PlayerStatEventKind::Shot)
            .count(),
        total_shots,
        "Expected one emitted shot event per replay-header shot"
    );
    assert_eq!(
        processor
            .player_stat_events
            .iter()
            .filter(|event| event.kind == PlayerStatEventKind::Save)
            .count(),
        total_saves,
        "Expected one emitted save event per replay-header save"
    );
    assert_eq!(
        processor
            .player_stat_events
            .iter()
            .filter(|event| event.kind == PlayerStatEventKind::Assist)
            .count(),
        total_assists,
        "Expected one emitted assist event per replay-header assist"
    );
}

#[test]
fn test_touch_attribution_usually_matches_goal_scorer() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let mut processor = ReplayProcessor::new(&replay).expect("Failed to construct processor");
    let mut counter = FrameCounter::new();
    processor
        .process(&mut counter)
        .expect("Failed to process replay for touch attribution quality");

    let mut matched = 0usize;
    let mut total_with_scorer = 0usize;
    for goal_event in processor
        .goal_events
        .iter()
        .filter(|event| event.player.is_some())
    {
        total_with_scorer += 1;
        let last_touch = processor.touch_events.iter().rev().find(|touch| {
            touch.frame <= goal_event.frame
                && touch.team_is_team_0 == goal_event.scoring_team_is_team_0
                && touch.player.is_some()
        });
        if last_touch
            .and_then(|touch| touch.player.as_ref())
            .zip(goal_event.player.as_ref())
            .map(|(touch_player, scorer)| touch_player == scorer)
            .unwrap_or(false)
        {
            matched += 1;
        }
    }

    assert!(
        total_with_scorer > 0,
        "Expected the replay to expose at least one goal scorer for attribution comparison"
    );
    assert!(
        matched * 2 >= total_with_scorer,
        "Expected motion-aware touch attribution to match the replay-derived goal scorer for a majority of scorable goals, matched {matched}/{total_with_scorer}"
    );
}

/// Regression: new-format demolish payloads still need car->player resolution even
/// when same-frame cleanup clears the player link to `ActorId(-1)`.
#[test]
fn test_new_demolition_format_replay_has_demolishes() {
    let replay =
        parse_replay("assets/replay-format-2026-01-14-v868-32-net10-demolish-extended.replay");
    let replay_data = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .expect("Failed to get replay data for replay-format-2026-01-14-v868-32-net10-demolish-extended.replay");

    assert_eq!(
        replay_data.demolish_infos.len(),
        10,
        "Expected 10 demolitions in replay-format-2026-01-14-v868-32-net10-demolish-extended.replay"
    );
    assert!(
        replay_data.demolish_infos.iter().all(|info| {
            info.victim_location.x != 0.0
                || info.victim_location.y != 0.0
                || info.victim_location.z != 0.0
        }),
        "Expected deleted-victim demolitions to keep a real last-known location instead of origin"
    );
}

#[test]
fn test_demolition_velocities_are_in_physical_units() {
    let replay =
        parse_replay("assets/replay-format-2026-01-14-v868-32-net10-demolish-extended.replay");
    let replay_data = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .expect("Failed to get replay data for replay-format-2026-01-14-v868-32-net10-demolish-extended.replay");

    let mut attacker_velocity_ratios = Vec::new();
    for demolish in &replay_data.demolish_infos {
        let attacker_data = replay_data
            .frame_data
            .players
            .iter()
            .find(|(player_id, _)| player_id == &demolish.attacker)
            .map(|(_, player_data)| player_data)
            .expect("Expected demolish attacker to have player data");
        if let Some(PlayerFrame::Data { rigid_body, .. }) =
            attacker_data.frames().get(demolish.frame)
        {
            if let Some(linear_velocity) = rigid_body.linear_velocity {
                let demo_speed = glam::Vec3::new(
                    demolish.attacker_velocity.x,
                    demolish.attacker_velocity.y,
                    demolish.attacker_velocity.z,
                )
                .length();
                let rigid_body_speed =
                    glam::Vec3::new(linear_velocity.x, linear_velocity.y, linear_velocity.z)
                        .length();
                if demo_speed.is_finite()
                    && rigid_body_speed.is_finite()
                    && demo_speed > 0.0
                    && rigid_body_speed > 0.0
                {
                    attacker_velocity_ratios.push(demo_speed / rigid_body_speed);
                }
            }
        }
    }

    attacker_velocity_ratios
        .sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let median_ratio = attacker_velocity_ratios[attacker_velocity_ratios.len() / 2];
    assert!(
        (0.5..=2.0).contains(&median_ratio),
        "Expected demolish attacker velocities to be on the same physical scale as rigid-body velocities, got median ratio {median_ratio}"
    );
}

/// Custom collector to verify frame processing works correctly
struct FrameCounter {
    frame_count: usize,
    times: Vec<f32>,
}

impl FrameCounter {
    fn new() -> Self {
        Self {
            frame_count: 0,
            times: Vec::new(),
        }
    }
}

impl Collector for FrameCounter {
    fn process_frame(
        &mut self,
        _processor: &dyn ProcessorView,
        _frame: &boxcars::Frame,
        _frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        self.frame_count += 1;
        self.times.push(current_time);
        Ok(TimeAdvance::NextFrame)
    }
}

/// Test custom collector receives all frames
#[test]
fn test_custom_collector_receives_frames() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");

    let counter = FrameCounter::new()
        .process_replay(&replay)
        .expect("Should process replay");

    assert!(counter.frame_count > 0, "Should have processed frames");
    assert_eq!(
        counter.times.len(),
        counter.frame_count,
        "Should have time for each frame"
    );

    // Verify times are monotonically increasing
    for window in counter.times.windows(2) {
        assert!(
            window[1] >= window[0],
            "Times should be monotonically increasing"
        );
    }
}

#[test]
#[allow(clippy::result_large_err)]
fn test_callback_collector_invokes_callback_for_each_frame_by_default() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let mut callback_frames = Vec::new();
    let mut collector = CallbackCollector::new(|_frame, frame_number, current_time| {
        callback_frames.push((frame_number, current_time));
        Ok(())
    });
    let mut processor = ReplayProcessor::new(&replay).expect("Should create processor");
    processor
        .process(&mut collector)
        .expect("Should process replay");

    let total_frames = replay
        .network_frames
        .as_ref()
        .expect("Replay should have network frames")
        .frames
        .len();

    assert_eq!(
        callback_frames.len(),
        total_frames,
        "Callback should be invoked once per processed frame by default"
    );
    assert_eq!(
        callback_frames
            .first()
            .map(|(frame_number, _)| *frame_number),
        Some(0),
        "Callback collector should begin at frame zero"
    );
}

#[test]
#[allow(clippy::result_large_err)]
fn test_callback_collector_honors_frame_interval() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let frame_interval = 100;
    let mut callback_frames = Vec::new();
    let mut collector = CallbackCollector::with_frame_interval(
        |_frame, frame_number, _current_time| {
            callback_frames.push(frame_number);
            Ok(())
        },
        frame_interval,
    );
    let mut processor = ReplayProcessor::new(&replay).expect("Should create processor");
    processor
        .process(&mut collector)
        .expect("Should process replay");

    let total_frames = replay
        .network_frames
        .as_ref()
        .expect("Replay should have network frames")
        .frames
        .len();
    let expected_frames: Vec<usize> = (0..total_frames).step_by(frame_interval).collect();

    assert_eq!(
        callback_frames, expected_frames,
        "Callback collector should only invoke the callback at the configured cadence"
    );
}
