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

/// Mirrors the struct layouts declared in `include/tem_recorder.h`; any
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

    assert_layout!(TrCapturedShot, size = 56, align = 8);
    assert_offset!(TrCapturedShot, ball, 0);
    assert_offset!(TrCapturedShot, time_limit, 36);
    assert_offset!(TrCapturedShot, cars, 40);
    assert_offset!(TrCapturedShot, car_count, 48);
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
    let scratch =
        std::env::temp_dir().join(format!("tem-recorder-ffi-smoke-{}", std::process::id()));
    let pack = tem_recorder_pack_create();
    assert!(!pack.is_null());
    unsafe {
        let name = CString::new("FFI Smoke Pack").unwrap();
        assert_eq!(tem_recorder_pack_set_name(pack, name.as_ptr()), 0);
        let creator = CString::new("subtr-actor").unwrap();
        assert_eq!(
            tem_recorder_pack_set_creator_name(pack, creator.as_ptr()),
            0
        );
        assert_eq!(tem_recorder_pack_set_difficulty(pack, 2), 0);
        assert_eq!(tem_recorder_pack_difficulty(pack), 2);

        let cars = [sample_car()];
        let shot = sample_shot(&cars);
        assert_eq!(tem_recorder_pack_add_shot(pack, &raw const shot), 0);
        assert_eq!(tem_recorder_pack_shot_count(pack), 1);

        let summary = read_string(tem_recorder_pack_shot_summary_len(pack, 0), |buf, cap| {
            tem_recorder_pack_write_shot_summary(pack, 0, buf, cap)
        });
        assert!(summary.contains("1 car"), "summary was {summary:?}");

        let mut guid_buffer = [0u8; 32];
        assert_eq!(
            tem_recorder_pack_guid_hex(pack, guid_buffer.as_mut_ptr(), guid_buffer.len()),
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
        assert_eq!(tem_recorder_pack_save(pack, save_path_c.as_ptr()), 0);

        let reopened = tem_recorder_pack_open(save_path_c.as_ptr());
        assert!(!reopened.is_null());
        assert_eq!(tem_recorder_pack_shot_count(reopened), 1);
        let reopened_name = read_string(tem_recorder_pack_name_len(reopened), |buf, cap| {
            tem_recorder_pack_write_name(reopened, buf, cap)
        });
        assert_eq!(reopened_name, "FFI Smoke Pack");
        tem_recorder_pack_destroy(reopened);
        tem_recorder_pack_destroy(pack);
    }
    let _ = std::fs::remove_dir_all(&scratch);
}

#[test]
fn ffi_open_missing_file_reports_global_error() {
    unsafe {
        let path = CString::new("/nonexistent/definitely-missing.Tem").unwrap();
        let pack = tem_recorder_pack_open(path.as_ptr());
        assert!(pack.is_null());
        let error = read_string(tem_recorder_last_error_len(), |buf, cap| {
            tem_recorder_write_last_error(buf, cap)
        });
        assert!(error.contains("could not read"), "error was {error:?}");
    }
}

#[test]
fn ffi_failed_remove_reports_pack_error() {
    let pack = tem_recorder_pack_create();
    unsafe {
        assert_eq!(tem_recorder_pack_remove_shot(pack, 5), 1);
        let error = read_string(tem_recorder_pack_last_error_len(pack), |buf, cap| {
            tem_recorder_pack_write_last_error(pack, buf, cap)
        });
        assert!(!error.is_empty());
        tem_recorder_pack_destroy(pack);
    }
}

#[test]
fn ffi_null_handles_are_safe() {
    unsafe {
        assert_eq!(tem_recorder_pack_shot_count(std::ptr::null()), 0);
        assert_eq!(tem_recorder_pack_remove_shot(std::ptr::null_mut(), 0), 1);
        assert_eq!(
            tem_recorder_pack_add_shot(std::ptr::null_mut(), std::ptr::null()),
            1
        );
        assert_eq!(tem_recorder_pack_last_error_len(std::ptr::null()), 0);
        tem_recorder_pack_destroy(std::ptr::null_mut());
    }
}
