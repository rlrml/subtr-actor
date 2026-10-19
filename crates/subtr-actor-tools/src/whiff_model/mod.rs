//! Experimental telemetry whiff model: shared telemetry capture, causal
//! candidate features, and inference for gradient-boosted trees exported by
//! `scripts/whiff_model`. Not used by the production whiff detector.

pub mod features;
pub mod model;
pub mod telemetry;
