use glam::{Quat, Vec3};
use subtr_actor::{PlayerFrame, ReplayDataCollector, quat_to_glam, vec_to_glam};
use subtr_actor_tools::replay_plausibility::{
    ReplayPlausibilityReport, evaluate_replay_plausibility,
};

const MIN_ANGULAR_VELOCITY_SPEED: f32 = 30.0;
const MIN_DERIVED_ORIENTATION_SPEED: f32 = 0.5;
const MAX_ORIENTATION_PAIR_DT_SECONDS: f32 = 0.2;

struct PostEacFixture {
    path: &'static str,
}

const POST_EAC_FIXTURES: &[PostEacFixture] = &[
    PostEacFixture {
        path: "post-eac-ranked-duel-2026-04-28-a.replay",
    },
    PostEacFixture {
        path: "post-eac-ranked-duel-2026-04-28-b.replay",
    },
    PostEacFixture {
        path: "post-eac-ranked-doubles-2026-04-28.replay",
    },
    PostEacFixture {
        path: "post-eac-ranked-standard-2026-04-28.replay",
    },
];

fn asset_path(path: &str) -> std::path::PathBuf {
    let relative_path = std::path::Path::new(path)
        .strip_prefix("assets")
        .unwrap_or_else(|_| std::path::Path::new(path));
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets")
        .join(relative_path)
}

fn parse_replay(path: &str) -> boxcars::Replay {
    let replay_path = asset_path(path);
    let data = std::fs::read(&replay_path)
        .unwrap_or_else(|_| panic!("failed to read replay file: {}", replay_path.display()));
    boxcars::ParserBuilder::new(&data)
        .must_parse_network_data()
        .on_error_check_crc()
        .parse()
        .unwrap_or_else(|_| panic!("failed to parse replay file: {}", replay_path.display()))
}

fn plausibility_report(path: &str) -> ReplayPlausibilityReport {
    let replay = parse_replay(path);
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
        "assets/replay-format-2022-09-29-v868-32-net10-legacy-boost.replay",
        "assets/replay-format-2025-06-10-v868-32-net10-replicated-boost.replay",
        "assets/replay-format-2020-09-25-v868-29-net10-tournament.replay",
        "assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay",
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
    let path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
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

#[test]
fn legacy_replay_rotation_roll_matches_angular_velocity() {
    let path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(path);
    let replay_data = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .unwrap_or_else(|_| panic!("failed to collect replay data for {path}"));
    let mut direction_dots = Vec::new();

    for (_player_id, player_data) in &replay_data.frame_data.players {
        for (frame_index, frame_pair) in player_data.frames().windows(2).enumerate() {
            let Some(previous_time) = replay_data
                .frame_data
                .metadata_frames
                .get(frame_index)
                .map(|frame| frame.time)
            else {
                continue;
            };
            let Some(current_time) = replay_data
                .frame_data
                .metadata_frames
                .get(frame_index + 1)
                .map(|frame| frame.time)
            else {
                continue;
            };
            let dt = current_time - previous_time;
            if !(0.0..=MAX_ORIENTATION_PAIR_DT_SECONDS).contains(&dt) {
                continue;
            }

            let (
                PlayerFrame::Data {
                    rigid_body: previous_body,
                    ..
                },
                PlayerFrame::Data {
                    rigid_body: current_body,
                    ..
                },
            ) = (&frame_pair[0], &frame_pair[1])
            else {
                continue;
            };
            let Some(reported_angular_velocity) = previous_body
                .angular_velocity
                .or(current_body.angular_velocity)
            else {
                continue;
            };
            let reported_angular_velocity = vec_to_glam(&reported_angular_velocity);
            if reported_angular_velocity.length() < MIN_ANGULAR_VELOCITY_SPEED {
                continue;
            }

            let Some(derived_angular_velocity) = derive_world_angular_velocity(
                quat_to_glam(&previous_body.rotation),
                quat_to_glam(&current_body.rotation),
                dt,
            ) else {
                continue;
            };
            if derived_angular_velocity.length() < MIN_DERIVED_ORIENTATION_SPEED {
                continue;
            }

            let direction_dot = derived_angular_velocity
                .normalize()
                .dot(reported_angular_velocity.normalize());
            if direction_dot.is_finite() {
                direction_dots.push(direction_dot);
            }
        }
    }

    let sample_count = direction_dots.len();
    let positive_fraction = direction_dots
        .iter()
        .filter(|direction_dot| **direction_dot > 0.0)
        .count() as f32
        / sample_count.max(1) as f32;
    let median_direction_dot = median(direction_dots).unwrap_or(f32::NAN);

    assert!(
        sample_count > 1_000,
        "expected {path} to provide enough angular samples, got {sample_count}"
    );
    assert!(
        median_direction_dot > 0.99,
        "expected {path} orientation deltas to align with angular velocity, got median {median_direction_dot}"
    );
    assert!(
        positive_fraction > 0.95,
        "expected {path} orientation deltas to be consistently signed with angular velocity, got positive fraction {positive_fraction}"
    );
}

#[test]
fn post_eac_replay_motion_plausibility_passes() {
    for fixture in POST_EAC_FIXTURES {
        let path = format!("assets/{}", fixture.path);
        let replay = parse_replay(&path);
        let replay_data = ReplayDataCollector::new()
            .get_replay_data(&replay)
            .unwrap_or_else(|error| panic!("failed to collect replay data for {path}: {error:?}"));
        let report = evaluate_replay_plausibility(&replay_data);

        assert!(
            report.all_motion_consistent(),
            "{path} should have plausible velocity/displacement consistency"
        );
        assert!(
            report.all_field_bounds_plausible(),
            "{path} should stay within plausible field and speed bounds"
        );
        assert!(
            report.all_quaternion_norms_plausible(),
            "{path} should expose unit-length rotations"
        );
    }
}

fn derive_world_angular_velocity(
    previous_rotation: Quat,
    mut current_rotation: Quat,
    dt: f32,
) -> Option<Vec3> {
    if dt <= 0.0 {
        return None;
    }
    if previous_rotation.dot(current_rotation) < 0.0 {
        current_rotation = Quat::from_xyzw(
            -current_rotation.x,
            -current_rotation.y,
            -current_rotation.z,
            -current_rotation.w,
        );
    }
    let delta = current_rotation * previous_rotation.inverse();
    let (axis, angle) = delta.to_axis_angle();
    let angular_velocity = axis * (angle / dt);
    angular_velocity.is_finite().then_some(angular_velocity)
}

fn median(mut values: Vec<f32>) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    Some(values[values.len() / 2])
}
