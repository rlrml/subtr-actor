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
