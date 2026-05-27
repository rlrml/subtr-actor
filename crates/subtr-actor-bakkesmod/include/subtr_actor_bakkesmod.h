#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct SaEngine SaEngine;
typedef struct SaReplayAnnotations SaReplayAnnotations;

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
  const char *player_name;
  uint8_t is_team_0;
  uint8_t has_rigid_body;
  SaRigidBody rigid_body;
  float boost_amount;
  float last_boost_amount;
  uint8_t boost_active;
  uint8_t jump_active;
  uint8_t double_jump_active;
  uint8_t dodge_active;
  uint8_t powerslide_active;
  uint8_t has_match_stats;
  int32_t match_goals;
  int32_t match_assists;
  int32_t match_saves;
  int32_t match_shots;
  int32_t match_score;
} SaPlayerFrame;

typedef struct SaEventTiming {
  uint64_t frame_number;
  float time;
  int32_t seconds_remaining;
  uint8_t has_timing;
  uint8_t has_seconds_remaining;
} SaEventTiming;

typedef struct SaTouchEvent {
  SaEventTiming timing;
  uint32_t player_index;
  uint8_t has_player;
  uint8_t is_team_0;
  float closest_approach_distance;
  uint8_t has_closest_approach_distance;
} SaTouchEvent;

typedef struct SaDodgeRefreshedEvent {
  SaEventTiming timing;
  uint32_t player_index;
  uint8_t is_team_0;
  int32_t counter_value;
} SaDodgeRefreshedEvent;

typedef enum SaBoostPadEventKind {
  SaBoostPadEventKindPickedUp = 1,
  SaBoostPadEventKindAvailable = 2,
} SaBoostPadEventKind;

typedef struct SaBoostPadEvent {
  SaEventTiming timing;
  uint32_t pad_id;
  SaBoostPadEventKind kind;
  uint8_t sequence;
  uint32_t player_index;
  uint8_t has_player;
} SaBoostPadEvent;

typedef struct SaGoalEvent {
  SaEventTiming timing;
  uint8_t scoring_team_is_team_0;
  uint32_t player_index;
  uint8_t has_player;
  int32_t team_zero_score;
  uint8_t has_team_zero_score;
  int32_t team_one_score;
  uint8_t has_team_one_score;
} SaGoalEvent;

typedef enum SaPlayerStatEventKind {
  SaPlayerStatEventKindShot = 1,
  SaPlayerStatEventKindSave = 2,
  SaPlayerStatEventKindAssist = 3,
} SaPlayerStatEventKind;

typedef struct SaPlayerStatEvent {
  SaEventTiming timing;
  uint32_t player_index;
  uint8_t is_team_0;
  SaPlayerStatEventKind kind;
  uint8_t has_shot_ball;
  SaRigidBody shot_ball;
  uint8_t has_shot_player;
  SaRigidBody shot_player;
} SaPlayerStatEvent;

typedef struct SaDemolishEvent {
  SaEventTiming timing;
  uint32_t attacker_index;
  uint32_t victim_index;
  SaVec3 attacker_velocity;
  SaVec3 victim_velocity;
  SaVec3 victim_location;
  float active_duration_seconds;
} SaDemolishEvent;

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
  SaRigidBody ball;
  const SaPlayerFrame *players;
  size_t player_count;
  const SaTouchEvent *touches;
  size_t touch_count;
  const SaDodgeRefreshedEvent *dodge_refreshes;
  size_t dodge_refresh_count;
  const SaBoostPadEvent *boost_pad_events;
  size_t boost_pad_event_count;
  const SaGoalEvent *goals;
  size_t goal_count;
  const SaPlayerStatEvent *player_stat_events;
  size_t player_stat_event_count;
  const SaDemolishEvent *demolishes;
  size_t demolish_count;
} SaLiveFrame;

typedef enum SaMechanicKind {
  SaMechanicKindSpeedFlip = 1,
  SaMechanicKindHalfFlip = 2,
  SaMechanicKindWavedash = 3,
  SaMechanicKindBallCarry = 4,
  SaMechanicKindAirDribble = 5,
  SaMechanicKindCeilingShot = 6,
  SaMechanicKindWallAerial = 7,
  SaMechanicKindWallAerialShot = 8,
  SaMechanicKindCenter = 9,
  SaMechanicKindFlipReset = 10,
  SaMechanicKindDoubleTap = 11,
  SaMechanicKindFlick = 12,
  SaMechanicKindMustyFlick = 13,
  SaMechanicKindOneTimer = 14,
  SaMechanicKindPass = 15,
  SaMechanicKindHalfVolley = 16,
  SaMechanicKindWhiff = 17,
  SaMechanicKindBump = 18,
  SaMechanicKindBackboard = 19,
  SaMechanicKindBoostPickup = 20,
  SaMechanicKindDemo = 21,
  SaMechanicKindFiftyFifty = 22,
  SaMechanicKindAerialGoal = 23,
  SaMechanicKindHighAerialGoal = 24,
  SaMechanicKindLongDistanceGoal = 25,
  SaMechanicKindOwnHalfGoal = 26,
  SaMechanicKindEmptyNetGoal = 27,
  SaMechanicKindCounterAttackGoal = 28,
  SaMechanicKindFlickGoal = 29,
  SaMechanicKindDoubleTapGoal = 30,
  SaMechanicKindOneTimerGoal = 31,
  SaMechanicKindAirDribbleGoal = 32,
  SaMechanicKindFlipResetGoal = 33,
  SaMechanicKindHalfVolleyGoal = 34,
  SaMechanicKindGoal = 35,
  SaMechanicKindShot = 36,
  SaMechanicKindSave = 37,
  SaMechanicKindAssist = 38,
  SaMechanicKindDeath = 39,
  SaMechanicKindPassingGoal = 40,
} SaMechanicKind;

typedef struct SaMechanicEvent {
  SaMechanicKind kind;
  uint32_t player_index;
  uint8_t is_team_0;
  uint64_t frame_number;
  float time;
  float confidence;
} SaMechanicEvent;

typedef struct SaReplayPlayerInfo {
  uint32_t player_index;
  uint8_t is_team_0;
  const char *name;
} SaReplayPlayerInfo;

typedef enum SaTeamEventKind {
  SaTeamEventKindRush = 1,
} SaTeamEventKind;

typedef struct SaTeamEvent {
  SaTeamEventKind kind;
  uint8_t is_team_0;
  uint64_t start_frame;
  uint64_t end_frame;
  float start_time;
  float end_time;
  uint32_t attackers;
  uint32_t defenders;
  float confidence;
} SaTeamEvent;

typedef enum SaGoalBuildupKind {
  SaGoalBuildupKindCounterAttack = 1,
  SaGoalBuildupKindSustainedPressure = 2,
  SaGoalBuildupKindOther = 3,
} SaGoalBuildupKind;

typedef struct SaGoalContextEvent {
  uint64_t frame_number;
  float time;
  uint8_t scoring_team_is_team_0;
  uint8_t has_scorer;
  uint32_t scorer_index;
  uint8_t has_scoring_team_most_back_player;
  uint32_t scoring_team_most_back_player_index;
  uint8_t has_defending_team_most_back_player;
  uint32_t defending_team_most_back_player_index;
  uint8_t has_ball_position;
  SaVec3 ball_position;
  uint8_t has_ball_air_time_before_goal;
  float ball_air_time_before_goal;
  SaGoalBuildupKind goal_buildup;
} SaGoalContextEvent;

SaEngine *subtr_actor_bakkesmod_engine_create(void);
SaReplayAnnotations *subtr_actor_bakkesmod_replay_annotations_create(
    const char *replay_path);
void subtr_actor_bakkesmod_replay_annotations_destroy(
    SaReplayAnnotations *annotations);
size_t subtr_actor_bakkesmod_replay_annotation_count(
    const SaReplayAnnotations *annotations);
size_t subtr_actor_bakkesmod_replay_annotation_player_count(
    const SaReplayAnnotations *annotations);
size_t subtr_actor_bakkesmod_write_replay_annotation_players(
    const SaReplayAnnotations *annotations,
    SaReplayPlayerInfo *out_players,
    size_t max_players);
size_t subtr_actor_bakkesmod_poll_replay_annotations(
    SaReplayAnnotations *annotations,
    float replay_time,
    SaMechanicEvent *out_events,
    size_t max_events);
void subtr_actor_bakkesmod_engine_destroy(SaEngine *engine);
void subtr_actor_bakkesmod_engine_reset(SaEngine *engine);
int32_t subtr_actor_bakkesmod_finish(SaEngine *engine);
int32_t subtr_actor_bakkesmod_process_frame(SaEngine *engine, const SaLiveFrame *frame);
size_t subtr_actor_bakkesmod_pending_event_count(const SaEngine *engine);
size_t subtr_actor_bakkesmod_pending_team_event_count(const SaEngine *engine);
size_t subtr_actor_bakkesmod_pending_goal_context_event_count(const SaEngine *engine);
size_t subtr_actor_bakkesmod_decoded_stats_player_config_json_len(
    const char *encoded_config);
size_t subtr_actor_bakkesmod_write_decoded_stats_player_config_json(
    const char *encoded_config,
    uint8_t *out_bytes,
    size_t max_bytes);
size_t subtr_actor_bakkesmod_events_json_len(const SaEngine *engine);
size_t subtr_actor_bakkesmod_write_events_json(
    const SaEngine *engine,
    uint8_t *out_bytes,
    size_t max_bytes);
size_t subtr_actor_bakkesmod_frame_json_len(const SaEngine *engine);
size_t subtr_actor_bakkesmod_write_frame_json(
    const SaEngine *engine,
    uint8_t *out_bytes,
    size_t max_bytes);
size_t subtr_actor_bakkesmod_timeline_json_len(const SaEngine *engine);
size_t subtr_actor_bakkesmod_write_timeline_json(
    const SaEngine *engine,
    uint8_t *out_bytes,
    size_t max_bytes);
size_t subtr_actor_bakkesmod_stats_json_len(const SaEngine *engine);
size_t subtr_actor_bakkesmod_write_stats_json(
    const SaEngine *engine,
    uint8_t *out_bytes,
    size_t max_bytes);
size_t subtr_actor_bakkesmod_stats_module_json_len(
    const SaEngine *engine,
    const char *module_name);
size_t subtr_actor_bakkesmod_write_stats_module_json(
    const SaEngine *engine,
    const char *module_name,
    uint8_t *out_bytes,
    size_t max_bytes);
size_t subtr_actor_bakkesmod_stats_module_frame_json_len(
    const SaEngine *engine,
    const char *module_name);
size_t subtr_actor_bakkesmod_write_stats_module_frame_json(
    const SaEngine *engine,
    const char *module_name,
    uint8_t *out_bytes,
    size_t max_bytes);
size_t subtr_actor_bakkesmod_stats_module_config_json_len(
    const SaEngine *engine,
    const char *module_name);
size_t subtr_actor_bakkesmod_write_stats_module_config_json(
    const SaEngine *engine,
    const char *module_name,
    uint8_t *out_bytes,
    size_t max_bytes);
size_t subtr_actor_bakkesmod_graph_output_json_len(
    const SaEngine *engine,
    const char *output_name);
size_t subtr_actor_bakkesmod_write_graph_output_json(
    const SaEngine *engine,
    const char *output_name,
    uint8_t *out_bytes,
    size_t max_bytes);
size_t subtr_actor_bakkesmod_analysis_node_json_len(
    const SaEngine *engine,
    const char *node_name);
size_t subtr_actor_bakkesmod_write_analysis_node_json(
    const SaEngine *engine,
    const char *node_name,
    uint8_t *out_bytes,
    size_t max_bytes);
size_t subtr_actor_bakkesmod_analysis_node_names_json_len(
    const SaEngine *engine);
size_t subtr_actor_bakkesmod_write_analysis_node_names_json(
    const SaEngine *engine,
    uint8_t *out_bytes,
    size_t max_bytes);
size_t subtr_actor_bakkesmod_graph_info_json_len(const SaEngine *engine);
size_t subtr_actor_bakkesmod_write_graph_info_json(
    const SaEngine *engine,
    uint8_t *out_bytes,
    size_t max_bytes);
size_t subtr_actor_bakkesmod_drain_events(
    SaEngine *engine,
    SaMechanicEvent *out_events,
    size_t max_events);
size_t subtr_actor_bakkesmod_drain_team_events(
    SaEngine *engine,
    SaTeamEvent *out_events,
    size_t max_events);
size_t subtr_actor_bakkesmod_drain_goal_context_events(
    SaEngine *engine,
    SaGoalContextEvent *out_events,
    size_t max_events);

#ifdef __cplusplus
}
#endif
