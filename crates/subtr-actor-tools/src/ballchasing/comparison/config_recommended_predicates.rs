use super::super::model::{ComparisonTarget, StatKey};

pub(super) fn boost_amount_key(target: &ComparisonTarget) -> bool {
    matches!(
        target.key,
        StatKey::AmountCollected
            | StatKey::AmountStolen
            | StatKey::AmountCollectedBig
            | StatKey::AmountStolenBig
            | StatKey::AmountCollectedSmall
            | StatKey::AmountStolenSmall
            | StatKey::AmountOverfill
            | StatKey::AmountOverfillStolen
            | StatKey::AmountUsedWhileSupersonic
    )
}

pub(super) fn boost_timing_key(target: &ComparisonTarget) -> bool {
    matches!(
        target.key,
        StatKey::Bpm
            | StatKey::AvgAmount
            | StatKey::TimeZeroBoost
            | StatKey::PercentZeroBoost
            | StatKey::TimeFullBoost
            | StatKey::PercentFullBoost
            | StatKey::TimeBoost0To25
            | StatKey::TimeBoost25To50
            | StatKey::TimeBoost50To75
            | StatKey::TimeBoost75To100
            | StatKey::PercentBoost0To25
            | StatKey::PercentBoost25To50
            | StatKey::PercentBoost50To75
            | StatKey::PercentBoost75To100
    )
}

pub(super) fn movement_timing_key(target: &ComparisonTarget) -> bool {
    matches!(
        target.key,
        StatKey::TimeSupersonicSpeed
            | StatKey::TimeBoostSpeed
            | StatKey::TimeSlowSpeed
            | StatKey::TimeGround
            | StatKey::TimeLowAir
            | StatKey::TimeHighAir
            | StatKey::TimePowerslide
            | StatKey::PercentSlowSpeed
            | StatKey::PercentBoostSpeed
            | StatKey::PercentSupersonicSpeed
            | StatKey::PercentGround
            | StatKey::PercentLowAir
            | StatKey::PercentHighAir
    )
}

pub(super) fn movement_distance_key(target: &ComparisonTarget) -> bool {
    matches!(
        target.key,
        StatKey::AvgSpeed
            | StatKey::AvgSpeedPercentage
            | StatKey::TotalDistance
            | StatKey::AvgPowerslideDuration
    )
}

pub(super) fn movement_distance_predicate(
    actual: f64,
    expected: f64,
    target: &ComparisonTarget,
) -> bool {
    let tol = match target.key {
        StatKey::AvgSpeed => 5.0,
        StatKey::AvgSpeedPercentage => 0.5,
        StatKey::TotalDistance => 2500.0,
        StatKey::AvgPowerslideDuration => 0.1,
        _ => 0.0,
    };
    (actual - expected).abs() <= tol
}

pub(super) fn positioning_predicate(actual: f64, expected: f64, target: &ComparisonTarget) -> bool {
    let tol = match target.key {
        StatKey::AvgDistanceToBall
        | StatKey::AvgDistanceToBallPossession
        | StatKey::AvgDistanceToBallNoPossession
        | StatKey::AvgDistanceToMates => 50.0,
        _ => 1.0,
    };
    (actual - expected).abs() <= tol
}
