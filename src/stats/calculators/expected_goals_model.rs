//! Deployed five-second scoring threat and the retained v6 baseline.
//!
//! The default path uses the calibrated entity model. The legacy 154-feature
//! function and coefficients remain available for historical parity checks.
//! See `scripts/threat_model/EVALUATION.md` for v7 validation and provenance.

use super::expected_goals::{THREAT_MODEL_FEATURE_COUNT, ThreatModelFeatures};

/// Label horizon the model is (to be) trained against: V estimates the
/// probability of the attacking team scoring within this many seconds.
pub const THREAT_HORIZON_SECONDS: f32 =
    super::expected_goals_entity_model::ENTITY_THREAT_HORIZON_SECONDS;

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

/// Identifies the deployed frame/touch model pair.
pub const THREAT_MODEL_VERSION: &str = "trained-v7-entity-precontact";

include!("expected_goals_model_weights.rs");

// GENERATED COEFFICIENTS -- END
// ---------------------------------------------------------------------------

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// Evaluate the threat model on one feature vector: the probability, in
/// (0, 1), that the attacking team scores within [`THREAT_HORIZON_SECONDS`].
pub fn threat_value(features: &ThreatModelFeatures) -> f32 {
    super::expected_goals_entity_model::entity_threat_value(features)
}

/// Evaluate the retained v6 baseline in [`ThreatModelFeatures::feature_names`]
/// order. This is not the default deployed inference path.
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
