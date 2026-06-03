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
