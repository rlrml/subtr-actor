use subtr_actor::*;

#[test]
fn test_game_state_feature_adders_registered() {
    // Test that the new feature adders are registered and can be used
    let collector = NDArrayCollector::<f32>::from_strings(
        &[
            "ReplicatedStateName",
            "ReplicatedGameStateTimeRemaining",
            "BallHasBeenHit",
        ],
        &[],
    );

    assert!(collector.is_ok(), "Feature adders should be registered");
}

#[test]
fn test_game_state_column_headers() {
    // Test that the column headers are correct
    let collector = NDArrayCollector::<f32>::from_strings(
        &[
            "ReplicatedStateName",
            "ReplicatedGameStateTimeRemaining",
            "BallHasBeenHit",
        ],
        &[],
    )
    .unwrap();

    let headers = collector.get_column_headers();

    assert_eq!(headers.global_headers.len(), 3);
    assert!(headers.global_headers.contains(&"game state".to_string()));
    assert!(headers
        .global_headers
        .contains(&"kickoff countdown".to_string()));
    assert!(headers
        .global_headers
        .contains(&"ball has been hit".to_string()));
}

#[test]
fn test_game_state_with_replay() {
    // Test extraction from an actual replay
    let replay_path = "assets/replays/old_boost_format.replay";
    let data = std::fs::read(replay_path).expect("Failed to read replay file");
    let replay = boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .expect("Failed to parse replay");

    let collector = NDArrayCollector::<f32>::from_strings(
        &[
            "BallRigidBody",
            "ReplicatedStateName",
            "ReplicatedGameStateTimeRemaining",
            "BallHasBeenHit",
        ],
        &["PlayerRigidBody"],
    )
    .unwrap();

    let (meta, array) = collector
        .process_replay(&replay)
        .expect("Failed to process replay")
        .get_meta_and_ndarray()
        .expect("Failed to get ndarray");

    // Verify we got data
    assert!(array.nrows() > 0, "Should have extracted frames");

    // Find column indices
    let headers = &meta.column_headers.global_headers;
    let state_idx = headers.iter().position(|h| h == "game state").unwrap();
    let countdown_idx = headers
        .iter()
        .position(|h| h == "kickoff countdown")
        .unwrap();
    let ball_hit_idx = headers
        .iter()
        .position(|h| h == "ball has been hit")
        .unwrap();

    // Verify game state returns numeric values (known values: 0, 55, 58, 86)
    let unique_states: std::collections::HashSet<i32> = array
        .rows()
        .into_iter()
        .map(|row| row[state_idx] as i32)
        .collect();
    assert!(!unique_states.is_empty(), "Should have game state values");

    // Verify countdown values are in valid range 0-3
    let mut ball_hit_true_count = 0usize;
    let mut ball_hit_false_count = 0usize;
    for row in array.rows() {
        let countdown = row[countdown_idx] as i32;
        assert!(
            (0..=3).contains(&countdown),
            "Countdown should be 0-3, got {}",
            countdown
        );

        let ball_hit = row[ball_hit_idx];
        assert!(
            ball_hit == 0.0 || ball_hit == 1.0,
            "Ball hit should be 0 or 1, got {}",
            ball_hit
        );
        if ball_hit == 1.0 {
            ball_hit_true_count += 1;
        } else {
            ball_hit_false_count += 1;
        }
    }

    // Regression test for https://github.com/rlrml/subtr-actor/issues/16
    // BallHasBeenHit should be 0 during kickoff and 1 after the ball is hit.
    // Previously it was always 1 because NDArrayCollector skipped frames where
    // the ball rigid body didn't exist, which coincided with kickoff (when
    // BallHasBeenHit is false).
    assert!(
        ball_hit_true_count > 0 && ball_hit_false_count > 0,
        "BallHasBeenHit should contain both 0 and 1 values across a full replay, \
         got {} true and {} false",
        ball_hit_true_count,
        ball_hit_false_count
    );
}
