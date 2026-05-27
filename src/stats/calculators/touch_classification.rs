use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TouchClassification {
    pub(crate) kind: TouchKind,
    pub(crate) height_band: PlayerVerticalBand,
    pub(crate) surface: TouchSurface,
    pub(crate) dodge_state: TouchDodgeState,
}

impl TouchClassification {
    pub(crate) fn labels(self) -> [StatLabel; 4] {
        [
            self.kind.as_label(),
            self.height_band.as_label(),
            self.surface.as_label(),
            self.dodge_state.as_label(),
        ]
    }
}

impl TouchCalculator {
    pub(crate) fn classify_touch(
        height_band: PlayerVerticalBand,
        surface: TouchSurface,
        dodge_state: TouchDodgeState,
        ball_speed_change: f32,
        controlled_touch_kind: Option<BallCarryKind>,
    ) -> TouchClassification {
        let kind = if controlled_touch_kind.is_some()
            || ball_speed_change <= SOFT_TOUCH_BALL_SPEED_CHANGE_THRESHOLD
        {
            TouchKind::Control
        } else if ball_speed_change < HARD_TOUCH_BALL_SPEED_CHANGE_THRESHOLD {
            TouchKind::MediumHit
        } else {
            TouchKind::HardHit
        };

        TouchClassification {
            kind,
            height_band,
            surface,
            dodge_state,
        }
    }
}
