use super::*;

/// The coefficient table must stay keyed exactly by
/// [`ThreatFeatures::FEATURE_NAMES`], in order -- pasting trained weights is a
/// mechanical edit only while this holds.
#[test]
fn weights_cover_every_feature_in_order() {
    assert_eq!(THREAT_MODEL_WEIGHTS.len(), THREAT_FEATURE_COUNT);
    for ((weight_name, _), feature_name) in THREAT_MODEL_WEIGHTS
        .iter()
        .zip(ThreatFeatures::FEATURE_NAMES.iter())
    {
        assert_eq!(weight_name, feature_name);
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
        nearest_attacker_dist: 0.2,
        attackers_ahead_of_ball: 0.5,
        attackers_behind_ball: 0.5,
        nearest_defender_dist: 0.5,
        nearest_defender_to_goal_dist: 0.35,
        defenders_goalside: 1.0,
        defender_in_net: 0.0,
        nearest_defender_boost: 0.5,
        attacking_team_size: 2.0,
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
        defenders_goalside: 0.0,
        nearest_defender_to_goal_dist: 0.3,
        ..neutral
    };

    let neutral_value = threat_value(&neutral);
    let dangerous_value = threat_value(&dangerous);
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

/// Real feature rows from the trained-v2 held-out set with the offline
/// pipeline's predicted probabilities (float64). The embedded f32 model must
/// reproduce them; regenerated alongside the coefficients by
/// scripts/threat_model/train_threat_model.py.
const TRAINING_PARITY_FIXTURE: &[(&[f32; THREAT_FEATURE_COUNT], f32)] = &[
    (
        &[
            -0.839109, 0.862713, 0.078395, 0.65384, -0.615211, 0.057195, 0.0, 0.0, 1.0, 0.5, 0.0,
            0.063089, 0.505834, 1.0, 0.0, 0.27451, 2.0,
        ],
        5.5895807e-06,
    ),
    (
        &[
            -0.876275, 0.865327, 0.628659, 0.247617, 0.210264, 0.058854, 0.0, 0.110026, 0.411007,
            1.0, 0.0, 0.464603, 0.484796, 1.0, 0.0, 0.121569, 2.0,
        ],
        0.0049046245,
    ),
    (
        &[
            -0.67292, 0.809615, 0.04566, 0.151471, 0.078867, 0.059097, 0.0, 0.075715, 0.110705,
            0.0, 1.0, 1.0, 0.314607, 1.0, 0.0, 0.793353, 2.0,
        ],
        0.019436132,
    ),
    (
        &[
            0.899693, 0.206376, 0.045572, 0.045879, 0.0169, 0.063362, 0.0, 0.0, 0.354955, 0.0, 1.0,
            0.182267, 0.012131, 0.5, 1.0, 0.133333, 2.0,
        ],
        0.06779221,
    ),
    (
        &[
            0.995201, 0.002314, 0.047216, 0.510596, 0.489777, 0.982483, 1.0, 0.992038, 0.305113,
            0.0, 1.0, 1.0, 0.912356, 0.0, 0.0, 0.047059, 1.0,
        ],
        0.99705017,
    ),
    (
        &[
            0.987828, 0.017902, 0.087539, 0.415882, 0.41333, 0.953978, 1.0, 0.919608, 0.06734, 0.0,
            1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 2.0,
        ],
        0.99744374,
    ),
];

/// The embedded f32 inference must agree with the float64 training-pipeline
/// predictions on held-out rows spanning the whole probability range.
#[test]
fn trained_model_matches_training_pipeline_predictions() {
    for (index, (features, expected)) in TRAINING_PARITY_FIXTURE.iter().enumerate() {
        let actual = threat_value_from_array(features);
        let absolute = (actual - expected).abs();
        assert!(
            absolute < 2e-4 && absolute / expected < 0.01,
            "fixture row {index}: rust={actual} python={expected}"
        );
    }
}
