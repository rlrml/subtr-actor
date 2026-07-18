use super::*;
use crate::{TeamThreatFeatures, ThreatFeatures};

/// The first-layer weight table must stay keyed exactly by
/// [`ThreatModelFeatures::feature_names`], in order -- pasting trained weights is a
/// mechanical edit only while this holds.
#[test]
fn weights_cover_every_feature_in_order() {
    assert_eq!(THREAT_MODEL_INPUT_WEIGHTS.len(), THREAT_MODEL_FEATURE_COUNT);
    for ((weight_name, weights), feature_name) in THREAT_MODEL_INPUT_WEIGHTS
        .iter()
        .zip(ThreatModelFeatures::feature_names())
    {
        assert_eq!(weight_name, feature_name);
        assert_eq!(weights.len(), THREAT_MODEL_HIDDEN_UNITS);
    }
    assert_eq!(THREAT_MODEL_HIDDEN_BIASES.len(), THREAT_MODEL_HIDDEN_UNITS);
    assert_eq!(THREAT_MODEL_OUTPUT_WEIGHTS.len(), THREAT_MODEL_HIDDEN_UNITS);
}

#[test]
fn rust_inference_matches_locked_training_pipeline() {
    let fixtures: &[(&[f32; THREAT_MODEL_FEATURE_COUNT], f32)] =
        include!("expected_goals_model_parity_fixture.rs");
    for &(features, expected) in fixtures {
        let actual = threat_value_from_array(features);
        assert!(
            (actual - expected).abs() < 2e-6,
            "training parity mismatch: expected {expected}, got {actual}"
        );
    }
}

fn neutral_features() -> ThreatFeatures {
    ThreatFeatures {
        ball_forward_y: 0.0,
        ball_dist_to_goal: 0.46,
        ball_height: 0.05,
        ball_speed: 0.2,
        ball_speed_toward_goal: 0.0,
        goal_open_angle: 0.11,
        on_target: 0.0,
        time_to_goal_line: 0.0,
        own_team: TeamThreatFeatures::default(),
        opponent_team: TeamThreatFeatures::default(),
    }
}

#[test]
fn threat_value_is_a_probability_and_orders_danger_over_neutral() {
    let neutral = neutral_features();
    let dangerous = ThreatFeatures {
        ball_forward_y: 0.8,
        ball_dist_to_goal: 0.12,
        ball_speed_toward_goal: 0.3,
        goal_open_angle: 0.45,
        on_target: 1.0,
        time_to_goal_line: 0.6,
        ..neutral
    };

    let neutral_value = threat_value(&ThreatModelFeatures::new(neutral, [None, None]));
    let dangerous_value = threat_value(&ThreatModelFeatures::new(dangerous, [None, None]));
    assert!(neutral_value > 0.0 && neutral_value < 1.0);
    assert!(dangerous_value > 0.0 && dangerous_value < 1.0);
    assert!(
        dangerous_value > neutral_value,
        "dangerous state ({dangerous_value}) must out-rank neutral ({neutral_value})"
    );
    // The model keeps neutral midfield states well under the episode
    // threshold and clear on-target chances above it.
    assert!(neutral_value < super::super::expected_goals::THREAT_EPISODE_THRESHOLD);
    assert!(dangerous_value > super::super::expected_goals::THREAT_EPISODE_THRESHOLD);
}
