#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/*
 * C ABI of replay_to_training.dll (crate `subtr-actor-replay-to-training`).
 *
 * Struct layouts are locked by the Rust tests in `src/lib_tests.rs`; any
 * change here must be mirrored there and in `src/abi.rs`.
 *
 * Conventions (mirroring subtr_actor_bakkesmod.h):
 *  - `TrPack` is an opaque handle owned by the caller and freed with
 *    `replay_to_training_pack_destroy`.
 *  - String outputs come as `..._len` / `..._write_...` pairs: the `len`
 *    function returns the UTF-8 byte length (no NUL terminator), the
 *    `write` function copies up to `max_bytes` bytes into the caller's
 *    buffer and returns the number of bytes written.
 *  - Fallible operations return 0 on success and 1 on failure; the message
 *    is retrievable through the pack-level (or, for constructor failures,
 *    global) last-error functions.
 */

typedef struct TrPack TrPack;

/* A position or velocity vector in Unreal units (BakkesMod `Vector`). */
typedef struct TrVec3 {
  float x;
  float y;
  float z;
} TrVec3;

/* A rotation in integer Unreal rotator units, 65536 = full turn
 * (BakkesMod `Rotator`). */
typedef struct TrRotator {
  int32_t pitch;
  int32_t yaw;
  int32_t roll;
} TrRotator;

/* Captured ball state. Angular velocity is carried for phase-3; the
 * current .tem archetype format cannot express it. */
typedef struct TrBallState {
  TrVec3 location;
  TrVec3 linear_velocity;
  TrVec3 angular_velocity;
} TrBallState;

/* Captured car state. Velocities and boost (0.0..=1.0) are carried for
 * phase-3; the current .tem archetype format only stores location and
 * rotation. `is_primary` marks the car the shot is "for" (the `IsPC` car). */
typedef struct TrCarState {
  TrVec3 location;
  TrRotator rotation;
  TrVec3 linear_velocity;
  TrVec3 angular_velocity;
  float boost_amount;
  uint8_t is_primary;
  uint8_t team;
} TrCarState;

/* One captured shot: ball plus `car_count` cars, a round time limit in
 * seconds, and per-capture options. `cars` may be null when `car_count`
 * is 0.
 *
 *  - `mode`: 0 = offensive shot capture, 1 = defensive save capture.
 *    Decides the pack training type the first capture assigns
 *    (Striker/Goalie) and the orientation convention mirroring enforces.
 *  - `mirror_by_team`: nonzero mirrors the whole scenario 180 degrees
 *    about field center when the captured primary car's team does not
 *    match the training convention for `mode` (blue/team 0 defends -Y,
 *    training scenarios are blue-oriented; see rust/src/mirror.rs).
 *  - `capture_momentum`: nonzero writes the primary car's forward speed
 *    into the spawn mesh's VelocityStartSpeed. */
typedef struct TrCapturedShot {
  TrBallState ball;
  float time_limit;
  const TrCarState *cars;
  size_t car_count;
  uint8_t mode;
  uint8_t mirror_by_team;
  uint8_t capture_momentum;
} TrCapturedShot;

/* Constructors / destructor. `create` never returns null; `open` returns
 * null on failure (see replay_to_training_last_error_len). */
TrPack *replay_to_training_pack_create(void);
TrPack *replay_to_training_pack_open(const char *path);
void replay_to_training_pack_destroy(TrPack *pack);

/* Metadata setters. Null `name`/`code`/`creator_name` clears the field. */
int32_t replay_to_training_pack_set_name(TrPack *pack, const char *name);
int32_t replay_to_training_pack_set_code(TrPack *pack, const char *code);
int32_t replay_to_training_pack_set_creator_name(TrPack *pack, const char *creator_name);
int32_t replay_to_training_pack_set_map_name(TrPack *pack, const char *map_name);
/* 0 = Easy, 1 = Medium, 2 = Hard. */
int32_t replay_to_training_pack_set_difficulty(TrPack *pack, uint32_t difficulty);
uint32_t replay_to_training_pack_difficulty(const TrPack *pack);

/* Pack training type (ETrainingType): 0 = None, 1 = Aerial, 2 = Goalie,
 * 3 = Striker. A fresh pack's type is UNSET until the first capture's mode
 * assigns it (shot -> Striker, save -> Goalie) or the setter overrides it;
 * the getter additionally reports 4 = unset and 5 = a type this crate does
 * not model. The setter marks the type assigned (later mismatched-mode
 * captures are REFUSED via add_shot's return code 2) and rejects values
 * > 3. */
int32_t replay_to_training_pack_set_training_type(TrPack *pack, uint32_t training_type);
uint32_t replay_to_training_pack_training_type(const TrPack *pack);

/* The capture mode the plugin's mode-selection cvar
 * (replay_to_training_capture_mode) should sync to when this pack becomes
 * active (the pack's assigned type is authoritative):
 *   0  sync to shot/striker (Striker pack),
 *   1  sync to save/goalie (Goalie pack),
 *  -1  leave the selection untouched (null pack, unset type, or an
 *      Aerial/None/unmodeled type).
 * The 0/1 values match the TrCapturedShot mode encoding. */
int32_t replay_to_training_pack_capture_mode_sync(const TrPack *pack);

/* Pack name readback for the settings UI. */
size_t replay_to_training_pack_name_len(const TrPack *pack);
size_t replay_to_training_pack_write_name(const TrPack *pack, uint8_t *out_bytes, size_t max_bytes);

/* Shots (training rounds). add_shot returns:
 *   0  added (mode agreed with, or assigned, the pack training type),
 *   1  failure (last-error set),
 *   2  REFUSED: the capture's mode conflicts with the pack's assigned
 *      training type (pack-level ETrainingType cannot tag rounds). Nothing
 *      was added — the pack is untouched — and the pack's last-error is
 *      set to an explanatory message. */
int32_t replay_to_training_pack_add_shot(TrPack *pack, const TrCapturedShot *shot);

/* Momentum-loss diagnostic for momentum capture. The spawn mesh class has
 * only a scalar VelocityStartSpeed (no direction property, per the game's
 * class dump), so only the along-facing share of a captured car's velocity
 * is representable. When a meaningful share is lost (total speed > 300
 * uu/s AND (lost magnitude > 400 uu/s OR velocity more than 30 degrees off
 * the facing)), writes a human-readable warning like
 *   "car moving 1320 uu/s at 74° off facing; only 338 uu/s representable
 *    as spawn momentum"
 * into `out_bytes` (up to `max_bytes`, no NUL) and returns bytes written.
 * Returns 0 when there is no warning (or `car` is null). Stateless. */
size_t replay_to_training_momentum_note(
    const TrCarState *car,
    uint8_t *out_bytes,
    size_t max_bytes);
int32_t replay_to_training_pack_remove_shot(TrPack *pack, size_t index);
size_t replay_to_training_pack_shot_count(const TrPack *pack);
size_t replay_to_training_pack_shot_summary_len(const TrPack *pack, size_t index);
size_t replay_to_training_pack_write_shot_summary(
    const TrPack *pack,
    size_t index,
    uint8_t *out_bytes,
    size_t max_bytes);

/* Writes the pack GUID as 32 uppercase hex characters (the game's `.Tem`
 * filename convention); returns bytes written. */
size_t replay_to_training_pack_guid_hex(const TrPack *pack, uint8_t *out_bytes, size_t max_bytes);

/* Serializes, encrypts, and writes the pack to `path` (creating parent
 * directories). */
int32_t replay_to_training_pack_save(TrPack *pack, const char *path);

/* Non-destructively saves the pack to a target `.tem` `path`. Returns:
 *   0  created (path did not exist),
 *   1  appended (path held this same pack; the write is non-destructive),
 *   2  refused (path holds a DIFFERENT pack this session did not load;
 *      nothing was written; last-error is set to an explanation),
 *  -1  error (null pack/path or filesystem/parse failure; last-error set).
 * Memory is the single source of truth, so this never merges at the I/O
 * layer. */
int32_t replay_to_training_pack_save_to_target(TrPack *pack, const char *path);

/* Writes the GUID of the `.tem` pack at `path` as 32 uppercase hex
 * characters; returns bytes written (32 when large enough), or 0 when the
 * path is missing/unreadable/unparseable. Does not touch any in-memory
 * pack. Used by the plugin's clobber guard. */
size_t replay_to_training_file_guid_hex(const char *path, uint8_t *out_bytes, size_t max_bytes);

/* Target path logic (account-dir aware: the game keeps the listing folders
 * under `<root>/<account>/MyTraining` etc., where <account> is a 16-digit
 * or online-id directory; a root-level `MyTraining` is also scanned for
 * robustness).
 *
 * - sanitize_target: normalizes a user-entered name; returns bytes written
 *   (0 when empty/invalid).
 * - targets_len / write_targets: newline-joined sorted names discovered
 *   under `root`; duplicate stems across accounts come back qualified as
 *   `<account>\Folder\<stem>`.
 * - resolve_target: >= 0 = bytes of the resolved on-disk path written;
 *   -2 = ambiguous (newline-joined qualified candidates written);
 *   -1 = invalid/empty name.
 * - default_save_dir: directory untargeted auto-GUID saves land in (the
 *   sole account's MyTraining when exactly one account dir exists, else
 *   `root`); returns bytes written. */
size_t replay_to_training_sanitize_target(const char *name, uint8_t *out_bytes, size_t max_bytes);
size_t replay_to_training_targets_len(const char *root);
size_t replay_to_training_write_targets(const char *root, uint8_t *out_bytes, size_t max_bytes);
int32_t replay_to_training_resolve_target(
    const char *root,
    const char *name,
    uint8_t *out_bytes,
    size_t max_bytes);
size_t replay_to_training_default_save_dir(
    const char *root,
    uint8_t *out_bytes,
    size_t max_bytes);

/* Per-pack last error (set by failed pack operations). */
size_t replay_to_training_pack_last_error_len(const TrPack *pack);
size_t replay_to_training_pack_write_last_error(
    const TrPack *pack,
    uint8_t *out_bytes,
    size_t max_bytes);

/* Global last error (set by failed `replay_to_training_pack_open` calls). */
size_t replay_to_training_last_error_len(void);
size_t replay_to_training_write_last_error(uint8_t *out_bytes, size_t max_bytes);

/* Build identifier of the Rust core DLL ("replay_to_training <version>
 * build=<hash> dirty=<0|1> commit_date=<date>"), logged by the plugin's
 * `replay_to_training_version` command next to the plugin's own build id so
 * mismatched DLL pairs are visible. */
size_t replay_to_training_build_info_len(void);
size_t replay_to_training_write_build_info(uint8_t *out_bytes, size_t max_bytes);

#ifdef __cplusplus
}
#endif
