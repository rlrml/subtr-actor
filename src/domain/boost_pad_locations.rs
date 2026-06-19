use std::sync::LazyLock;

use crate::replay_model::BoostPadSize;

pub(crate) const STANDARD_PAD_MATCH_RADIUS_SMALL: f32 = 450.0;
pub(crate) const STANDARD_PAD_MATCH_RADIUS_BIG: f32 = 1000.0;
pub(crate) const BOOST_PAD_MIDFIELD_TOLERANCE_Y: f32 = 128.0;
const BOOST_PAD_SMALL_Z: f32 = 70.0;
const BOOST_PAD_BIG_Z: f32 = 73.0;
pub(crate) const BOOST_PAD_BACK_CORNER_X: f32 = 3072.0;
pub(crate) const BOOST_PAD_BACK_CORNER_Y: f32 = 4096.0;
const BOOST_PAD_BACK_LANE_X: f32 = 1792.0;
const BOOST_PAD_BACK_LANE_Y: f32 = 4184.0;
const BOOST_PAD_BACK_MID_X: f32 = 940.0;
const BOOST_PAD_BACK_MID_Y: f32 = 3308.0;
const BOOST_PAD_CENTER_BACK_Y: f32 = 2816.0;
const BOOST_PAD_SIDE_WALL_X: f32 = 3584.0;
const BOOST_PAD_SIDE_WALL_Y: f32 = 2484.0;
const BOOST_PAD_SIDE_LANE_X: f32 = 1788.0;
pub(crate) const BOOST_PAD_SIDE_LANE_Y: f32 = 2300.0;
const BOOST_PAD_FRONT_LANE_X: f32 = 2048.0;
const BOOST_PAD_FRONT_LANE_Y: f32 = 1036.0;
const BOOST_PAD_CENTER_X: f32 = 1024.0;
const BOOST_PAD_CENTER_MID_Y: f32 = 1024.0;
const BOOST_PAD_GOAL_LINE_Y: f32 = 4240.0;

fn push_pad(
    pads: &mut Vec<(glam::Vec3, BoostPadSize)>,
    x: f32,
    y: f32,
    z: f32,
    size: BoostPadSize,
) {
    pads.push((glam::Vec3::new(x, y, z), size));
}

fn push_mirror_x(
    pads: &mut Vec<(glam::Vec3, BoostPadSize)>,
    x: f32,
    y: f32,
    z: f32,
    size: BoostPadSize,
) {
    push_pad(pads, -x, y, z, size);
    push_pad(pads, x, y, z, size);
}

fn push_mirror_y(
    pads: &mut Vec<(glam::Vec3, BoostPadSize)>,
    x: f32,
    y: f32,
    z: f32,
    size: BoostPadSize,
) {
    push_pad(pads, x, -y, z, size);
    push_pad(pads, x, y, z, size);
}

fn push_mirror_xy(
    pads: &mut Vec<(glam::Vec3, BoostPadSize)>,
    x: f32,
    y: f32,
    z: f32,
    size: BoostPadSize,
) {
    push_mirror_x(pads, x, -y, z, size);
    push_mirror_x(pads, x, y, z, size);
}

fn build_standard_soccar_boost_pad_layout() -> Vec<(glam::Vec3, BoostPadSize)> {
    let mut pads = Vec::with_capacity(34);

    push_mirror_y(
        &mut pads,
        0.0,
        BOOST_PAD_GOAL_LINE_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_xy(
        &mut pads,
        BOOST_PAD_BACK_LANE_X,
        BOOST_PAD_BACK_LANE_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_xy(
        &mut pads,
        BOOST_PAD_BACK_CORNER_X,
        BOOST_PAD_BACK_CORNER_Y,
        BOOST_PAD_BIG_Z,
        BoostPadSize::Big,
    );
    push_mirror_xy(
        &mut pads,
        BOOST_PAD_BACK_MID_X,
        BOOST_PAD_BACK_MID_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_y(
        &mut pads,
        0.0,
        BOOST_PAD_CENTER_BACK_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_xy(
        &mut pads,
        BOOST_PAD_SIDE_WALL_X,
        BOOST_PAD_SIDE_WALL_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_xy(
        &mut pads,
        BOOST_PAD_SIDE_LANE_X,
        BOOST_PAD_SIDE_LANE_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_xy(
        &mut pads,
        BOOST_PAD_FRONT_LANE_X,
        BOOST_PAD_FRONT_LANE_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_y(
        &mut pads,
        0.0,
        BOOST_PAD_CENTER_MID_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_x(
        &mut pads,
        BOOST_PAD_SIDE_WALL_X,
        0.0,
        BOOST_PAD_BIG_Z,
        BoostPadSize::Big,
    );
    push_mirror_x(
        &mut pads,
        BOOST_PAD_CENTER_X,
        0.0,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );

    pads
}

pub(crate) static STANDARD_SOCCAR_BOOST_PAD_LAYOUT: LazyLock<Vec<(glam::Vec3, BoostPadSize)>> =
    LazyLock::new(build_standard_soccar_boost_pad_layout);

pub fn standard_soccar_boost_pad_layout() -> &'static [(glam::Vec3, BoostPadSize)] {
    STANDARD_SOCCAR_BOOST_PAD_LAYOUT.as_slice()
}

pub(crate) fn standard_soccar_boost_pad_position(index: usize) -> glam::Vec3 {
    STANDARD_SOCCAR_BOOST_PAD_LAYOUT[index].0
}

#[derive(Debug, Clone, Default)]
pub(crate) struct PadPositionEstimate {
    observations: Vec<glam::Vec3>,
}

impl PadPositionEstimate {
    pub(crate) fn observe(&mut self, position: glam::Vec3) {
        self.observations.push(position);
    }

    pub(crate) fn observations(&self) -> &[glam::Vec3] {
        self.observations.as_slice()
    }

    pub(crate) fn mean(&self) -> Option<glam::Vec3> {
        if self.observations.is_empty() {
            return None;
        }

        let sum = self
            .observations
            .iter()
            .copied()
            .fold(glam::Vec3::ZERO, |acc, position| acc + position);
        Some(sum / self.observations.len() as f32)
    }
}
