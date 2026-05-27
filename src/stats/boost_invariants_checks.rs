use super::boost_invariants_amounts::nominal_pickup_tolerance;
use super::*;

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
    push_nominal_pickup_violations(&mut violations, stats);
    if let Some(current_boost_amount) = observed_boost_amount {
        push_violation(
            &mut violations,
            BoostInvariantKind::CurrentAmount,
            current_boost_amount,
            stats.amount_obtained() - stats.amount_used,
            1.0,
        );
    }
    push_violation(
        &mut violations,
        BoostInvariantKind::UsedSplitAmounts,
        stats.amount_used,
        stats.amount_used_by_vertical_band(),
        1.0,
    );

    violations
}

fn push_nominal_pickup_violations(
    violations: &mut Vec<BoostInvariantViolation>,
    stats: &BoostStats,
) {
    push_violation(
        violations,
        BoostInvariantKind::NominalPickupAmount,
        nominal_pickup_amount_from_counts(stats),
        stats.amount_collected + stats.overfill_total,
        nominal_pickup_tolerance(stats.big_pads_collected + stats.small_pads_collected),
    );
    push_violation(
        violations,
        BoostInvariantKind::NominalStolenPickupAmount,
        nominal_stolen_pickup_amount_from_counts(stats),
        stats.amount_stolen + stats.overfill_from_stolen,
        nominal_pickup_tolerance(stats.big_pads_stolen + stats.small_pads_stolen),
    );
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
