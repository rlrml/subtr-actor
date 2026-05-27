use super::*;

pub(super) fn boost_transaction_label(kind: &'static str) -> StatLabel {
    StatLabel::new("transaction", kind)
}

pub(super) fn boost_pad_size_label(pad_size: Option<BoostPadSize>) -> StatLabel {
    match pad_size {
        Some(BoostPadSize::Big) => StatLabel::new("pad_size", "big"),
        Some(BoostPadSize::Small) => StatLabel::new("pad_size", "small"),
        None => StatLabel::new("pad_size", "unknown"),
    }
}

pub(super) fn boost_activity_label(activity: BoostPickupActivity) -> StatLabel {
    match activity {
        BoostPickupActivity::Active => StatLabel::new("activity", "active"),
        BoostPickupActivity::Inactive => StatLabel::new("activity", "inactive"),
        BoostPickupActivity::Unknown => StatLabel::new("activity", "unknown"),
    }
}

pub(super) fn boost_field_half_label(field_half: BoostPickupFieldHalf) -> StatLabel {
    match field_half {
        BoostPickupFieldHalf::Own => StatLabel::new("field_half", "own"),
        BoostPickupFieldHalf::Opponent => StatLabel::new("field_half", "opponent"),
        BoostPickupFieldHalf::Unknown => StatLabel::new("field_half", "unknown"),
    }
}

pub(super) fn boost_supersonic_label(supersonic: bool) -> StatLabel {
    if supersonic {
        StatLabel::new("supersonic", "true")
    } else {
        StatLabel::new("supersonic", "false")
    }
}
