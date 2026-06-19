use std::path::Path;
use subtr_actor::collector::replay_data::{BallFrame, PlayerFrame, ReplayDataCollector};
use subtr_actor::{
    BOOST_KICKOFF_START_AMOUNT, PlayerStatEventKind, ReplayProcessor,
    ShotGoalLineCrossingPredictionKind, ShotGoalLineCrossingUnavailableReason,
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
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
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
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let expected = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .expect("Failed to collect replay data from convenience API");

    let mut processor = ReplayProcessor::new(&replay).expect("Failed to build replay processor");
    let mut replay_data_collector = ReplayDataCollector::new();

    processor
        .process_all(&mut [&mut replay_data_collector])
        .expect("Failed to process replay with composed collectors");

    let actual = replay_data_collector
        .into_replay_data(processor)
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
    assert_eq!(actual.player_stat_events, expected.player_stat_events);
    assert_eq!(actual.goal_events, expected.goal_events);
}

#[test]
fn saved_shot_goal_line_crossing_counts_are_stable_in_replay_fixtures() {
    let cases = [
        (
            "assets/nuttrback-double-tap-goal-7-2026-06-01.replay",
            5,
            5,
            5,
            3,
            2,
            None,
        ),
        (
            "assets/air-dribble-goal-mouth-2026-05-24.replay",
            6,
            6,
            5,
            5,
            1,
            None,
        ),
        (
            "assets/replay-format-2017-03-16-v868-17-net-none-online.replay",
            8,
            4,
            2,
            4,
            0,
            Some([
                ShotGoalLineCrossingUnavailableReason::NoGoalLineCrossingBeforeSaveReference,
                ShotGoalLineCrossingUnavailableReason::NoGoalwardBallBeforeSaveReference,
                ShotGoalLineCrossingUnavailableReason::NoGoalwardBallBeforeSaveReference,
                ShotGoalLineCrossingUnavailableReason::OnlyUnphysicalFreeFlightCrossings,
            ]),
        ),
    ];

    for (
        path,
        expected_saved,
        expected_projected,
        expected_inside_goal_mouth,
        expected_surface,
        expected_free_flight,
        expected_unavailable_reasons,
    ) in cases
    {
        let replay = parse_replay(path);
        let replay_data = ReplayDataCollector::new()
            .get_replay_data(&replay)
            .unwrap_or_else(|_| panic!("failed to collect replay data for {path}"));

        let saved_shots = replay_data
            .player_stat_events
            .iter()
            .filter(|event| event.kind == PlayerStatEventKind::Shot)
            .filter_map(|event| {
                let shot = event.shot.as_ref()?;
                let save = shot.resulting_save.as_ref()?;
                Some((shot, save.time - event.time))
            })
            .collect::<Vec<_>>();
        let saved_shot_crossings = saved_shots
            .iter()
            .filter_map(|(shot, save_time_after_shot)| {
                let crossing = shot.projected_goal_line_crossing.as_ref()?;
                Some((crossing, *save_time_after_shot))
            })
            .collect::<Vec<_>>();

        let surface_count = saved_shot_crossings
            .iter()
            .filter(|(crossing, _)| {
                matches!(
                    crossing.prediction_kind,
                    ShotGoalLineCrossingPredictionKind::SurfaceBounces
                        | ShotGoalLineCrossingPredictionKind::SavedShotPreSaveSurfaceBounces
                )
            })
            .count();
        let free_flight_count = saved_shot_crossings
            .iter()
            .filter(|(crossing, _)| {
                matches!(
                    crossing.prediction_kind,
                    ShotGoalLineCrossingPredictionKind::FreeFlight
                        | ShotGoalLineCrossingPredictionKind::SavedShotPreSaveFreeFlight
                )
            })
            .count();
        let inside_goal_mouth_count = saved_shot_crossings
            .iter()
            .filter(|(crossing, _)| crossing.inside_goal_mouth)
            .count();
        let unavailable_reasons = saved_shots
            .iter()
            .filter_map(|(shot, _)| shot.projected_goal_line_crossing_unavailable_reason)
            .collect::<Vec<_>>();

        assert_eq!(saved_shots.len(), expected_saved, "{path}");
        assert_eq!(saved_shot_crossings.len(), expected_projected, "{path}");
        assert_eq!(
            inside_goal_mouth_count, expected_inside_goal_mouth,
            "{path}"
        );
        assert_eq!(surface_count, expected_surface, "{path}");
        assert_eq!(free_flight_count, expected_free_flight, "{path}");
        if let Some(expected_unavailable_reasons) = expected_unavailable_reasons {
            assert_eq!(unavailable_reasons, expected_unavailable_reasons, "{path}");
        } else {
            assert!(unavailable_reasons.is_empty(), "{path}");
        }
    }
}
