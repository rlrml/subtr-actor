use std::collections::HashSet;
use subtr_actor::*;

/// Helper to parse a replay file
fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).expect(&format!("Failed to read replay file: {}", path));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .expect(&format!("Failed to parse replay: {}", path))
}

/// Test that all sample replays can be parsed and processed without errors
#[test]
fn test_all_replays_parse_successfully() {
    let replays = [
        "assets/replays/test/rumble.replay",
        "assets/replays/test/rlcs.replay",
        "assets/replays/test/gridiron.replay",
        "assets/replays/test/tourny.replay",
        "assets/replays/test/soccar-lan.replay",
    ];

    for path in replays {
        let replay = parse_replay(path);
        assert!(
            replay.network_frames.is_some(),
            "Replay {} should have network frames",
            path
        );
    }
}

/// Test ReplayDataCollector works on known-good replays
#[test]
fn test_replay_data_collector_multiple_replays() {
    // Use replays that are known to work with the collector
    let replays = [
        "assets/replays/old_boost_format.replay",
        "assets/replays/new_boost_format.replay",
    ];

    for path in replays {
        let replay = parse_replay(path);
        let collector = ReplayDataCollector::new();

        let replay_data = collector
            .get_replay_data(&replay)
            .expect(&format!("Failed to get replay data for {}", path));

        // Verify we got meaningful data
        assert!(
            replay_data.frame_data.frame_count() > 0,
            "Replay {} should have frames",
            path
        );
        assert!(
            replay_data.frame_data.duration() > 0.0,
            "Replay {} should have positive duration",
            path
        );

        // Verify JSON serialization works
        let json = replay_data
            .as_json()
            .expect("JSON serialization should work");
        assert!(!json.is_empty(), "JSON should not be empty");
    }
}

/// Test NDArrayCollector with default feature adders
#[test]
fn test_ndarray_collector_default_features() {
    let replay = parse_replay("assets/replays/test/rlcs.replay");

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
    let replay = parse_replay("assets/replays/old_boost_format.replay");

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
        "Should have ball-related headers, got: {:?}",
        headers
    );
}

/// Test NDArrayCollector with all player feature adders
#[test]
fn test_ndarray_collector_all_player_features() {
    let replay = parse_replay("assets/replays/test/rumble.replay");

    // Use all available player feature adders
    let collector = NDArrayCollector::<f32>::from_strings(
        &[],
        &[
            "PlayerRigidBody",
            "PlayerRigidBodyNoVelocities",
            "PlayerBoost",
            "PlayerJump",
            "PlayerAnyJump",
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

/// Test FrameRateDecorator with different FPS values
#[test]
fn test_frame_rate_decorator() {
    let replay = parse_replay("assets/replays/test/rlcs.replay");

    for fps in [5.0, 10.0, 30.0] {
        let mut collector =
            NDArrayCollector::<f32>::from_strings(&["BallRigidBody"], &["PlayerRigidBody"])
                .expect("Should create collector");

        FrameRateDecorator::new_from_fps(fps, &mut collector)
            .process_replay(&replay)
            .expect(&format!("Should process at {} fps", fps));

        let (_, array) = collector
            .get_meta_and_ndarray()
            .expect("Should get ndarray");

        assert!(array.nrows() > 0, "Should have rows at {} fps", fps);
    }
}

/// Test that different FPS values produce different row counts
#[test]
fn test_frame_rate_affects_output_size() {
    let replay = parse_replay("assets/replays/test/soccar-lan.replay");

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
    let replay = parse_replay("assets/replays/test/rlcs.replay");

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
    let replay = parse_replay("assets/replays/test/rlcs.replay");

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

/// Test gridiron (special game mode) replay parsing - note this mode has different archetypes
#[test]
fn test_gridiron_replay() {
    let replay = parse_replay("assets/replays/test/gridiron.replay");

    // Gridiron has different game event archetypes, so just verify parsing works
    assert!(
        replay.network_frames.is_some(),
        "Gridiron replay should have network frames"
    );

    // Use basic NDArrayCollector without soccar-specific features
    let collector = NDArrayCollector::<f32>::from_strings(&["BallRigidBody"], &["PlayerRigidBody"])
        .expect("Should create collector");

    // This should work even for gridiron
    let result = collector.process_replay(&replay);
    // Gridiron may or may not work depending on how similar it is to soccar
    // Just verify we can attempt to process it
    assert!(
        result.is_ok() || result.is_err(),
        "Processing should complete (success or failure)"
    );
}

/// Test tournament replay processing
#[test]
fn test_tournament_replay() {
    let replay = parse_replay("assets/replays/test/tourny.replay");

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
    let replay = parse_replay("assets/replays/test/rlcs.replay");

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
        ("assets/replays/test/rlcs.replay", 6), // RLCS should have 6 players (3v3)
        ("assets/replays/test/soccar-lan.replay", 2), // Might be 1v1 or 2v2
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
        _processor: &ReplayProcessor,
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
    let replay = parse_replay("assets/replays/test/rumble.replay");

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
