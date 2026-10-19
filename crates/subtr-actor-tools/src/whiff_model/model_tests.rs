use super::*;
use crate::whiff_model::features::{FEATURE_NAMES, FEATURE_VERSION};

fn stump(feature: i64, threshold: f64, left: f64, right: f64) -> Tree {
    Tree {
        feature: vec![feature, -2, -2],
        threshold: vec![threshold, -2.0, -2.0],
        left: vec![1, -1, -1],
        right: vec![2, -1, -1],
        value: vec![0.0, left, right],
    }
}

fn model(thresholds: &[(&str, f64)]) -> WhiffModel {
    WhiffModel {
        name: "test".to_owned(),
        classes: vec!["clear_whiff".to_owned(), "successful_contact".to_owned()],
        feature_names: FEATURE_NAMES.iter().map(|n| n.to_string()).collect(),
        feature_version: FEATURE_VERSION.to_owned(),
        missing_value: -999.0,
        learning_rate: 1.0,
        init_log_prior: vec![0.0, 0.0],
        stages: vec![vec![stump(0, 10.0, 2.0, -2.0), stump(0, 10.0, -2.0, 2.0)]],
        thresholds: thresholds
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect(),
    }
}

fn features(gap: f64) -> Vec<f64> {
    let mut x = vec![0.0; FEATURE_NAMES.len()];
    x[0] = gap;
    x
}

#[test]
fn trees_split_left_on_less_or_equal_and_softmax() {
    let m = model(&[]);
    let at_threshold = m.predict_proba(&features(10.0));
    let expected = 1.0 / (1.0 + (-4.0f64).exp());
    assert!((at_threshold[0] - expected).abs() < 1e-12);
    assert_eq!(m.predict_class(&features(11.0)), "successful_contact");
}

#[test]
fn nan_features_use_the_missing_value() {
    let m = model(&[]);
    assert_eq!(
        m.predict_proba(&features(f64::NAN)),
        m.predict_proba(&features(-999.0))
    );
}

#[test]
fn event_class_below_threshold_falls_back_to_next_non_event_class() {
    let m = model(&[("clear_whiff", 0.999)]);
    assert_eq!(m.predict_class(&features(0.0)), "successful_contact");
}

#[test]
fn mismatched_feature_order_is_rejected() {
    let mut m = model(&[]);
    m.feature_names.swap(0, 1);
    let text = serde_json::json!({
        "name": m.name, "classes": m.classes, "feature_names": m.feature_names,
        "feature_version": m.feature_version, "missing_value": -999.0, "learning_rate": 1.0,
        "init_log_prior": [0.0, 0.0], "stages": [],
    });
    assert!(WhiffModel::from_json(&text.to_string()).is_err());
}
