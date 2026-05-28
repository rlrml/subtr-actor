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

constexpr wchar_t RUST_DLL_NAME[] = L"subtr_actor_bakkesmod.dll";
constexpr char BALL_TOUCH_EVENT[] = "Function TAGame.Ball_TA.OnCarTouch";
constexpr char BOOST_PICKED_UP_EVENT[] = "Function TAGame.VehiclePickup_TA.EventPickedUp";
constexpr char BOOST_SPAWNED_EVENT[] = "Function TAGame.VehiclePickup_TA.EventSpawned";
constexpr char GOAL_SCORED_EVENT[] = "Function TAGame.GameEvent_Soccar_TA.EventGoalScored";
constexpr char CAR_DEMOLISHED_EVENT[] = "Function TAGame.Car_TA.Demolish";
constexpr char GRAPH_OUTPUT_USAGE[] =
    "subtr_actor_dump_graph_output "
    "<events|frame|timeline|stats|analysis_nodes|event_history|graph_info> [finish]";

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
