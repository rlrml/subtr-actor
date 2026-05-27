use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TouchKind {
    Control,
    MediumHit,
    HardHit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TouchSurface {
    Ground,
    Air,
    Wall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TouchDodgeState {
    NoDodge,
    Dodge,
}

pub(crate) const ALL_TOUCH_KINDS: [TouchKind; 3] =
    [TouchKind::Control, TouchKind::MediumHit, TouchKind::HardHit];
pub(crate) const ALL_TOUCH_SURFACES: [TouchSurface; 3] =
    [TouchSurface::Ground, TouchSurface::Air, TouchSurface::Wall];
pub(crate) const ALL_TOUCH_DODGE_STATES: [TouchDodgeState; 2] =
    [TouchDodgeState::NoDodge, TouchDodgeState::Dodge];

impl TouchKind {
    pub(crate) fn as_label_value(self) -> &'static str {
        match self {
            Self::Control => "control",
            Self::MediumHit => "medium_hit",
            Self::HardHit => "hard_hit",
        }
    }

    pub(crate) fn as_label(self) -> StatLabel {
        StatLabel::new("kind", self.as_label_value())
    }
}

impl TouchSurface {
    pub(crate) fn as_label_value(self) -> &'static str {
        match self {
            Self::Ground => "ground",
            Self::Air => "air",
            Self::Wall => "wall",
        }
    }

    pub(crate) fn as_label(self) -> StatLabel {
        StatLabel::new("surface", self.as_label_value())
    }
}

impl TouchDodgeState {
    pub(crate) fn from_dodge_active(dodge_active: bool) -> Self {
        if dodge_active {
            Self::Dodge
        } else {
            Self::NoDodge
        }
    }

    pub(crate) fn as_label_value(self) -> &'static str {
        match self {
            Self::NoDodge => "no_dodge",
            Self::Dodge => "dodge",
        }
    }

    pub(crate) fn as_label(self) -> StatLabel {
        StatLabel::new("dodge_state", self.as_label_value())
    }
}
