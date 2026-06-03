use crate::StatLabel;

pub(crate) const CONFIDENCE_BAND_LABELS: [StatLabel; 2] = [
    StatLabel::new("confidence_band", "standard"),
    StatLabel::new("confidence_band", "high"),
];

pub(crate) const VERTICAL_STATE_LABELS: [StatLabel; 2] = [
    StatLabel::new("vertical_state", "grounded"),
    StatLabel::new("vertical_state", "aerial"),
];

pub(crate) const CAR_MAX_SPEED: f32 = 2300.0;

pub(crate) fn confidence_band_label(high_confidence: bool) -> StatLabel {
    if high_confidence {
        StatLabel::new("confidence_band", "high")
    } else {
        StatLabel::new("confidence_band", "standard")
    }
}

pub(crate) fn vertical_state_label(aerial: bool) -> StatLabel {
    if aerial {
        StatLabel::new("vertical_state", "aerial")
    } else {
        StatLabel::new("vertical_state", "grounded")
    }
}
