use super::ReplayProcessor;
use crate::{Collector, FrameRateDecorator, ProcessorView, SubtrActorErrorVariant, TimeAdvance};

fn single_frame_replay() -> boxcars::Replay {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/post-eac-ranked-duel-2026-04-28-a.replay"
    ))
    .unwrap();
    let mut replay = boxcars::ParserBuilder::new(&bytes)
        .must_parse_network_data()
        .parse()
        .unwrap();
    replay.network_frames.as_mut().unwrap().frames.truncate(1);
    replay
}

#[test]
fn process_rejects_nonadvancing_and_nonfinite_collector_times() {
    let replay = single_frame_replay();
    let time = replay.network_frames.as_ref().unwrap().frames[0].time;
    for target in [time, time - 1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut calls = 0;
        let mut collector = |_: &dyn ProcessorView, _: &boxcars::Frame, _: usize, _: f32| {
            calls += 1;
            Ok(if calls == 1 {
                TimeAdvance::Time(target)
            } else {
                TimeAdvance::NextFrame
            })
        };
        let error = ReplayProcessor::new(&replay)
            .unwrap()
            .process(&mut collector)
            .unwrap_err();
        assert!(matches!(
            error.variant,
            SubtrActorErrorVariant::InvalidTimeAdvance { .. }
        ));
        assert_eq!(calls, 1);
    }
}

#[test]
fn process_rejects_invalid_frame_rate_without_looping() {
    let replay = single_frame_replay();
    for fps in [0.0, -30.0, f32::INFINITY, f32::NAN, f32::MAX] {
        let mut calls = 0;
        let mut collector = |_: &dyn ProcessorView, _: &boxcars::Frame, _: usize, _: f32| {
            calls += 1;
            if calls > 1 {
                return crate::SubtrActorError::new_result(
                    SubtrActorErrorVariant::FinishProcessingEarly,
                );
            }
            Ok(TimeAdvance::NextFrame)
        };
        let error = FrameRateDecorator::new_from_fps(fps, &mut collector)
            .process_replay(&replay)
            .err()
            .expect("invalid frame rates must fail");
        assert!(matches!(
            error.variant,
            SubtrActorErrorVariant::InvalidTimeAdvance { .. }
        ));
        assert_eq!(calls, 1);
    }
}

#[test]
fn process_accepts_advancing_collector_times() {
    let replay = single_frame_replay();
    let mut collector = |_: &dyn ProcessorView, _: &boxcars::Frame, _: usize, time: f32| {
        Ok(TimeAdvance::Time(time + 1.0 / 30.0))
    };
    ReplayProcessor::new(&replay)
        .unwrap()
        .process(&mut collector)
        .unwrap();
}

#[test]
fn rigid_body_normalization_factors_split_at_expected_legacy_boundary() {
    assert_eq!(
        ReplayProcessor::rigid_body_location_normalization_factor_for_net_version(None),
        100.0
    );
    assert_eq!(
        ReplayProcessor::rigid_body_velocity_normalization_factor_for_net_version(None),
        10.0
    );
    assert_eq!(
        ReplayProcessor::rigid_body_location_normalization_factor_for_net_version(Some(2)),
        100.0
    );
    assert_eq!(
        ReplayProcessor::rigid_body_velocity_normalization_factor_for_net_version(Some(2)),
        10.0
    );
    assert!(ReplayProcessor::uses_legacy_rigid_body_rotation_for_net_version(Some(2)));
    assert_eq!(
        ReplayProcessor::rigid_body_location_normalization_factor_for_net_version(Some(5)),
        1.0
    );
    assert_eq!(
        ReplayProcessor::rigid_body_velocity_normalization_factor_for_net_version(Some(5)),
        1.0
    );
    assert!(ReplayProcessor::uses_legacy_rigid_body_rotation_for_net_version(Some(5)));
    assert_eq!(
        ReplayProcessor::rigid_body_location_normalization_factor_for_net_version(Some(10)),
        1.0
    );
    assert_eq!(
        ReplayProcessor::rigid_body_velocity_normalization_factor_for_net_version(Some(10)),
        1.0
    );
    assert!(!ReplayProcessor::uses_legacy_rigid_body_rotation_for_net_version(Some(7)));
    assert!(!ReplayProcessor::uses_legacy_rigid_body_rotation_for_net_version(Some(10)));
}
