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

/// Real feature rows from the trained-v1 held-out set with the offline
/// pipeline's predicted probabilities (float64). The embedded f32 model must
/// reproduce them; regenerated alongside the coefficients by
/// scripts/threat_model/train_threat_model.py.
const TRAINING_PARITY_FIXTURE: &[(&[f32; THREAT_FEATURE_COUNT], f32)] = &[
    (
        &[
            -0.839_109, 0.862_713, 0.078_395, 0.653_84, -0.615_211, 0.057_195, 0.0, 0.0, 1.0, 0.5,
            0.0, 0.063_089, 0.505_834, 1.0, 0.0, 0.274_51, 2.0,
        ],
        5.375_665e-6,
    ),
    (
        &[
            -0.981_82, 0.914_914, 0.100_039, 0.062_519, 0.001_412, 0.054_801, 0.0, 0.0, 0.362_575,
            1.0, 0.0, 0.330_344, 0.813_828, 1.0, 0.0, 0.239_216, 1.0,
        ],
        0.004_856_322_6,
    ),
    (
        &[
            -0.850_684, 0.855_126, 0.070_44, 0.175_894, 0.167_881, 0.058_558, 0.0, 0.092_799,
            0.214_89, 0.0, 1.0, 0.194_378, 0.571_433, 0.5, 0.0, 0.819_608, 2.0,
        ],
        0.019_329_575,
    ),
    (
        &[
            0.639_963, 0.219_892, 0.155_705, 0.295_717, 0.001_887, 0.179_142, 0.0, 0.388_036,
            0.254_12, 0.0, 1.0, 0.680_158, 0.102_873, 0.5, 0.0, 1.0, 2.0,
        ],
        0.067_572_535,
    ),
    (
        &[
            0.995_201, 0.002_314, 0.047_216, 0.510_596, 0.489_777, 0.982_483, 1.0, 0.992_038,
            0.305_113, 0.0, 1.0, 1.0, 0.912_356, 0.0, 0.0, 0.047_059, 1.0,
        ],
        0.997_098_6,
    ),
    (
        &[
            0.987_828, 0.017_902, 0.087_539, 0.415_882, 0.413_33, 0.953_978, 1.0, 0.919_608,
            0.067_34, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 2.0,
        ],
        0.997_472_2,
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
