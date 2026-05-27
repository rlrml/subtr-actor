use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MovementSpeedBand {
    Slow,
    Boost,
    Supersonic,
}

pub(crate) const ALL_MOVEMENT_SPEED_BANDS: [MovementSpeedBand; 3] = [
    MovementSpeedBand::Slow,
    MovementSpeedBand::Boost,
    MovementSpeedBand::Supersonic,
];

impl MovementSpeedBand {
    pub(super) fn as_label_value(self) -> &'static str {
        match self {
            Self::Slow => "slow",
            Self::Boost => "boost",
            Self::Supersonic => "supersonic",
        }
    }

    pub(super) fn as_label(self) -> StatLabel {
        StatLabel::new("speed_band", self.as_label_value())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MovementClassification {
    pub(super) speed_band: MovementSpeedBand,
    pub(super) height_band: PlayerVerticalBand,
}

impl MovementClassification {
    pub(super) fn labels(self) -> [StatLabel; 2] {
        [self.speed_band.as_label(), self.height_band.as_label()]
    }
}

impl MovementCalculator {
    pub(super) fn classify_movement(
        speed: f32,
        height_band: PlayerVerticalBand,
    ) -> MovementClassification {
        let speed_band = if speed >= SUPERSONIC_SPEED_THRESHOLD {
            MovementSpeedBand::Supersonic
        } else if speed >= BOOST_SPEED_THRESHOLD {
            MovementSpeedBand::Boost
        } else {
            MovementSpeedBand::Slow
        };

        MovementClassification {
            speed_band,
            height_band,
        }
    }
}
