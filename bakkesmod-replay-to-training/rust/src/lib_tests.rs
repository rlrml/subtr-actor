use std::ffi::CString;

use crate::abi::{TrBallState, TrCapturedShot, TrCarState, TrRotator, TrVec3};
use crate::ffi::*;

macro_rules! assert_layout {
    ($ty:ty, size = $size:expr, align = $align:expr) => {
        assert_eq!(
            std::mem::size_of::<$ty>(),
            $size,
            "size of {}",
            stringify!($ty)
        );
        assert_eq!(
            std::mem::align_of::<$ty>(),
            $align,
            "alignment of {}",
            stringify!($ty)
        );
    };
}

macro_rules! assert_offset {
    ($ty:ty, $field:ident, $offset:expr) => {
        assert_eq!(
            std::mem::offset_of!($ty, $field),
            $offset,
            "offset of {}.{}",
            stringify!($ty),
            stringify!($field)
        );
    };
}

/// Mirrors the struct layouts declared in `include/replay_to_training.h`; any
/// change here must be reflected in the header and the C++ plugin.
#[test]
fn abi_layout_matches_plugin_header_expectations() {
    assert_layout!(TrVec3, size = 12, align = 4);
    assert_offset!(TrVec3, x, 0);
    assert_offset!(TrVec3, y, 4);
    assert_offset!(TrVec3, z, 8);

    assert_layout!(TrRotator, size = 12, align = 4);
    assert_offset!(TrRotator, pitch, 0);
    assert_offset!(TrRotator, yaw, 4);
    assert_offset!(TrRotator, roll, 8);

    assert_layout!(TrBallState, size = 36, align = 4);
    assert_offset!(TrBallState, location, 0);
    assert_offset!(TrBallState, linear_velocity, 12);
    assert_offset!(TrBallState, angular_velocity, 24);

    assert_layout!(TrCarState, size = 56, align = 4);
    assert_offset!(TrCarState, location, 0);
    assert_offset!(TrCarState, rotation, 12);
    assert_offset!(TrCarState, linear_velocity, 24);
    assert_offset!(TrCarState, angular_velocity, 36);
    assert_offset!(TrCarState, boost_amount, 48);
    assert_offset!(TrCarState, is_primary, 52);
    assert_offset!(TrCarState, team, 53);

    assert_layout!(TrCapturedShot, size = 64, align = 8);
    assert_offset!(TrCapturedShot, ball, 0);
    assert_offset!(TrCapturedShot, time_limit, 36);
    assert_offset!(TrCapturedShot, cars, 40);
    assert_offset!(TrCapturedShot, car_count, 48);
    assert_offset!(TrCapturedShot, mode, 56);
    assert_offset!(TrCapturedShot, mirror_by_team, 57);
    assert_offset!(TrCapturedShot, capture_momentum, 58);
}

fn sample_shot(cars: &[TrCarState]) -> TrCapturedShot {
    TrCapturedShot {
        ball: TrBallState {
            location: TrVec3 {
                x: 24.5,
                y: 4269.25,
                z: 224.4375,
            },
            linear_velocity: TrVec3 {
                x: 100.0,
                y: -250.0,
                z: 500.0,
            },
            angular_velocity: TrVec3::default(),
        },
        time_limit: 10.0,
        cars: if cars.is_empty() {
            std::ptr::null()
        } else {
            cars.as_ptr()
        },
        car_count: cars.len(),
        // Shot mode with mirroring/momentum off: these smoke tests assert
        // pre-existing byte-level behavior, and the sample car is team 0 so
        // mirroring would be a no-op anyway.
        mode: 0,
        mirror_by_team: 0,
        capture_momentum: 0,
    }
}

fn sample_car() -> TrCarState {
    TrCarState {
        location: TrVec3 {
            x: -600.0,
            y: -700.0,
            z: 530.0,
        },
        rotation: TrRotator {
            pitch: -837,
            yaw: 3634,
            roll: 0,
        },
        boost_amount: 0.33,
        is_primary: 1,
        ..TrCarState::default()
    }
}

fn read_string(length: usize, write: impl FnOnce(*mut u8, usize) -> usize) -> String {
    let mut buffer = vec![0u8; length];
    let written = write(buffer.as_mut_ptr(), buffer.len());
    assert_eq!(written, length);
    String::from_utf8(buffer).expect("ABI strings are UTF-8")
}

#[test]
fn ffi_smoke_create_edit_save_open_destroy() {
    let scratch = std::env::temp_dir().join(format!(
        "replay-to-training-ffi-smoke-{}",
        std::process::id()
    ));
    let pack = replay_to_training_pack_create();
    assert!(!pack.is_null());
    unsafe {
        let name = CString::new("FFI Smoke Pack").unwrap();
        assert_eq!(replay_to_training_pack_set_name(pack, name.as_ptr()), 0);
        let creator = CString::new("subtr-actor").unwrap();
        assert_eq!(
            replay_to_training_pack_set_creator_name(pack, creator.as_ptr()),
            0
        );
        assert_eq!(replay_to_training_pack_set_difficulty(pack, 2), 0);
        assert_eq!(replay_to_training_pack_difficulty(pack), 2);

        let cars = [sample_car()];
        let shot = sample_shot(&cars);
        assert_eq!(replay_to_training_pack_add_shot(pack, &raw const shot), 0);
        assert_eq!(replay_to_training_pack_shot_count(pack), 1);

        let summary = read_string(
            replay_to_training_pack_shot_summary_len(pack, 0),
            |buf, cap| replay_to_training_pack_write_shot_summary(pack, 0, buf, cap),
        );
        assert!(summary.contains("1 car"), "summary was {summary:?}");

        let mut guid_buffer = [0u8; 32];
        assert_eq!(
            replay_to_training_pack_guid_hex(pack, guid_buffer.as_mut_ptr(), guid_buffer.len()),
            32
        );
        let guid_hex = std::str::from_utf8(&guid_buffer).unwrap();
        assert!(
            guid_hex
                .chars()
                .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_lowercase()),
            "guid hex was {guid_hex:?}"
        );

        let save_path = scratch.join(format!("{guid_hex}.Tem"));
        let save_path_c = CString::new(save_path.to_str().unwrap()).unwrap();
        assert_eq!(replay_to_training_pack_save(pack, save_path_c.as_ptr()), 0);

        let reopened = replay_to_training_pack_open(save_path_c.as_ptr());
        assert!(!reopened.is_null());
        assert_eq!(replay_to_training_pack_shot_count(reopened), 1);
        let reopened_name = read_string(replay_to_training_pack_name_len(reopened), |buf, cap| {
            replay_to_training_pack_write_name(reopened, buf, cap)
        });
        assert_eq!(reopened_name, "FFI Smoke Pack");
        replay_to_training_pack_destroy(reopened);
        replay_to_training_pack_destroy(pack);
    }
    let _ = std::fs::remove_dir_all(&scratch);
}

/// Mode-driven training-type assignment across the ABI: a fresh pack
/// reports unset (4), the first capture's mode assigns the type, a
/// mismatched-mode capture is REFUSED with code 2 (nothing added,
/// explanatory last-error), and the manual override setter re-types the
/// pack. The capture-mode-sync export follows the assigned type.
#[test]
fn ffi_training_type_assignment_and_mismatch_refusal() {
    let pack = replay_to_training_pack_create();
    unsafe {
        assert_eq!(replay_to_training_pack_training_type(pack), 4);
        // Unset type: the plugin's selection cvar is left untouched.
        assert_eq!(replay_to_training_pack_capture_mode_sync(pack), -1);
        let cars = [sample_car()];
        let mut shot = sample_shot(&cars);
        shot.mode = 1; // defensive save capture
        assert_eq!(replay_to_training_pack_add_shot(pack, &raw const shot), 0);
        // First capture assigned Goalie; sync now selects save/goalie.
        assert_eq!(replay_to_training_pack_training_type(pack), 2);
        assert_eq!(replay_to_training_pack_capture_mode_sync(pack), 1);
        // A mismatched shot capture is REFUSED: code 2, nothing added, and
        // the last-error explains what to do instead.
        shot.mode = 0;
        assert_eq!(replay_to_training_pack_add_shot(pack, &raw const shot), 2);
        assert_eq!(replay_to_training_pack_shot_count(pack), 1);
        let error = read_string(replay_to_training_pack_last_error_len(pack), |buf, cap| {
            replay_to_training_pack_write_last_error(pack, buf, cap)
        });
        assert!(
            error.contains("refused") && error.contains("Training_Goalie"),
            "error was {error:?}"
        );
        // Manual override to Striker; shot captures are accepted again and
        // the sync flips to shot/striker.
        assert_eq!(replay_to_training_pack_set_training_type(pack, 3), 0);
        assert_eq!(replay_to_training_pack_training_type(pack), 3);
        assert_eq!(replay_to_training_pack_capture_mode_sync(pack), 0);
        assert_eq!(replay_to_training_pack_add_shot(pack, &raw const shot), 0);
        assert_eq!(replay_to_training_pack_shot_count(pack), 2);
        // Aerial/None overrides never drive the selection.
        assert_eq!(replay_to_training_pack_set_training_type(pack, 1), 0);
        assert_eq!(replay_to_training_pack_capture_mode_sync(pack), -1);
        assert_eq!(replay_to_training_pack_set_training_type(pack, 0), 0);
        assert_eq!(replay_to_training_pack_capture_mode_sync(pack), -1);
        // Out-of-range values are rejected.
        assert_eq!(replay_to_training_pack_set_training_type(pack, 9), 1);
        assert_eq!(
            replay_to_training_pack_training_type(std::ptr::null()),
            4,
            "null pack reports unset"
        );
        assert_eq!(
            replay_to_training_pack_capture_mode_sync(std::ptr::null()),
            -1,
            "null pack leaves the selection untouched"
        );
        replay_to_training_pack_destroy(pack);
    }
}

#[test]
fn ffi_open_missing_file_reports_global_error() {
    unsafe {
        let path = CString::new("/nonexistent/definitely-missing.Tem").unwrap();
        let pack = replay_to_training_pack_open(path.as_ptr());
        assert!(pack.is_null());
        let error = read_string(replay_to_training_last_error_len(), |buf, cap| {
            replay_to_training_write_last_error(buf, cap)
        });
        assert!(error.contains("could not read"), "error was {error:?}");
    }
}

#[test]
fn ffi_failed_remove_reports_pack_error() {
    let pack = replay_to_training_pack_create();
    unsafe {
        assert_eq!(replay_to_training_pack_remove_shot(pack, 5), 1);
        let error = read_string(replay_to_training_pack_last_error_len(pack), |buf, cap| {
            replay_to_training_pack_write_last_error(pack, buf, cap)
        });
        assert!(!error.is_empty());
        replay_to_training_pack_destroy(pack);
    }
}

#[test]
fn ffi_null_handles_are_safe() {
    unsafe {
        assert_eq!(replay_to_training_pack_shot_count(std::ptr::null()), 0);
        assert_eq!(
            replay_to_training_pack_remove_shot(std::ptr::null_mut(), 0),
            1
        );
        assert_eq!(
            replay_to_training_pack_add_shot(std::ptr::null_mut(), std::ptr::null()),
            1
        );
        assert_eq!(replay_to_training_pack_last_error_len(std::ptr::null()), 0);
        replay_to_training_pack_destroy(std::ptr::null_mut());
    }
}

#[test]
fn ffi_target_save_appends_and_guards_against_clobber() {
    let scratch =
        std::env::temp_dir().join(format!("replay-to-training-target-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    let target = scratch.join("target.Tem");
    let target_c = CString::new(target.to_str().unwrap()).unwrap();

    unsafe {
        // A brand-new target path: the file GUID probe reports nothing yet.
        let mut probe = [0u8; 32];
        assert_eq!(
            replay_to_training_file_guid_hex(target_c.as_ptr(), probe.as_mut_ptr(), probe.len()),
            0
        );

        // Author a pack with one shot and save it to the target (Created).
        let pack = replay_to_training_pack_create();
        let cars = [sample_car()];
        let shot = sample_shot(&cars);
        assert_eq!(replay_to_training_pack_add_shot(pack, &raw const shot), 0);
        assert_eq!(
            replay_to_training_pack_save_to_target(pack, target_c.as_ptr()),
            0
        );
        // The probe now reports this pack's GUID (always 32 hex bytes).
        let pack_guid = read_string(32, |buf, cap| {
            replay_to_training_pack_guid_hex(pack, buf, cap)
        });
        let probed = read_string(32, |buf, cap| {
            replay_to_training_file_guid_hex(target_c.as_ptr(), buf, cap)
        });
        assert_eq!(pack_guid, probed);

        // Capture another shot and re-save to the same target: Appended.
        assert_eq!(replay_to_training_pack_add_shot(pack, &raw const shot), 0);
        assert_eq!(
            replay_to_training_pack_save_to_target(pack, target_c.as_ptr()),
            1
        );
        // Reopening the target sees both shots (append, not clobber).
        let reopened = replay_to_training_pack_open(target_c.as_ptr());
        assert!(!reopened.is_null());
        assert_eq!(replay_to_training_pack_shot_count(reopened), 2);
        replay_to_training_pack_destroy(reopened);

        // A fresh, unrelated pack must refuse to clobber the target (code 2).
        let foreign = replay_to_training_pack_create();
        assert_eq!(
            replay_to_training_pack_add_shot(foreign, &raw const shot),
            0
        );
        assert_eq!(
            replay_to_training_pack_save_to_target(foreign, target_c.as_ptr()),
            2
        );
        let error = read_string(
            replay_to_training_pack_last_error_len(foreign),
            |buf, cap| replay_to_training_pack_write_last_error(foreign, buf, cap),
        );
        assert!(error.contains("different pack"), "error was {error:?}");
        // The target on disk still has exactly the two appended shots.
        let after = replay_to_training_pack_open(target_c.as_ptr());
        assert_eq!(replay_to_training_pack_shot_count(after), 2);
        replay_to_training_pack_destroy(after);
        replay_to_training_pack_destroy(foreign);
        replay_to_training_pack_destroy(pack);
    }
    let _ = std::fs::remove_dir_all(&scratch);
}

/// The build-info exports must produce a well-formed identifier whatever
/// the build environment (env override, git, or the unknown fallback).
#[test]
fn ffi_build_info_reports_versioned_identifier() {
    let info = read_string(replay_to_training_build_info_len(), |buf, cap| unsafe {
        replay_to_training_write_build_info(buf, cap)
    });
    assert!(
        info.starts_with(&format!(
            "replay_to_training {} ",
            env!("CARGO_PKG_VERSION")
        )),
        "build info was {info:?}"
    );
    assert!(info.contains(" build="), "build info was {info:?}");
    assert!(
        info.contains(" dirty=0") || info.contains(" dirty=1"),
        "build info was {info:?}"
    );
    assert!(info.contains(" commit_date="), "build info was {info:?}");
}
