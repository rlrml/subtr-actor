//! Inference for gradient-boosted trees exported by `scripts/whiff_model`.
//!
//! The export mirrors scikit-learn's multiclass `GradientBoostingClassifier`:
//! raw scores start at the log class prior, each stage adds
//! `learning_rate * tree(x)` per class, and probabilities are the softmax.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Tree {
    pub feature: Vec<i64>,
    pub threshold: Vec<f64>,
    pub left: Vec<i64>,
    pub right: Vec<i64>,
    pub value: Vec<f64>,
}

impl Tree {
    fn predict(&self, x: &[f64]) -> f64 {
        let mut node = 0usize;
        while self.left[node] >= 0 {
            let feature = self.feature[node] as usize;
            node = if x[feature] <= self.threshold[node] {
                self.left[node] as usize
            } else {
                self.right[node] as usize
            };
        }
        self.value[node]
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WhiffModel {
    pub name: String,
    pub classes: Vec<String>,
    pub feature_names: Vec<String>,
    pub feature_version: String,
    pub missing_value: f64,
    pub learning_rate: f64,
    pub init_log_prior: Vec<f64>,
    pub stages: Vec<Vec<Tree>>,
    /// Minimum probability for event classes to win the argmax.
    #[serde(default)]
    pub thresholds: std::collections::BTreeMap<String, f64>,
}

impl WhiffModel {
    pub fn from_json(text: &str) -> anyhow::Result<Self> {
        let model: Self = serde_json::from_str(text)?;
        anyhow::ensure!(
            model.feature_names.len() == super::features::FEATURE_NAMES.len()
                && model
                    .feature_names
                    .iter()
                    .zip(super::features::FEATURE_NAMES)
                    .all(|(a, b)| a == b),
            "model feature order does not match this build"
        );
        anyhow::ensure!(
            model.feature_version == super::features::FEATURE_VERSION,
            "model feature version {} does not match {}",
            model.feature_version,
            super::features::FEATURE_VERSION
        );
        Ok(model)
    }

    pub fn predict_proba(&self, features: &[f64]) -> Vec<f64> {
        let x: Vec<f64> = features
            .iter()
            .map(|v| if v.is_nan() { self.missing_value } else { *v })
            .collect();
        let mut raw = self.init_log_prior.clone();
        for stage in &self.stages {
            for (k, tree) in stage.iter().enumerate() {
                raw[k] += self.learning_rate * tree.predict(&x);
            }
        }
        let max = raw.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let exp: Vec<f64> = raw.iter().map(|r| (r - max).exp()).collect();
        let total: f64 = exp.iter().sum();
        exp.iter().map(|e| e / total).collect()
    }

    /// Thresholded argmax, matching `modeling.predict_classes`.
    pub fn predict_class(&self, features: &[f64]) -> &str {
        let proba = self.predict_proba(features);
        let mut order: Vec<usize> = (0..proba.len()).collect();
        order.sort_by(|a, b| proba[*b].total_cmp(&proba[*a]));
        let best = order[0];
        let name = self.classes[best].as_str();
        match self.thresholds.get(name) {
            Some(threshold) if proba[best] < *threshold => order
                .iter()
                .map(|k| self.classes[*k].as_str())
                .find(|class| !self.thresholds.contains_key(*class))
                .unwrap_or(name),
            _ => name,
        }
    }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
