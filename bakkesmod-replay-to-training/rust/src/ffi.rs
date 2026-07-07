//! C ABI exported to the BakkesMod plugin.
//!
//! Conventions (mirroring `subtr-actor-bakkesmod`):
//! * `TrPack` is an opaque handle owned by the caller and freed with
//!   [`replay_to_training_pack_destroy`].
//! * String outputs come as `..._len` / `..._write_...` pairs: the `len`
//!   function returns the UTF-8 byte length (no NUL), the `write` function
//!   copies up to `max_bytes` bytes into the caller's buffer and returns
//!   the number of bytes written.
//! * Fallible operations return `0` on success and `1` on failure; the
//!   message is retrievable through the pack-level (or, for constructor
//!   failures, global) last-error functions.

use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::Mutex;

use subtr_actor_training::{Difficulty, TrainingType};

use crate::abi::TrCapturedShot;
use crate::mirror::CaptureMode;
use crate::recorder::{
    AddShotOutcome, RecorderPack, ShotOptions, TargetSaveOutcome, file_guid_hex,
};

/// Opaque pack handle exposed through the C ABI.
pub struct TrPack {
    pub(crate) inner: RecorderPack,
}

static GLOBAL_LAST_ERROR: Mutex<String> = Mutex::new(String::new());

fn set_global_error(message: String) {
    if let Ok(mut guard) = GLOBAL_LAST_ERROR.lock() {
        *guard = message;
    }
}

fn global_error() -> String {
    GLOBAL_LAST_ERROR
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_default()
}

unsafe fn pack_ref<'a>(pack: *const TrPack) -> Option<&'a TrPack> {
    unsafe { pack.as_ref() }
}

unsafe fn pack_mut<'a>(pack: *mut TrPack) -> Option<&'a mut TrPack> {
    unsafe { pack.as_mut() }
}

/// Reads a C string into `Option<&str>`; `None` for null or invalid UTF-8.
unsafe fn utf8_arg<'a>(value: *const c_char) -> Option<&'a str> {
    if value.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(value) }.to_str().ok()
}

/// Copies up to `max_bytes` of `text` into `out_bytes`, returning the
/// number of bytes written (no NUL terminator; pair with the `_len`
/// functions).
unsafe fn write_text(text: &str, out_bytes: *mut u8, max_bytes: usize) -> usize {
    if out_bytes.is_null() || max_bytes == 0 {
        return 0;
    }
    let bytes = text.as_bytes();
    let count = bytes.len().min(max_bytes);
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_bytes, count) };
    count
}

/// Runs a fallible pack operation, recording the error on the pack.
/// Returns 0 on success, 1 on failure (including a null pack).
unsafe fn with_pack_mut(
    pack: *mut TrPack,
    operation: impl FnOnce(&mut RecorderPack) -> Result<(), String>,
) -> i32 {
    let Some(pack) = (unsafe { pack_mut(pack) }) else {
        return 1;
    };
    match operation(&mut pack.inner) {
        Ok(()) => 0,
        Err(error) => {
            pack.inner.record_error(error);
            1
        }
    }
}

/// Creates a fresh in-memory training pack with generated GUID and default
/// metadata.
///
/// The caller owns the returned pointer and must free it with
/// `replay_to_training_pack_destroy`. Never returns null.
#[unsafe(no_mangle)]
pub extern "C" fn replay_to_training_pack_create() -> *mut TrPack {
    Box::into_raw(Box::new(TrPack {
        inner: RecorderPack::new(),
    }))
}

/// Opens an existing `.tem` file so new shots append to it.
///
/// Returns null on failure; the message is retrievable through
/// `replay_to_training_last_error_len` / `replay_to_training_write_last_error`.
///
/// # Safety
///
/// `path` must be null or a valid NUL-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_open(path: *const c_char) -> *mut TrPack {
    let Some(path) = (unsafe { utf8_arg(path) }) else {
        set_global_error("open: path is null or not valid UTF-8".to_string());
        return std::ptr::null_mut();
    };
    match RecorderPack::open(&PathBuf::from(path)) {
        Ok(inner) => Box::into_raw(Box::new(TrPack { inner })),
        Err(error) => {
            set_global_error(error);
            std::ptr::null_mut()
        }
    }
}

/// Destroys a pack handle allocated by `replay_to_training_pack_create` or
/// `replay_to_training_pack_open`.
///
/// # Safety
///
/// `pack` must be null or a pointer returned by one of the constructors,
/// not yet destroyed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_destroy(pack: *mut TrPack) {
    if !pack.is_null() {
        drop(unsafe { Box::from_raw(pack) });
    }
}

/// Sets the pack display name. Returns 0 on success.
///
/// # Safety
///
/// `pack` must be a valid pack handle; `name` must be null (clears the
/// name) or a valid NUL-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_set_name(
    pack: *mut TrPack,
    name: *const c_char,
) -> i32 {
    let name = unsafe { utf8_arg(name) }.map(str::to_string);
    unsafe { with_pack_mut(pack, |inner| inner.set_name(name.as_deref())) }
}

/// Sets the pack share code (null clears it). Returns 0 on success.
///
/// # Safety
///
/// `pack` must be a valid pack handle; `code` must be null or a valid
/// NUL-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_set_code(
    pack: *mut TrPack,
    code: *const c_char,
) -> i32 {
    let code = unsafe { utf8_arg(code) }.map(str::to_string);
    unsafe { with_pack_mut(pack, |inner| inner.set_code(code.as_deref())) }
}

/// Sets the creator display name. Returns 0 on success.
///
/// # Safety
///
/// `pack` must be a valid pack handle; `creator_name` must be null or a
/// valid NUL-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_set_creator_name(
    pack: *mut TrPack,
    creator_name: *const c_char,
) -> i32 {
    let creator_name = unsafe { utf8_arg(creator_name) }.map(str::to_string);
    unsafe {
        with_pack_mut(pack, |inner| {
            inner.set_creator_name(creator_name.as_deref())
        })
    }
}

/// Sets the map name the pack loads into. Returns 0 on success.
///
/// # Safety
///
/// `pack` must be a valid pack handle; `map_name` must be a valid
/// NUL-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_set_map_name(
    pack: *mut TrPack,
    map_name: *const c_char,
) -> i32 {
    let Some(map_name) = (unsafe { utf8_arg(map_name) }).map(str::to_string) else {
        return 1;
    };
    unsafe { with_pack_mut(pack, |inner| inner.set_map_name(&map_name)) }
}

/// Sets the pack difficulty: 0 = Easy, 1 = Medium, 2 = Hard. Returns 0 on
/// success.
///
/// # Safety
///
/// `pack` must be a valid pack handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_set_difficulty(
    pack: *mut TrPack,
    difficulty: u32,
) -> i32 {
    let difficulty = match difficulty {
        0 => Difficulty::Easy,
        1 => Difficulty::Medium,
        2 => Difficulty::Hard,
        _ => return 1,
    };
    unsafe { with_pack_mut(pack, |inner| inner.set_difficulty(&difficulty)) }
}

/// Returns the pack difficulty as 0 = Easy, 1 = Medium, 2 = Hard
/// (unknown/other values report 1).
///
/// # Safety
///
/// `pack` must be null or a valid pack handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_difficulty(pack: *const TrPack) -> u32 {
    let Some(pack) = (unsafe { pack_ref(pack) }) else {
        return 1;
    };
    match pack.inner.pack().map(|pack| pack.difficulty) {
        Ok(Difficulty::Easy) => 0,
        Ok(Difficulty::Hard) => 2,
        _ => 1,
    }
}

/// Returns the UTF-8 byte length of the pack display name.
///
/// # Safety
///
/// `pack` must be null or a valid pack handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_name_len(pack: *const TrPack) -> usize {
    unsafe { pack_ref(pack) }
        .and_then(|pack| pack.inner.pack().ok())
        .and_then(|pack| pack.name)
        .map(|name| name.len())
        .unwrap_or(0)
}

/// Copies the pack display name into `out_bytes` (up to `max_bytes`, no
/// NUL); returns bytes written.
///
/// # Safety
///
/// `pack` must be null or a valid pack handle; `out_bytes` must be null or
/// valid for `max_bytes` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_write_name(
    pack: *const TrPack,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let Some(name) = (unsafe { pack_ref(pack) })
        .and_then(|pack| pack.inner.pack().ok())
        .and_then(|pack| pack.name)
    else {
        return 0;
    };
    unsafe { write_text(&name, out_bytes, max_bytes) }
}

/// Appends a captured shot to the pack as a new round, honoring the
/// per-capture options carried in the shot (mode, mirror-by-team,
/// momentum). Returns:
///
/// * `0` — added; the mode agreed with (or, on the first capture into a
///   fresh pack, assigned) the pack training type,
/// * `1` — failure (null pack/shot or serialization error; last-error set),
/// * `2` — added, but the mode conflicts with the pack's already-assigned
///   training type (`ETrainingType` is pack-level, so the round cannot be
///   tagged differently); the caller should warn.
///
/// # Safety
///
/// `pack` must be a valid pack handle; `shot` must be null or point to a
/// valid `TrCapturedShot` whose `cars` pointer is valid for `car_count`
/// elements.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_add_shot(
    pack: *mut TrPack,
    shot: *const TrCapturedShot,
) -> i32 {
    let Some(shot) = (unsafe { shot.as_ref() }) else {
        return 1;
    };
    let cars = if shot.cars.is_null() || shot.car_count == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(shot.cars, shot.car_count) }
    };
    let options = ShotOptions {
        mode: CaptureMode::from_abi(shot.mode),
        mirror_by_team: shot.mirror_by_team != 0,
        capture_momentum: shot.capture_momentum != 0,
    };
    let ball = shot.ball;
    let time_limit = shot.time_limit;
    let Some(pack) = (unsafe { pack_mut(pack) }) else {
        return 1;
    };
    match pack.inner.add_shot(&ball, cars, time_limit, &options) {
        Ok(AddShotOutcome::Added) => 0,
        Ok(AddShotOutcome::AddedTypeMismatch) => 2,
        Err(error) => {
            pack.inner.record_error(error);
            1
        }
    }
}

/// ABI encoding of the pack training type (`ETrainingType`):
/// `0` = None, `1` = Aerial, `2` = Goalie, `3` = Striker. The getter
/// additionally reports `4` = unset (fresh pack; the first capture's mode
/// will assign the type) and `5` = a type this crate does not model.
const TR_TRAINING_TYPE_NONE: u32 = 0;
const TR_TRAINING_TYPE_AERIAL: u32 = 1;
const TR_TRAINING_TYPE_GOALIE: u32 = 2;
const TR_TRAINING_TYPE_STRIKER: u32 = 3;
const TR_TRAINING_TYPE_UNSET: u32 = 4;
const TR_TRAINING_TYPE_OTHER: u32 = 5;

/// Manually sets (overrides) the pack training type: 0 = None, 1 = Aerial,
/// 2 = Goalie, 3 = Striker. Marks the type as assigned, so later captures
/// warn on mode mismatch instead of re-assigning. Returns 0 on success.
///
/// # Safety
///
/// `pack` must be a valid pack handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_set_training_type(
    pack: *mut TrPack,
    training_type: u32,
) -> i32 {
    let training_type = match training_type {
        TR_TRAINING_TYPE_NONE => TrainingType::None,
        TR_TRAINING_TYPE_AERIAL => TrainingType::Aerial,
        TR_TRAINING_TYPE_GOALIE => TrainingType::Goalie,
        TR_TRAINING_TYPE_STRIKER => TrainingType::Striker,
        _ => return 1,
    };
    unsafe { with_pack_mut(pack, |inner| inner.set_training_type(&training_type)) }
}

/// Returns the pack training type in the encoding documented on
/// `replay_to_training_pack_set_training_type`, plus `4` for a fresh pack
/// whose type is still unset (first capture decides) and `5` for a type
/// this crate does not model. A null pack reports unset.
///
/// # Safety
///
/// `pack` must be null or a valid pack handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_training_type(pack: *const TrPack) -> u32 {
    let Some(pack) = (unsafe { pack_ref(pack) }) else {
        return TR_TRAINING_TYPE_UNSET;
    };
    if !pack.inner.training_type_assigned() {
        return TR_TRAINING_TYPE_UNSET;
    }
    match pack.inner.training_type() {
        Ok(TrainingType::None) => TR_TRAINING_TYPE_NONE,
        Ok(TrainingType::Aerial) => TR_TRAINING_TYPE_AERIAL,
        Ok(TrainingType::Goalie) => TR_TRAINING_TYPE_GOALIE,
        Ok(TrainingType::Striker) => TR_TRAINING_TYPE_STRIKER,
        Ok(_) | Err(_) => TR_TRAINING_TYPE_OTHER,
    }
}

/// Removes the shot (round) at `index`. Returns 0 on success.
///
/// # Safety
///
/// `pack` must be a valid pack handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_remove_shot(
    pack: *mut TrPack,
    index: usize,
) -> i32 {
    unsafe { with_pack_mut(pack, |inner| inner.remove_shot(index)) }
}

/// Number of shots (rounds) in the pack, including rounds already present
/// in an opened file.
///
/// # Safety
///
/// `pack` must be null or a valid pack handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_shot_count(pack: *const TrPack) -> usize {
    unsafe { pack_ref(pack) }
        .map(|pack| pack.inner.shot_count())
        .unwrap_or(0)
}

/// Returns the UTF-8 byte length of the shot summary at `index` (0 when
/// out of range).
///
/// # Safety
///
/// `pack` must be null or a valid pack handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_shot_summary_len(
    pack: *const TrPack,
    index: usize,
) -> usize {
    unsafe { pack_ref(pack) }
        .and_then(|pack| pack.inner.shot_summary(index))
        .map(|summary| summary.len())
        .unwrap_or(0)
}

/// Copies the shot summary at `index` into `out_bytes` (up to `max_bytes`,
/// no NUL); returns bytes written.
///
/// # Safety
///
/// `pack` must be null or a valid pack handle; `out_bytes` must be null or
/// valid for `max_bytes` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_write_shot_summary(
    pack: *const TrPack,
    index: usize,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let Some(summary) = (unsafe { pack_ref(pack) }).and_then(|pack| pack.inner.shot_summary(index))
    else {
        return 0;
    };
    unsafe { write_text(&summary, out_bytes, max_bytes) }
}

/// Copies the pack GUID as 32 uppercase hex characters (the game's `.Tem`
/// filename convention) into `out_bytes`; returns bytes written (32 when
/// `max_bytes` is large enough).
///
/// # Safety
///
/// `pack` must be null or a valid pack handle; `out_bytes` must be null or
/// valid for `max_bytes` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_guid_hex(
    pack: *const TrPack,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let Some(pack) = (unsafe { pack_ref(pack) }) else {
        return 0;
    };
    unsafe { write_text(&pack.inner.guid_hex(), out_bytes, max_bytes) }
}

/// Serializes, encrypts, and writes the pack to `path` (creating parent
/// directories). Returns 0 on success.
///
/// # Safety
///
/// `pack` must be a valid pack handle; `path` must be a valid
/// NUL-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_save(
    pack: *mut TrPack,
    path: *const c_char,
) -> i32 {
    let Some(path) = (unsafe { utf8_arg(path) }).map(PathBuf::from) else {
        return 1;
    };
    unsafe { with_pack_mut(pack, move |inner| inner.save(&path)) }
}

/// Non-destructively saves the pack to a target `.tem` `path`. Return codes:
///
/// * `0` — created (path did not exist),
/// * `1` — appended (path held this same pack; write is non-destructive),
/// * `2` — refused (path holds a DIFFERENT pack this session did not load;
///   nothing was written), and the pack's last-error is set to an
///   explanatory message,
/// * `-1` — a real error occurred (null pack/path, filesystem/parse error);
///   the pack's last-error is set.
///
/// See [`RecorderPack::save_to_target`]; memory is the single source of
/// truth, so this never merges at the I/O layer.
///
/// # Safety
///
/// `pack` must be a valid pack handle; `path` must be a valid
/// NUL-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_save_to_target(
    pack: *mut TrPack,
    path: *const c_char,
) -> i32 {
    let Some(pack) = (unsafe { pack_mut(pack) }) else {
        return -1;
    };
    let Some(path) = (unsafe { utf8_arg(path) }).map(PathBuf::from) else {
        pack.inner
            .record_error("save target: path is null or not valid UTF-8".to_string());
        return -1;
    };
    match pack.inner.save_to_target(&path) {
        Ok(TargetSaveOutcome::Created) => 0,
        Ok(TargetSaveOutcome::Appended) => 1,
        Ok(TargetSaveOutcome::RefusedDifferentPack) => {
            pack.inner.record_error(
                "target already contains a different pack; open/target it first to append"
                    .to_string(),
            );
            2
        }
        Err(error) => {
            pack.inner.record_error(error);
            -1
        }
    }
}

/// Writes the GUID of the `.tem` pack at `path` as 32 uppercase hex
/// characters into `out_bytes`; returns bytes written (32 when large
/// enough), or `0` when the path is missing/unreadable/unparseable (the
/// caller cannot distinguish those cases — it only needs "is there a
/// readable pack GUID here"). Does not touch the in-memory pack.
///
/// # Safety
///
/// `path` must be a valid NUL-terminated C string; `out_bytes` must be null
/// or valid for `max_bytes` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_file_guid_hex(
    path: *const c_char,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let Some(path) = (unsafe { utf8_arg(path) }) else {
        return 0;
    };
    match file_guid_hex(std::path::Path::new(path)) {
        Ok(Some(hex)) => unsafe { write_text(&hex, out_bytes, max_bytes) },
        _ => 0,
    }
}

// --- target path logic (see `targets` module docs for the account-dir
// layout) ---

/// Sanitizes a user-entered target name (slash/case/extension
/// normalization; a pasted full path collapses to
/// `<account>\Folder\<stem>`). Writes the result into `out_bytes`; returns
/// bytes written (`0` for a null/invalid or empty-after-sanitizing name).
///
/// # Safety
///
/// `name` must be null or a valid NUL-terminated C string; `out_bytes` must
/// be null or valid for `max_bytes` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_sanitize_target(
    name: *const c_char,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let Some(name) = (unsafe { utf8_arg(name) }) else {
        return 0;
    };
    unsafe {
        write_text(
            &crate::targets::sanitize_target_name(name),
            out_bytes,
            max_bytes,
        )
    }
}

fn joined_targets(root: &str) -> String {
    crate::targets::discover_targets(std::path::Path::new(root)).join("\n")
}

/// Returns the UTF-8 byte length of the newline-joined target list under
/// the Training `root` (see `replay_to_training_write_targets`).
///
/// # Safety
///
/// `root` must be null or a valid NUL-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_targets_len(root: *const c_char) -> usize {
    let Some(root) = (unsafe { utf8_arg(root) }) else {
        return 0;
    };
    joined_targets(root).len()
}

/// Writes the newline-joined, sorted target names discovered under the
/// Training `root` — scanning both `<root>/<Folder>` and every
/// `<root>/<account>/<Folder>` — into `out_bytes`; returns bytes written.
/// Duplicate stems across accounts come back qualified as
/// `<account>\Folder\<stem>`.
///
/// # Safety
///
/// `root` must be null or a valid NUL-terminated C string; `out_bytes` must
/// be null or valid for `max_bytes` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_write_targets(
    root: *const c_char,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let Some(root) = (unsafe { utf8_arg(root) }) else {
        return 0;
    };
    unsafe { write_text(&joined_targets(root), out_bytes, max_bytes) }
}

/// Resolves a target `name` against the Training `root` (account-dir
/// aware). Returns:
///
/// * `>= 0` — bytes of the resolved on-disk path written into `out_bytes`
///   (the file may not exist yet; the first save creates it),
/// * `-2` — the unqualified name matched existing files in several
///   locations; the newline-joined qualified candidate names are written
///   into `out_bytes` so the caller can tell the user what to pick,
/// * `-1` — null/invalid arguments or an empty-after-sanitizing name.
///
/// Both negative outcomes also record a human-readable message through the
/// global last-error functions (`replay_to_training_last_error_len` /
/// `replay_to_training_write_last_error`), so callers can surface it
/// without decoding `out_bytes`.
///
/// # Safety
///
/// `root` and `name` must be null or valid NUL-terminated C strings;
/// `out_bytes` must be null or valid for `max_bytes` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_resolve_target(
    root: *const c_char,
    name: *const c_char,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> i32 {
    let (Some(root), Some(name)) = (unsafe { utf8_arg(root) }, unsafe { utf8_arg(name) }) else {
        set_global_error("resolve target: null or non-UTF-8 argument".to_string());
        return -1;
    };
    match crate::targets::resolve_target_path(std::path::Path::new(root), name) {
        Ok(crate::targets::ResolvedTarget::Path(path)) => {
            let text = path.to_string_lossy();
            unsafe { write_text(&text, out_bytes, max_bytes) as i32 }
        }
        Ok(crate::targets::ResolvedTarget::Ambiguous(candidates)) => {
            set_global_error(format!(
                "target name is ambiguous across accounts; use the qualified \
                 form: {}",
                candidates.join(" or ")
            ));
            let joined = candidates.join("\n");
            unsafe { write_text(&joined, out_bytes, max_bytes) };
            -2
        }
        Err(error) => {
            set_global_error(error);
            -1
        }
    }
}

/// Writes the directory untargeted (auto-GUID) saves should land in: the
/// sole account directory's `MyTraining\` when the Training `root` has
/// exactly one account directory, otherwise `root` itself. Returns bytes
/// written.
///
/// # Safety
///
/// `root` must be null or a valid NUL-terminated C string; `out_bytes` must
/// be null or valid for `max_bytes` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_default_save_dir(
    root: *const c_char,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let Some(root) = (unsafe { utf8_arg(root) }) else {
        return 0;
    };
    let dir = crate::targets::default_save_dir(std::path::Path::new(root));
    let text = dir.to_string_lossy();
    unsafe { write_text(&text, out_bytes, max_bytes) }
}

/// Returns the UTF-8 byte length of the pack's last error message.
///
/// # Safety
///
/// `pack` must be null or a valid pack handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_last_error_len(pack: *const TrPack) -> usize {
    unsafe { pack_ref(pack) }
        .map(|pack| pack.inner.last_error().len())
        .unwrap_or(0)
}

/// Copies the pack's last error message into `out_bytes` (up to
/// `max_bytes`, no NUL); returns bytes written.
///
/// # Safety
///
/// `pack` must be null or a valid pack handle; `out_bytes` must be null or
/// valid for `max_bytes` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_pack_write_last_error(
    pack: *const TrPack,
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    let Some(pack) = (unsafe { pack_ref(pack) }) else {
        return 0;
    };
    unsafe { write_text(pack.inner.last_error(), out_bytes, max_bytes) }
}

/// Returns the UTF-8 byte length of the global last error (set by failed
/// `replay_to_training_pack_open` calls).
#[unsafe(no_mangle)]
pub extern "C" fn replay_to_training_last_error_len() -> usize {
    global_error().len()
}

/// Copies the global last error into `out_bytes` (up to `max_bytes`, no
/// NUL); returns bytes written.
///
/// # Safety
///
/// `out_bytes` must be null or valid for `max_bytes` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_write_last_error(
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    unsafe { write_text(&global_error(), out_bytes, max_bytes) }
}

/// Build identifier of this Rust core DLL, e.g.
/// `replay_to_training 1.1.0 build=20eb058 dirty=0 commit_date=2026-07-01T…`.
/// The hash/dirty/date values are embedded by `build.rs` (environment
/// override, then git, then `unknown`); the C++ plugin logs this next to its
/// own build id so mismatched DLL pairs are visible.
pub(crate) fn build_info() -> String {
    format!(
        "replay_to_training {} build={} dirty={} commit_date={}",
        env!("CARGO_PKG_VERSION"),
        env!("REPLAY_TO_TRAINING_GIT_HASH"),
        env!("REPLAY_TO_TRAINING_GIT_DIRTY"),
        env!("REPLAY_TO_TRAINING_COMMIT_DATE"),
    )
}

/// Returns the UTF-8 byte length of the build identifier string.
#[unsafe(no_mangle)]
pub extern "C" fn replay_to_training_build_info_len() -> usize {
    build_info().len()
}

/// Copies the build identifier into `out_bytes` (up to `max_bytes`, no
/// NUL); returns bytes written.
///
/// # Safety
///
/// `out_bytes` must be null or valid for `max_bytes` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn replay_to_training_write_build_info(
    out_bytes: *mut u8,
    max_bytes: usize,
) -> usize {
    unsafe { write_text(&build_info(), out_bytes, max_bytes) }
}
