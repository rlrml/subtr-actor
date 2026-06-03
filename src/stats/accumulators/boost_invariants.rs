use std::collections::HashMap;

use super::*;
use crate::PlayerId;

/// A small tolerance for replay-boost accounting checks.
///
/// Some pickup amounts are inferred from frame deltas before the pad is fully
/// resolved, so nominal pad-value identities are expected to be approximate
/// rather than bit-exact in production. The drift grows slowly with pickup
/// count, so use a small base tolerance plus a per-pickup allowance.
pub const BOOST_INVARIANT_BASE_TOLERANCE_RAW: f32 = 2.0;
pub const BOOST_INVARIANT_PER_PICKUP_TOLERANCE_RAW: f32 = 0.5;
pub const BOOST_USED_SPLIT_TOLERANCE_RAW: f32 = 2.0;
pub const BOOST_CURRENT_AMOUNT_SETTLE_FRAME_WINDOW: usize = 4;
pub const BOOST_CURRENT_AMOUNT_DRIFT_WARN_THRESHOLD_BYTES: i16 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoostInvariantKind {
    BucketAmounts,
    NominalPickupAmount,
    NominalStolenPickupAmount,
    CurrentAmount,
    UsedSplitAmounts,
}

impl BoostInvariantKind {
    pub const ALL: [Self; 5] = [
        Self::BucketAmounts,
        Self::NominalPickupAmount,
        Self::NominalStolenPickupAmount,
        Self::CurrentAmount,
        Self::UsedSplitAmounts,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::BucketAmounts => "bucket_amounts",
            Self::NominalPickupAmount => "nominal_pickup_amount",
            Self::NominalStolenPickupAmount => "nominal_stolen_pickup_amount",
            Self::CurrentAmount => "current_amount",
            Self::UsedSplitAmounts => "used_split_amounts",
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
            BoostInvariantKind::UsedSplitAmounts => format!(
                "amount_used_while_grounded + amount_used_while_airborne should match amount_used \
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
/// from cumulative `BoostStats`.
pub fn boost_invariant_violations(stats: &BoostStats) -> Vec<BoostInvariantViolation> {
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
    push_violation(
        &mut violations,
        BoostInvariantKind::UsedSplitAmounts,
        stats.amount_used,
        stats.amount_used_by_vertical_band(),
        BOOST_USED_SPLIT_TOLERANCE_RAW,
    );

    violations
}

pub fn projected_current_boost_byte(ledger_current_amount: f32) -> u8 {
    ledger_current_amount.round().clamp(0.0, BOOST_MAX_AMOUNT) as u8
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoostCurrentAmountDriftWarning {
    pub player_id: PlayerId,
    pub first_frame: usize,
    pub frame: usize,
    pub time: f32,
    pub ledger_current_amount: f32,
    pub ledger_projected_byte: u8,
    pub observed_byte: u8,
    pub diff_bytes: i16,
}

impl BoostCurrentAmountDriftWarning {
    pub fn message(&self) -> String {
        format!(
            "current boost byte did not resync within {} frames \
             (ledger_projected={}, observed={}, diff_bytes={}, ledger_raw={:.3}, first_frame={})",
            BOOST_CURRENT_AMOUNT_SETTLE_FRAME_WINDOW,
            self.ledger_projected_byte,
            self.observed_byte,
            self.diff_bytes,
            self.ledger_current_amount,
            self.first_frame,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
struct PendingBoostCurrentAmountDrift {
    first_frame: usize,
    latest: BoostCurrentAmountDriftWarning,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BoostCurrentAmountConsistencyTracker {
    pending_drifts: HashMap<PlayerId, PendingBoostCurrentAmountDrift>,
}

impl BoostCurrentAmountConsistencyTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn observe(
        &mut self,
        frame: usize,
        time: f32,
        player_id: &PlayerId,
        stats: &BoostStats,
        observed_byte: u8,
    ) {
        let ledger_current_amount = stats.amount_obtained() - stats.amount_used;
        let ledger_projected_byte = projected_current_boost_byte(ledger_current_amount);
        let diff_bytes = ledger_projected_byte as i16 - observed_byte as i16;
        if diff_bytes.abs() <= BOOST_CURRENT_AMOUNT_DRIFT_WARN_THRESHOLD_BYTES {
            self.pending_drifts.remove(player_id);
            return;
        }

        let warning = BoostCurrentAmountDriftWarning {
            player_id: player_id.clone(),
            first_frame: frame,
            frame,
            time,
            ledger_current_amount,
            ledger_projected_byte,
            observed_byte,
            diff_bytes,
        };
        self.pending_drifts
            .entry(player_id.clone())
            .and_modify(|pending| {
                pending.latest = BoostCurrentAmountDriftWarning {
                    first_frame: pending.first_frame,
                    ..warning.clone()
                };
            })
            .or_insert(PendingBoostCurrentAmountDrift {
                first_frame: frame,
                latest: warning,
            });
    }

    pub fn unresolved_warnings(&self) -> Vec<BoostCurrentAmountDriftWarning> {
        self.pending_drifts
            .values()
            .filter(|pending| {
                pending.latest.frame.saturating_sub(pending.first_frame)
                    > BOOST_CURRENT_AMOUNT_SETTLE_FRAME_WINDOW
            })
            .map(|pending| pending.latest.clone())
            .collect()
    }
}

#[cfg(test)]
#[path = "boost_invariants_tests.rs"]
mod tests;
