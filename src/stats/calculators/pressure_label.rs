use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum PressureHalfLabel {
    TeamZeroSide,
    TeamOneSide,
    #[default]
    Neutral,
}

impl PressureHalfLabel {
    pub(super) fn as_label_value(self) -> &'static str {
        match self {
            Self::TeamZeroSide => "team_zero_side",
            Self::TeamOneSide => "team_one_side",
            Self::Neutral => "neutral",
        }
    }

    pub(super) fn as_label(self) -> StatLabel {
        StatLabel::new("field_half", self.as_label_value())
    }
}

pub(crate) fn team_relative_pressure_label(label: &StatLabel, is_team_zero: bool) -> StatLabel {
    match (label.key, label.value) {
        ("field_half", "team_zero_side") => {
            StatLabel::new("field_half", relative_half_label(is_team_zero))
        }
        ("field_half", "team_one_side") => {
            StatLabel::new("field_half", relative_half_label(!is_team_zero))
        }
        _ => label.clone(),
    }
}

fn relative_half_label(defensive: bool) -> &'static str {
    if defensive {
        "defensive_half"
    } else {
        "offensive_half"
    }
}
