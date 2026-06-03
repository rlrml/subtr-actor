use super::*;

#[test]
fn reports_used_split_mismatch() {
    let stats = BoostStats {
        amount_used: 20.0,
        amount_used_while_grounded: 8.0,
        amount_used_while_airborne: 9.0,
        ..Default::default()
    };

    let violations = boost_invariant_violations(&stats, None);

    assert!(violations
        .iter()
        .any(|violation| violation.kind == BoostInvariantKind::UsedSplitAmounts));
}

#[test]
fn accepts_matching_used_split() {
    let stats = BoostStats {
        amount_used: 20.0,
        amount_used_while_grounded: 8.0,
        amount_used_while_airborne: 12.0,
        ..Default::default()
    };

    let violations = boost_invariant_violations(&stats, None);

    assert!(!violations
        .iter()
        .any(|violation| violation.kind == BoostInvariantKind::UsedSplitAmounts));
}

#[test]
fn current_amount_warns_only_when_ledger_overstates_observed_boost() {
    let stats = BoostStats {
        amount_respawned: 100.0,
        amount_used: 20.0,
        ..Default::default()
    };

    let replay_resync_violations = boost_invariant_violations(&stats, Some(90.0));
    assert!(!replay_resync_violations
        .iter()
        .any(|violation| violation.kind == BoostInvariantKind::CurrentAmount));

    let overstated_violations = boost_invariant_violations(&stats, Some(70.0));
    assert!(overstated_violations
        .iter()
        .any(|violation| violation.kind == BoostInvariantKind::CurrentAmount));
}
