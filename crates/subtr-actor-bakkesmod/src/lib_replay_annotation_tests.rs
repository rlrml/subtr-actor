use super::*;
use std::ffi::CString;

fn real_replay_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay")
        .canonicalize()
        .expect("real replay fixture should resolve")
}

#[test]
fn replay_annotations_parse_real_replay_and_poll_by_time() {
    let replay_path = CString::new(real_replay_path().to_string_lossy().as_bytes())
        .expect("fixture path should not contain interior nulls");
    let annotations =
        unsafe { subtr_actor_bakkesmod_replay_annotations_create(replay_path.as_ptr()) };
    assert!(!annotations.is_null());

    let annotation_count = unsafe { subtr_actor_bakkesmod_replay_annotation_count(annotations) };
    assert!(annotation_count > 0);
    let final_time = unsafe { (*annotations).events.last().expect("events").time + 1.0 };

    let mut events = vec![
        SaMechanicEvent {
            kind: SaMechanicKind::SpeedFlip,
            player_index: 0,
            is_team_0: 0,
            frame_number: 0,
            time: 0.0,
            confidence: 0.0,
        };
        annotation_count
    ];
    let initial_drained = unsafe {
        subtr_actor_bakkesmod_poll_replay_annotations(
            annotations,
            0.0,
            events.as_mut_ptr(),
            events.len(),
        )
    };
    let drained = initial_drained
        + unsafe {
            subtr_actor_bakkesmod_poll_replay_annotations(
                annotations,
                final_time,
                events.as_mut_ptr().add(initial_drained),
                events.len() - initial_drained,
            )
        };
    assert_eq!(drained, annotation_count);
    assert!(events[..drained]
        .windows(2)
        .all(|pair| pair[0].time <= pair[1].time));

    unsafe { subtr_actor_bakkesmod_replay_annotations_destroy(annotations) };
}

#[test]
fn replay_annotations_reject_null_path() {
    let annotations = unsafe { subtr_actor_bakkesmod_replay_annotations_create(std::ptr::null()) };
    assert!(annotations.is_null());
}
