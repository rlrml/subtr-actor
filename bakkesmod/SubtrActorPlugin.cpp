#include "SubtrActorPlugin.h"

#include <algorithm>
#include <array>
#include <cctype>
#include <cmath>
#include <cstddef>
#include <cstdlib>
#include <fstream>
#include <format>
#include <type_traits>

BAKKESMOD_PLUGIN(
    SubtrActorPlugin,
    "subtr-actor mechanic overlay",
    "0.1.0",
    PLUGINTYPE_FREEPLAY | PLUGINTYPE_CUSTOM_TRAINING | PLUGINTYPE_REPLAY)

namespace {

constexpr float PI = 3.14159265358979323846f;
constexpr float UNREAL_ROTATOR_TO_RADIANS = (2.0f * PI) / 65536.0f;
constexpr wchar_t RUST_DLL_NAME[] = L"subtr_actor_bakkesmod.dll";
constexpr char BALL_TOUCH_EVENT[] = "Function TAGame.Ball_TA.OnCarTouch";
constexpr char BOOST_PICKED_UP_EVENT[] = "Function TAGame.VehiclePickup_TA.EventPickedUp";
constexpr char BOOST_SPAWNED_EVENT[] = "Function TAGame.VehiclePickup_TA.EventSpawned";
constexpr char GOAL_SCORED_EVENT[] = "Function TAGame.GameEvent_Soccar_TA.EventGoalScored";
constexpr char CAR_DEMOLISHED_EVENT[] = "Function TAGame.Car_TA.Demolish";
constexpr char GRAPH_OUTPUT_USAGE[] =
    "subtr_actor_dump_graph_output "
    "<events|frame|timeline|stats|analysis_nodes|event_history|graph_info> [finish]";
constexpr float BOOST_PICKUP_ATTRIBUTION_RADIUS = 450.0f;
constexpr float STANDARD_BOOST_PAD_MATCH_RADIUS = 900.0f;
constexpr float DEMO_ACTIVE_DURATION_SECONDS = 3.0f;
constexpr float GOAL_EVENT_DEDUPE_WINDOW_SECONDS = 3.0f;
constexpr uint32_t NON_STANDARD_BOOST_PAD_ID_START = 1000;
constexpr uint64_t DODGE_REFRESH_TOUCH_FRAME_WINDOW = 2;
constexpr int GAME_STATE_KICKOFF_COUNTDOWN = 55;
constexpr int GAME_STATE_GOAL_SCORED_REPLAY = 86;

int moduleAnchor = 0;

bool isAbsoluteWindowsPath(const std::string &path) {
  return path.size() >= 3 && std::isalpha(static_cast<unsigned char>(path[0])) != 0 &&
         path[1] == ':' && (path[2] == '\\' || path[2] == '/');
}

std::optional<std::filesystem::path> existingReplayPathCandidate(
    const std::filesystem::path &path) {
  std::error_code error;
  if (!std::filesystem::exists(path, error)) {
    return std::nullopt;
  }
  const auto canonical = std::filesystem::weakly_canonical(path, error);
  return error ? path : canonical;
}

std::string normalizedReplayPathString(const std::filesystem::path &path) {
  std::error_code error;
  const auto canonical = std::filesystem::weakly_canonical(path, error);
  return (error ? path : canonical).string();
}

static_assert(sizeof(SaBoostPadEventKind) == 4);
static_assert(sizeof(SaPlayerStatEventKind) == 4);

static_assert(std::is_standard_layout_v<SaEventTiming>);
static_assert(sizeof(SaEventTiming) == 24);
static_assert(alignof(SaEventTiming) == 8);
static_assert(offsetof(SaEventTiming, frame_number) == 0);
static_assert(offsetof(SaEventTiming, time) == 8);
static_assert(offsetof(SaEventTiming, seconds_remaining) == 12);
static_assert(offsetof(SaEventTiming, has_timing) == 16);
static_assert(offsetof(SaEventTiming, has_seconds_remaining) == 17);
static_assert(sizeof(SaMechanicKind) == 4);
static_assert(sizeof(SaTeamEventKind) == 4);
static_assert(sizeof(SaGoalBuildupKind) == 4);

static_assert(std::is_standard_layout_v<SaVec3>);
static_assert(sizeof(SaVec3) == 12);
static_assert(alignof(SaVec3) == 4);
static_assert(offsetof(SaVec3, x) == 0);
static_assert(offsetof(SaVec3, y) == 4);
static_assert(offsetof(SaVec3, z) == 8);

static_assert(std::is_standard_layout_v<SaQuat>);
static_assert(sizeof(SaQuat) == 16);
static_assert(alignof(SaQuat) == 4);
static_assert(offsetof(SaQuat, x) == 0);
static_assert(offsetof(SaQuat, y) == 4);
static_assert(offsetof(SaQuat, z) == 8);
static_assert(offsetof(SaQuat, w) == 12);

static_assert(std::is_standard_layout_v<SaRigidBody>);
static_assert(sizeof(SaRigidBody) == 56);
static_assert(alignof(SaRigidBody) == 4);
static_assert(offsetof(SaRigidBody, location) == 0);
static_assert(offsetof(SaRigidBody, rotation) == 12);
static_assert(offsetof(SaRigidBody, linear_velocity) == 28);
static_assert(offsetof(SaRigidBody, angular_velocity) == 40);
static_assert(offsetof(SaRigidBody, has_linear_velocity) == 52);
static_assert(offsetof(SaRigidBody, has_angular_velocity) == 53);
static_assert(offsetof(SaRigidBody, sleeping) == 54);

static_assert(std::is_standard_layout_v<SaPlayerFrame>);
static_assert(sizeof(SaPlayerFrame) == 112);
static_assert(alignof(SaPlayerFrame) == 8);
static_assert(offsetof(SaPlayerFrame, player_index) == 0);
static_assert(offsetof(SaPlayerFrame, player_name) == 8);
static_assert(offsetof(SaPlayerFrame, is_team_0) == 16);
static_assert(offsetof(SaPlayerFrame, has_rigid_body) == 17);
static_assert(offsetof(SaPlayerFrame, rigid_body) == 20);
static_assert(offsetof(SaPlayerFrame, boost_amount) == 76);
static_assert(offsetof(SaPlayerFrame, last_boost_amount) == 80);
static_assert(offsetof(SaPlayerFrame, boost_active) == 84);
static_assert(offsetof(SaPlayerFrame, jump_active) == 85);
static_assert(offsetof(SaPlayerFrame, double_jump_active) == 86);
static_assert(offsetof(SaPlayerFrame, dodge_active) == 87);
static_assert(offsetof(SaPlayerFrame, powerslide_active) == 88);
static_assert(offsetof(SaPlayerFrame, has_match_stats) == 89);
static_assert(offsetof(SaPlayerFrame, match_goals) == 92);
static_assert(offsetof(SaPlayerFrame, match_assists) == 96);
static_assert(offsetof(SaPlayerFrame, match_saves) == 100);
static_assert(offsetof(SaPlayerFrame, match_shots) == 104);
static_assert(offsetof(SaPlayerFrame, match_score) == 108);

static_assert(std::is_standard_layout_v<SaTouchEvent>);
static_assert(sizeof(SaTouchEvent) == 40);
static_assert(alignof(SaTouchEvent) == 8);
static_assert(offsetof(SaTouchEvent, timing) == 0);
static_assert(offsetof(SaTouchEvent, player_index) == 24);
static_assert(offsetof(SaTouchEvent, has_player) == 28);
static_assert(offsetof(SaTouchEvent, is_team_0) == 29);
static_assert(offsetof(SaTouchEvent, closest_approach_distance) == 32);
static_assert(offsetof(SaTouchEvent, has_closest_approach_distance) == 36);

static_assert(std::is_standard_layout_v<SaDodgeRefreshedEvent>);
static_assert(sizeof(SaDodgeRefreshedEvent) == 40);
static_assert(alignof(SaDodgeRefreshedEvent) == 8);
static_assert(offsetof(SaDodgeRefreshedEvent, timing) == 0);
static_assert(offsetof(SaDodgeRefreshedEvent, player_index) == 24);
static_assert(offsetof(SaDodgeRefreshedEvent, is_team_0) == 28);
static_assert(offsetof(SaDodgeRefreshedEvent, counter_value) == 32);

static_assert(std::is_standard_layout_v<SaBoostPadEvent>);
static_assert(sizeof(SaBoostPadEvent) == 48);
static_assert(alignof(SaBoostPadEvent) == 8);
static_assert(offsetof(SaBoostPadEvent, timing) == 0);
static_assert(offsetof(SaBoostPadEvent, pad_id) == 24);
static_assert(offsetof(SaBoostPadEvent, kind) == 28);
static_assert(offsetof(SaBoostPadEvent, sequence) == 32);
static_assert(offsetof(SaBoostPadEvent, player_index) == 36);
static_assert(offsetof(SaBoostPadEvent, has_player) == 40);

static_assert(std::is_standard_layout_v<SaGoalEvent>);
static_assert(sizeof(SaGoalEvent) == 56);
static_assert(alignof(SaGoalEvent) == 8);
static_assert(offsetof(SaGoalEvent, timing) == 0);
static_assert(offsetof(SaGoalEvent, scoring_team_is_team_0) == 24);
static_assert(offsetof(SaGoalEvent, player_index) == 28);
static_assert(offsetof(SaGoalEvent, has_player) == 32);
static_assert(offsetof(SaGoalEvent, team_zero_score) == 36);
static_assert(offsetof(SaGoalEvent, has_team_zero_score) == 40);
static_assert(offsetof(SaGoalEvent, team_one_score) == 44);
static_assert(offsetof(SaGoalEvent, has_team_one_score) == 48);

static_assert(std::is_standard_layout_v<SaPlayerStatEvent>);
static_assert(sizeof(SaPlayerStatEvent) == 160);
static_assert(alignof(SaPlayerStatEvent) == 8);
static_assert(offsetof(SaPlayerStatEvent, timing) == 0);
static_assert(offsetof(SaPlayerStatEvent, player_index) == 24);
static_assert(offsetof(SaPlayerStatEvent, is_team_0) == 28);
static_assert(offsetof(SaPlayerStatEvent, kind) == 32);
static_assert(offsetof(SaPlayerStatEvent, has_shot_ball) == 36);
static_assert(offsetof(SaPlayerStatEvent, shot_ball) == 40);
static_assert(offsetof(SaPlayerStatEvent, has_shot_player) == 96);
static_assert(offsetof(SaPlayerStatEvent, shot_player) == 100);

static_assert(std::is_standard_layout_v<SaDemolishEvent>);
static_assert(sizeof(SaDemolishEvent) == 72);
static_assert(alignof(SaDemolishEvent) == 8);
static_assert(offsetof(SaDemolishEvent, timing) == 0);
static_assert(offsetof(SaDemolishEvent, attacker_index) == 24);
static_assert(offsetof(SaDemolishEvent, victim_index) == 28);
static_assert(offsetof(SaDemolishEvent, attacker_velocity) == 32);
static_assert(offsetof(SaDemolishEvent, victim_velocity) == 44);
static_assert(offsetof(SaDemolishEvent, victim_location) == 56);
static_assert(offsetof(SaDemolishEvent, active_duration_seconds) == 68);

static_assert(std::is_standard_layout_v<SaLiveFrame>);
static_assert(sizeof(SaLiveFrame) == 232);
static_assert(alignof(SaLiveFrame) == 8);
static_assert(offsetof(SaLiveFrame, frame_number) == 0);
static_assert(offsetof(SaLiveFrame, time) == 8);
static_assert(offsetof(SaLiveFrame, dt) == 12);
static_assert(offsetof(SaLiveFrame, seconds_remaining) == 16);
static_assert(offsetof(SaLiveFrame, has_seconds_remaining) == 20);
static_assert(offsetof(SaLiveFrame, game_state) == 24);
static_assert(offsetof(SaLiveFrame, has_game_state) == 28);
static_assert(offsetof(SaLiveFrame, kickoff_countdown_time) == 32);
static_assert(offsetof(SaLiveFrame, has_kickoff_countdown_time) == 36);
static_assert(offsetof(SaLiveFrame, ball_has_been_hit) == 37);
static_assert(offsetof(SaLiveFrame, has_ball_has_been_hit) == 38);
static_assert(offsetof(SaLiveFrame, team_zero_score) == 40);
static_assert(offsetof(SaLiveFrame, has_team_zero_score) == 44);
static_assert(offsetof(SaLiveFrame, team_one_score) == 48);
static_assert(offsetof(SaLiveFrame, has_team_one_score) == 52);
static_assert(offsetof(SaLiveFrame, possession_team_is_team_0) == 53);
static_assert(offsetof(SaLiveFrame, has_possession_team) == 54);
static_assert(offsetof(SaLiveFrame, scored_on_team_is_team_0) == 55);
static_assert(offsetof(SaLiveFrame, has_scored_on_team) == 56);
static_assert(offsetof(SaLiveFrame, live_play) == 57);
static_assert(offsetof(SaLiveFrame, has_live_play) == 58);
static_assert(offsetof(SaLiveFrame, has_ball) == 59);
static_assert(offsetof(SaLiveFrame, ball) == 60);
static_assert(offsetof(SaLiveFrame, players) == 120);
static_assert(offsetof(SaLiveFrame, player_count) == 128);
static_assert(offsetof(SaLiveFrame, touches) == 136);
static_assert(offsetof(SaLiveFrame, touch_count) == 144);
static_assert(offsetof(SaLiveFrame, dodge_refreshes) == 152);
static_assert(offsetof(SaLiveFrame, dodge_refresh_count) == 160);
static_assert(offsetof(SaLiveFrame, boost_pad_events) == 168);
static_assert(offsetof(SaLiveFrame, boost_pad_event_count) == 176);
static_assert(offsetof(SaLiveFrame, goals) == 184);
static_assert(offsetof(SaLiveFrame, goal_count) == 192);
static_assert(offsetof(SaLiveFrame, player_stat_events) == 200);
static_assert(offsetof(SaLiveFrame, player_stat_event_count) == 208);
static_assert(offsetof(SaLiveFrame, demolishes) == 216);
static_assert(offsetof(SaLiveFrame, demolish_count) == 224);

static_assert(std::is_standard_layout_v<SaMechanicEvent>);
static_assert(sizeof(SaMechanicEvent) == 32);
static_assert(alignof(SaMechanicEvent) == 8);
static_assert(offsetof(SaMechanicEvent, kind) == 0);
static_assert(offsetof(SaMechanicEvent, player_index) == 4);
static_assert(offsetof(SaMechanicEvent, is_team_0) == 8);
static_assert(offsetof(SaMechanicEvent, frame_number) == 16);
static_assert(offsetof(SaMechanicEvent, time) == 24);
static_assert(offsetof(SaMechanicEvent, confidence) == 28);

static_assert(std::is_standard_layout_v<SaTeamEvent>);
static_assert(sizeof(SaTeamEvent) == 48);
static_assert(alignof(SaTeamEvent) == 8);
static_assert(offsetof(SaTeamEvent, kind) == 0);
static_assert(offsetof(SaTeamEvent, is_team_0) == 4);
static_assert(offsetof(SaTeamEvent, start_frame) == 8);
static_assert(offsetof(SaTeamEvent, end_frame) == 16);
static_assert(offsetof(SaTeamEvent, start_time) == 24);
static_assert(offsetof(SaTeamEvent, end_time) == 28);
static_assert(offsetof(SaTeamEvent, attackers) == 32);
static_assert(offsetof(SaTeamEvent, defenders) == 36);
static_assert(offsetof(SaTeamEvent, confidence) == 40);

static_assert(std::is_standard_layout_v<SaGoalContextEvent>);
static_assert(sizeof(SaGoalContextEvent) == 64);
static_assert(alignof(SaGoalContextEvent) == 8);
static_assert(offsetof(SaGoalContextEvent, frame_number) == 0);
static_assert(offsetof(SaGoalContextEvent, time) == 8);
static_assert(offsetof(SaGoalContextEvent, scoring_team_is_team_0) == 12);
static_assert(offsetof(SaGoalContextEvent, has_scorer) == 13);
static_assert(offsetof(SaGoalContextEvent, scorer_index) == 16);
static_assert(offsetof(SaGoalContextEvent, has_scoring_team_most_back_player) == 20);
static_assert(offsetof(SaGoalContextEvent, scoring_team_most_back_player_index) == 24);
static_assert(offsetof(SaGoalContextEvent, has_defending_team_most_back_player) == 28);
static_assert(offsetof(SaGoalContextEvent, defending_team_most_back_player_index) == 32);
static_assert(offsetof(SaGoalContextEvent, has_ball_position) == 36);
static_assert(offsetof(SaGoalContextEvent, ball_position) == 40);
static_assert(offsetof(SaGoalContextEvent, has_ball_air_time_before_goal) == 52);
static_assert(offsetof(SaGoalContextEvent, ball_air_time_before_goal) == 56);
static_assert(offsetof(SaGoalContextEvent, goal_buildup) == 60);

struct BallTouchParams {
  uintptr_t hitCar;
  uint8_t hitType;
};

struct GoalScoredParams {
  uintptr_t gameEvent;
  uintptr_t ball;
  uintptr_t goal;
  int scoreIndex;
  int assistIndex;
};

struct CarDemolishedParams {
  uintptr_t demolisher;
};

struct StandardBoostPad {
  uint32_t id;
  Vector location;
};

const std::array<StandardBoostPad, 34> STANDARD_BOOST_PADS{{
    {1, {0.0f, -4240.0f, 70.0f}},
    {2, {0.0f, 4240.0f, 70.0f}},
    {3, {-1792.0f, -4184.0f, 70.0f}},
    {4, {1792.0f, -4184.0f, 70.0f}},
    {5, {-1792.0f, 4184.0f, 70.0f}},
    {6, {1792.0f, 4184.0f, 70.0f}},
    {7, {-3072.0f, -4096.0f, 73.0f}},
    {8, {3072.0f, -4096.0f, 73.0f}},
    {9, {-3072.0f, 4096.0f, 73.0f}},
    {10, {3072.0f, 4096.0f, 73.0f}},
    {11, {-940.0f, -3308.0f, 70.0f}},
    {12, {940.0f, -3308.0f, 70.0f}},
    {13, {-940.0f, 3308.0f, 70.0f}},
    {14, {940.0f, 3308.0f, 70.0f}},
    {15, {0.0f, -2816.0f, 70.0f}},
    {16, {0.0f, 2816.0f, 70.0f}},
    {17, {-3584.0f, -2484.0f, 70.0f}},
    {18, {3584.0f, -2484.0f, 70.0f}},
    {19, {-3584.0f, 2484.0f, 70.0f}},
    {20, {3584.0f, 2484.0f, 70.0f}},
    {21, {-1788.0f, -2300.0f, 70.0f}},
    {22, {1788.0f, -2300.0f, 70.0f}},
    {23, {-1788.0f, 2300.0f, 70.0f}},
    {24, {1788.0f, 2300.0f, 70.0f}},
    {25, {-2048.0f, -1036.0f, 70.0f}},
    {26, {2048.0f, -1036.0f, 70.0f}},
    {27, {-2048.0f, 1036.0f, 70.0f}},
    {28, {2048.0f, 1036.0f, 70.0f}},
    {29, {0.0f, -1024.0f, 70.0f}},
    {30, {0.0f, 1024.0f, 70.0f}},
    {31, {-3584.0f, 0.0f, 73.0f}},
    {32, {3584.0f, 0.0f, 73.0f}},
    {33, {-1024.0f, 0.0f, 70.0f}},
    {34, {1024.0f, 0.0f, 70.0f}},
}};

std::optional<uint32_t> nearestStandardBoostPadId(Vector location) {
  std::optional<uint32_t> bestId;
  float bestDistance = STANDARD_BOOST_PAD_MATCH_RADIUS;
  for (const auto &pad : STANDARD_BOOST_PADS) {
    const float distance = (location - pad.location).magnitude();
    if (distance <= bestDistance) {
      bestDistance = distance;
      bestId = pad.id;
    }
  }
  return bestId;
}

SaVec3 toSaVec3(Vector value) {
  return SaVec3{value.X, value.Y, value.Z};
}

SaQuat rotatorToQuat(Rotator rotation) {
  const float pitch = rotation.Pitch * UNREAL_ROTATOR_TO_RADIANS;
  const float yaw = rotation.Yaw * UNREAL_ROTATOR_TO_RADIANS;
  const float roll = rotation.Roll * UNREAL_ROTATOR_TO_RADIANS;

  const float cy = std::cos(yaw * 0.5f);
  const float sy = std::sin(yaw * 0.5f);
  const float cp = std::cos(pitch * 0.5f);
  const float sp = std::sin(pitch * 0.5f);
  const float cr = std::cos(roll * 0.5f);
  const float sr = std::sin(roll * 0.5f);

  return SaQuat{
      sr * cp * cy - cr * sp * sy,
      cr * sp * cy + sr * cp * sy,
      cr * cp * sy - sr * sp * cy,
      cr * cp * cy + sr * sp * sy,
  };
}

std::string mechanicLabel(SaMechanicKind kind) {
  switch (kind) {
  case SaMechanicKindAerialGoal:
    return "Aerial goal";
  case SaMechanicKindAirDribble:
    return "Air dribble";
  case SaMechanicKindAirDribbleGoal:
    return "Air dribble goal";
  case SaMechanicKindAssist:
    return "Assist";
  case SaMechanicKindBallCarry:
    return "Ball carry";
  case SaMechanicKindBackboard:
    return "Backboard";
  case SaMechanicKindBoostPickup:
    return "Boost pickup";
  case SaMechanicKindCeilingShot:
    return "Ceiling shot";
  case SaMechanicKindCenter:
    return "Center";
  case SaMechanicKindCounterAttackGoal:
    return "Counter attack goal";
  case SaMechanicKindDemo:
    return "Demo";
  case SaMechanicKindDeath:
    return "Demolished";
  case SaMechanicKindDoubleTapGoal:
    return "Double tap goal";
  case SaMechanicKindEmptyNetGoal:
    return "Empty net goal";
  case SaMechanicKindFiftyFifty:
    return "Fifty fifty";
  case SaMechanicKindDoubleTap:
    return "Double tap";
  case SaMechanicKindFlick:
    return "Flick";
  case SaMechanicKindFlickGoal:
    return "Flick goal";
  case SaMechanicKindFlipReset:
    return "Flip reset";
  case SaMechanicKindFlipResetGoal:
    return "Flip reset goal";
  case SaMechanicKindGoal:
    return "Goal";
  case SaMechanicKindSpeedFlip:
    return "Speed flip";
  case SaMechanicKindHalfFlip:
    return "Half flip";
  case SaMechanicKindHalfVolley:
    return "Half volley";
  case SaMechanicKindHalfVolleyGoal:
    return "Half volley goal";
  case SaMechanicKindHighAerialGoal:
    return "High aerial goal";
  case SaMechanicKindLongDistanceGoal:
    return "Long distance goal";
  case SaMechanicKindMustyFlick:
    return "Musty flick";
  case SaMechanicKindOneTimer:
    return "One timer";
  case SaMechanicKindOneTimerGoal:
    return "One timer goal";
  case SaMechanicKindOwnHalfGoal:
    return "Own half goal";
  case SaMechanicKindPass:
    return "Pass";
  case SaMechanicKindPassingGoal:
    return "Passing goal";
  case SaMechanicKindSave:
    return "Save";
  case SaMechanicKindShot:
    return "Shot";
  case SaMechanicKindWallAerial:
    return "Wall aerial";
  case SaMechanicKindWallAerialShot:
    return "Wall aerial shot";
  case SaMechanicKindWavedash:
    return "Wavedash";
  case SaMechanicKindWhiff:
    return "Whiff";
  case SaMechanicKindBump:
    return "Bump";
  default:
    return "Mechanic";
  }
}

std::string teamEventLabel(const SaTeamEvent &event) {
  switch (event.kind) {
  case SaTeamEventKindRush:
    return std::format("{}v{} rush", event.attackers, event.defenders);
  default:
    return "Team event";
  }
}

std::string goalBuildupLabel(SaGoalBuildupKind kind) {
  switch (kind) {
  case SaGoalBuildupKindCounterAttack:
    return "counter attack";
  case SaGoalBuildupKindSustainedPressure:
    return "sustained pressure";
  case SaGoalBuildupKindOther:
    return "goal";
  default:
    return "goal";
  }
}

std::string goalContextLabel(const SaGoalContextEvent &event) {
  if (event.has_ball_air_time_before_goal != 0) {
    return std::format(
        "{} context ({:.1f}s air)",
        goalBuildupLabel(event.goal_buildup),
        event.ball_air_time_before_goal);
  }
  return std::format("{} context", goalBuildupLabel(event.goal_buildup));
}

std::filesystem::path currentModuleDirectory() {
  HMODULE module = nullptr;
  const BOOL foundModule = GetModuleHandleExW(
      GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS |
          GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
      reinterpret_cast<LPCWSTR>(&moduleAnchor),
      &module);
  if (!foundModule || !module) {
    return {};
  }

  std::array<wchar_t, 32768> pathBuffer{};
  const DWORD length =
      GetModuleFileNameW(module, pathBuffer.data(), static_cast<DWORD>(pathBuffer.size()));
  if (length == 0 || length >= pathBuffer.size()) {
    return {};
  }

  return std::filesystem::path(pathBuffer.data()).parent_path();
}

std::vector<std::filesystem::path> rustLibrarySearchPaths(GameWrapper *gameWrapper) {
  std::vector<std::filesystem::path> paths;
  const auto moduleDirectory = currentModuleDirectory();
  if (!moduleDirectory.empty()) {
    paths.push_back(moduleDirectory / RUST_DLL_NAME);
  }
  if (gameWrapper) {
    paths.push_back(gameWrapper->GetDataFolder() / "subtr-actor" / RUST_DLL_NAME);
  }
  paths.emplace_back(RUST_DLL_NAME);
  return paths;
}

} // namespace

void SubtrActorPlugin::onLoad() {
  cvarManager->registerCvar(
      "subtr_actor_enabled",
      "0",
      "Enable live subtr-actor frame processing and analysis graph evaluation.",
      true,
      true,
      0,
      true,
      1);
  cvarManager->registerCvar(
      "subtr_actor_overlay_enabled",
      "1",
      "Draw subtr-actor mechanic overlay messages.",
      true,
      true,
      0,
      true,
      1);
  cvarManager->registerCvar(
      "subtr_actor_replay_annotations_enabled",
      "1",
      "Draw annotations from normal replay processing while watching Rocket League replays.",
      true,
      true,
      0,
      true,
      1);
  cvarManager->registerCvar(
      "subtr_actor_status_overlay_enabled",
      "1",
      "Draw subtr-actor live processing status.",
      true,
      true,
      0,
      true,
      1);
  cvarManager->registerCvar(
      "subtr_actor_sample_interval_ms",
      "8",
      "Minimum elapsed game time between live frame samples.",
      true,
      true,
      1,
      true,
      1000);
  cvarManager->registerCvar(
      "subtr_actor_profile_enabled",
      "0",
      "Log average live sampling, graph processing, and event drain timings.",
      true,
      true,
      0,
      true,
      1);
  cvarManager->registerCvar(
      "subtr_actor_profile_log_every",
      "120",
      "Number of processed live samples per profiling log line.",
      true,
      true,
      1,
      true,
      10000);

  loaded = loadRustLibrary();
  if (!loaded) {
    cvarManager->log("subtr-actor: failed to load subtr_actor_bakkesmod.dll");
    return;
  }

  liveTickCancelled = std::make_shared<bool>(false);
  gameWrapper->RegisterDrawable([this](CanvasWrapper canvas) { render(canvas); });
  scheduleLiveTick();
  cvarManager->registerNotifier(
      "subtr_actor_dump_graph",
      [this](std::vector<std::string> params) { dumpGraphJson(params); },
      "Writes graph metadata, timeline, events, frame, stats, and analysis node JSON. "
      "Pass 'finish' to flush delayed graph events first.",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "subtr_actor_dump_stats_module",
      [this](std::vector<std::string> params) { dumpStatsModuleJson(params); },
      "Writes one named graph-backed stats module JSON. Usage: "
      "subtr_actor_dump_stats_module <module_name> [finish]",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "subtr_actor_dump_stats_module_frame",
      [this](std::vector<std::string> params) { dumpStatsModuleFrameJson(params); },
      "Writes one named graph-backed stats module frame JSON. Usage: "
      "subtr_actor_dump_stats_module_frame <module_name> [finish]",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "subtr_actor_dump_stats_module_config",
      [this](std::vector<std::string> params) { dumpStatsModuleConfigJson(params); },
      "Writes one named graph-backed stats module config JSON. Usage: "
      "subtr_actor_dump_stats_module_config <module_name> [finish]",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "subtr_actor_dump_graph_output",
      [this](std::vector<std::string> params) { dumpGraphOutputJson(params); },
      std::string("Writes one named graph output JSON. Usage: ") + GRAPH_OUTPUT_USAGE,
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "subtr_actor_dump_analysis_node",
      [this](std::vector<std::string> params) { dumpAnalysisNodeJson(params); },
      "Writes one named graph-backed analysis node JSON. Usage: "
      "subtr_actor_dump_analysis_node <node_name> [finish]",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "subtr_actor_verify_graph",
      [this](std::vector<std::string> params) { verifyGraphRuntime(params); },
      "Calls the live graph outputs and every callable analysis node, logging byte sizes. "
      "Pass 'finish' to flush delayed graph events first; pass 'require_graph_events' "
      "or 'require_event_history' for strict event checks.",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "subtr_actor_self_test_graph",
      [this](std::vector<std::string> params) { selfTestGraphRuntime(params); },
      "Feeds a synthetic live frame with every required event family, then runs "
      "strict graph verification against a temporary Rust engine. Pass 'dump' "
      "to also write synthetic graph JSON snapshots.",
      PERMISSION_ALL);
  hookGameEvents();

  cvarManager->log("subtr-actor: mechanic overlay loaded");
}

void SubtrActorPlugin::onUnload() {
  if (liveTickCancelled) {
    *liveTickCancelled = true;
  }
  gameWrapper->UnregisterDrawables();
  unhookGameEvents();
  unloadRustLibrary();
}

void SubtrActorPlugin::hookGameEvents() {
  gameWrapper->HookEventWithCallerPost<BallWrapper>(
      BALL_TOUCH_EVENT,
      [this](BallWrapper, void *params, std::string) {
        if (!liveProcessingEnabled()) {
          return;
        }
        if (!params) {
          return;
        }
        const auto *touchParams = static_cast<const BallTouchParams *>(params);
        recordTouch(CarWrapper(touchParams->hitCar));
      });

  gameWrapper->HookEventWithCallerPost<ActorWrapper>(
      BOOST_PICKED_UP_EVENT,
      [this](ActorWrapper pickup, void *, std::string) {
        if (!liveProcessingEnabled()) {
          return;
        }
        recordBoostPadEvent(pickup, SaBoostPadEventKindPickedUp);
      });
  gameWrapper->HookEventWithCallerPost<ActorWrapper>(
      BOOST_SPAWNED_EVENT,
      [this](ActorWrapper pickup, void *, std::string) {
        if (!liveProcessingEnabled()) {
          return;
        }
        recordBoostPadEvent(pickup, SaBoostPadEventKindAvailable);
      });

  gameWrapper->HookEventWithCallerPost<ServerWrapper>(
      GOAL_SCORED_EVENT,
      [this](ServerWrapper server, void *params, std::string) {
        if (!liveProcessingEnabled()) {
          return;
        }
        auto goal = GoalWrapper(0);
        int scoreIndex = -1;
        int assistIndex = -1;
        if (params) {
          const auto *goalParams = static_cast<const GoalScoredParams *>(params);
          goal = GoalWrapper(goalParams->goal);
          scoreIndex = goalParams->scoreIndex;
          assistIndex = goalParams->assistIndex;
        }
        recordGoal(server, goal, scoreIndex, assistIndex);
      });

  gameWrapper->HookEventWithCallerPost<CarWrapper>(
      CAR_DEMOLISHED_EVENT,
      [this](CarWrapper victim, void *params, std::string) {
        if (!liveProcessingEnabled()) {
          return;
        }
        if (!params) {
          return;
        }
        const auto *demolishParams = static_cast<const CarDemolishedParams *>(params);
        recordDemolish(victim, ActorWrapper(demolishParams->demolisher));
      });
}

void SubtrActorPlugin::unhookGameEvents() {
  gameWrapper->UnhookEventPost(BALL_TOUCH_EVENT);
  gameWrapper->UnhookEventPost(BOOST_PICKED_UP_EVENT);
  gameWrapper->UnhookEventPost(BOOST_SPAWNED_EVENT);
  gameWrapper->UnhookEventPost(GOAL_SCORED_EVENT);
  gameWrapper->UnhookEventPost(CAR_DEMOLISHED_EVENT);
}

bool SubtrActorPlugin::loadRustLibrary() {
  for (const auto &dllPath : rustLibrarySearchPaths(gameWrapper.get())) {
    rustLibrary = LoadLibraryW(dllPath.c_str());
    if (rustLibrary) {
      cvarManager->log(std::format("subtr-actor: loaded Rust ABI from {}", dllPath.string()));
      break;
    }
  }
  if (!rustLibrary) {
    cvarManager->log(
        std::format("subtr-actor: LoadLibrary failed with error {}", GetLastError()));
    return false;
  }

  engineCreate = reinterpret_cast<EngineCreate>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_engine_create"));
  engineDestroy = reinterpret_cast<EngineDestroy>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_engine_destroy"));
  engineReset = reinterpret_cast<EngineReset>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_engine_reset"));
  engineFinish = reinterpret_cast<EngineFinish>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_finish"));
  processFrame = reinterpret_cast<ProcessFrame>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_process_frame"));
  eventsJsonLen = reinterpret_cast<EventsJsonLen>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_events_json_len"));
  writeEventsJson = reinterpret_cast<WriteEventsJson>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_write_events_json"));
  frameJsonLen = reinterpret_cast<FrameJsonLen>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_frame_json_len"));
  writeFrameJson = reinterpret_cast<WriteFrameJson>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_write_frame_json"));
  timelineJsonLen = reinterpret_cast<TimelineJsonLen>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_timeline_json_len"));
  writeTimelineJson = reinterpret_cast<WriteTimelineJson>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_write_timeline_json"));
  statsJsonLen = reinterpret_cast<StatsJsonLen>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_stats_json_len"));
  writeStatsJson = reinterpret_cast<WriteStatsJson>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_write_stats_json"));
  statsModuleJsonLen = reinterpret_cast<StatsModuleJsonLen>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_stats_module_json_len"));
  writeStatsModuleJson = reinterpret_cast<WriteStatsModuleJson>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_write_stats_module_json"));
  statsModuleFrameJsonLen = reinterpret_cast<StatsModuleFrameJsonLen>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_stats_module_frame_json_len"));
  writeStatsModuleFrameJson = reinterpret_cast<WriteStatsModuleFrameJson>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_write_stats_module_frame_json"));
  statsModuleConfigJsonLen = reinterpret_cast<StatsModuleConfigJsonLen>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_stats_module_config_json_len"));
  writeStatsModuleConfigJson = reinterpret_cast<WriteStatsModuleConfigJson>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_write_stats_module_config_json"));
  graphOutputJsonLen = reinterpret_cast<GraphOutputJsonLen>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_graph_output_json_len"));
  writeGraphOutputJson = reinterpret_cast<WriteGraphOutputJson>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_write_graph_output_json"));
  analysisNodeJsonLen = reinterpret_cast<AnalysisNodeJsonLen>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_analysis_node_json_len"));
  writeAnalysisNodeJson = reinterpret_cast<WriteAnalysisNodeJson>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_write_analysis_node_json"));
  analysisNodeNamesJsonLen = reinterpret_cast<AnalysisNodeNamesJsonLen>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_analysis_node_names_json_len"));
  writeAnalysisNodeNamesJson = reinterpret_cast<WriteAnalysisNodeNamesJson>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_write_analysis_node_names_json"));
  graphInfoJsonLen = reinterpret_cast<GraphInfoJsonLen>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_graph_info_json_len"));
  writeGraphInfoJson = reinterpret_cast<WriteGraphInfoJson>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_write_graph_info_json"));
  drainEvents = reinterpret_cast<DrainEvents>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_drain_events"));
  drainTeamEvents = reinterpret_cast<DrainTeamEvents>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_drain_team_events"));
  drainGoalContextEvents = reinterpret_cast<DrainGoalContextEvents>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_drain_goal_context_events"));
  replayAnnotationsCreate = reinterpret_cast<ReplayAnnotationsCreate>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_replay_annotations_create"));
  replayAnnotationsDestroy = reinterpret_cast<ReplayAnnotationsDestroy>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_replay_annotations_destroy"));
  replayAnnotationCount = reinterpret_cast<ReplayAnnotationCount>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_replay_annotation_count"));
  pollReplayAnnotations = reinterpret_cast<PollReplayAnnotations>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_poll_replay_annotations"));

  if (!engineCreate || !engineDestroy || !engineReset || !engineFinish || !processFrame ||
      !eventsJsonLen || !writeEventsJson || !frameJsonLen || !writeFrameJson ||
      !timelineJsonLen || !writeTimelineJson || !statsJsonLen || !writeStatsJson ||
      !statsModuleJsonLen || !writeStatsModuleJson || !statsModuleFrameJsonLen ||
      !writeStatsModuleFrameJson || !statsModuleConfigJsonLen ||
      !writeStatsModuleConfigJson || !graphOutputJsonLen || !writeGraphOutputJson ||
      !analysisNodeJsonLen || !writeAnalysisNodeJson || !analysisNodeNamesJsonLen ||
      !writeAnalysisNodeNamesJson || !graphInfoJsonLen || !writeGraphInfoJson ||
      !drainEvents || !drainTeamEvents || !drainGoalContextEvents ||
      !replayAnnotationsCreate || !replayAnnotationsDestroy || !replayAnnotationCount ||
      !pollReplayAnnotations) {
    unloadRustLibrary();
    return false;
  }

  engine = engineCreate();
  return engine != nullptr;
}

void SubtrActorPlugin::unloadRustLibrary() {
  resetReplayAnnotations();
  if (engine && engineFinish) {
    finishAndDrainPendingEvents("plugin unload");
  }
  if (engine && engineDestroy) {
    engineDestroy(engine);
  }
  engine = nullptr;

  if (rustLibrary) {
    FreeLibrary(rustLibrary);
  }
  rustLibrary = nullptr;
  engineCreate = nullptr;
  engineDestroy = nullptr;
  engineReset = nullptr;
  engineFinish = nullptr;
  processFrame = nullptr;
  eventsJsonLen = nullptr;
  writeEventsJson = nullptr;
  frameJsonLen = nullptr;
  writeFrameJson = nullptr;
  timelineJsonLen = nullptr;
  writeTimelineJson = nullptr;
  statsJsonLen = nullptr;
  writeStatsJson = nullptr;
  statsModuleJsonLen = nullptr;
  writeStatsModuleJson = nullptr;
  statsModuleFrameJsonLen = nullptr;
  writeStatsModuleFrameJson = nullptr;
  statsModuleConfigJsonLen = nullptr;
  writeStatsModuleConfigJson = nullptr;
  graphOutputJsonLen = nullptr;
  writeGraphOutputJson = nullptr;
  analysisNodeJsonLen = nullptr;
  writeAnalysisNodeJson = nullptr;
  analysisNodeNamesJsonLen = nullptr;
  writeAnalysisNodeNamesJson = nullptr;
  graphInfoJsonLen = nullptr;
  writeGraphInfoJson = nullptr;
  drainEvents = nullptr;
  drainTeamEvents = nullptr;
  drainGoalContextEvents = nullptr;
  replayAnnotationsCreate = nullptr;
  replayAnnotationsDestroy = nullptr;
  replayAnnotationCount = nullptr;
  pollReplayAnnotations = nullptr;
}

void SubtrActorPlugin::scheduleLiveTick(float delaySeconds) {
  auto cancelled = liveTickCancelled;
  gameWrapper->SetTimeout(
      [this, cancelled](GameWrapper *) {
        if (!cancelled || *cancelled) {
          return;
        }

        tick("");
        const bool replayPolling = replayAnnotationsEnabled() && gameWrapper->IsInReplay();
        const float nextDelay =
            liveProcessingEnabled() ? sampleIntervalSeconds() : replayPolling ? 0.05f : 0.25f;
        scheduleLiveTick(nextDelay);
      },
      delaySeconds);
}

bool SubtrActorPlugin::liveProcessingEnabled() {
  auto enabledCvar = cvarManager->getCvar("subtr_actor_enabled");
  return static_cast<bool>(enabledCvar) && enabledCvar.getBoolValue();
}

bool SubtrActorPlugin::replayAnnotationsEnabled() {
  auto enabledCvar = cvarManager->getCvar("subtr_actor_replay_annotations_enabled");
  return !static_cast<bool>(enabledCvar) || enabledCvar.getBoolValue();
}

float SubtrActorPlugin::sampleIntervalSeconds() {
  auto intervalCvar = cvarManager->getCvar("subtr_actor_sample_interval_ms");
  const float intervalMs =
      std::clamp(static_cast<bool>(intervalCvar) ? intervalCvar.getFloatValue() : 50.0f,
                 1.0f,
                 1000.0f);
  return intervalMs / 1000.0f;
}

bool SubtrActorPlugin::profileTimingEnabled() {
  auto profileCvar = cvarManager->getCvar("subtr_actor_profile_enabled");
  return static_cast<bool>(profileCvar) && profileCvar.getBoolValue();
}

uint64_t SubtrActorPlugin::profileLogEvery() {
  auto logEveryCvar = cvarManager->getCvar("subtr_actor_profile_log_every");
  return static_cast<uint64_t>(
      std::max(1, static_cast<bool>(logEveryCvar) ? logEveryCvar.getIntValue() : 120));
}

void SubtrActorPlugin::recordProfileTiming(
    double samplingMs,
    double processingMs,
    double drainMs) {
  profileSampleCount += 1;
  profileSamplingMs += samplingMs;
  profileProcessingMs += processingMs;
  profileDrainMs += drainMs;

  const uint64_t logEvery = profileLogEvery();
  if (profileSampleCount < logEvery) {
    return;
  }

  const double divisor = static_cast<double>(profileSampleCount);
  cvarManager->log(std::format(
      "subtr-actor: live profile over {} samples: sample={:.3f}ms process={:.3f}ms "
      "drain={:.3f}ms total={:.3f}ms",
      profileSampleCount,
      profileSamplingMs / divisor,
      profileProcessingMs / divisor,
      profileDrainMs / divisor,
      (profileSamplingMs + profileProcessingMs + profileDrainMs) / divisor));
  resetProfileTiming();
}

void SubtrActorPlugin::resetProfileTiming() {
  profileSampleCount = 0;
  profileSamplingMs = 0.0;
  profileProcessingMs = 0.0;
  profileDrainMs = 0.0;
}

void SubtrActorPlugin::resetReplayAnnotations() {
  if (replayAnnotations && replayAnnotationsDestroy) {
    replayAnnotationsDestroy(replayAnnotations);
  }
  replayAnnotations = nullptr;
  replayAnnotationPath.clear();
  replayAnnotationLoadFailed = false;
}

std::optional<std::string> SubtrActorPlugin::currentReplayPath(ReplayServerWrapper replayServer) {
  if (replayServer.IsNull()) {
    return std::nullopt;
  }
  ReplayWrapper replay = replayServer.GetReplay();
  if (replay.IsNull()) {
    return std::nullopt;
  }
  std::string replayPath = replay.GetFilePath().ToString();
  if (replayPath.empty()) {
    return std::nullopt;
  }

  if (isAbsoluteWindowsPath(replayPath)) {
    return normalizedReplayPathString(std::filesystem::path(replayPath));
  }

  if (const auto path = existingReplayPathCandidate(std::filesystem::path(replayPath))) {
    return path->string();
  }

  const char *userProfile = std::getenv("USERPROFILE");
  if (userProfile != nullptr && *userProfile != '\0') {
    const std::filesystem::path rocketLeagueDocuments =
        std::filesystem::path(userProfile) / "Documents" / "My Games" / "Rocket League";
    for (const auto &base : {
             rocketLeagueDocuments / "TAGame" / "Logs",
             rocketLeagueDocuments / "TAGame" / "Cache" / "WebCache",
             rocketLeagueDocuments,
         }) {
      if (const auto path = existingReplayPathCandidate(base / replayPath)) {
        return path->string();
      }
    }
  }

  return replayPath;
}

void SubtrActorPlugin::tickReplayAnnotations() {
  if (!replayAnnotationsEnabled() || !replayAnnotationsCreate || !pollReplayAnnotations) {
    resetReplayAnnotations();
    return;
  }
  if (!gameWrapper->IsInReplay()) {
    resetReplayAnnotations();
    return;
  }

  ReplayServerWrapper replayServer = gameWrapper->GetGameEventAsReplay();
  if (replayServer.IsNull()) {
    return;
  }
  auto replayPath = currentReplayPath(replayServer);
  if (!replayPath) {
    return;
  }
  const std::string rawReplayPath = replayServer.GetReplay().GetFilePath().ToString();

  if (!replayAnnotations && replayAnnotationLoadFailed && replayAnnotationPath == *replayPath) {
    return;
  }
  if (replayAnnotationPath != *replayPath) {
    resetReplayAnnotations();
  }
  if (!replayAnnotations) {
    replayAnnotationPath = *replayPath;
    replayAnnotations = replayAnnotationsCreate(replayAnnotationPath.c_str());
    if (!replayAnnotations) {
      if (!replayAnnotationLoadFailed) {
        cvarManager->log(
            std::format(
                "subtr-actor: failed to process replay annotations for {} (raw path {})",
                *replayPath,
                rawReplayPath));
      }
      replayAnnotationLoadFailed = true;
      return;
    }
    replayAnnotationLoadFailed = false;
    const size_t annotationCount =
        replayAnnotationCount ? replayAnnotationCount(replayAnnotations) : 0;
    cvarManager->log(std::format(
        "subtr-actor: loaded {} replay annotations from normal replay processor for {}",
        annotationCount,
        *replayPath));
  }

  std::array<SaMechanicEvent, 64> replayEvents{};
  const size_t eventCount = pollReplayAnnotations(
      replayAnnotations,
      replayServer.GetReplayTimeElapsed(),
      replayEvents.data(),
      replayEvents.size());
  for (size_t i = 0; i < eventCount; i += 1) {
    pushEventMessage(replayEvents[i]);
  }
}

void SubtrActorPlugin::tick(std::string) {
  if (!loaded || !engine) {
    return;
  }

  tickReplayAnnotations();

  if (!liveProcessingEnabled()) {
    if (wasInGame && engineReset) {
      finishAndDrainPendingEvents("live processing disabled");
      engineReset(engine);
      resetLiveState();
    }
    wasInGame = false;
    clearPendingFrameEvents();
    return;
  }

  if (!gameWrapper->IsInGame()) {
    if (wasInGame && engineReset) {
      finishAndDrainPendingEvents("game exit");
      engineReset(engine);
      resetLiveState();
    }
    wasInGame = false;
    return;
  }
  if (!wasInGame && engineReset) {
    engineReset(engine);
    resetLiveState();
  }
  wasInGame = true;

  ServerWrapper server = gameWrapper->GetGameEventAsServer();
  if (!server.IsNull()) {
    const float now = server.GetSecondsElapsed();
    if (lastProcessedGameTime && now >= *lastProcessedGameTime &&
        now - *lastProcessedGameTime < sampleIntervalSeconds()) {
      return;
    }
    lastProcessedGameTime = now;
  }

  inputTickNumber += 1;
  const auto sampleStarted = std::chrono::steady_clock::now();
  SaLiveFrame frame = sampleFrame();
  const auto processStarted = std::chrono::steady_clock::now();
  const int32_t processResult = processFrame(engine, &frame);
  const auto drainStarted = std::chrono::steady_clock::now();
  if (processResult != 0) {
    cvarManager->log(
        std::format("subtr-actor: live frame processing failed: {}", processResult));
    return;
  }

  commitPendingFrameEvents();
  clearPendingFrameEvents();
  drainPendingEvents();
  const auto drainFinished = std::chrono::steady_clock::now();

  if (profileTimingEnabled()) {
    const double samplingMs =
        std::chrono::duration<double, std::milli>(processStarted - sampleStarted).count();
    const double processingMs =
        std::chrono::duration<double, std::milli>(drainStarted - processStarted).count();
    const double drainMs =
        std::chrono::duration<double, std::milli>(drainFinished - drainStarted).count();
    recordProfileTiming(samplingMs, processingMs, drainMs);
  } else {
    resetProfileTiming();
  }
}

SaLiveFrame SubtrActorPlugin::sampleFrame() {
  ServerWrapper server = gameWrapper->GetGameEventAsServer();
  CarWrapper car = gameWrapper->GetLocalCar();

  const float now = server.IsNull() ? lastTime : server.GetSecondsElapsed();
  const float dt = frameNumber == 0 ? 0.0f : std::max(0.0f, now - lastTime);
  lastTime = now;

  samplePlayers(server, car);

  SaLiveFrame frame{};
  frame.frame_number = frameNumber++;
  frame.time = now;
  frame.dt = dt;
  frame.live_play = 0;
  frame.has_live_play = 0;
  frame.ball_has_been_hit =
      server.IsNull() ? 1 : static_cast<uint8_t>(server.GetbBallHasBeenHit() != 0);
  frame.has_ball_has_been_hit = 1;
  frame.players = sampledPlayers.empty() ? nullptr : sampledPlayers.data();
  frame.player_count = sampledPlayers.size();
  if (!server.IsNull()) {
    frame.seconds_remaining = server.GetSecondsRemaining();
    frame.has_seconds_remaining = 1;
    frame.kickoff_countdown_time = server.GetReplicatedGameStateTimeRemaining();
    frame.has_kickoff_countdown_time = 1;
    if (server.GetbPlayReplays() != 0) {
      frame.game_state = GAME_STATE_GOAL_SCORED_REPLAY;
      frame.has_game_state = 1;
    } else if (frame.kickoff_countdown_time > 0) {
      frame.game_state = GAME_STATE_KICKOFF_COUNTDOWN;
      frame.has_game_state = 1;
    }
    sampleTeamScores(server, frame);
    rememberTeamScores(frame);
    const unsigned char scoredOnTeam = server.GetReplicatedScoredOnTeam();
    if (scoredOnTeam == 0 || scoredOnTeam == 1) {
      frame.scored_on_team_is_team_0 = scoredOnTeam == 0 ? 1 : 0;
      frame.has_scored_on_team = 1;
    }
  }

  if (!server.IsNull()) {
    BallWrapper ball = server.GetBall();
    if (!ball.IsNull()) {
      frame.has_ball = 1;
      frame.ball = sampleRigidBody(ball);
      const unsigned char hitTeam = ball.GetHitTeamNum();
      if (hitTeam == 0 || hitTeam == 1) {
        frame.possession_team_is_team_0 = hitTeam == 0 ? 1 : 0;
        frame.has_possession_team = 1;
      }
    }
  }

  attachPendingFrameEvents(frame);
  return frame;
}

void SubtrActorPlugin::samplePlayers(ServerWrapper server, CarWrapper localCar) {
  sampledPlayers.clear();
  sampledPlayerNames.clear();
  carPlayerIndices.clear();
  priPlayerIndices.clear();

  if (!server.IsNull()) {
    ArrayWrapper<CarWrapper> cars = server.GetCars();
    ArrayWrapper<PriWrapper> pris = server.GetPRIs();
    const int carCount = cars.IsNull() ? 0 : cars.Count();
    const int priCount = pris.IsNull() ? 0 : pris.Count();
    const auto reserveCount = static_cast<size_t>(std::max(0, carCount + priCount));
    sampledPlayers.reserve(reserveCount);
    sampledPlayerNames.reserve(reserveCount);

    if (!cars.IsNull()) {
      for (int i = 0; i < carCount; i += 1) {
        CarWrapper car = cars.Get(i);
        if (!car.IsNull()) {
          sampledPlayers.push_back(samplePlayer(car, static_cast<uint32_t>(i)));
        }
      }
    }

    if (!pris.IsNull()) {
      for (int i = 0; i < priCount; i += 1) {
        PriWrapper pri = pris.Get(i);
        if (pri.IsNull() || priPlayerIndices.find(pri.memory_address) != priPlayerIndices.end()) {
          continue;
        }
        sampledPlayers.push_back(
            samplePlayer(pri, static_cast<uint32_t>(sampledPlayers.size())));
      }
    }
  }

  if (sampledPlayers.empty() && !localCar.IsNull()) {
    sampledPlayers.reserve(1);
    sampledPlayerNames.reserve(1);
    sampledPlayers.push_back(samplePlayer(localCar, 0));
  }
}

SaRigidBody SubtrActorPlugin::sampleRigidBody(ActorWrapper actor) {
  SaRigidBody body{};
  if (actor.IsNull()) {
    body.sleeping = 1;
    return body;
  }

  body.location = toSaVec3(actor.GetLocation());
  body.rotation = rotatorToQuat(actor.GetRotation());
  body.linear_velocity = toSaVec3(actor.GetVelocity());
  body.angular_velocity = toSaVec3(actor.GetAngularVelocity());
  body.has_linear_velocity = 1;
  body.has_angular_velocity = 1;
  body.sleeping = 0;
  return body;
}

SaPlayerFrame SubtrActorPlugin::samplePlayer(PriWrapper pri, uint32_t playerIndex) {
  SaPlayerFrame player{};
  player.player_index = playerIndex;
  player.is_team_0 = 1;
  populatePlayerFromPri(player, pri, playerIndex);
  return player;
}

void SubtrActorPlugin::populatePlayerFromPri(
    SaPlayerFrame &player,
    PriWrapper pri,
    uint32_t fallbackIndex) {
  if (pri.IsNull()) {
    return;
  }

  const uint32_t playerIndex = stablePlayerIndexForPri(pri, fallbackIndex);
  player.player_index = playerIndex;
  sampledPlayerNames.push_back(pri.GetPlayerName().ToString());
  player.player_name = sampledPlayerNames.back().c_str();
  player.is_team_0 = pri.GetTeamNum() == 0 ? 1 : 0;
  player.has_match_stats = 1;
  player.match_goals = pri.GetMatchGoals();
  player.match_assists = pri.GetMatchAssists();
  player.match_saves = pri.GetMatchSaves();
  player.match_shots = pri.GetMatchShots();
  player.match_score = pri.GetMatchScore();
  priPlayerIndices[pri.memory_address] = playerIndex;
  recordPlayerStatDeltas(pri, playerIndex, player.is_team_0);
}

SaPlayerFrame SubtrActorPlugin::samplePlayer(CarWrapper car, uint32_t playerIndex) {
  SaPlayerFrame player{};
  player.player_index = playerIndex;
  player.is_team_0 = 1;
  if (car.IsNull()) {
    player.has_rigid_body = 0;
    return player;
  }

  PriWrapper pri = car.GetPRI();
  if (!pri.IsNull()) {
    populatePlayerFromPri(player, pri, playerIndex);
    playerIndex = player.player_index;
  } else {
    nextPlayerIndex = std::max(nextPlayerIndex, playerIndex + 1);
  }
  carPlayerIndices[car.memory_address] = playerIndex;
  recordDodgeRefreshFromJumpState(car, playerIndex, player.is_team_0);

  player.has_rigid_body = 1;
  player.rigid_body = sampleRigidBody(car);
  player.jump_active = car.GetbJumped() != 0;
  player.double_jump_active = car.GetbDoubleJumped() != 0;
  player.dodge_active =
      car.GetDodgeComponent().IsNull() ? 0 : car.GetDodgeComponent().GetbActive();
  player.powerslide_active = car.GetbReplicatedHandbrake() != 0;

  BoostWrapper boost = car.GetBoostComponent();
  if (!boost.IsNull()) {
    const auto previousBoost = lastBoostAmounts.find(playerIndex);
    player.boost_amount = static_cast<float>(boost.GetReplicatedBoostAmount());
    player.last_boost_amount =
        previousBoost == lastBoostAmounts.end() ? player.boost_amount : previousBoost->second;
    player.boost_active = player.boost_amount < player.last_boost_amount ? 1 : 0;
    lastBoostAmounts[playerIndex] = player.boost_amount;
  }

  return player;
}

void SubtrActorPlugin::resetLiveState() {
  frameNumber = 0;
  inputTickNumber = 0;
  lastTime = 0.0f;
  lastProcessedGameTime.reset();
  resetProfileTiming();
  sampledPlayers.clear();
  sampledPlayerNames.clear();
  clearPendingFrameEvents();
  lastBoostAmounts.clear();
  carPlayerIndices.clear();
  priPlayerIndices.clear();
  uniqueIdPlayerIndices.clear();
  stablePriPlayerIndices.clear();
  lastPlayerStats.clear();
  suppressedPlayerStatDeltas.clear();
  lastDoubleJumped.clear();
  lastCanJump.clear();
  lastBallTouchFrames.clear();
  dodgeRefreshCounters.clear();
  boostPadIds.clear();
  boostPadSequences.clear();
  lastTeamScores.reset();
  lastGoalEvent.reset();
  lastTouch.reset();
  nextPlayerIndex = 0;
  nextBoostPadId = 1;
  messages.clear();
}

void SubtrActorPlugin::clearPendingFrameEvents() {
  pendingTouches.clear();
  pendingDodgeRefreshes.clear();
  pendingBoostPadEvents.clear();
  pendingGoals.clear();
  pendingPlayerStatEvents.clear();
  pendingDemolishes.clear();
}

void SubtrActorPlugin::commitPendingFrameEvents() {
  if (!pendingGoals.empty()) {
    lastGoalEvent = pendingGoals.back();
  }
}

void SubtrActorPlugin::attachPendingFrameEvents(SaLiveFrame &frame) {
  frame.touches = pendingTouches.empty() ? nullptr : pendingTouches.data();
  frame.touch_count = pendingTouches.size();
  frame.dodge_refreshes = pendingDodgeRefreshes.empty() ? nullptr : pendingDodgeRefreshes.data();
  frame.dodge_refresh_count = pendingDodgeRefreshes.size();
  frame.boost_pad_events =
      pendingBoostPadEvents.empty() ? nullptr : pendingBoostPadEvents.data();
  frame.boost_pad_event_count = pendingBoostPadEvents.size();
  frame.goals = pendingGoals.empty() ? nullptr : pendingGoals.data();
  frame.goal_count = pendingGoals.size();
  frame.player_stat_events =
      pendingPlayerStatEvents.empty() ? nullptr : pendingPlayerStatEvents.data();
  frame.player_stat_event_count = pendingPlayerStatEvents.size();
  frame.demolishes = pendingDemolishes.empty() ? nullptr : pendingDemolishes.data();
  frame.demolish_count = pendingDemolishes.size();
}

SaEventTiming SubtrActorPlugin::currentEventTiming() {
  SaEventTiming timing{};
  timing.frame_number = frameNumber;
  ServerWrapper server = gameWrapper->GetGameEventAsServer();
  timing.time = server.IsNull() ? lastTime : server.GetSecondsElapsed();
  if (!server.IsNull()) {
    timing.seconds_remaining = server.GetSecondsRemaining();
    timing.has_seconds_remaining = 1;
  }
  timing.has_timing = 1;
  return timing;
}

void SubtrActorPlugin::recordTouch(CarWrapper car) {
  if (car.IsNull()) {
    return;
  }

  SaTouchEvent event{};
  event.timing = currentEventTiming();
  if (auto playerIndex = playerIndexForCar(car)) {
    event.player_index = *playerIndex;
    event.has_player = 1;
  }

  bool hasHitTeam = false;
  ServerWrapper server = gameWrapper->GetGameEventAsServer();
  if (!server.IsNull()) {
    BallWrapper ball = server.GetBall();
    if (!ball.IsNull()) {
      const unsigned char hitTeam = ball.GetHitTeamNum();
      if (hitTeam == 0 || hitTeam == 1) {
        event.is_team_0 = hitTeam == 0 ? 1 : 0;
        hasHitTeam = true;
      }
      event.closest_approach_distance = (ball.GetLocation() - car.GetLocation()).magnitude();
      event.has_closest_approach_distance = 1;
    }
  }
  if (!hasHitTeam) {
    PriWrapper pri = car.GetPRI();
    event.is_team_0 = pri.IsNull() || pri.GetTeamNum() == 0 ? 1 : 0;
  }
  if (event.has_player != 0) {
    lastTouch = TouchAttribution{
        event.player_index,
        event.is_team_0,
    };
    lastBallTouchFrames[event.player_index] = frameNumber;
  }
  pendingTouches.push_back(event);
}

void SubtrActorPlugin::recordDodgeRefreshFromJumpState(
    CarWrapper car,
    uint32_t playerIndex,
    uint8_t isTeam0) {
  if (car.IsNull()) {
    return;
  }

  const bool canJump = car.GetbCanJump() != 0;
  const bool onGround = car.GetbOnGround() != 0 || car.IsOnGround();
  lastDoubleJumped[playerIndex] = car.GetbDoubleJumped() != 0;
  const auto previousCanJump = lastCanJump.find(playerIndex);
  const bool canJumpWasKnown = previousCanJump != lastCanJump.end();
  const bool regainedJump = canJumpWasKnown && !previousCanJump->second && canJump;
  lastCanJump[playerIndex] = canJump;

  const auto touchFrame = lastBallTouchFrames.find(playerIndex);
  const bool recentlyTouchedBall =
      touchFrame != lastBallTouchFrames.end() &&
      frameNumber >= touchFrame->second &&
      frameNumber - touchFrame->second <= DODGE_REFRESH_TOUCH_FRAME_WINDOW;
  if (!regainedJump || onGround || !recentlyTouchedBall) {
    return;
  }

  SaDodgeRefreshedEvent event{};
  event.timing = currentEventTiming();
  event.player_index = playerIndex;
  event.is_team_0 = isTeam0;
  event.counter_value = ++dodgeRefreshCounters[playerIndex];
  pendingDodgeRefreshes.push_back(event);
}

void SubtrActorPlugin::recordBoostPadEvent(ActorWrapper pickup, SaBoostPadEventKind kind) {
  if (pickup.IsNull()) {
    return;
  }

  SaBoostPadEvent event{};
  event.timing = currentEventTiming();
  event.pad_id = boostPadId(pickup);
  event.kind = kind;
  if (kind == SaBoostPadEventKindPickedUp) {
    event.sequence = ++boostPadSequences[pickup.memory_address];
    if (auto playerIndex = playerIndexForNearestCar(pickup, BOOST_PICKUP_ATTRIBUTION_RADIUS)) {
      event.player_index = *playerIndex;
      event.has_player = 1;
    }
  }
  pendingBoostPadEvents.push_back(event);
}

void SubtrActorPlugin::recordGoal(
    ServerWrapper server,
    GoalWrapper goal,
    int scoreIndex,
    int assistIndex) {
  SaGoalEvent event{};
  event.timing = currentEventTiming();
  sampleTeamScores(server, event);
  if (auto scoringTeam = scoringTeamFromScoreDelta(event)) {
    event.scoring_team_is_team_0 = *scoringTeam ? 1 : 0;
  } else if (!goal.IsNull()) {
    event.scoring_team_is_team_0 = goal.GetTeamNum() == 0 ? 0 : 1;
  } else if (!server.IsNull()) {
    const unsigned char scoredOnTeam = server.GetReplicatedScoredOnTeam();
    if (scoredOnTeam == 0 || scoredOnTeam == 1) {
      event.scoring_team_is_team_0 = scoredOnTeam == 0 ? 0 : 1;
    }
  }
  if (auto scorerIndex = playerIndexForScoreIndex(server, scoreIndex)) {
    event.player_index = *scorerIndex;
    event.has_player = 1;
  } else if (lastTouch && lastTouch->is_team_0 == event.scoring_team_is_team_0) {
    event.player_index = lastTouch->player_index;
    event.has_player = 1;
  }
  if (goalEventIsDuplicate(event)) {
    return;
  }
  rememberTeamScores(event);
  pendingGoals.push_back(event);

  recordExplicitPlayerStat(priForScoreIndex(server, assistIndex), SaPlayerStatEventKindAssist);
}

void SubtrActorPlugin::recordDemolish(CarWrapper victim, ActorWrapper demolisher) {
  if (victim.IsNull() || demolisher.IsNull()) {
    return;
  }

  CarWrapper attacker(demolisher.memory_address);
  const auto victimIndex = playerIndexForCar(victim);
  const auto attackerIndex = playerIndexForCar(attacker);
  if (!victimIndex || !attackerIndex) {
    return;
  }

  SaDemolishEvent event{};
  event.timing = currentEventTiming();
  event.attacker_index = *attackerIndex;
  event.victim_index = *victimIndex;
  event.attacker_velocity = toSaVec3(attacker.GetVelocity());
  event.victim_velocity = toSaVec3(victim.GetVelocity());
  event.victim_location = toSaVec3(victim.GetLocation());
  event.active_duration_seconds = DEMO_ACTIVE_DURATION_SECONDS;
  pendingDemolishes.push_back(event);
}

void SubtrActorPlugin::recordPlayerStatDeltas(
    PriWrapper pri,
    uint32_t playerIndex,
    uint8_t isTeam0) {
  if (pri.IsNull()) {
    return;
  }

  const PlayerStatSnapshot current{
      pri.GetMatchShots(),
      pri.GetMatchSaves(),
      pri.GetMatchAssists(),
      pri.GetMatchDemolishes(),
  };
  auto [it, inserted] = lastPlayerStats.emplace(pri.memory_address, current);
  if (inserted) {
    return;
  }

  auto suppressions = suppressedPlayerStatDeltas.find(pri.memory_address);
  auto consumeSuppressed = [&](int count, int PlayerStatSnapshot::*field) {
    if (count <= 0 || suppressions == suppressedPlayerStatDeltas.end()) {
      return count;
    }

    int &suppressed = suppressions->second.*field;
    const int consumed = std::min(count, suppressed);
    suppressed -= consumed;
    return count - consumed;
  };
  auto pushStats = [&](int previous, int next, SaPlayerStatEventKind kind, int PlayerStatSnapshot::*field) {
    const int count = consumeSuppressed(next - previous, field);
    for (int i = 0; i < count; i += 1) {
      SaPlayerStatEvent event{};
      event.timing = currentEventTiming();
      event.player_index = playerIndex;
      event.is_team_0 = isTeam0;
      event.kind = kind;
      if (kind == SaPlayerStatEventKindShot) {
        ServerWrapper server = gameWrapper->GetGameEventAsServer();
        if (!server.IsNull()) {
          BallWrapper ball = server.GetBall();
          if (!ball.IsNull()) {
            event.has_shot_ball = 1;
            event.shot_ball = sampleRigidBody(ball);
          }
        }

        CarWrapper car = pri.GetCar();
        if (!car.IsNull()) {
          event.has_shot_player = 1;
          event.shot_player = sampleRigidBody(car);
        }
      }
      pendingPlayerStatEvents.push_back(event);
    }
  };
  pushStats(it->second.shots, current.shots, SaPlayerStatEventKindShot, &PlayerStatSnapshot::shots);
  pushStats(it->second.saves, current.saves, SaPlayerStatEventKindSave, &PlayerStatSnapshot::saves);
  pushStats(
      it->second.assists,
      current.assists,
      SaPlayerStatEventKindAssist,
      &PlayerStatSnapshot::assists);
  it->second = current;
  if (suppressions != suppressedPlayerStatDeltas.end() &&
      suppressions->second.shots == 0 &&
      suppressions->second.saves == 0 &&
      suppressions->second.assists == 0 &&
      suppressions->second.demolishes == 0) {
    suppressedPlayerStatDeltas.erase(suppressions);
  }
}

void SubtrActorPlugin::recordExplicitPlayerStat(PriWrapper pri, SaPlayerStatEventKind kind) {
  if (pri.IsNull()) {
    return;
  }

  const auto playerIndex = playerIndexForPri(pri);
  if (!playerIndex) {
    return;
  }

  SaPlayerStatEvent event{};
  event.timing = currentEventTiming();
  event.player_index = *playerIndex;
  event.is_team_0 = pri.GetTeamNum() == 0 ? 1 : 0;
  event.kind = kind;
  pendingPlayerStatEvents.push_back(event);

  if (lastPlayerStats.find(pri.memory_address) == lastPlayerStats.end()) {
    return;
  }

  PlayerStatSnapshot &suppressed = suppressedPlayerStatDeltas[pri.memory_address];
  if (kind == SaPlayerStatEventKindShot) {
    suppressed.shots += 1;
  } else if (kind == SaPlayerStatEventKindSave) {
    suppressed.saves += 1;
  } else if (kind == SaPlayerStatEventKindAssist) {
    suppressed.assists += 1;
  }
}

std::optional<uint32_t> SubtrActorPlugin::playerIndexForCar(CarWrapper car) {
  if (car.IsNull()) {
    return std::nullopt;
  }

  const auto carMatch = carPlayerIndices.find(car.memory_address);
  if (carMatch != carPlayerIndices.end()) {
    return carMatch->second;
  }

  PriWrapper pri = car.GetPRI();
  if (!pri.IsNull()) {
    return playerIndexForPri(pri);
  }
  return std::nullopt;
}

std::optional<uint32_t> SubtrActorPlugin::playerIndexForPri(PriWrapper pri) {
  if (pri.IsNull()) {
    return std::nullopt;
  }

  const auto priMatch = priPlayerIndices.find(pri.memory_address);
  if (priMatch != priPlayerIndices.end()) {
    return priMatch->second;
  }

  const uint32_t playerIndex = stablePlayerIndexForPri(pri, nextPlayerIndex);
  priPlayerIndices[pri.memory_address] = playerIndex;
  return playerIndex;
}

PriWrapper SubtrActorPlugin::priForScoreIndex(ServerWrapper server, int scoreIndex) {
  if (server.IsNull() || scoreIndex < 0) {
    return PriWrapper(0);
  }

  ArrayWrapper<PriWrapper> pris = server.GetPRIs();
  if (pris.IsNull() || scoreIndex >= pris.Count()) {
    return PriWrapper(0);
  }

  return pris.Get(scoreIndex);
}

std::optional<uint32_t> SubtrActorPlugin::playerIndexForScoreIndex(
    ServerWrapper server,
    int scoreIndex) {
  return playerIndexForPri(priForScoreIndex(server, scoreIndex));
}

std::optional<uint32_t> SubtrActorPlugin::playerIndexForNearestCar(
    ActorWrapper actor,
    float maxDistance) {
  if (actor.IsNull()) {
    return std::nullopt;
  }

  ServerWrapper server = gameWrapper->GetGameEventAsServer();
  if (server.IsNull()) {
    return std::nullopt;
  }

  ArrayWrapper<CarWrapper> cars = server.GetCars();
  if (cars.IsNull()) {
    return std::nullopt;
  }

  const Vector actorLocation = actor.GetLocation();
  const int carCount = cars.Count();
  std::optional<uint32_t> bestIndex;
  float bestDistance = maxDistance;
  for (int i = 0; i < carCount; i += 1) {
    CarWrapper car = cars.Get(i);
    if (car.IsNull()) {
      continue;
    }

    const float distance = (car.GetLocation() - actorLocation).magnitude();
    if (distance <= bestDistance) {
      if (auto playerIndex = playerIndexForCar(car)) {
        bestDistance = distance;
        bestIndex = *playerIndex;
      }
    }
  }

  return bestIndex;
}

uint32_t SubtrActorPlugin::stablePlayerIndexForPri(PriWrapper pri, uint32_t fallbackIndex) {
  if (pri.IsNull()) {
    return fallbackIndex;
  }

  const std::string uniqueId = pri.GetbBot() != 0 ? "" : pri.GetUniqueIdWrapper().GetIdString();
  if (!uniqueId.empty()) {
    const auto existing = uniqueIdPlayerIndices.find(uniqueId);
    if (existing != uniqueIdPlayerIndices.end()) {
      return existing->second;
    }

    const uint32_t playerIndex = nextPlayerIndex++;
    uniqueIdPlayerIndices[uniqueId] = playerIndex;
    return playerIndex;
  }

  const auto existing = stablePriPlayerIndices.find(pri.memory_address);
  if (existing != stablePriPlayerIndices.end()) {
    return existing->second;
  }

  const uint32_t playerIndex = nextPlayerIndex++;
  stablePriPlayerIndices[pri.memory_address] = playerIndex;
  return playerIndex;
}

uint32_t SubtrActorPlugin::boostPadId(ActorWrapper pickup) {
  if (!pickup.IsNull()) {
    if (auto standardPadId = nearestStandardBoostPadId(pickup.GetLocation())) {
      return *standardPadId;
    }
  }

  const uintptr_t pickupAddress = pickup.memory_address;
  const auto existing = boostPadIds.find(pickupAddress);
  if (existing != boostPadIds.end()) {
    return existing->second;
  }

  const uint32_t id = NON_STANDARD_BOOST_PAD_ID_START + nextBoostPadId++;
  boostPadIds[pickupAddress] = id;
  return id;
}

void SubtrActorPlugin::sampleTeamScores(ServerWrapper server, SaLiveFrame &frame) {
  if (server.IsNull()) {
    return;
  }

  ArrayWrapper<TeamWrapper> teams = server.GetTeams();
  if (teams.IsNull()) {
    return;
  }

  const int teamCount = teams.Count();
  for (int i = 0; i < teamCount; i += 1) {
    TeamWrapper team = teams.Get(i);
    if (team.IsNull()) {
      continue;
    }
    const int teamIndex = team.GetTeamIndex();
    if (teamIndex == 0) {
      frame.team_zero_score = team.GetScore();
      frame.has_team_zero_score = 1;
    } else if (teamIndex == 1) {
      frame.team_one_score = team.GetScore();
      frame.has_team_one_score = 1;
    }
  }
}

void SubtrActorPlugin::sampleTeamScores(ServerWrapper server, SaGoalEvent &goal) {
  if (server.IsNull()) {
    return;
  }

  ArrayWrapper<TeamWrapper> teams = server.GetTeams();
  if (teams.IsNull()) {
    return;
  }

  const int teamCount = teams.Count();
  for (int i = 0; i < teamCount; i += 1) {
    TeamWrapper team = teams.Get(i);
    if (team.IsNull()) {
      continue;
    }
    const int teamIndex = team.GetTeamIndex();
    if (teamIndex == 0) {
      goal.team_zero_score = team.GetScore();
      goal.has_team_zero_score = 1;
    } else if (teamIndex == 1) {
      goal.team_one_score = team.GetScore();
      goal.has_team_one_score = 1;
    }
  }
}

std::optional<bool> SubtrActorPlugin::scoringTeamFromScoreDelta(
    const SaGoalEvent &goal) const {
  if (!lastTeamScores || goal.has_team_zero_score == 0 || goal.has_team_one_score == 0) {
    return std::nullopt;
  }

  const bool teamZeroScored = goal.team_zero_score > lastTeamScores->first;
  const bool teamOneScored = goal.team_one_score > lastTeamScores->second;
  if (teamZeroScored == teamOneScored) {
    return std::nullopt;
  }
  return teamZeroScored;
}

void SubtrActorPlugin::rememberTeamScores(const SaLiveFrame &frame) {
  if (frame.has_team_zero_score != 0 && frame.has_team_one_score != 0) {
    lastTeamScores = std::make_pair(frame.team_zero_score, frame.team_one_score);
  }
}

void SubtrActorPlugin::rememberTeamScores(const SaGoalEvent &goal) {
  if (goal.has_team_zero_score != 0 && goal.has_team_one_score != 0) {
    lastTeamScores = std::make_pair(goal.team_zero_score, goal.team_one_score);
  }
}

bool SubtrActorPlugin::goalEventIsDuplicate(const SaGoalEvent &goal) const {
  const SaGoalEvent *previous = nullptr;
  if (!pendingGoals.empty()) {
    previous = &pendingGoals.back();
  } else if (lastGoalEvent) {
    previous = &*lastGoalEvent;
  }
  if (!previous) {
    return false;
  }

  if (goal.has_team_zero_score != 0 && goal.has_team_one_score != 0 &&
      previous->has_team_zero_score != 0 && previous->has_team_one_score != 0) {
    return goal.team_zero_score == previous->team_zero_score &&
           goal.team_one_score == previous->team_one_score;
  }

  return goal.scoring_team_is_team_0 == previous->scoring_team_is_team_0 &&
         std::abs(goal.timing.time - previous->timing.time) <=
             GOAL_EVENT_DEDUPE_WINDOW_SECONDS;
}

bool SubtrActorPlugin::finishAndDrainPendingEvents(std::string_view context) {
  if (!engine || !engineFinish) {
    return false;
  }

  const int32_t finishResult = engineFinish(engine);
  if (finishResult != 0) {
    cvarManager->log(std::format(
        "subtr-actor: live graph finalization failed during {}: {}",
        context,
        finishResult));
    return false;
  }

  drainPendingEvents();
  return true;
}

void SubtrActorPlugin::drainPendingEvents() {
  if (!engine || !drainEvents || !drainTeamEvents || !drainGoalContextEvents) {
    return;
  }

  SaMechanicEvent events[16];
  size_t count = 0;
  do {
    count = drainEvents(engine, events, 16);
    for (size_t i = 0; i < count; i += 1) {
      pushEventMessage(events[i]);
    }
  } while (count == 16);

  SaTeamEvent teamEvents[16];
  do {
    count = drainTeamEvents(engine, teamEvents, 16);
    for (size_t i = 0; i < count; i += 1) {
      pushTeamEventMessage(teamEvents[i]);
    }
  } while (count == 16);

  SaGoalContextEvent goalContextEvents[16];
  do {
    count = drainGoalContextEvents(engine, goalContextEvents, 16);
    for (size_t i = 0; i < count; i += 1) {
      pushGoalContextEventMessage(goalContextEvents[i]);
    }
  } while (count == 16);
}

void SubtrActorPlugin::pushEventMessage(const SaMechanicEvent &event) {
  const bool isBlue = event.is_team_0 != 0;
  const std::string label = event.confidence < 0.999f
                                ? std::format(
                                      "{} ({:.0f}%)",
                                      mechanicLabel(event.kind),
                                      event.confidence * 100.0f)
                                : mechanicLabel(event.kind);
  OverlayMessage message{
      label,
      isBlue ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255},
      std::chrono::steady_clock::now() + std::chrono::seconds(2),
  };
  messages.push_back(message);
}

void SubtrActorPlugin::pushTeamEventMessage(const SaTeamEvent &event) {
  const bool isBlue = event.is_team_0 != 0;
  const std::string label = event.confidence < 0.999f
                                ? std::format(
                                      "{} ({:.0f}%)",
                                      teamEventLabel(event),
                                      event.confidence * 100.0f)
                                : teamEventLabel(event);
  OverlayMessage message{
      label,
      isBlue ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255},
      std::chrono::steady_clock::now() + std::chrono::seconds(2),
  };
  messages.push_back(message);
}

void SubtrActorPlugin::pushGoalContextEventMessage(const SaGoalContextEvent &event) {
  const bool isBlue = event.scoring_team_is_team_0 != 0;
  OverlayMessage message{
      goalContextLabel(event),
      isBlue ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255},
      std::chrono::steady_clock::now() + std::chrono::seconds(2),
  };
  messages.push_back(message);
}

void SubtrActorPlugin::render(CanvasWrapper canvas) {
  auto overlayEnabledCvar = cvarManager->getCvar("subtr_actor_overlay_enabled");
  const bool overlayEnabled =
      !static_cast<bool>(overlayEnabledCvar) || overlayEnabledCvar.getBoolValue();
  auto statusOverlayEnabledCvar = cvarManager->getCvar("subtr_actor_status_overlay_enabled");
  const bool statusOverlayEnabled = !static_cast<bool>(statusOverlayEnabledCvar) ||
                                    statusOverlayEnabledCvar.getBoolValue();

  if (statusOverlayEnabled) {
    const bool processingEnabled = liveProcessingEnabled();
    const bool replayAnnotationActive = replayAnnotationsEnabled() && replayAnnotations != nullptr;
    const bool inGame = gameWrapper->IsInGame();
    const float intervalMs = sampleIntervalSeconds() * 1000.0f;
    const std::string status =
        replayAnnotationActive
            ? std::format(
                  "subtr-actor REPLAY | annotations={}",
                  replayAnnotationCount ? replayAnnotationCount(replayAnnotations) : 0)
            : !processingEnabled
                  ? "subtr-actor OFF"
                  : inGame ? std::format(
                                 "subtr-actor LIVE | frames={} | interval={:.0f}ms",
                                 frameNumber,
                                 intervalMs)
                           : "subtr-actor ON | waiting for game";
    canvas.SetPosition(Vector2{64, 240});
    canvas.SetColor((processingEnabled || replayAnnotationActive)
                        ? LinearColor{80, 255, 150, 255}
                        : LinearColor{180, 180, 180, 255});
    canvas.DrawString(status, 1.0f, 1.0f, true);
  }

  if (!overlayEnabled) {
    return;
  }

  const auto now = std::chrono::steady_clock::now();
  while (!messages.empty() && messages.front().expires_at <= now) {
    messages.pop_front();
  }

  Vector2 position{64, 280};
  for (const OverlayMessage &message : messages) {
    canvas.SetPosition(position);
    canvas.SetColor(message.color);
    canvas.DrawString(message.text, 1.4f, 1.4f, true);
    position.Y += 34;
  }
}
