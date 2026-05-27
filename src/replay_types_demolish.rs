use serde::Serialize;

use super::PlayerId;

/// Represents which demolition format a replay uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemolishFormat {
    /// Old format (pre-September 2024): uses `ReplicatedDemolishGoalExplosion`
    Fx,
    /// New format (September 2024+): uses `ReplicatedDemolishExtended`
    Extended,
}

/// Wrapper enum for different demolition attribute formats across Rocket League versions.
///
/// Rocket League changed the demolition data structure around September 2024 (v2.43+),
/// moving from `DemolishFx` to `DemolishExtended`. This enum provides a unified interface
/// for both formats.
#[derive(Debug, Clone, PartialEq)]
pub enum DemolishAttribute {
    Fx(boxcars::DemolishFx),
    Extended(boxcars::DemolishExtended),
}

impl DemolishAttribute {
    pub fn attacker_actor_id(&self) -> boxcars::ActorId {
        match self {
            DemolishAttribute::Fx(fx) => fx.attacker,
            DemolishAttribute::Extended(ext) => ext.attacker.actor,
        }
    }

    pub fn victim_actor_id(&self) -> boxcars::ActorId {
        match self {
            DemolishAttribute::Fx(fx) => fx.victim,
            DemolishAttribute::Extended(ext) => ext.victim.actor,
        }
    }

    pub fn attacker_velocity(&self) -> boxcars::Vector3f {
        match self {
            DemolishAttribute::Fx(fx) => fx.attack_velocity,
            DemolishAttribute::Extended(ext) => ext.attacker_velocity,
        }
    }

    pub fn victim_velocity(&self) -> boxcars::Vector3f {
        match self {
            DemolishAttribute::Fx(fx) => fx.victim_velocity,
            DemolishAttribute::Extended(ext) => ext.victim_velocity,
        }
    }
}

/// [`DemolishInfo`] struct represents data related to a demolition event in the game.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DemolishInfo {
    pub time: f32,
    pub seconds_remaining: i32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub attacker: PlayerId,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub victim: PlayerId,
    #[ts(as = "crate::ts_bindings::Vector3fTs")]
    pub attacker_velocity: boxcars::Vector3f,
    #[ts(as = "crate::ts_bindings::Vector3fTs")]
    pub victim_velocity: boxcars::Vector3f,
    #[ts(as = "crate::ts_bindings::Vector3fTs")]
    pub victim_location: boxcars::Vector3f,
}
