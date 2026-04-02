use super::calculators::BoostStats;
use crate::*;

/// A small tolerance for replay-boost accounting checks.
///
/// Some pickup amounts are inferred from frame deltas before the pad is fully
/// resolved, so nominal pad-value identities are expected to be approximate
/// rather than bit-exact in production. The drift grows slowly with pickup
/// count, so use a small base tolerance plus a per-pickup allowance.
pub const BOOST_INVARIANT_BASE_TOLERANCE_RAW: f32 = 2.0;
pub const BOOST_INVARIANT_PER_PICKUP_TOLERANCE_RAW: f32 = 0.3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoostInvariantKind {
    BucketAmounts,
    NominalPickupAmount,
    NominalStolenPickupAmount,
    CurrentAmount,
}

impl BoostInvariantKind {
    pub const ALL: [Self; 4] = [
        Self::BucketAmounts,
        Self::NominalPickupAmount,
        Self::NominalStolenPickupAmount,
        Self::CurrentAmount,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::BucketAmounts => "bucket_amounts",
            Self::NominalPickupAmount => "nominal_pickup_amount",
            Self::NominalStolenPickupAmount => "nominal_stolen_pickup_amount",
            Self::CurrentAmount => "current_amount",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoostInvariantViolation {
    pub kind: BoostInvariantKind,
    pub expected: f32,
    pub actual: f32,
    pub diff: f32,
    pub tolerance: f32,
}

impl BoostInvariantViolation {
    pub fn message(&self) -> String {
        match self.kind {
            BoostInvariantKind::BucketAmounts => format!(
                "amount_collected_big + amount_collected_small should match amount_collected \
                 (actual={:.1}, expected={:.1}, diff={:.1}, tolerance={:.1})",
                self.actual, self.expected, self.diff, self.tolerance
            ),
            BoostInvariantKind::NominalPickupAmount => format!(
                "amount_collected + overfill_total should match nominal pad value from pickup counts \
                 (actual={:.1}, expected={:.1}, diff={:.1}, tolerance={:.1})",
                self.actual, self.expected, self.diff, self.tolerance
            ),
            BoostInvariantKind::NominalStolenPickupAmount => format!(
                "amount_stolen + overfill_from_stolen should match nominal stolen pad value from pickup counts \
                 (actual={:.1}, expected={:.1}, diff={:.1}, tolerance={:.1})",
                self.actual, self.expected, self.diff, self.tolerance
            ),
            BoostInvariantKind::CurrentAmount => format!(
                "amount_obtained - amount_used should match observed current boost \
                 (actual={:.1}, expected={:.1}, diff={:.1}, tolerance={:.1})",
                self.actual, self.expected, self.diff, self.tolerance
            ),
        }
    }
}

pub fn nominal_pickup_amount_from_counts(stats: &BoostStats) -> f32 {
    stats.big_pads_collected as f32 * BOOST_MAX_AMOUNT
        + stats.small_pads_collected as f32 * boost_percent_to_amount(12.0)
}

pub fn nominal_stolen_pickup_amount_from_counts(stats: &BoostStats) -> f32 {
    stats.big_pads_stolen as f32 * BOOST_MAX_AMOUNT
        + stats.small_pads_stolen as f32 * boost_percent_to_amount(12.0)
}

fn nominal_pickup_tolerance(pickup_count: u32) -> f32 {
    BOOST_INVARIANT_BASE_TOLERANCE_RAW
        + BOOST_INVARIANT_PER_PICKUP_TOLERANCE_RAW * pickup_count as f32
}

fn push_violation(
    violations: &mut Vec<BoostInvariantViolation>,
    kind: BoostInvariantKind,
    expected: f32,
    actual: f32,
    tolerance: f32,
) {
    let diff = (actual - expected).abs();
    if diff > tolerance {
        violations.push(BoostInvariantViolation {
            kind,
            expected,
            actual,
            diff,
            tolerance,
        });
    }
}

/// Returns the per-snapshot boost accounting violations that can be checked
/// from cumulative `BoostStats`, plus an optional observed current boost amount.
pub fn boost_invariant_violations(
    stats: &BoostStats,
    observed_boost_amount: Option<f32>,
) -> Vec<BoostInvariantViolation> {
    let mut violations = Vec::new();

    push_violation(
        &mut violations,
        BoostInvariantKind::BucketAmounts,
        stats.amount_collected,
        stats.amount_collected_big + stats.amount_collected_small,
        1.0,
    );
    push_violation(
        &mut violations,
        BoostInvariantKind::NominalPickupAmount,
        nominal_pickup_amount_from_counts(stats),
        stats.amount_collected + stats.overfill_total,
        nominal_pickup_tolerance(stats.big_pads_collected + stats.small_pads_collected),
    );
    push_violation(
        &mut violations,
        BoostInvariantKind::NominalStolenPickupAmount,
        nominal_stolen_pickup_amount_from_counts(stats),
        stats.amount_stolen + stats.overfill_from_stolen,
        nominal_pickup_tolerance(stats.big_pads_stolen + stats.small_pads_stolen),
    );
    if let Some(current_boost_amount) = observed_boost_amount {
        push_violation(
            &mut violations,
            BoostInvariantKind::CurrentAmount,
            current_boost_amount,
            stats.amount_obtained() - stats.amount_used,
            1.0,
        );
    }

    violations
}
