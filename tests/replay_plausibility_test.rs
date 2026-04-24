mod common;

use subtr_actor::{evaluate_replay_plausibility, ReplayDataCollector, ReplayPlausibilityReport};

fn plausibility_report(path: &str) -> ReplayPlausibilityReport {
    let replay = common::parse_replay(path);
    let replay_data = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .unwrap_or_else(|_| panic!("failed to collect replay data for {path}"));
    let report = evaluate_replay_plausibility(&replay_data);
    println!("{path}: {report:#?}");
    report
}

#[test]
fn modern_replay_motion_consistency_passes() {
    for path in [
        "assets/old_boost_format.replay",
        "assets/new_boost_format.replay",
        "assets/tourny.replay",
        "assets/dodges_refreshed_counter.replay",
    ] {
        let report = plausibility_report(path);
        assert!(
            report.all_motion_consistent(),
            "expected {path} to have plausible velocity/displacement consistency"
        );
        assert!(
            report.all_field_bounds_plausible(),
            "expected {path} to stay within plausible field and speed bounds"
        );
        assert!(
            report.all_quaternion_norms_plausible(),
            "expected {path} to expose unit-length rotations"
        );
        assert!(
            report
                .players
                .median_grounded_forward_alignment
                .is_some_and(|alignment| alignment > 0.95),
            "expected {path} grounded player forward vectors to align with travel direction"
        );
        assert!(
            report
                .players
                .grounded_forward_alignment_positive_fraction
                .is_some_and(|fraction| fraction > 0.9),
            "expected {path} grounded player forward vectors to be mostly forward-facing"
        );
    }
}

#[test]
fn legacy_replay_rigid_body_normalization_passes() {
    let path = "assets/rlcs.replay";
    let report = plausibility_report(path);
    assert!(
        report.all_motion_consistent(),
        "expected {path} legacy rigid-body velocities to be motion-consistent after normalization"
    );
    assert!(
        report.all_field_bounds_plausible(),
        "expected {path} legacy positions and velocities to stay within plausible bounds"
    );
    assert!(
        report.all_quaternion_norms_plausible(),
        "expected {path} legacy compressed rotations to normalize into unit quaternions"
    );
    assert!(
        report
            .players
            .median_grounded_forward_alignment
            .is_some_and(|alignment| alignment > 0.95),
        "expected {path} grounded player forward vectors to align with travel direction"
    );
    assert!(
        report
            .players
            .grounded_forward_alignment_positive_fraction
            .is_some_and(|fraction| fraction > 0.9),
        "expected {path} grounded player forward vectors to be mostly forward-facing"
    );
}
