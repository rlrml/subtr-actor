#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct QuaternionMode {
    pub(crate) missing_slot: usize,
    pub(crate) order: [usize; 3],
    pub(crate) signs: [i8; 3],
    pub(crate) reconstruct_missing: bool,
}

impl QuaternionMode {
    pub(crate) fn label(&self) -> String {
        format!(
            "{}@{} order={:?} signs={:?}",
            if self.reconstruct_missing {
                "reconstruct"
            } else {
                "zero"
            },
            self.missing_slot,
            self.order,
            self.signs
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct EulerMode {
    pub(crate) order: [usize; 3],
    pub(crate) signs: [i8; 3],
    pub(crate) scale: EulerScale,
    pub(crate) rotation_order: EulerRotationOrder,
}

impl EulerMode {
    pub(crate) fn label(&self) -> String {
        format!(
            "euler order={:?} signs={:?} scale={:?} rot={:?}",
            self.order, self.signs, self.scale, self.rotation_order
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum EulerScale {
    Pi,
    TwoPi,
    HalfPi,
}

impl EulerScale {
    pub(crate) fn factor(self) -> f32 {
        match self {
            Self::Pi => std::f32::consts::PI,
            Self::TwoPi => std::f32::consts::TAU,
            Self::HalfPi => std::f32::consts::FRAC_PI_2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum EulerRotationOrder {
    Xyz,
    Xzy,
    Yxz,
    Yzx,
    Zxy,
    Zyx,
}

impl EulerRotationOrder {
    pub(crate) fn to_glam(self) -> glam::EulerRot {
        match self {
            Self::Xyz => glam::EulerRot::XYZ,
            Self::Xzy => glam::EulerRot::XZY,
            Self::Yxz => glam::EulerRot::YXZ,
            Self::Yzx => glam::EulerRot::YZX,
            Self::Zxy => glam::EulerRot::ZXY,
            Self::Zyx => glam::EulerRot::ZYX,
        }
    }
}
