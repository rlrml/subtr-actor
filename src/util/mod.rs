//! Small shared helpers used throughout the crate.
//!
//! - [`geometry`] — vector/quaternion/rotation helpers for working with
//!   Rocket League's coordinate space (also re-exported at the crate root as
//!   `geometry`).
//! - [`search`] — frame/time search utilities (e.g. locating frames by time);
//!   re-exported at the crate root as `search`.
//! - `vec_map` — a crate-internal small-map structure.

pub mod geometry;
pub mod search;
pub(crate) mod vec_map;
