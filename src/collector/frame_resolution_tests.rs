use super::{FinalStatsFrameAction, StatsFramePersistenceController, StatsFrameResolution};

#[test]
fn every_frame_resolution_emits_every_frame() {
    let mut controller = StatsFramePersistenceController::new(StatsFrameResolution::EveryFrame);

    assert_eq!(controller.on_frame(10, 0.0), Some(0.0));
    assert_eq!(controller.on_frame(11, 0.1), Some(0.1));
    assert_eq!(controller.on_frame(12, 0.25), Some(0.15));
    assert_eq!(
        controller.final_frame_action(12, 0.25),
        Some(FinalStatsFrameAction::ReplaceLast { dt: 0.15 })
    );
}

#[test]
fn timestep_resolution_emits_crossings_and_appends_final_frame() {
    let mut controller =
        StatsFramePersistenceController::new(StatsFrameResolution::TimeStep { seconds: 0.5 });

    assert_eq!(controller.on_frame(0, 0.0), Some(0.0));
    assert_eq!(controller.on_frame(1, 0.2), None);
    assert_eq!(controller.on_frame(2, 0.49), None);
    assert_eq!(controller.on_frame(3, 0.5), Some(0.5));
    assert_eq!(controller.on_frame(4, 0.74), None);
    match controller.final_frame_action(4, 0.74) {
        Some(FinalStatsFrameAction::Append { dt }) => {
            assert!(
                (dt - 0.24).abs() < 1e-6,
                "expected dt close to 0.24, got {dt}"
            );
        }
        action => panic!("expected append action, got {action:?}"),
    }
}
