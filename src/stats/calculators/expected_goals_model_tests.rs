use super::*;
use crate::TeamThreatFeatures;

/// The first-layer weight table must stay keyed exactly by
/// [`ThreatFeatures::FEATURE_NAMES`], in order -- pasting trained weights is a
/// mechanical edit only while this holds.
#[test]
fn weights_cover_every_feature_in_order() {
    assert_eq!(THREAT_MODEL_INPUT_WEIGHTS.len(), THREAT_FEATURE_COUNT);
    for ((weight_name, weights), feature_name) in THREAT_MODEL_INPUT_WEIGHTS
        .iter()
        .zip(ThreatFeatures::FEATURE_NAMES.iter())
    {
        assert_eq!(weight_name, feature_name);
        assert_eq!(weights.len(), THREAT_MODEL_HIDDEN_UNITS);
    }
    assert_eq!(THREAT_MODEL_HIDDEN_BIASES.len(), THREAT_MODEL_HIDDEN_UNITS);
    assert_eq!(THREAT_MODEL_OUTPUT_WEIGHTS.len(), THREAT_MODEL_HIDDEN_UNITS);
}

#[test]
fn rust_inference_matches_locked_training_pipeline() {
    // Low, median, and high temporal-holdout rows emitted by the uv/Nix-locked
    // trainer. Three rows cover the numerical range without checking in a
    // bulky dataset fixture.
    let fixtures: &[(&[f32; THREAT_FEATURE_COUNT], f32)] = &[
        (
            &[
                -0.96218, 0.925188, 0.104428, 0.249714, -0.241733, 0.053084, 0.0, 0.0, -0.487075,
                -0.062788, 0.039922, 0.024665, -0.847772, -0.041446, -0.27894, -0.611723,
                -0.214726, 0.955397, 0.551591, 0.201961, 1.0, 0.0, 0.0, 0.0, 0.795908, 0.657334,
                0.018151, 0.163557, 0.062961, 0.07677, 0.141425, 0.523912, 1.308831, 0.089206,
                0.170685, 0.403922, 0.0, 0.0, 0.0, 0.0, -0.196389, -0.357214, 0.008319, -0.301372,
                -0.054663, 0.000135, -0.473223, -0.44209, -0.00963, 0.825421, 0.672191, 1.0, 1.0,
                0.0, 1.0, 0.0, 1.396052, 0.296608, 5e-06, 0.618239, 0.220761, 5.2e-05, 1.04027,
                1.113524, 0.000169, 0.349158, 0.179705, 0.0, 0.0, 0.0, 0.0, 0.0,
            ],
            1.2671989e-05,
        ),
        (
            &[
                -0.834768, 0.878126, 0.194393, 0.14428, 0.032285, 0.055151, 0.0, 0.0, 0.584054,
                -0.853876, 0.055987, 0.43948, 0.10223, 0.129237, 0.858966, 0.196267, 0.141177,
                0.153861, 0.874028, 0.790474, 0.5, 0.0, 1.0, 0.0, 0.000884, 0.071093, 0.077821,
                0.128004, 0.311243, 0.001274, 0.138747, 0.834277, 0.315651, 0.032005, 0.03156,
                0.320915, 1.0, 0.0, 0.0, 0.0, 0.61513, -0.356371, 0.016677, 0.265415, -0.11932,
                -0.073148, 0.013493, -0.18861, -0.156092, 0.575371, 0.659995, 0.647059, 0.5, 0.0,
                0.5, 0.0, 0.371751, 1.077598, 0.016729, 1.363196, 0.072961, 0.146757, 1.913777,
                0.105561, 0.292978, 0.849259, 0.509122, 0.705882, 1.0, 0.0, 1.0, 0.0,
            ],
            0.0021030817,
        ),
        (
            &[
                0.066164, 0.538284, 0.300059, 0.342233, 0.057796, 0.075575, 0.0, 0.0, -0.022155,
                0.014199, 0.026363, -0.055111, -0.047265, -0.003167, 0.469069, 0.430992, -0.485502,
                0.777381, 0.49211, 0.268627, 0.5, 0.0, 0.5, 0.0, 1.046838, 0.457496, 0.036081,
                1.139352, 0.971261, 0.006552, 0.56392, 0.458414, 0.951779, 0.445237, 0.185265,
                0.537255, 1.0, 0.0, 1.0, 0.0, 0.684725, 0.250195, 0.008322, 0.543522, 0.527483,
                0.000117, 0.692867, 0.583069, -0.009586, 0.448671, 0.428114, 0.0, 0.5, 0.0, 1.0,
                0.0, 0.139915, 0.54652, 0.0, 0.200922, 0.7114, 0.0, 0.546158, 0.649006, 2e-06,
                0.433581, 0.229961, 0.0, 1.0, 0.0, 0.0, 0.0,
            ],
            0.012173547,
        ),
        (
            &[
                0.43535, 0.436157, 0.378987, 0.198702, 0.095721, 0.071475, 0.0, 0.194624, 0.805072,
                -0.131209, 0.151426, 0.276896, -0.272502, 0.216659, 0.313352, -0.13858, 0.425392,
                0.533471, 0.627687, 0.0, 0.0, 0.0, 1.0, 0.0, 0.38156, 1.117762, 0.286208, 0.55427,
                1.076161, 0.433074, 0.60767, 1.296356, 0.869935, 0.933058, 0.353907, 0.0, 0.0, 0.0,
                0.0, 0.0, -0.011678, 0.433968, 0.035881, -0.212563, 0.501893, 0.23715, -0.178231,
                0.670245, 0.411837, 0.816757, 0.434632, 0.5, 0.5, 0.0, 1.0, 0.0, 1.667278,
                0.938955, 0.055127, 0.671822, 0.3839, 0.473943, 0.767197, 0.313906, 0.842886,
                0.366487, 0.261577, 1.0, 1.0, 0.0, 0.0, 0.0,
            ],
            0.047502104,
        ),
        (
            &[
                0.966717, 0.029268, 0.047515, 0.367709, 0.34372, 0.867728, 1.0, 0.910348, 0.0083,
                0.789181, 0.019889, 0.759365, 0.429302, 0.055039, 0.742522, 0.592933, -0.10626,
                0.240938, 0.096608, 0.492157, 0.0, 0.0, 0.5, 0.0, 0.022379, 0.081877, 0.023134,
                0.469, 0.841839, 0.109843, 0.327925, 0.445301, 0.193319, 0.107006, 0.037799,
                0.262745, 0.0, 0.0, 1.0, 0.0, -0.678044, -0.087629, 0.025289, 0.125807, 0.853424,
                -0.001824, -0.264759, 0.882893, 0.111929, 1.0, 0.565673, 0.545201, 0.0, 0.0, 0.5,
                0.0, 0.420317, 0.363465, 0.034276, 0.244196, 0.1095, 0.009804, 0.663393, 0.229643,
                0.242214, 0.0, 0.078619, 0.815892, 0.0, 0.0, 1.0, 0.0,
            ],
            0.9994111,
        ),
        (
            &[
                0.952699, 0.021898, 0.045572, 0.332711, 0.325819, 0.831078, 1.0, 0.885155,
                0.035382, 0.388345, 0.295964, -0.13335, 0.870457, 0.109611, 0.214073, -0.189625,
                -0.133566, 0.742218, 0.293057, 0.396078, 0.0, 0.0, 0.0, 0.0, 0.1228, 0.500603,
                0.49774, 0.481222, 0.235687, 0.192752, 0.201112, 0.929666, 1.642186, 0.515564,
                0.202191, 0.792157, 0.0, 0.0, 0.0, 0.0, 0.447262, -0.203876, 0.030824, -0.266467,
                0.901317, -0.007328, -0.663567, 0.640405, 0.214863, 1.0, 0.599773, 0.037255, 0.0,
                0.0, 0.5, 0.0, 0.798191, 0.497635, 0.046208, 0.502578, 0.121104, 0.001204,
                0.244412, 0.400742, 0.439597, 0.0, 0.12917, 0.07451, 0.0, 0.0, 1.0, 0.0,
            ],
            0.99955446,
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
