use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum PossessionStateLabel {
    TeamZero,
    TeamOne,
    #[default]
    Neutral,
}

impl PossessionStateLabel {
    pub(super) fn as_label_value(self) -> &'static str {
        match self {
            Self::TeamZero => "team_zero",
            Self::TeamOne => "team_one",
            Self::Neutral => "neutral",
        }
    }

    pub(super) fn as_label(self) -> StatLabel {
        StatLabel::new("possession_state", self.as_label_value())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum FieldThirdLabel {
    TeamZeroThird,
    NeutralThird,
    TeamOneThird,
}

impl FieldThirdLabel {
    pub(super) fn from_ball(ball: &BallSample) -> Self {
        let ball_y = ball.position().y;
        if ball_y < -FIELD_ZONE_BOUNDARY_Y {
            Self::TeamZeroThird
        } else if ball_y > FIELD_ZONE_BOUNDARY_Y {
            Self::TeamOneThird
        } else {
            Self::NeutralThird
        }
    }

    pub(super) fn as_label_value(self) -> &'static str {
        match self {
            Self::TeamZeroThird => "team_zero_third",
            Self::NeutralThird => "neutral_third",
            Self::TeamOneThird => "team_one_third",
        }
    }

    pub(super) fn as_label(self) -> StatLabel {
        StatLabel::new("field_third", self.as_label_value())
    }
}
