use super::*;

#[test]
fn reports_used_split_mismatch() {
    let stats = BoostStats {
        amount_used: 20.0,
        amount_used_while_grounded: 8.0,
        amount_used_while_airborne: 9.0,
        ..Default::default()
    };

    let violations = boost_invariant_violations(&stats);

    assert!(
        violations
            .iter()
            .any(|violation| violation.kind == BoostInvariantKind::UsedSplitAmounts)
    );
}

#[test]
fn accepts_matching_used_split() {
    let stats = BoostStats {
        amount_used: 20.0,
        amount_used_while_grounded: 8.0,
        amount_used_while_airborne: 12.0,
        ..Default::default()
    };

    let violations = boost_invariant_violations(&stats);

    assert!(
        !violations
            .iter()
            .any(|violation| violation.kind == BoostInvariantKind::UsedSplitAmounts)
    );
}

#[test]
fn projected_current_boost_byte_uses_replay_byte_rounding() {
    let stats = BoostStats {
        amount_collected_small: boost_percent_to_amount(12.0),
        ..Default::default()
    };

    assert_eq!(
        projected_current_boost_byte(stats.amount_obtained() - stats.amount_used),
        31
    );
}

#[test]
fn current_amount_drift_warns_only_after_settle_window() {
    let player_id = PlayerId::Steam(1);
    let stats = BoostStats {
        amount_respawned: 85.0,
        ..Default::default()
    };
    let mut tracker = BoostCurrentAmountConsistencyTracker::new();

    for frame in 10..=10 + BOOST_CURRENT_AMOUNT_SETTLE_FRAME_WINDOW {
        tracker.observe(frame, frame as f32, &player_id, &stats, 79);
        assert!(tracker.unresolved_warnings().is_empty());
    }

    tracker.observe(
        11 + BOOST_CURRENT_AMOUNT_SETTLE_FRAME_WINDOW,
        1.0,
        &player_id,
        &stats,
        79,
    );
    let warnings = tracker.unresolved_warnings();
    let [warning] = warnings.as_slice() else {
        panic!("persistent current boost drift should warn");
    };
    assert_eq!(warning.first_frame, 10);
    assert_eq!(warning.ledger_projected_byte, 85);
    assert_eq!(warning.observed_byte, 79);
}

#[test]
fn current_amount_drift_clears_when_byte_resyncs() {
    let player_id = PlayerId::Steam(1);
    let stats = BoostStats {
        amount_respawned: 85.0,
        ..Default::default()
    };
    let mut tracker = BoostCurrentAmountConsistencyTracker::new();

    tracker.observe(10, 1.0, &player_id, &stats, 79);
    tracker.observe(11, 1.1, &player_id, &stats, 85);
    assert!(tracker.unresolved_warnings().is_empty());

    for frame in 20..=25 {
        tracker.observe(frame, frame as f32, &player_id, &stats, 79);
    }
    let warnings = tracker.unresolved_warnings();
    let [warning] = warnings.as_slice() else {
        panic!("second persistent current boost drift should warn");
    };
    assert_eq!(warning.first_frame, 20);
}

#[test]
fn current_amount_drift_ignores_small_byte_residue() {
    let player_id = PlayerId::Steam(1);
    let stats = BoostStats {
        amount_respawned: 85.0,
        ..Default::default()
    };
    let mut tracker = BoostCurrentAmountConsistencyTracker::new();

    for frame in 10..=100 {
        tracker.observe(frame, frame as f32, &player_id, &stats, 82);
        assert!(tracker.unresolved_warnings().is_empty());
    }
}
