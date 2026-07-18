//! Versioned nonlinear threat model for the expected-goals stat.
//!
//! The model maps a [`ThreatModelFeatures`] vector to
//! an eight-hidden-unit tanh MLP: the probability that the attacking team
//! scores within the next [`THREAT_HORIZON_SECONDS`] seconds, evaluated on raw
//! (unstandardized) features. The offline training pipeline folds input
//! standardization into the published first-layer weights, so inference here
//! is a pair of small matrix-vector products over
//! [`ThreatModelFeatures::to_array`].
//!
//! Replacing the model is a mechanical edit confined to the generated
//! coefficients module and version/provenance section below. The input-weight
//! table has one named row per [`ThreatModelFeatures::feature_names`] entry, in
//! the same order.

use super::expected_goals::{THREAT_MODEL_FEATURE_COUNT, ThreatModelFeatures};

/// Label horizon the model is (to be) trained against: V estimates the
/// probability of the attacking team scoring within this many seconds.
pub const THREAT_HORIZON_SECONDS: f32 = 5.0;

// ---------------------------------------------------------------------------
// GENERATED COEFFICIENTS -- BEGIN
//
// trained-v6 provenance: 8-hidden-unit tanh MLP fit by
// scripts/threat_model/train_threat_model.py (uv-locked environment) on
// 5.22M live-play team rows sampled at 4 Hz from 2,544 rank-stratified
// ranked-doubles replays (rocket-sense production corpus, rank tiers 3-22,
// 2026-07-18). Inputs include the instantaneous symmetric state plus causal
// 0.5s and 1.0s changes. The newest 20% of replays were held out temporally:
// log_loss 0.12936, Brier 0.03444, AUC 0.8962, and 15-bin ECE 0.00117 (linear
// baseline log_loss 0.13433). The published model was refit on the full
// corpus. Input standardization is folded into the first-layer weights, which
// apply directly to raw features.
// ---------------------------------------------------------------------------

/// Identifies the generated model embedded below.
pub const THREAT_MODEL_VERSION: &str = "trained-v6-temporal";

include!("expected_goals_model_weights.rs");

// GENERATED COEFFICIENTS -- END
// ---------------------------------------------------------------------------

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// Evaluate the threat model on one feature vector: the probability, in
/// (0, 1), that the attacking team scores within [`THREAT_HORIZON_SECONDS`].
pub fn threat_value(features: &ThreatModelFeatures) -> f32 {
    threat_value_from_array(&features.to_array())
}

/// Evaluate the model on a raw feature vector in
/// [`ThreatModelFeatures::feature_names`] order. This is the exact inference
/// path [`threat_value`] uses; it is public so parity against the offline
/// training pipeline's predictions can be asserted on shared fixtures.
pub fn threat_value_from_array(values: &[f32; THREAT_MODEL_FEATURE_COUNT]) -> f32 {
    let mut hidden = THREAT_MODEL_HIDDEN_BIASES;
    for ((_, weights), value) in THREAT_MODEL_INPUT_WEIGHTS.iter().zip(values.iter()) {
        for (activation, weight) in hidden.iter_mut().zip(weights) {
            *activation += weight * value;
        }
    }
    hidden.iter_mut().for_each(|activation| {
        *activation = activation.tanh();
    });
    let logit = hidden
        .iter()
        .zip(THREAT_MODEL_OUTPUT_WEIGHTS)
        .fold(THREAT_MODEL_OUTPUT_BIAS, |sum, (activation, weight)| {
            sum + activation * weight
        });
    sigmoid(logit)
}

#[cfg(test)]
#[path = "expected_goals_model_tests.rs"]
mod tests;
