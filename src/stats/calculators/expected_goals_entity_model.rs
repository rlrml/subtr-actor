//! Permutation-invariant inference shared by frame threat and pre-contact xG.

use super::expected_goals::{
    THREAT_ENTITY_FEATURE_COUNT, THREAT_ENTITY_GLOBAL_COUNT, ThreatModelFeatures,
};

pub const TOUCH_XG_HORIZON_SECONDS: f32 = touch::HORIZON_SECONDS;
pub const ENTITY_THREAT_HORIZON_SECONDS: f32 = frame::HORIZON_SECONDS;

struct Weights {
    reflection: &'static [f32; THREAT_ENTITY_FEATURE_COUNT],
    mean: &'static [f32; THREAT_ENTITY_FEATURE_COUNT],
    scale: &'static [f32; THREAT_ENTITY_FEATURE_COUNT],
    player1_weight: &'static [[f32; 23]; 32],
    player1_bias: &'static [f32; 32],
    player2_weight: &'static [[f32; 32]; 16],
    player2_bias: &'static [f32; 16],
    hidden1_weight: &'static [[f32; 224]; 64],
    hidden1_bias: &'static [f32; 64],
    hidden2_weight: &'static [[f32; 64]; 32],
    hidden2_bias: &'static [f32; 32],
    output_weight: &'static [[f32; 32]; 1],
    output_bias: &'static [f32; 1],
    calibration_slope: f32,
    calibration_intercept: f32,
}

macro_rules! model {
    ($name:ident, $file:literal) => {
        mod $name {
            include!($file);
            pub(super) const WEIGHTS: super::Weights = super::Weights {
                mean: &MEAN,
                scale: &SCALE,
                reflection: &REFLECTION,
                player1_weight: &PLAYER1_WEIGHT,
                player1_bias: &PLAYER1_BIAS,
                player2_weight: &PLAYER2_WEIGHT,
                player2_bias: &PLAYER2_BIAS,
                hidden1_weight: &HIDDEN1_WEIGHT,
                hidden1_bias: &HIDDEN1_BIAS,
                hidden2_weight: &HIDDEN2_WEIGHT,
                hidden2_bias: &HIDDEN2_BIAS,
                output_weight: &OUTPUT_WEIGHT,
                output_bias: &OUTPUT_BIAS,
                calibration_slope: CALIBRATION_SLOPE,
                calibration_intercept: CALIBRATION_INTERCEPT,
            };
        }
    };
}

model!(frame, "expected_goals_entity_frame_weights.rs");
model!(touch, "expected_goals_entity_touch_weights.rs");

fn linear<const I: usize, const O: usize>(
    input: &[f32; I],
    weights: &[[f32; I]; O],
    bias: &[f32; O],
) -> [f32; O] {
    std::array::from_fn(|out| {
        input
            .iter()
            .zip(weights[out])
            .fold(bias[out], |sum, (x, w)| sum + x * w)
    })
}

fn raw_probability(values: &[f32; THREAT_ENTITY_FEATURE_COUNT], weights: &Weights) -> f32 {
    let scaled: [f32; THREAT_ENTITY_FEATURE_COUNT] =
        std::array::from_fn(|i| (values[i] - weights.mean[i]) / weights.scale[i]);
    let players: [[f32; 16]; 4] = std::array::from_fn(|player| {
        let inputs = std::array::from_fn(|i| {
            if i < 17 {
                scaled[THREAT_ENTITY_GLOBAL_COUNT + player * 17 + i]
            } else {
                scaled[154 + i - 17]
            }
        });
        let hidden = linear(&inputs, weights.player1_weight, weights.player1_bias).map(f32::tanh);
        linear(&hidden, weights.player2_weight, weights.player2_bias).map(f32::tanh)
    });
    let mut context = [0.0; 224];
    context[..THREAT_ENTITY_GLOBAL_COUNT].copy_from_slice(&scaled[..THREAT_ENTITY_GLOBAL_COUNT]);
    for team in 0..2 {
        for feature in 0..16 {
            let first = players[team * 2][feature];
            let second = players[team * 2 + 1][feature];
            context[THREAT_ENTITY_GLOBAL_COUNT + team * 32 + feature] = (first + second) * 0.5;
            context[THREAT_ENTITY_GLOBAL_COUNT + team * 32 + 16 + feature] = (first - second).abs();
        }
    }
    let hidden = linear(&context, weights.hidden1_weight, weights.hidden1_bias).map(f32::tanh);
    let hidden = linear(&hidden, weights.hidden2_weight, weights.hidden2_bias).map(f32::tanh);
    let logit = linear(&hidden, weights.output_weight, weights.output_bias)[0];
    1.0 / (1.0 + (-logit).exp())
}

fn evaluate(values: &[f32; THREAT_ENTITY_FEATURE_COUNT], weights: &Weights) -> f32 {
    let reflected = std::array::from_fn(|i| values[i] * weights.reflection[i]);
    let p = ((raw_probability(values, weights) + raw_probability(&reflected, weights)) * 0.5)
        .clamp(1e-7, 1.0 - 1e-7);
    let logit = (p / (1.0 - p)).ln() * weights.calibration_slope + weights.calibration_intercept;
    1.0 / (1.0 + (-logit).exp())
}

pub fn entity_threat_value(features: &ThreatModelFeatures) -> f32 {
    entity_threat_value_from_array(&features.to_entity_array())
}

pub(crate) fn entity_threat_value_from_array(values: &[f32; THREAT_ENTITY_FEATURE_COUNT]) -> f32 {
    evaluate(values, &frame::WEIGHTS)
}

/// Valid only at detected touches, using the pre-contact feature snapshot.
pub fn touch_xg_value(features: &super::expected_goals::PreContactThreatFeatures) -> f32 {
    evaluate(&features.to_entity_array(), &touch::WEIGHTS)
}

#[cfg(test)]
#[path = "expected_goals_entity_model_tests.rs"]
mod tests;

#[cfg(test)]
pub(crate) fn entity_model_reflections() -> [&'static [f32; THREAT_ENTITY_FEATURE_COUNT]; 2] {
    [&frame::REFLECTION, &touch::REFLECTION]
}
