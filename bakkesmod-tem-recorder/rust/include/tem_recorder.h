#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/*
 * C ABI of tem_recorder.dll (crate `subtr-actor-tem-recorder`).
 *
 * Struct layouts are locked by the Rust tests in `src/lib_tests.rs`; any
 * change here must be mirrored there and in `src/abi.rs`.
 *
 * Conventions (mirroring subtr_actor_bakkesmod.h):
 *  - `TrPack` is an opaque handle owned by the caller and freed with
 *    `tem_recorder_pack_destroy`.
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

/* One captured shot: ball plus `car_count` cars and a round time limit in
 * seconds. `cars` may be null when `car_count` is 0. */
typedef struct TrCapturedShot {
  TrBallState ball;
  float time_limit;
  const TrCarState *cars;
  size_t car_count;
} TrCapturedShot;

/* Constructors / destructor. `create` never returns null; `open` returns
 * null on failure (see tem_recorder_last_error_len). */
TrPack *tem_recorder_pack_create(void);
TrPack *tem_recorder_pack_open(const char *path);
void tem_recorder_pack_destroy(TrPack *pack);

/* Metadata setters. Null `name`/`code`/`creator_name` clears the field. */
int32_t tem_recorder_pack_set_name(TrPack *pack, const char *name);
int32_t tem_recorder_pack_set_code(TrPack *pack, const char *code);
int32_t tem_recorder_pack_set_creator_name(TrPack *pack, const char *creator_name);
int32_t tem_recorder_pack_set_map_name(TrPack *pack, const char *map_name);
/* 0 = Easy, 1 = Medium, 2 = Hard. */
int32_t tem_recorder_pack_set_difficulty(TrPack *pack, uint32_t difficulty);
uint32_t tem_recorder_pack_difficulty(const TrPack *pack);

/* Pack name readback for the settings UI. */
size_t tem_recorder_pack_name_len(const TrPack *pack);
size_t tem_recorder_pack_write_name(const TrPack *pack, uint8_t *out_bytes, size_t max_bytes);

/* Shots (training rounds). */
int32_t tem_recorder_pack_add_shot(TrPack *pack, const TrCapturedShot *shot);
int32_t tem_recorder_pack_remove_shot(TrPack *pack, size_t index);
size_t tem_recorder_pack_shot_count(const TrPack *pack);
size_t tem_recorder_pack_shot_summary_len(const TrPack *pack, size_t index);
size_t tem_recorder_pack_write_shot_summary(
    const TrPack *pack,
    size_t index,
    uint8_t *out_bytes,
    size_t max_bytes);

/* Writes the pack GUID as 32 uppercase hex characters (the game's `.Tem`
 * filename convention); returns bytes written. */
size_t tem_recorder_pack_guid_hex(const TrPack *pack, uint8_t *out_bytes, size_t max_bytes);

/* Serializes, encrypts, and writes the pack to `path` (creating parent
 * directories). */
int32_t tem_recorder_pack_save(TrPack *pack, const char *path);

/* Per-pack last error (set by failed pack operations). */
size_t tem_recorder_pack_last_error_len(const TrPack *pack);
size_t tem_recorder_pack_write_last_error(
    const TrPack *pack,
    uint8_t *out_bytes,
    size_t max_bytes);

/* Global last error (set by failed `tem_recorder_pack_open` calls). */
size_t tem_recorder_last_error_len(void);
size_t tem_recorder_write_last_error(uint8_t *out_bytes, size_t max_bytes);

#ifdef __cplusplus
}
#endif
