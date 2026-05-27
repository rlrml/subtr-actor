use super::*;

#[path = "builtins_rigid_body_core.rs"]
mod builtins_rigid_body_core;
pub use builtins_rigid_body_core::*;
#[path = "builtins_rigid_body_defaults.rs"]
mod builtins_rigid_body_defaults;
pub(crate) use builtins_rigid_body_defaults::*;
