//! Deployed whiff attempt classifier: gradient-boosted trees over
//! [`super::whiff_features`] vectors.
//!
//! The export mirrors scikit-learn's multiclass `GradientBoostingClassifier`:
//! raw scores start at the log class prior, each stage adds
//! `learning_rate * tree(x)` per class, and probabilities are the softmax.
//! See `scripts/whiff_model/EVALUATION.md` for training data and validation.

use super::whiff_features::{WHIFF_FEATURE_COUNT, WhiffFeatures};

/// Outcome classes, in model output order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhiffAttemptClass {
    /// Attempted, no contact, not disrupted.
    CleanWhiff,
    /// Attempted, no contact, another player's touch took the ball first.
    DisruptedAttempt,
    SuccessfulContact,
    NoObservableAttempt,
}

pub const WHIFF_ATTEMPT_CLASSES: [WhiffAttemptClass; 4] = [
    WhiffAttemptClass::CleanWhiff,
    WhiffAttemptClass::DisruptedAttempt,
    WhiffAttemptClass::SuccessfulContact,
    WhiffAttemptClass::NoObservableAttempt,
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WhiffClassification {
    pub class: WhiffAttemptClass,
    /// Class probabilities in [`WHIFF_ATTEMPT_CLASSES`] order.
    pub probabilities: [f64; 4],
}

impl WhiffClassification {
    pub fn probability(&self) -> f64 {
        let index = WHIFF_ATTEMPT_CLASSES
            .iter()
            .position(|class| *class == self.class)
            .expect("class is listed");
        self.probabilities[index]
    }
}

/// One tree node; leaves have `feature < 0`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct WhiffTreeNode {
    feature: i16,
    threshold: f64,
    left: i16,
    right: i16,
    value: f64,
}

const fn n(feature: i16, threshold: f64, left: i16, right: i16, value: f64) -> WhiffTreeNode {
    WhiffTreeNode {
        feature,
        threshold,
        left,
        right,
        value,
    }
}

fn tree_value(nodes: &[WhiffTreeNode], x: &[f64; WHIFF_FEATURE_COUNT]) -> f64 {
    let mut node = &nodes[0];
    while node.feature >= 0 {
        let next = if x[node.feature as usize] <= node.threshold {
            node.left
        } else {
            node.right
        };
        node = &nodes[next as usize];
    }
    node.value
}

// ---------------------------------------------------------------------------
// GENERATED MODEL -- BEGIN (scripts/whiff_model `export-rust`)
include!("whiff_model_weights.rs");
// GENERATED MODEL -- END
// ---------------------------------------------------------------------------

/// Class probabilities in [`WHIFF_ATTEMPT_CLASSES`] order.
pub fn whiff_class_probabilities(features: &WhiffFeatures) -> [f64; 4] {
    let mut x = *features;
    for value in &mut x {
        if value.is_nan() {
            *value = WHIFF_MODEL_MISSING_VALUE;
        }
    }
    let mut raw = WHIFF_MODEL_INIT_LOG_PRIOR;
    for stage in WHIFF_MODEL_STAGES {
        for (score, tree) in raw.iter_mut().zip(stage.iter()) {
            *score += WHIFF_MODEL_LEARNING_RATE * tree_value(tree, &x);
        }
    }
    let max = raw.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mut exp = raw.map(|r| (r - max).exp());
    let total: f64 = exp.iter().sum();
    for value in &mut exp {
        *value /= total;
    }
    exp
}

/// Thresholded argmax: an event class wins only at or above its threshold;
/// otherwise the most probable non-event class is chosen.
pub fn classify_whiff_candidate(features: &WhiffFeatures) -> WhiffClassification {
    let probabilities = whiff_class_probabilities(features);
    let mut order = [0usize, 1, 2, 3];
    order.sort_by(|a, b| probabilities[*b].total_cmp(&probabilities[*a]));
    let threshold = |class: WhiffAttemptClass| match class {
        WhiffAttemptClass::CleanWhiff => Some(WHIFF_MODEL_CLEAN_WHIFF_THRESHOLD),
        WhiffAttemptClass::DisruptedAttempt => Some(WHIFF_MODEL_DISRUPTED_THRESHOLD),
        _ => None,
    };
    let best = WHIFF_ATTEMPT_CLASSES[order[0]];
    let class = match threshold(best) {
        Some(minimum) if probabilities[order[0]] < minimum => order
            .iter()
            .map(|index| WHIFF_ATTEMPT_CLASSES[*index])
            .find(|class| threshold(*class).is_none())
            .expect("non-event classes exist"),
        _ => best,
    };
    WhiffClassification {
        class,
        probabilities,
    }
}

#[cfg(test)]
#[path = "whiff_model_tests.rs"]
mod tests;
