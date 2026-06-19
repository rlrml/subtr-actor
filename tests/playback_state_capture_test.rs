//! End-to-end checks that the replay-data collector captures the replay-driven
//! camera and vehicle-input state added for richer playback rendering.

mod common;

use subtr_actor::{PlayerFrame, ReplayData, ReplayDataCollector};

fn collect_replay_data(replay_path: &str) -> ReplayData {
    let replay = common::parse_replay(replay_path);
    ReplayDataCollector::new()
        .get_replay_data(&replay)
        .expect("collector should produce replay data")
}

fn collect_player_frames(replay_path: &str) -> Vec<PlayerFrame> {
    collect_replay_data(replay_path)
        .frame_data
        .players
        .into_iter()
        .flat_map(|(_player_id, player_data)| player_data.frames().to_vec())
        .collect()
}

/// Discrete camera toggles are emitted as a coalesced event stream rather than
/// stored per frame. A modern (post-EAC) replay carries per-player
/// camera-settings actors, so we expect ball-cam events and far fewer of them
/// than there are frames.
#[test]
fn captures_camera_events_for_modern_replay() {
    let replay_data = collect_replay_data("assets/post-eac-ranked-doubles-2026-04-28.replay");
    let grouped = &replay_data.player_camera_events;

    let total_changes: usize = grouped.iter().map(|(_player, changes)| changes.len()).sum();
    let ball_cam_seen = grouped
        .iter()
        .flat_map(|(_player, changes)| changes)
        .any(|change| change.ball_cam_active.is_some());
    assert!(
        ball_cam_seen,
        "expected at least one coalesced camera change with a ball-cam value",
    );

    // Coalescing must collapse the per-frame signal: there should be far fewer
    // changes than player frames.
    let player_frame_count: usize = replay_data
        .frame_data
        .players
        .iter()
        .map(|(_player_id, player_data)| player_data.frames().len())
        .sum();
    assert!(
        total_changes < player_frame_count / 4,
        "camera changes ({total_changes}) should be far fewer than player frames ({player_frame_count})",
    );

    // Within each player, changes must be coalesced (each differs from the
    // previous discrete state) and ordered by frame.
    for (_player, changes) in grouped {
        for window in changes.windows(2) {
            let [previous, current] = window else {
                continue;
            };
            assert!(
                current.frame >= previous.frame,
                "camera changes must be ordered by frame",
            );
            assert_ne!(
                (
                    previous.ball_cam_active,
                    previous.behind_view_active,
                    previous.driving
                ),
                (
                    current.ball_cam_active,
                    current.behind_view_active,
                    current.driving
                ),
                "consecutive camera changes for a player must differ",
            );
        }
    }
}

/// Continuous camera look angles stay on the per-frame structure.
#[test]
fn captures_camera_look_angles_for_modern_replay() {
    let frames = collect_player_frames("assets/post-eac-ranked-doubles-2026-04-28.replay");

    let look_angle_seen = frames.iter().any(|frame| {
        matches!(frame, PlayerFrame::Data { camera, .. } if camera.pitch.is_some() || camera.yaw.is_some())
    });

    assert!(
        look_angle_seen,
        "expected at least one frame with a replicated camera pitch/yaw",
    );
}

/// The same replay should expose replicated vehicle inputs (throttle/steer) that
/// drive accurate wheel rendering during playback.
#[test]
fn captures_vehicle_input_for_modern_replay() {
    let frames = collect_player_frames("assets/post-eac-ranked-doubles-2026-04-28.replay");

    let mut steer_seen = false;
    let mut throttle_seen = false;
    for frame in &frames {
        if let PlayerFrame::Data { input, .. } = frame {
            steer_seen |= input.steer.is_some();
            throttle_seen |= input.throttle.is_some();
        }
    }

    assert!(
        steer_seen,
        "expected at least one frame with replicated steer"
    );
    assert!(
        throttle_seen,
        "expected at least one frame with replicated throttle",
    );
}
