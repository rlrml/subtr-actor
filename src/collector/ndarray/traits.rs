/// Re-export of `derive_new` used by the public ndarray feature macros.
pub use ::derive_new;
/// Re-export of `paste` used by the public ndarray feature macros.
pub use ::paste;

#[path = "traits_conversion.rs"]
mod traits_conversion;
#[path = "traits_feature.rs"]
mod traits_feature;
#[path = "traits_macros_global.rs"]
mod traits_macros_global;
#[path = "traits_macros_player.rs"]
mod traits_macros_player;
#[path = "traits_player.rs"]
mod traits_player;
#[path = "traits_tuple_impls.rs"]
mod traits_tuple_impls;

pub use traits_conversion::*;
pub use traits_feature::*;
pub use traits_player::*;
