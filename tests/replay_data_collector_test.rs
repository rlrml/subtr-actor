use std::path::Path;
use subtr_actor::collector::replay_data::{
    BallFrame, PlayerFrame, ReplayDataCollector, ReplayDataSupplementalData,
};
use subtr_actor::{
    FlipResetTracker, ReplayProcessor, ResolvedBoostPadCollector, BOOST_KICKOFF_START_AMOUNT,
};

fn parse_replay(path: &str) -> boxcars::Replay {
    let replay_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let data = std::fs::read(&replay_path)
        .unwrap_or_else(|_| panic!("Failed to read replay file: {}", replay_path.display()));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay: {}", replay_path.display()))
}

#[test]
fn kickoff_replay_data_preserves_ball_countdown_and_initial_boost_state() {
    let replay = parse_replay("assets/replays/rlcs.replay");
    let replay_data = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .expect("Failed to collect replay data");

    let first_touch_time = replay_data
        .touch_events
        .first()
        .map(|event| event.time)
        .expect("Expected replay to contain at least one touch event");

    let pre_touch_ball_frames = replay_data
        .frame_data
        .metadata_frames
        .iter()
        .zip(replay_data.frame_data.ball_data.frames().iter())
        .take_while(|(metadata, _)| metadata.time < first_touch_time);

    let non_empty_pre_touch_frames = pre_touch_ball_frames
        .filter(|(_, frame)| matches!(frame, BallFrame::Data { .. }))
        .count();

    assert!(
        non_empty_pre_touch_frames > 0,
        "Expected kickoff frames before the first touch to retain ball position data"
    );

    let countdowns: Vec<i32> = replay_data
        .frame_data
        .metadata_frames
        .iter()
        .map(|metadata| metadata.replicated_game_state_time_remaining)
        .collect();

    assert!(
        countdowns
            .iter()
            .all(|countdown| (0..=3).contains(countdown)),
        "Expected kickoff countdown metadata to stay within the replay's 0-3 range"
    );
    assert!(
        countdowns.iter().any(|countdown| *countdown > 0),
        "Expected replay metadata export to include non-zero kickoff countdown frames"
    );

    let kickoff_frame_index = replay_data
        .frame_data
        .metadata_frames
        .iter()
        .position(|metadata| {
            metadata.replicated_game_state_name == 55
                || metadata.replicated_game_state_time_remaining > 0
        })
        .expect("Expected replay to contain kickoff countdown frames");

    let kickoff_boosts: Vec<f32> = replay_data
        .frame_data
        .players
        .iter()
        .filter_map(
            |(_, player_data)| match player_data.frames().get(kickoff_frame_index) {
                Some(PlayerFrame::Data { boost_amount, .. }) => Some(*boost_amount),
                _ => None,
            },
        )
        .collect();

    assert!(
        !kickoff_boosts.is_empty(),
        "Expected kickoff countdown frames to expose at least one active player"
    );
    assert!(
        kickoff_boosts
            .iter()
            .all(|boost| (*boost - BOOST_KICKOFF_START_AMOUNT).abs() < f32::EPSILON),
        "Expected kickoff player boosts to start at {BOOST_KICKOFF_START_AMOUNT}, got {kickoff_boosts:?}"
    );
}

#[test]
fn replay_data_collectors_can_be_composed_in_a_single_processor_pass() {
    let replay = parse_replay("assets/replays/rlcs.replay");
    let expected = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .expect("Failed to collect replay data from convenience API");

    let mut processor = ReplayProcessor::new(&replay).expect("Failed to build replay processor");
    let mut replay_data_collector = ReplayDataCollector::new();
    let mut flip_reset_tracker = FlipResetTracker::new();
    let mut boost_pad_collector = ResolvedBoostPadCollector::new();

    processor
        .process_all(&mut [
            &mut replay_data_collector,
            &mut flip_reset_tracker,
            &mut boost_pad_collector,
        ])
        .expect("Failed to process replay with composed collectors");

    let supplemental_data = ReplayDataSupplementalData::from_flip_reset_tracker(flip_reset_tracker)
        .with_boost_pads(boost_pad_collector.into_resolved_boost_pads());

    let actual = replay_data_collector
        .into_replay_data_with_supplemental_data(processor, supplemental_data)
        .expect("Failed to assemble replay data from composed collectors");

    assert_eq!(actual.frame_data, expected.frame_data);
    assert_eq!(actual.meta.team_zero, expected.meta.team_zero);
    assert_eq!(actual.meta.team_one, expected.meta.team_one);
    assert_eq!(
        actual.meta.all_headers.len(),
        expected.meta.all_headers.len()
    );
    assert_eq!(actual.demolish_infos, expected.demolish_infos);
    assert_eq!(actual.boost_pad_events, expected.boost_pad_events);
    assert_eq!(actual.boost_pads.len(), expected.boost_pads.len());
    for (actual_pad, expected_pad) in actual.boost_pads.iter().zip(expected.boost_pads.iter()) {
        assert_eq!(actual_pad.index, expected_pad.index);
        assert_eq!(actual_pad.size, expected_pad.size);
        assert_eq!(actual_pad.position, expected_pad.position);
        assert_eq!(actual_pad.pad_id.is_some(), expected_pad.pad_id.is_some());
    }
    assert_eq!(actual.touch_events, expected.touch_events);
    assert_eq!(
        actual.dodge_refreshed_events,
        expected.dodge_refreshed_events
    );
    assert_eq!(
        actual.heuristic_data.flip_reset_events,
        expected.heuristic_data.flip_reset_events
    );
    assert_eq!(
        actual.heuristic_data.post_wall_dodge_events,
        expected.heuristic_data.post_wall_dodge_events
    );
    assert_eq!(
        actual.heuristic_data.flip_reset_followup_dodge_events,
        expected.heuristic_data.flip_reset_followup_dodge_events
    );
    assert_eq!(actual.player_stat_events, expected.player_stat_events);
    assert_eq!(actual.goal_events, expected.goal_events);
}
