#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/*
 * C ABI of state_export.dll (crate `subtr-actor-state-export`).
 *
 * The engine is a thin export shell: sampled `SeFrame`s are converted to the
 * owned subtr-actor live model and broadcast over the `subtr-actor-live`
 * WebSocket protocol. No stats graph or analysis runs in-process.
 *
 * Struct layouts are locked by the Rust tests in `src/lib_tests/`; any
 * change here must be mirrored there and in `src/abi.rs`.
 *
 * Conventions (mirroring subtr_actor_bakkesmod.h / replay_to_training.h):
 *  - `SeEngine` is an opaque handle owned by the caller and freed with
 *    `state_export_engine_destroy`, which shuts the server down (joining
 *    every server thread) and MUST run before this DLL unloads.
 *  - The `Se*` frame/event structs mirror the `Sa*` shapes from
 *    subtr_actor_bakkesmod.h so sampling code ports mechanically;
 *    `SePlayerFrame` additionally carries controller input, camera state,
 *    dodge impulse/torque, and platform identity.
 *  - Optional fields use `has_` flag pairs (nonzero = present).
 *  - String outputs come as `..._len` / `..._write_...` pairs: the `len`
 *    function returns the UTF-8 byte length (no NUL terminator), the
 *    `write` function copies up to `max_bytes` bytes into the caller's
 *    buffer and returns the number of bytes written.
 *  - Fallible operations return 0 on success, -1 for invalid pointers, and
 *    -2 for operation failures; the message is retrievable through the
 *    engine last-error functions.
 */

typedef struct SeEngine SeEngine;

/* Recommended TCP port for the export server; the plugin's port cvar should
 * default to it (matches subtr-actor-live's DEFAULT_STATE_EXPORT_PORT). In
 * `SeConfig`, port 0 means "bind an ephemeral port" (read the bound port
 * back via state_export_status), NOT this default. */
#define SE_DEFAULT_STATE_EXPORT_PORT 49109

typedef struct SeVec3 {
  float x;
  float y;
  float z;
} SeVec3;

typedef struct SeQuat {
  float x;
  float y;
  float z;
  float w;
} SeQuat;

typedef struct SeRigidBody {
  SeVec3 location;
  SeQuat rotation;
  SeVec3 linear_velocity;
  SeVec3 angular_velocity;
  uint8_t has_linear_velocity;
  uint8_t has_angular_velocity;
  uint8_t sleeping;
} SeRigidBody;

/* Controller input axes in -1..1 plus button states (nonzero = pressed). */
typedef struct SeControllerInput {
  float throttle;
  float steer;
  float pitch;
  float yaw;
  float roll;
  float dodge_forward;
  float dodge_strafe;
  uint8_t handbrake;
  uint8_t jump;
  uint8_t activate_boost;
  uint8_t holding_boost;
} SeControllerInput;

/* Replay-style camera state; each part is independently optional. */
typedef struct SeCameraState {
  uint8_t pitch;
  uint8_t yaw;
  uint8_t has_pitch;
  uint8_t has_yaw;
  uint8_t ball_cam_active;
  uint8_t has_ball_cam;
} SeCameraState;

/* Platform discriminants for SeRemoteId.platform. Only platforms whose
 * boxcars identity is lossless from (platform, online_id | epic_id) map to
 * a concrete RemoteId:
 *   0 none        -> no identity (falls back to SplitScreen(player_index))
 *   1 steam       -> Steam(online_id)
 *   2 epic        -> Epic(epic_id)   (null/empty epic_id -> no identity)
 *   3 xbox        -> Xbox(online_id)
 *   4 psynet      -> no identity (PsyNet ids carry opaque payload bytes)
 *   5 switch      -> Switch(online_id)
 *   6 splitscreen -> SplitScreen(splitscreen_index)
 *   7 playstation -> no identity (PS4 ids carry a name + payload bytes)
 *   8 qq          -> QQ(online_id)
 * Unknown values behave like none. */
#define SE_REMOTE_ID_PLATFORM_NONE 0
#define SE_REMOTE_ID_PLATFORM_STEAM 1
#define SE_REMOTE_ID_PLATFORM_EPIC 2
#define SE_REMOTE_ID_PLATFORM_XBOX 3
#define SE_REMOTE_ID_PLATFORM_PSYNET 4
#define SE_REMOTE_ID_PLATFORM_SWITCH 5
#define SE_REMOTE_ID_PLATFORM_SPLITSCREEN 6
#define SE_REMOTE_ID_PLATFORM_PLAYSTATION 7
#define SE_REMOTE_ID_PLATFORM_QQ 8

/* Platform-qualified player identity. `epic_id` is only read when
 * `platform` is SE_REMOTE_ID_PLATFORM_EPIC. */
typedef struct SeRemoteId {
  uint64_t online_id;
  const char *epic_id;
  uint32_t splitscreen_index;
  uint8_t platform;
} SeRemoteId;

typedef struct SePlayerFrame {
  uint32_t player_index;
  const char *player_name;
  uint8_t is_team_0;
  uint8_t has_rigid_body;
  SeRigidBody rigid_body;
  float boost_amount;
  float last_boost_amount;
  uint8_t boost_active;
  uint8_t jump_active;
  uint8_t double_jump_active;
  uint8_t dodge_active;
  uint8_t powerslide_active;
  int32_t car_body_id;
  uint8_t has_car_body_id;
  uint8_t has_match_stats;
  int32_t match_goals;
  int32_t match_assists;
  int32_t match_saves;
  int32_t match_shots;
  int32_t match_score;
  uint8_t has_input;
  SeControllerInput input;
  SeCameraState camera;
  uint8_t has_dodge_impulse;
  SeVec3 dodge_impulse;
  uint8_t has_dodge_torque;
  SeVec3 dodge_torque;
  SeRemoteId remote_id;
} SePlayerFrame;

typedef struct SeEventTiming {
  uint64_t frame_number;
  float time;
  int32_t seconds_remaining;
  uint8_t has_timing;
  uint8_t has_seconds_remaining;
} SeEventTiming;

typedef struct SeTouchEvent {
  SeEventTiming timing;
  uint32_t player_index;
  uint8_t has_player;
  uint8_t is_team_0;
  float closest_approach_distance;
  uint8_t has_closest_approach_distance;
} SeTouchEvent;

typedef struct SeDodgeRefreshedEvent {
  SeEventTiming timing;
  uint32_t player_index;
  uint8_t is_team_0;
  int32_t counter_value;
} SeDodgeRefreshedEvent;

typedef enum SeBoostPadEventKind {
  SeBoostPadEventKindPickedUp = 1,
  SeBoostPadEventKindAvailable = 2,
} SeBoostPadEventKind;

typedef struct SeBoostPadEvent {
  SeEventTiming timing;
  uint32_t pad_id;
  SeBoostPadEventKind kind;
  uint8_t sequence;
  uint32_t player_index;
  uint8_t has_player;
} SeBoostPadEvent;

typedef struct SeGoalEvent {
  SeEventTiming timing;
  uint8_t scoring_team_is_team_0;
  uint32_t player_index;
  uint8_t has_player;
  int32_t team_zero_score;
  uint8_t has_team_zero_score;
  int32_t team_one_score;
  uint8_t has_team_one_score;
} SeGoalEvent;

typedef enum SePlayerStatEventKind {
  SePlayerStatEventKindShot = 1,
  SePlayerStatEventKindSave = 2,
  SePlayerStatEventKindAssist = 3,
} SePlayerStatEventKind;

typedef struct SePlayerStatEvent {
  SeEventTiming timing;
  uint32_t player_index;
  uint8_t is_team_0;
  SePlayerStatEventKind kind;
  uint8_t has_shot_ball;
  SeRigidBody shot_ball;
  uint8_t has_shot_player;
  SeRigidBody shot_player;
} SePlayerStatEvent;

typedef struct SeDemolishEvent {
  SeEventTiming timing;
  uint32_t attacker_index;
  uint32_t victim_index;
  SeVec3 attacker_velocity;
  SeVec3 victim_velocity;
  SeVec3 victim_location;
  float active_duration_seconds;
} SeDemolishEvent;

typedef struct SeFrame {
  uint64_t frame_number;
  float time;
  float dt;
  int32_t seconds_remaining;
  uint8_t has_seconds_remaining;
  int32_t game_state;
  uint8_t has_game_state;
  int32_t kickoff_countdown_time;
  uint8_t has_kickoff_countdown_time;
  uint8_t ball_has_been_hit;
  uint8_t has_ball_has_been_hit;
  int32_t team_zero_score;
  uint8_t has_team_zero_score;
  int32_t team_one_score;
  uint8_t has_team_one_score;
  uint8_t possession_team_is_team_0;
  uint8_t has_possession_team;
  uint8_t scored_on_team_is_team_0;
  uint8_t has_scored_on_team;
  uint8_t live_play;
  uint8_t has_live_play;
  uint8_t has_ball;
  SeRigidBody ball;
  const SePlayerFrame *players;
  size_t player_count;
  const SeTouchEvent *touches;
  size_t touch_count;
  const SeDodgeRefreshedEvent *dodge_refreshes;
  size_t dodge_refresh_count;
  const SeBoostPadEvent *boost_pad_events;
  size_t boost_pad_event_count;
  const SeGoalEvent *goals;
  size_t goal_count;
  const SePlayerStatEvent *player_stat_events;
  size_t player_stat_event_count;
  const SeDemolishEvent *demolishes;
  size_t demolish_count;
} SeFrame;

/* Server configuration. Zero selects defaults: port 0 binds an ephemeral
 * port (the plugin should pass its port cvar, defaulted to
 * SE_DEFAULT_STATE_EXPORT_PORT), bind_any_interface 0 binds 127.0.0.1
 * (nonzero binds 0.0.0.0), max_queued_frames 0 = 256, max_client_queue
 * 0 = 512, null server_name = "subtr-actor-state-export". */
typedef struct SeConfig {
  const char *server_name;
  uint32_t max_queued_frames;
  uint32_t max_client_queue;
  uint16_t port;
  uint8_t bind_any_interface;
} SeConfig;

/* SeStatus.state values. */
#define SE_STATE_STOPPED 0
#define SE_STATE_LISTENING 1
#define SE_STATE_ERROR 2

/* Engine status; cheap to poll every tick (atomic reads only). `port` is
 * the actually-bound port (resolves port 0). `frames_sent` counts broadcast
 * frames (once per frame, not per client); `frames_dropped` counts raw
 * frames dropped by ingest backpressure (their explicit events are
 * coalesced forward, not lost). */
typedef struct SeStatus {
  int32_t state;
  uint32_t client_count;
  uint16_t port;
  uint64_t frames_sent;
  uint64_t frames_dropped;
} SeStatus;

/* Match-level context that cannot be derived from player frames. All
 * pointers nullable; null (or has_playlist_id 0) clears the field. */
typedef struct SeMatchContext {
  const char *match_guid;
  const char *map_name;
  int32_t playlist_id;
  uint8_t has_playlist_id;
} SeMatchContext;

/* Creates the engine and starts its WebSocket server. Never returns null:
 * on bind failure the engine is returned in the error state
 * (state == SE_STATE_ERROR) with the message readable through the
 * last-error functions. Null `config` selects the documented defaults,
 * including SE_DEFAULT_STATE_EXPORT_PORT. */
SeEngine *state_export_engine_create(const SeConfig *config);

/* Shuts the server down (joining every server thread; worst case bounded by
 * one socket write timeout, currently 5s) and destroys the engine. MUST run
 * before this DLL unloads. */
void state_export_engine_destroy(SeEngine *engine);

/* Stops the running server (if any) and starts a fresh one with `config`
 * (settings-window Apply). The held match context is re-applied. Returns 0
 * on success, -1 for a null engine, -2 when the new server fails to start
 * (engine left stopped in the error state; last-error set). */
int32_t state_export_engine_restart(SeEngine *engine, const SeConfig *config);

/* Converts one sampled frame and enqueues it for broadcast. Never blocks
 * beyond a short mutex hold; the bounded ingest queue drops the oldest
 * frame on overflow (coalescing its explicit events forward). Returns 0 on
 * success, -1 for invalid pointers or malformed arrays (null pointer with
 * nonzero count), -2 when no server is running. */
int32_t state_export_push_frame(SeEngine *engine, const SeFrame *frame);

/* Sets (null `context` clears) the match context merged into the broadcast
 * match meta; a mid-match change is broadcast as a roster update. Held
 * across restarts, cleared by state_export_notify_match_end. Returns 0 on
 * success, -1 for a null engine. */
int32_t state_export_set_match_context(SeEngine *engine, const SeMatchContext *context);

/* Broadcasts MatchEnd (if a match was live) and resets the export stream
 * (event history, roster, match context) for the next match. Returns 0 on
 * success, -1 for a null engine, -2 when no server is running. */
int32_t state_export_notify_match_end(SeEngine *engine);

/* Writes the engine status into `out_status`. Returns 0 on success, -1 for
 * null pointers. */
int32_t state_export_status(const SeEngine *engine, SeStatus *out_status);

/* Engine last error (set by failed operations; empty when healthy). */
size_t state_export_last_error_len(const SeEngine *engine);
size_t state_export_write_last_error(
    const SeEngine *engine,
    uint8_t *out_bytes,
    size_t max_bytes);

/* Build identifier of the Rust core DLL ("state_export <version>
 * build=<hash> dirty=<0|1> commit_date=<date>"), logged by the plugin next
 * to its own build id so mismatched DLL pairs are visible. */
size_t state_export_build_info_len(void);
size_t state_export_write_build_info(uint8_t *out_bytes, size_t max_bytes);

#ifdef __cplusplus
}
#endif
