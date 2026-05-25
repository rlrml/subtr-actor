#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct SaEngine SaEngine;

typedef struct SaVec3 {
  float x;
  float y;
  float z;
} SaVec3;

typedef struct SaQuat {
  float x;
  float y;
  float z;
  float w;
} SaQuat;

typedef struct SaRigidBody {
  SaVec3 location;
  SaQuat rotation;
  SaVec3 linear_velocity;
  SaVec3 angular_velocity;
  uint8_t has_linear_velocity;
  uint8_t has_angular_velocity;
  uint8_t sleeping;
} SaRigidBody;

typedef struct SaPlayerFrame {
  uint32_t player_index;
  uint8_t is_team_0;
  uint8_t has_rigid_body;
  SaRigidBody rigid_body;
  float boost_amount;
  float last_boost_amount;
  uint8_t boost_active;
  uint8_t dodge_active;
  uint8_t powerslide_active;
} SaPlayerFrame;

typedef struct SaLiveFrame {
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
  uint8_t live_play;
  uint8_t has_ball;
  SaRigidBody ball;
  const SaPlayerFrame *players;
  size_t player_count;
} SaLiveFrame;

typedef enum SaMechanicKind {
  SaMechanicKindSpeedFlip = 1,
  SaMechanicKindHalfFlip = 2,
  SaMechanicKindWavedash = 3,
} SaMechanicKind;

typedef struct SaMechanicEvent {
  SaMechanicKind kind;
  uint32_t player_index;
  uint8_t is_team_0;
  uint64_t frame_number;
  float time;
  float confidence;
} SaMechanicEvent;

SaEngine *subtr_actor_bakkesmod_engine_create(void);
void subtr_actor_bakkesmod_engine_destroy(SaEngine *engine);
void subtr_actor_bakkesmod_engine_reset(SaEngine *engine);
int32_t subtr_actor_bakkesmod_process_frame(SaEngine *engine, const SaLiveFrame *frame);
size_t subtr_actor_bakkesmod_pending_event_count(const SaEngine *engine);
size_t subtr_actor_bakkesmod_drain_events(
    SaEngine *engine,
    SaMechanicEvent *out_events,
    size_t max_events);

#ifdef __cplusplus
}
#endif
