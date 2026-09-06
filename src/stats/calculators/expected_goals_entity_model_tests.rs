use super::*;

#[test]
fn entity_inference_matches_training_and_player_permutations() {
    let frame_fixtures: &[(&[f32; THREAT_ENTITY_FEATURE_COUNT], f32)] =
        include!("expected_goals_entity_frame_parity.rs");
    let touch_fixtures: &[(&[f32; THREAT_ENTITY_FEATURE_COUNT], f32)] =
        include!("expected_goals_entity_touch_parity.rs");
    for (weights, fixtures) in [
        (&frame::WEIGHTS, frame_fixtures),
        (&touch::WEIGHTS, touch_fixtures),
    ] {
        for &(features, expected) in fixtures {
            let actual = evaluate(features, weights);
            assert!(
                (actual - expected).abs() < 1e-5,
                "expected {expected}, actual {actual}"
            );
            let mut swapped = *features;
            for team in 0..2 {
                let start = THREAT_ENTITY_GLOBAL_COUNT + team * 34;
                for field in 0..17 {
                    swapped.swap(start + field, start + 17 + field);
                }
            }
            assert!((evaluate(&swapped, weights) - actual).abs() < 1e-6);
            let reflected = std::array::from_fn(|i| features[i] * weights.reflection[i]);
            assert!((evaluate(&reflected, weights) - actual).abs() < 1e-6);
        }
    }
}

#[test]
fn learned_player_context_distinguishes_boost_assignment() {
    let fixtures: &[(&[f32; THREAT_ENTITY_FEATURE_COUNT], f32)] =
        include!("expected_goals_entity_touch_parity.rs");
    let largest_change = fixtures
        .iter()
        .map(|(features, _)| {
            let mut transferred = **features;
            transferred.swap(
                THREAT_ENTITY_GLOBAL_COUNT + 2 * 17 + 11,
                THREAT_ENTITY_GLOBAL_COUNT + 3 * 17 + 11,
            );
            (evaluate(&transferred, &touch::WEIGHTS) - evaluate(features, &touch::WEIGHTS)).abs()
        })
        .fold(0.0_f32, f32::max);
    assert!(largest_change > 1e-5);
}

#[test]
fn deployed_horizons_match_the_scoring_contracts() {
    assert_eq!(ENTITY_THREAT_HORIZON_SECONDS, 5.0);
    assert_eq!(TOUCH_XG_HORIZON_SECONDS, 10.0);
}
