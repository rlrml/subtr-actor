use super::super::whiff_features::{WHIFF_FEATURE_NAMES, WHIFF_FEATURE_VERSION};
use super::*;

#[test]
fn deployed_model_matches_current_feature_layout() {
    assert_eq!(WHIFF_MODEL_FEATURE_VERSION, WHIFF_FEATURE_VERSION);
    assert_eq!(WHIFF_FEATURE_NAMES.len(), WHIFF_FEATURE_COUNT);
    for stage in WHIFF_MODEL_STAGES {
        for tree in stage {
            for node in *tree {
                assert!((node.feature as usize) < WHIFF_FEATURE_COUNT || node.feature < 0);
            }
        }
    }
}

#[test]
fn rust_inference_matches_training_pipeline() {
    let fixtures: &[(&[f64; WHIFF_FEATURE_COUNT], [f64; 4])] =
        include!("whiff_model_parity_fixture.rs");
    assert!(!fixtures.is_empty());
    for (features, expected) in fixtures {
        let actual = whiff_class_probabilities(features);
        for (a, e) in actual.iter().zip(expected) {
            assert!(
                (a - e).abs() < 1e-9,
                "expected {expected:?}, got {actual:?}"
            );
        }
    }
}
