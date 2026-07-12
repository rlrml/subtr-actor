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
// trained-v1 provenance: logistic regression fit by
// scripts/threat_model/train_threat_model.py on 10.3M live-play rows sampled
// at 4 Hz from 5,280 rank-stratified ranked-duels/-doubles replays
// (rocket-sense production corpus, rank tiers 1-22, 2026-07-12). Grouped
// train/test split by replay. Held-out: log_loss 0.169, Brier 0.0468,
// AUC 0.885 (constant-rate baseline log_loss 0.252); GBT reference ceiling
// log_loss 0.154. Calibration tracks observed frequency within ~10% relative
// across all 15 prediction-quantile bins and across rank tiers.
// Standardization is folded in; weights apply to raw features.
// ---------------------------------------------------------------------------

/// Identifies the coefficient set embedded below. `heuristic-v0` marks the
/// hand-tuned placeholder; trained models use `trained-v<N>` stamps.
pub const THREAT_MODEL_VERSION: &str = "trained-v1";

pub const THREAT_MODEL_BIAS: f32 = -0.441_601_53;

/// One weight per [`ThreatFeatures::FEATURE_NAMES`] entry, in that exact
/// order. The pairing is enforced by `weights_cover_every_feature` in the
/// adjacent test module.
pub const THREAT_MODEL_WEIGHTS: [(&str, f32); THREAT_FEATURE_COUNT] = [
    ("ball_forward_y", -0.030_214_028),
    ("ball_dist_to_goal", -4.603_913),
    ("ball_height", -0.096_484_2),
    ("ball_speed", -2.722_955_2),
    ("ball_speed_toward_goal", 5.354_401_5),
    ("goal_open_angle", 1.949_296_7),
    ("on_target", 1.312_370_6),
    ("time_to_goal_line", -0.260_346_38),
    ("nearest_attacker_dist", -2.061_250_7),
    ("attackers_ahead_of_ball", -0.552_131_2),
    ("attackers_behind_ball", 0.387_900_9),
    ("nearest_defender_dist", 1.003_443_6),
    ("nearest_defender_to_goal_dist", 1.805_623),
    ("defenders_goalside", -0.772_091_76),
    ("defender_in_net", -0.183_698_83),
    ("nearest_defender_boost", -0.404_413_3),
    ("attacking_team_size", -0.265_604_6),
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
