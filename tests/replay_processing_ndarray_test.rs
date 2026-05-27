mod common;

use std::collections::HashSet;

use common::parse_replay;
use subtr_actor::*;

fn max_abs_position_from_ndarray(
    replay: &boxcars::Replay,
    global_feature_adders: &[&str],
    player_feature_adders: &[&str],
) -> f32 {
    let collector =
        NDArrayCollector::<f32>::from_strings(global_feature_adders, player_feature_adders)
            .expect("Should create collector");
    let (meta, array) = collector
        .process_replay(replay)
        .expect("Should process replay")
        .get_meta_and_ndarray()
        .expect("Should get ndarray");

    let headers = meta.headers_vec();
    let mut max_abs_position = 0.0f32;
    for (index, header) in headers.iter().enumerate() {
        if !header.contains("position ") {
            continue;
        }
        for value in array.column(index).iter().copied() {
            max_abs_position = max_abs_position.max(value.abs());
        }
    }
    max_abs_position
}

#[test]
fn test_legacy_replay_ndarray_positions_are_normalized_to_field_units() {
    for path in [
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
        "assets/replay-format-2016-07-21-v868-12-net-none-lan.replay",
    ] {
        let replay = parse_replay(path);
        let max_abs_position = max_abs_position_from_ndarray(
            &replay,
            &["InterpolatedBallRigidBodyNoVelocities"],
            &["InterpolatedPlayerRigidBodyNoVelocities"],
        );
        assert!(
            max_abs_position > 1000.0,
            "Expected ndarray positions for {path} to reach Rocket League field units, got {max_abs_position}"
        );
        assert!(
            max_abs_position < 10000.0,
            "Expected ndarray positions for {path} to stay within plausible Rocket League field bounds, got {max_abs_position}"
        );
    }
}

/// Test NDArrayCollector with default feature adders
#[test]
fn test_ndarray_collector_default_features() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");

    let collector = NDArrayCollector::<f32>::from_strings(
        &["BallRigidBody"],
        &["PlayerRigidBody", "PlayerBoost"],
    )
    .expect("Should create collector");

    let (meta, array) = collector
        .process_replay(&replay)
        .expect("Should process replay")
        .get_meta_and_ndarray()
        .expect("Should get ndarray");

    // Verify shape
    assert!(array.nrows() > 0, "Should have rows");
    assert!(array.ncols() > 0, "Should have columns");

    // Verify metadata
    assert!(meta.replay_meta.player_count() > 0, "Should have players");
    assert!(
        !meta.column_headers.global_headers.is_empty(),
        "Should have global headers"
    );
}

/// Test NDArrayCollector with all available global feature adders
#[test]
fn test_ndarray_collector_all_global_features() {
    // Use old_boost_format replay which is known to work with soccar features
    let replay = parse_replay("assets/replay-format-2022-09-29-v868-32-net10-legacy-boost.replay");

    let collector = NDArrayCollector::<f32>::from_strings(
        &[
            "BallRigidBody",
            "SecondsRemaining",
            "ReplicatedStateName",
            "ReplicatedGameStateTimeRemaining",
            "BallHasBeenHit",
        ],
        &[],
    )
    .expect("Should create collector with all global features");

    let (meta, _array) = collector
        .process_replay(&replay)
        .expect("Should process replay")
        .get_meta_and_ndarray()
        .expect("Should get ndarray");

    // Verify we got the expected headers
    let headers = &meta.column_headers.global_headers;
    assert!(
        headers
            .iter()
            .any(|h| h.to_lowercase().contains("ball") || h.contains("Ball")),
        "Should have ball-related headers, got: {headers:?}",
    );
}

/// Test NDArrayCollector with all player feature adders
#[test]
fn test_ndarray_collector_all_player_features() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");

    // Use all available player feature adders
    let collector = NDArrayCollector::<f32>::from_strings(
        &[],
        &[
            "PlayerRigidBody",
            "PlayerRigidBodyNoVelocities",
            "PlayerBallDistance",
            "PlayerBoost",
            "PlayerJump",
            "PlayerAnyJump",
            "PlayerDodgeRefreshed",
        ],
    )
    .expect("Should create collector with all player features");

    let (meta, array) = collector
        .process_replay(&replay)
        .expect("Should process replay")
        .get_meta_and_ndarray()
        .expect("Should get ndarray");

    // Verify player headers exist
    assert!(
        !meta.column_headers.player_headers.is_empty(),
        "Should have player headers"
    );
    assert!(array.ncols() > 0, "Should have columns");
}

#[test]
fn test_ndarray_collector_player_ball_distance_feature() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");

    let (meta, array) = NDArrayCollector::<f32>::from_strings(&[], &["PlayerBallDistance"])
        .expect("Should create collector")
        .process_replay(&replay)
        .expect("Should process replay")
        .get_meta_and_ndarray()
        .expect("Should get ndarray");

    assert_eq!(
        meta.column_headers.player_headers,
        vec!["distance to ball".to_string()],
        "Should expose the distance-to-ball player header"
    );
    assert_eq!(
        array.ncols(),
        meta.replay_meta.player_count(),
        "Should add one distance-to-ball column per player"
    );
    assert!(
        array.iter().any(|value| *value > 0.0),
        "Should emit positive ball-distance values for at least some frames"
    );
}

#[test]
fn test_ndarray_collector_player_dodge_refreshed_feature() {
    let replay =
        parse_replay("assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay");
    let replay_data = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .expect("Should collect replay data");

    let (meta, array) = NDArrayCollector::<f32>::from_strings(&[], &["PlayerDodgeRefreshed"])
        .expect("Should create collector")
        .process_replay(&replay)
        .expect("Should process replay")
        .get_meta_and_ndarray()
        .expect("Should get ndarray");

    assert_eq!(
        meta.column_headers.player_headers,
        vec!["dodge refresh count".to_string()],
        "Should expose the dodge refresh player header"
    );
    assert_eq!(
        array.ncols(),
        meta.replay_meta.player_count(),
        "Should add one dodge refresh column per player"
    );
    assert!(
        array.iter().any(|value| *value > 0.0),
        "Should emit non-zero values on frames with dodge refreshes"
    );
    assert_eq!(
        array.sum(),
        replay_data.dodge_refreshed_events.len() as f32,
        "Should preserve the exact total count of dodge refresh events"
    );
}

/// Test FrameRateDecorator with different FPS values
#[test]
fn test_frame_rate_decorator() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");

    for fps in [5.0, 10.0, 30.0] {
        let mut collector =
            NDArrayCollector::<f32>::from_strings(&["BallRigidBody"], &["PlayerRigidBody"])
                .expect("Should create collector");

        FrameRateDecorator::new_from_fps(fps, &mut collector)
            .process_replay(&replay)
            .unwrap_or_else(|_| panic!("Should process at {fps} fps"));

        let (_, array) = collector
            .get_meta_and_ndarray()
            .expect("Should get ndarray");

        assert!(array.nrows() > 0, "Should have rows at {fps} fps");
    }
}

/// Test that different FPS values produce different row counts
#[test]
fn test_frame_rate_affects_output_size() {
    let replay = parse_replay("assets/replay-format-2016-07-21-v868-12-net-none-lan.replay");

    let mut collector_low =
        NDArrayCollector::<f32>::from_strings(&["BallRigidBody"], &[]).expect("Should create");
    FrameRateDecorator::new_from_fps(5.0, &mut collector_low)
        .process_replay(&replay)
        .expect("Should process");
    let (_, array_low) = collector_low.get_meta_and_ndarray().expect("Should get");

    let mut collector_high =
        NDArrayCollector::<f32>::from_strings(&["BallRigidBody"], &[]).expect("Should create");
    FrameRateDecorator::new_from_fps(30.0, &mut collector_high)
        .process_replay(&replay)
        .expect("Should process");
    let (_, array_high) = collector_high.get_meta_and_ndarray().expect("Should get");

    // Higher FPS should produce more rows
    assert!(
        array_high.nrows() > array_low.nrows(),
        "30 fps ({} rows) should produce more rows than 5 fps ({} rows)",
        array_high.nrows(),
        array_low.nrows()
    );
}

/// Test ball position changes over time
#[test]
fn test_ball_position_changes() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");

    let collector = NDArrayCollector::<f32>::from_strings(&["BallRigidBody"], &[])
        .expect("Should create collector");

    let (meta, array) = collector
        .process_replay(&replay)
        .expect("Should process replay")
        .get_meta_and_ndarray()
        .expect("Should get ndarray");

    // Find position columns (usually first 3 columns for x, y, z)
    let headers = &meta.column_headers.global_headers;
    let x_idx = headers
        .iter()
        .position(|h| h.contains("pos x") || h.contains("location x"))
        .unwrap_or(0);

    // Collect unique x positions (rounded to avoid floating point issues)
    let unique_x: HashSet<i32> = array
        .rows()
        .into_iter()
        .map(|row| (row[x_idx] * 10.0) as i32)
        .collect();

    assert!(
        unique_x.len() > 10,
        "Ball x position should change significantly over time, got {} unique values",
        unique_x.len()
    );
}

/// Test player rigid body extraction
#[test]
fn test_player_rigid_body_extraction() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");

    let collector = NDArrayCollector::<f32>::from_strings(&[], &["PlayerRigidBody"])
        .expect("Should create collector");

    let (meta, array) = collector
        .process_replay(&replay)
        .expect("Should process replay")
        .get_meta_and_ndarray()
        .expect("Should get ndarray");

    // Verify we got player data
    assert!(meta.replay_meta.player_count() > 0, "Should have players");
    assert!(
        !meta.column_headers.player_headers.is_empty(),
        "Should have player headers"
    );

    // Expected properties per player: location (3), rotation (4), velocity (3), angular velocity (3) = 13
    // But it may vary based on implementation
    let expected_cols_per_player = meta.column_headers.player_headers.len();
    let total_player_cols = expected_cols_per_player * meta.replay_meta.player_count();

    assert!(
        array.ncols() >= total_player_cols,
        "Should have enough columns for all players"
    );
}

/// Test tournament replay processing
#[test]
fn test_tournament_replay() {
    let replay = parse_replay("assets/replay-format-2020-09-25-v868-29-net10-tournament.replay");

    let collector =
        NDArrayCollector::<f32>::from_strings(&["BallRigidBody", "SecondsRemaining"], &[])
            .expect("Should create collector");

    let (meta, array) = collector
        .process_replay(&replay)
        .expect("Should process tournament replay")
        .get_meta_and_ndarray()
        .expect("Should get ndarray");

    assert!(array.nrows() > 0, "Should have rows");
    assert!(
        meta.column_headers
            .global_headers
            .iter()
            .any(|h| h.contains("seconds")),
        "Should have seconds remaining header"
    );
}

/// Test that player order is consistent
#[test]
fn test_player_order_consistency() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");

    // Process twice and verify player order is the same
    let collector1 =
        NDArrayCollector::<f32>::from_strings(&[], &["PlayerRigidBody"]).expect("Should create");
    let (meta1, _) = collector1
        .process_replay(&replay)
        .expect("Should process")
        .get_meta_and_ndarray()
        .expect("Should get");

    let collector2 =
        NDArrayCollector::<f32>::from_strings(&[], &["PlayerRigidBody"]).expect("Should create");
    let (meta2, _) = collector2
        .process_replay(&replay)
        .expect("Should process")
        .get_meta_and_ndarray()
        .expect("Should get");

    let players1: Vec<_> = meta1.replay_meta.player_order().collect();
    let players2: Vec<_> = meta2.replay_meta.player_order().collect();

    assert_eq!(
        players1.len(),
        players2.len(),
        "Player count should be consistent"
    );

    for (p1, p2) in players1.iter().zip(players2.iter()) {
        assert_eq!(p1.name, p2.name, "Player order should be consistent");
    }
}

/// Test that column headers are properly generated
#[test]
fn test_column_header_generation() {
    let collector = NDArrayCollector::<f32>::from_strings(
        &["BallRigidBody", "SecondsRemaining"],
        &["PlayerRigidBody", "PlayerBoost"],
    )
    .expect("Should create collector");

    let headers = collector.get_column_headers();

    // Check global headers
    assert!(
        !headers.global_headers.is_empty(),
        "Should have global headers"
    );

    // Check player headers
    assert!(
        !headers.player_headers.is_empty(),
        "Should have player headers"
    );

    // Verify we have headers containing expected strings (case-insensitive)
    let has_ball_header = headers
        .global_headers
        .iter()
        .any(|h| h.to_lowercase().contains("ball") || h.contains("Ball"));
    assert!(
        has_ball_header,
        "Should have ball-related headers, got: {:?}",
        headers.global_headers
    );

    let has_boost_header = headers
        .player_headers
        .iter()
        .any(|h| h.to_lowercase().contains("boost"));
    assert!(
        has_boost_header,
        "Should have boost header, got: {:?}",
        headers.player_headers
    );
}

/// Test replay metadata extraction
#[test]
fn test_replay_meta_extraction() {
    let replays = [
        (
            "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
            6,
        ), // RLCS should have 6 players (3v3)
        (
            "assets/replay-format-2016-07-21-v868-12-net-none-lan.replay",
            2,
        ), // Might be 1v1 or 2v2
    ];

    for (path, min_players) in replays {
        let replay = parse_replay(path);

        let mut collector =
            NDArrayCollector::<f32>::from_strings(&["BallRigidBody"], &["PlayerRigidBody"])
                .expect("Should create");

        let meta = collector
            .process_and_get_meta_and_headers(&replay)
            .expect("Should get meta");

        assert!(
            meta.replay_meta.player_count() >= min_players,
            "Replay {} should have at least {} players, got {}",
            path,
            min_players,
            meta.replay_meta.player_count()
        );
    }
}

/// Test that invalid feature adder names are rejected
#[test]
fn test_invalid_feature_adder_rejected() {
    let result = NDArrayCollector::<f32>::from_strings(&["NonExistentFeature"], &[]);
    assert!(
        result.is_err(),
        "Should reject invalid global feature adder"
    );

    let result = NDArrayCollector::<f32>::from_strings(&[], &["NonExistentPlayerFeature"]);
    assert!(
        result.is_err(),
        "Should reject invalid player feature adder"
    );
}
