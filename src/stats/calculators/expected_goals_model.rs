//! Versioned logistic threat model for the expected-goals stat.
//!
//! The model maps a [`ThreatFeatures`] vector to
//! `V = sigmoid(bias + w . features)`: the probability that the attacking team
//! scores within the next [`THREAT_HORIZON_SECONDS`] seconds, evaluated on RAW
//! (unstandardized) features. The offline training pipeline standardizes
//! features internally and folds the standardization back into the published
//! bias/weights, so inference here stays a plain dot product over
//! [`ThreatFeatures::to_array`].
//!
//! Replacing the model is a mechanical edit confined to the generated
//! coefficients section below: paste a new `THREAT_MODEL_VERSION`,
//! `THREAT_MODEL_BIAS`, and `THREAT_MODEL_WEIGHTS` table (one weight per
//! [`ThreatFeatures::FEATURE_NAMES`] entry, same order).

use super::expected_goals::{THREAT_FEATURE_COUNT, ThreatFeatures};

/// Label horizon the model is (to be) trained against: V estimates the
/// probability of the attacking team scoring within this many seconds.
pub const THREAT_HORIZON_SECONDS: f32 = 5.0;

// ---------------------------------------------------------------------------
// GENERATED COEFFICIENTS -- BEGIN
//
// Everything between these markers is replaced wholesale when the offline
// training script publishes a fitted model.
//
// trained-v4 provenance: logistic regression fit by
// scripts/threat_model/train_threat_model.py (uv-locked environment) on
// 5.22M live-play team rows sampled at 4 Hz from 2,544 rank-stratified
// ranked-doubles replays (rocket-sense production corpus, rank tiers 3-22,
// 2026-07-12). Every player uses the same 16-field transform; each two-player
// team is represented by permutation-invariant mean/spread aggregates. The
// newest 20% of replays were held out temporally: log_loss 0.1355, Brier
// 0.0359, AUC 0.8837 (constant-rate baseline log_loss 0.1964); GBT reference
// ceiling log_loss 0.1287. Published coefficients were then refit on the full
// corpus. Standardization is folded in; weights apply to raw features.
// ---------------------------------------------------------------------------

/// Identifies the coefficient set embedded below. `heuristic-v0` marks the
/// hand-tuned placeholder; trained models use `trained-v<N>` stamps.
pub const THREAT_MODEL_VERSION: &str = "trained-v4";

pub const THREAT_MODEL_BIAS: f32 = -1.3350503;
pub const THREAT_MODEL_WEIGHTS: [(&str, f32); THREAT_FEATURE_COUNT] = [
    ("ball_forward_y", -0.5779634),
    ("ball_dist_to_goal", -5.708698),
    ("ball_height", -0.099439904),
    ("ball_speed", -2.3141916),
    ("ball_speed_toward_goal", 3.9342568),
    ("goal_open_angle", 1.511522),
    ("on_target", 1.6267366),
    ("time_to_goal_line", -0.3590436),
    ("own_team_mean_position_x", 0.018717207),
    ("own_team_mean_position_y", 0.30805534),
    ("own_team_mean_position_z", -0.41071174),
    ("own_team_mean_velocity_x", -0.0424128),
    ("own_team_mean_velocity_y", 0.78416425),
    ("own_team_mean_velocity_z", 0.9413253),
    ("own_team_mean_forward_x", 0.02993548),
    ("own_team_mean_forward_y", 0.4687313),
    ("own_team_mean_forward_z", 0.039111286),
    ("own_team_mean_distance_to_ball", -1.4057058),
    ("own_team_mean_distance_to_goal", -0.7276727),
    ("own_team_mean_boost", 0.46965355),
    ("own_team_mean_is_goalside", -1.4693916),
    ("own_team_mean_in_net", 0.13337676),
    ("own_team_mean_dodge_available", 0.29578996),
    ("own_team_mean_demoed", 0.80989605),
    ("own_team_spread_position_x", 0.15600729),
    ("own_team_spread_position_y", -0.35254636),
    ("own_team_spread_position_z", 0.1198017),
    ("own_team_spread_velocity_x", 0.061510094),
    ("own_team_spread_velocity_y", 0.410587),
    ("own_team_spread_velocity_z", 0.3519663),
    ("own_team_spread_forward_x", -0.009791375),
    ("own_team_spread_forward_y", 0.17669073),
    ("own_team_spread_forward_z", -0.008849148),
    ("own_team_spread_distance_to_ball", 0.45837495),
    ("own_team_spread_distance_to_goal", 0.4484276),
    ("own_team_spread_boost", -0.15753853),
    ("own_team_spread_is_goalside", 0.53830355),
    ("own_team_spread_in_net", -0.10456076),
    ("own_team_spread_dodge_available", -0.04883177),
    ("own_team_spread_demoed", -0.2931511),
    ("opponent_team_mean_position_x", -0.008389273),
    ("opponent_team_mean_position_y", 0.47667402),
    ("opponent_team_mean_position_z", 0.7273689),
    ("opponent_team_mean_velocity_x", -0.018365024),
    ("opponent_team_mean_velocity_y", -0.14907622),
    ("opponent_team_mean_velocity_z", -1.0305297),
    ("opponent_team_mean_forward_x", -0.016495481),
    ("opponent_team_mean_forward_y", 0.06543813),
    ("opponent_team_mean_forward_z", -0.008724887),
    ("opponent_team_mean_distance_to_ball", 0.6860846),
    ("opponent_team_mean_distance_to_goal", 3.8129063),
    ("opponent_team_mean_boost", -0.51951563),
    ("opponent_team_mean_is_goalside", -0.67251307),
    ("opponent_team_mean_in_net", -0.011148059),
    ("opponent_team_mean_dodge_available", -0.1358538),
    ("opponent_team_mean_demoed", -0.31512168),
    ("opponent_team_spread_position_x", 0.058660593),
    ("opponent_team_spread_position_y", 0.3145817),
    ("opponent_team_spread_position_z", -0.10809895),
    ("opponent_team_spread_velocity_x", -0.17743108),
    ("opponent_team_spread_velocity_y", -0.12635869),
    ("opponent_team_spread_velocity_z", -0.35638762),
    ("opponent_team_spread_forward_x", 0.043583564),
    ("opponent_team_spread_forward_y", 0.05177123),
    ("opponent_team_spread_forward_z", 0.103313714),
    ("opponent_team_spread_distance_to_ball", -0.33313403),
    ("opponent_team_spread_distance_to_goal", -1.4004678),
    ("opponent_team_spread_boost", 0.040076595),
    ("opponent_team_spread_is_goalside", -0.15839666),
    ("opponent_team_spread_in_net", -0.004282083),
    ("opponent_team_spread_dodge_available", -0.012042165),
    ("opponent_team_spread_demoed", 0.05703548),
];

// ---------------------------------------------------------------------------
// GENERATED COEFFICIENTS -- END
// ---------------------------------------------------------------------------

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// Evaluate the threat model on one feature vector: the probability, in
/// (0, 1), that the attacking team scores within [`THREAT_HORIZON_SECONDS`].
pub fn threat_value(features: &ThreatFeatures) -> f32 {
    threat_value_from_array(&features.to_array())
}

/// Evaluate the model on a raw feature vector in
/// [`ThreatFeatures::FEATURE_NAMES`] order. This is the exact inference path
/// [`threat_value`] uses; it is public so parity against the offline training
/// pipeline's predictions can be asserted on shared fixtures.
pub fn threat_value_from_array(values: &[f32; THREAT_FEATURE_COUNT]) -> f32 {
    let mut logit = THREAT_MODEL_BIAS;
    for ((_, weight), value) in THREAT_MODEL_WEIGHTS.iter().zip(values.iter()) {
        logit += weight * value;
    }
    sigmoid(logit)
}

#[cfg(test)]
#[path = "expected_goals_model_tests.rs"]
mod tests;
