use super::*;
use crate::TeamThreatFeatures;

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

#[test]
fn rust_inference_matches_locked_training_pipeline() {
    // Low, median, and high temporal-holdout rows emitted by the uv/Nix-locked
    // trainer. Three rows cover the numerical range without checking in a
    // bulky dataset fixture.
    let fixtures: &[(&[f32; THREAT_FEATURE_COUNT], f32)] = &[
        (
            &[
                -0.806533, 0.852562, 0.715602, 0.675648, -0.617448, 0.058722, 0.0, 0.0, 0.189132,
                -0.114463, 0.008182, -0.037291, -0.516876, 0.002107, -0.052617, -0.780234,
                -0.009459, 1.0, 0.514733, 0.0, 1.0, 0.0, 1.0, 0.0, 0.212444, 0.503934, 0.00025,
                0.518965, 0.43723, 0.003222, 1.243573, 0.083867, 0.000331, 0.0, 0.238454, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.126997, 0.073385, 0.008324, -0.247852, -0.409815, 0.000122,
                -0.615041, -0.705672, -0.009603, 1.0, 0.430744, 0.694118, 1.0, 0.0, 1.0, 0.0,
                0.415264, 0.750788, 5e-06, 0.205609, 0.501952, 9.6e-05, 0.530194, 0.462099,
                0.000124, 0.0, 0.353896, 0.031373, 0.0, 0.0, 0.0, 0.0,
            ],
            4.3319797e-06,
        ),
        (
            &[
                -0.0, 0.457143, 0.045377, 0.0, 0.0, 0.1099, 0.0, 0.0, 0.240743, -0.681881,
                0.008322, -0.101115, 0.249102, 1.3e-05, -0.346748, 0.853473, -0.009506, 0.89641,
                0.780111, 0.225292, 0.0, 0.0, 1.0, 0.0, 0.480979, 0.393789, 0.0, 0.2105, 0.085465,
                0.0, 0.72064, 0.29278, 1.5e-05, 0.20718, 0.157569, 0.0, 0.0, 0.0, 0.0, 0.0,
                -0.2403, 0.686037, 0.008322, 0.110463, -0.190139, 6.5e-05, 0.384705, -0.852546,
                -0.00949, 0.89641, 0.17304, 0.279313, 1.0, 0.5, 1.0, 0.0, 0.481865, 0.402102, 0.0,
                0.191804, 0.032461, 0.000104, 0.644725, 0.290927, 2.5e-05, 0.20718, 0.241964,
                0.108041, 0.0, 1.0, 0.0, 0.0,
            ],
            0.01381855,
        ),
        (
            &[
                0.858832, 0.064548, 0.045572, 0.382405, 0.377735, 0.56668, 1.0, 0.758773, 0.372847,
                0.280386, 0.008324, 0.051628, 0.011826, 0.000222, 0.033638, 0.00966, -0.013242,
                0.524791, 0.361114, 0.22549, 0.0, 0.0, 1.0, 0.0, 0.778916, 1.088385, 2.4e-05,
                0.499065, 1.908748, 7e-05, 0.549057, 1.921688, 0.007092, 0.950417, 0.560816, 0.2,
                0.0, 0.0, 0.0, 0.0, 0.368682, -0.479594, 0.0217, 0.001815, 0.179693, -0.161724,
                0.010377, 0.393655, -0.308104, 1.0, 0.967672, 0.254533, 0.0, 0.0, 0.0, 0.5,
                0.737363, 0.959188, 0.0434, 0.00363, 0.359387, 0.323448, 0.020754, 0.787309,
                0.616209, 0.0, 0.064655, 0.408582, 0.0, 0.0, 0.0, 1.0,
            ],
            0.99742234,
        ),
    ];

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
