use std::sync::LazyLock;

use super::*;

#[path = "soccar_boost_pads_builder.rs"]
mod soccar_boost_pads_builder;
#[path = "soccar_boost_pads_estimate.rs"]
mod soccar_boost_pads_estimate;

pub(crate) use soccar_boost_pads_estimate::PadPositionEstimate;

pub(crate) static STANDARD_SOCCAR_BOOST_PAD_LAYOUT: LazyLock<Vec<(glam::Vec3, BoostPadSize)>> =
    LazyLock::new(soccar_boost_pads_builder::build_standard_soccar_boost_pad_layout);

pub(crate) fn standard_soccar_boost_pad_layout() -> &'static [(glam::Vec3, BoostPadSize)] {
    STANDARD_SOCCAR_BOOST_PAD_LAYOUT.as_slice()
}

pub(crate) fn standard_soccar_boost_pad_position(index: usize) -> glam::Vec3 {
    STANDARD_SOCCAR_BOOST_PAD_LAYOUT[index].0
}
