use std::collections::HashMap;

use super::rotation_modes::{build_euler_modes, build_modes};
use super::rotation_types::{EulerMode, QuaternionMode};

#[derive(Debug, Default)]
pub(crate) struct ModeAccumulator {
    pub(crate) alignments: Vec<f32>,
    pub(crate) up_zs: Vec<f32>,
}

#[derive(Debug, Default)]
pub(crate) struct VelocityScaleAccumulator {
    pub(crate) ratios: Vec<f32>,
}

#[derive(Debug, Default)]
pub(crate) struct AngularVelocityAccumulator {
    pub(crate) direction_dots: Vec<f32>,
}

#[derive(Debug)]
pub(crate) struct LegacyRotationProbe {
    pub(crate) modes: Vec<QuaternionMode>,
    pub(crate) accumulators: HashMap<QuaternionMode, ModeAccumulator>,
    pub(crate) euler_modes: Vec<EulerMode>,
    pub(crate) euler_accumulators: HashMap<EulerMode, ModeAccumulator>,
    pub(crate) euler_angular_accumulators: HashMap<EulerMode, AngularVelocityAccumulator>,
    pub(crate) velocity_accumulators: Vec<(f32, VelocityScaleAccumulator)>,
    pub(crate) previous_bodies: HashMap<subtr_actor::PlayerId, (f32, boxcars::RigidBody)>,
}

impl LegacyRotationProbe {
    pub(crate) fn new() -> Self {
        let modes = build_modes();
        let accumulators = modes
            .iter()
            .copied()
            .map(|mode| (mode, ModeAccumulator::default()))
            .collect();
        let euler_modes = build_euler_modes();
        let euler_accumulators = euler_modes
            .iter()
            .copied()
            .map(|mode| (mode, ModeAccumulator::default()))
            .collect();
        let euler_angular_accumulators = euler_modes
            .iter()
            .copied()
            .map(|mode| (mode, AngularVelocityAccumulator::default()))
            .collect();
        let velocity_accumulators = [1.0, 0.1, 0.01]
            .into_iter()
            .map(|scale| (scale, VelocityScaleAccumulator::default()))
            .collect();
        Self {
            modes,
            accumulators,
            euler_modes,
            euler_accumulators,
            euler_angular_accumulators,
            velocity_accumulators,
            previous_bodies: HashMap::new(),
        }
    }
}
