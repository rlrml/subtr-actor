#include "SubtrActorPlugin.h"

#include <algorithm>
#include <array>
#include <cctype>
#include <cmath>
#include <cstddef>
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
constexpr std::array<const char *, 7> VERIFY_GRAPH_OUTPUTS{
    "events",
    "frame",
    "timeline",
    "stats",
    "analysis_nodes",
    "event_history",
    "graph_info",
};
constexpr char FRAME_EVENTS_STATE_NODE[] = "frame_events_state";
constexpr std::array<const char *, 7> FRAME_EVENTS_STATE_EVENT_FIELDS{
    "active_demos",
    "demo_events",
    "boost_pad_events",
    "touch_events",
    "dodge_refreshed_events",
    "player_stat_events",
    "goal_events",
};
constexpr std::array<const char *, 6> REQUIRED_EVENT_HISTORY_FIELDS{
    "demo_events",
    "boost_pad_events",
    "touch_events",
    "dodge_refreshed_events",
    "player_stat_events",
    "goal_events",
};
constexpr std::array<const char *, 40> GRAPH_EVENT_FIELDS{
    "timeline",
    "mechanics",
    "goal_context",
    "core_player",
    "core_team",
    "possession",
    "pressure",
    "movement",
    "positioning",
    "rotation_player",
    "rotation_team",
    "backboard",
    "ball_carry",
    "ceiling_shot",
    "wall_aerial",
    "wall_aerial_shot",
    "center",
    "double_tap",
    "fifty_fifty",
    "flick",
    "musty_flick",
    "one_timer",
    "pass",
    "pass_last_completed",
    "goal_tags",
    "rush",
    "speed_flip",
    "half_flip",
    "half_volley",
    "wavedash",
    "whiff",
    "dodge_reset",
    "powerslide",
    "boost_pickups",
    "boost_ledger",
    "boost_state",
    "bump",
    "touch",
    "touch_last_touch",
    "touch_ball_movement",
};
constexpr std::array<const char *, 3> REQUIRED_GRAPH_EVENT_FIELDS{
    "timeline",
    "goal_context",
    "boost_pickups",
};
constexpr float BOOST_PICKUP_ATTRIBUTION_RADIUS = 450.0f;
constexpr float STANDARD_BOOST_PAD_MATCH_RADIUS = 900.0f;
constexpr float DEMO_ACTIVE_DURATION_SECONDS = 3.0f;
constexpr float GOAL_EVENT_DEDUPE_WINDOW_SECONDS = 3.0f;
constexpr uint32_t NON_STANDARD_BOOST_PAD_ID_START = 1000;
constexpr uint64_t DODGE_REFRESH_TOUCH_FRAME_WINDOW = 2;
constexpr int GAME_STATE_KICKOFF_COUNTDOWN = 55;
constexpr int GAME_STATE_GOAL_SCORED_REPLAY = 86;

int moduleAnchor = 0;

bool wantsRequiredEventHistory(const std::vector<std::string> &params) {
  return std::find_if(params.begin(), params.end(), [](const std::string &param) {
           return param == "require_event_history" || param == "require-event-history" ||
                  param == "require_events" || param == "require-events";
         }) != params.end();
}

bool wantsRequiredGraphEvents(const std::vector<std::string> &params) {
  return std::find_if(params.begin(), params.end(), [](const std::string &param) {
           return param == "require_graph_events" || param == "require-graph-events" ||
                  param == "require_timeline_events" || param == "require-timeline-events";
         }) != params.end();
}

template <typename Array>
std::vector<std::string> stringVectorFromArray(const Array &values) {
  std::vector<std::string> strings;
  strings.reserve(values.size());
  for (const char *value : values) {
    strings.emplace_back(value);
  }
  return strings;
}

std::vector<std::string> defaultGraphEventFields() {
  return stringVectorFromArray(GRAPH_EVENT_FIELDS);
}

std::vector<std::string> defaultRequiredGraphEventFields() {
  return stringVectorFromArray(REQUIRED_GRAPH_EVENT_FIELDS);
}

std::vector<std::string> defaultEventHistoryFields() {
  return stringVectorFromArray(FRAME_EVENTS_STATE_EVENT_FIELDS);
}

std::vector<std::string> defaultRequiredEventHistoryFields() {
  return stringVectorFromArray(REQUIRED_EVENT_HISTORY_FIELDS);
}

bool containsString(const std::vector<std::string> &values, std::string_view value) {
  return std::find_if(values.begin(), values.end(), [value](const std::string &candidate) {
           return candidate == value;
         }) != values.end();
}

void skipJsonWhitespace(const std::string &json, size_t &offset) {
  while (offset < json.size() &&
         std::isspace(static_cast<unsigned char>(json[offset])) != 0) {
    ++offset;
  }
}

std::optional<std::string> parseJsonString(const std::string &json, size_t &offset) {
  if (offset >= json.size() || json[offset] != '"') {
    return std::nullopt;
  }
  ++offset;
  std::string value;
  while (offset < json.size()) {
    const char ch = json[offset++];
    if (ch == '"') {
      return value;
    }
    if (ch != '\\') {
      value.push_back(ch);
      continue;
    }
    if (offset >= json.size()) {
      return std::nullopt;
    }
    const char escaped = json[offset++];
    switch (escaped) {
    case '"':
    case '\\':
    case '/':
      value.push_back(escaped);
      break;
    case 'b':
      value.push_back('\b');
      break;
    case 'f':
      value.push_back('\f');
      break;
    case 'n':
      value.push_back('\n');
      break;
    case 'r':
      value.push_back('\r');
      break;
    case 't':
      value.push_back('\t');
      break;
    default:
      return std::nullopt;
    }
  }
  return std::nullopt;
}

std::optional<std::vector<std::string>> parseJsonStringArrayValue(
    const std::string &json,
    size_t &offset) {
  std::vector<std::string> values;
  skipJsonWhitespace(json, offset);
  if (offset >= json.size() || json[offset] != '[') {
    return std::nullopt;
  }
  ++offset;
  skipJsonWhitespace(json, offset);
  if (offset < json.size() && json[offset] == ']') {
    ++offset;
    return values;
  }

  while (offset < json.size()) {
    auto value = parseJsonString(json, offset);
    if (!value) {
      return std::nullopt;
    }
    values.push_back(*value);
    skipJsonWhitespace(json, offset);
    if (offset < json.size() && json[offset] == ',') {
      ++offset;
      skipJsonWhitespace(json, offset);
      continue;
    }
    if (offset < json.size() && json[offset] == ']') {
      ++offset;
      return values;
    }
    return std::nullopt;
  }
  return std::nullopt;
}

std::vector<std::string> parseJsonStringArray(const std::string &json) {
  size_t offset = 0;
  auto values = parseJsonStringArrayValue(json, offset);
  if (!values) {
    return {};
  }
  skipJsonWhitespace(json, offset);
  return offset == json.size() ? *values : std::vector<std::string>{};
}

std::vector<std::string> parseJsonStringArrayProperty(
    const std::string &json,
    const std::string &propertyName) {
  const std::string needle = std::format("\"{}\"", propertyName);
  size_t offset = json.find(needle);
  if (offset == std::string::npos) {
    return {};
  }
  offset += needle.size();
  skipJsonWhitespace(json, offset);
  if (offset >= json.size() || json[offset] != ':') {
    return {};
  }
  ++offset;
  auto values = parseJsonStringArrayValue(json, offset);
  return values.value_or(std::vector<std::string>{});
}

bool skipJsonValue(const std::string &json, size_t &offset) {
  skipJsonWhitespace(json, offset);
  if (offset >= json.size()) {
    return false;
  }

  if (json[offset] == '"') {
    return parseJsonString(json, offset).has_value();
  }

  if (json[offset] == '{') {
    ++offset;
    skipJsonWhitespace(json, offset);
    if (offset < json.size() && json[offset] == '}') {
      ++offset;
      return true;
    }
    while (offset < json.size()) {
      if (!parseJsonString(json, offset)) {
        return false;
      }
      skipJsonWhitespace(json, offset);
      if (offset >= json.size() || json[offset] != ':') {
        return false;
      }
      ++offset;
      if (!skipJsonValue(json, offset)) {
        return false;
      }
      skipJsonWhitespace(json, offset);
      if (offset < json.size() && json[offset] == ',') {
        ++offset;
        skipJsonWhitespace(json, offset);
        continue;
      }
      if (offset < json.size() && json[offset] == '}') {
        ++offset;
        return true;
      }
      return false;
    }
    return false;
  }

  if (json[offset] == '[') {
    ++offset;
    skipJsonWhitespace(json, offset);
    if (offset < json.size() && json[offset] == ']') {
      ++offset;
      return true;
    }
    while (offset < json.size()) {
      if (!skipJsonValue(json, offset)) {
        return false;
      }
      skipJsonWhitespace(json, offset);
      if (offset < json.size() && json[offset] == ',') {
        ++offset;
        skipJsonWhitespace(json, offset);
        continue;
      }
      if (offset < json.size() && json[offset] == ']') {
        ++offset;
        return true;
      }
      return false;
    }
    return false;
  }

  if (json.compare(offset, 4, "true") == 0) {
    offset += 4;
    return true;
  }
  if (json.compare(offset, 5, "false") == 0) {
    offset += 5;
    return true;
  }
  if (json.compare(offset, 4, "null") == 0) {
    offset += 4;
    return true;
  }

  const size_t start = offset;
  if (json[offset] == '-') {
    ++offset;
  }
  const size_t integerStart = offset;
  while (offset < json.size() &&
         std::isdigit(static_cast<unsigned char>(json[offset])) != 0) {
    ++offset;
  }
  if (offset == integerStart) {
    return false;
  }
  if (offset < json.size() && json[offset] == '.') {
    ++offset;
    const size_t fractionStart = offset;
    while (offset < json.size() &&
           std::isdigit(static_cast<unsigned char>(json[offset])) != 0) {
      ++offset;
    }
    if (offset == fractionStart) {
      return false;
    }
  }
  if (offset < json.size() && (json[offset] == 'e' || json[offset] == 'E')) {
    ++offset;
    if (offset < json.size() && (json[offset] == '+' || json[offset] == '-')) {
      ++offset;
    }
    const size_t exponentStart = offset;
    while (offset < json.size() &&
           std::isdigit(static_cast<unsigned char>(json[offset])) != 0) {
      ++offset;
    }
    if (offset == exponentStart) {
      return false;
    }
  }
  return offset > start;
}

std::vector<std::string> parseJsonObjectKeys(const std::string &json) {
  std::vector<std::string> keys;
  size_t offset = 0;
  skipJsonWhitespace(json, offset);
  if (offset >= json.size() || json[offset] != '{') {
    return {};
  }
  ++offset;
  skipJsonWhitespace(json, offset);
  if (offset < json.size() && json[offset] == '}') {
    ++offset;
    skipJsonWhitespace(json, offset);
    return offset == json.size() ? keys : std::vector<std::string>{};
  }

  while (offset < json.size()) {
    auto key = parseJsonString(json, offset);
    if (!key) {
      return {};
    }
    skipJsonWhitespace(json, offset);
    if (offset >= json.size() || json[offset] != ':') {
      return {};
    }
    ++offset;
    if (!skipJsonValue(json, offset)) {
      return {};
    }
    keys.push_back(*key);
    skipJsonWhitespace(json, offset);
    if (offset < json.size() && json[offset] == ',') {
      ++offset;
      skipJsonWhitespace(json, offset);
      continue;
    }
    if (offset < json.size() && json[offset] == '}') {
      ++offset;
      skipJsonWhitespace(json, offset);
      return offset == json.size() ? keys : std::vector<std::string>{};
    }
    return {};
  }
  return {};
}

std::optional<size_t> parseJsonArrayPropertyElementCount(
    const std::string &json,
    const std::string &propertyName) {
  const std::string needle = std::format("\"{}\"", propertyName);
  size_t offset = json.find(needle);
  if (offset == std::string::npos) {
    return std::nullopt;
  }
  offset += needle.size();
  skipJsonWhitespace(json, offset);
  if (offset >= json.size() || json[offset] != ':') {
    return std::nullopt;
  }
  ++offset;
  skipJsonWhitespace(json, offset);
  if (offset >= json.size() || json[offset] != '[') {
    return std::nullopt;
  }
  ++offset;
  skipJsonWhitespace(json, offset);
  if (offset < json.size() && json[offset] == ']') {
    return 0;
  }

  size_t count = 0;
  while (offset < json.size()) {
    if (!skipJsonValue(json, offset)) {
      return std::nullopt;
    }
    count += 1;
    skipJsonWhitespace(json, offset);
    if (offset < json.size() && json[offset] == ',') {
      ++offset;
      skipJsonWhitespace(json, offset);
      continue;
    }
    if (offset < json.size() && json[offset] == ']') {
      return count;
    }
    return std::nullopt;
  }
  return std::nullopt;
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

std::string safeModuleFileStem(std::string moduleName) {
  for (char &ch : moduleName) {
    const unsigned char value = static_cast<unsigned char>(ch);
    if (!std::isalnum(value) && ch != '_' && ch != '-') {
      ch = '_';
    }
  }
  return moduleName.empty() ? "module" : moduleName;
}

SaEventTiming syntheticTiming(uint64_t frameNumber, float time) {
  SaEventTiming timing{};
  timing.frame_number = frameNumber;
  timing.time = time;
  timing.seconds_remaining = 300;
  timing.has_timing = 1;
  timing.has_seconds_remaining = 1;
  return timing;
}

SaRigidBody syntheticRigidBody(
    float x,
    float y,
    float z,
    float vx = 0.0f,
    float vy = 0.0f,
    float vz = 0.0f) {
  SaRigidBody body{};
  body.location = SaVec3{x, y, z};
  body.rotation = SaQuat{0.0f, 0.0f, 0.0f, 1.0f};
  body.linear_velocity = SaVec3{vx, vy, vz};
  body.angular_velocity = SaVec3{0.0f, 0.0f, 0.0f};
  body.has_linear_velocity = 1;
  body.has_angular_velocity = 1;
  body.sleeping = 0;
  return body;
}

SaPlayerFrame syntheticPlayer(
    uint32_t playerIndex,
    const char *name,
    uint8_t isTeam0,
    float x,
    float y,
    float z) {
  SaPlayerFrame player{};
  player.player_index = playerIndex;
  player.player_name = name;
  player.is_team_0 = isTeam0;
  player.has_rigid_body = 1;
  player.rigid_body = syntheticRigidBody(x, y, z);
  player.boost_amount = isTeam0 != 0 ? 72.0f : 41.0f;
  player.last_boost_amount = player.boost_amount;
  player.has_match_stats = 1;
  player.match_goals = isTeam0 != 0 ? 1 : 0;
  player.match_assists = 0;
  player.match_saves = isTeam0 != 0 ? 0 : 1;
  player.match_shots = isTeam0 != 0 ? 1 : 0;
  player.match_score = isTeam0 != 0 ? 100 : 50;
  return player;
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
            std::format("subtr-actor: failed to process replay annotations for {}", *replayPath));
      }
      replayAnnotationLoadFailed = true;
      return;
    }
    replayAnnotationLoadFailed = false;
    const size_t annotationCount =
        replayAnnotationCount ? replayAnnotationCount(replayAnnotations) : 0;
    cvarManager->log(std::format(
        "subtr-actor: loaded {} replay annotations from normal replay processor",
        annotationCount));
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

std::string SubtrActorPlugin::readJsonBuffer(JsonLen len, WriteJson write) {
  if (!engine || !len || !write) {
    return {};
  }

  const size_t byteCount = len(engine);
  if (byteCount == 0) {
    return {};
  }

  std::string buffer(byteCount, '\0');
  const size_t written =
      write(engine, reinterpret_cast<uint8_t *>(buffer.data()), buffer.size());
  buffer.resize(written);
  return buffer;
}

std::string SubtrActorPlugin::readNamedJsonBuffer(
    NamedJsonLen len,
    WriteNamedJson write,
    const std::string &name) {
  if (!engine || !len || !write) {
    return {};
  }

  const size_t byteCount = len(engine, name.c_str());
  if (byteCount == 0) {
    return {};
  }

  std::string buffer(byteCount, '\0');
  const size_t written =
      write(engine, name.c_str(), reinterpret_cast<uint8_t *>(buffer.data()), buffer.size());
  buffer.resize(written);
  return buffer;
}

void SubtrActorPlugin::dumpGraphJson(std::vector<std::string> params) {
  if (!loaded || !engine) {
    cvarManager->log("subtr-actor: graph dump requested before engine was loaded");
    return;
  }

  const bool shouldFinish =
      std::find_if(params.begin(), params.end(), [](const std::string &param) {
        return param == "finish" || param == "finalize";
      }) != params.end();
  if (shouldFinish) {
    if (!engineFinish) {
      cvarManager->log("subtr-actor: graph dump requested finish but finish ABI is unavailable");
      return;
    }
    const int32_t finishResult = engineFinish(engine);
    if (finishResult != 0) {
      cvarManager->log(
          std::format("subtr-actor: graph finish failed before dump: {}", finishResult));
      return;
    }
    drainPendingEvents();
  }

  const std::filesystem::path outputDirectory =
      gameWrapper->GetDataFolder() / "subtr-actor";
  std::error_code error;
  std::filesystem::create_directories(outputDirectory, error);
  if (error) {
    cvarManager->log(
        std::format("subtr-actor: failed to create graph dump directory: {}", error.message()));
    return;
  }

  const std::string eventsJson = readJsonBuffer(eventsJsonLen, writeEventsJson);
  const std::string frameJson = readJsonBuffer(frameJsonLen, writeFrameJson);
  const std::string timelineJson = readJsonBuffer(timelineJsonLen, writeTimelineJson);
  const std::string statsJson = readJsonBuffer(statsJsonLen, writeStatsJson);
  const std::string analysisNodesJson =
      readNamedJsonBuffer(graphOutputJsonLen, writeGraphOutputJson, "analysis_nodes");
  const std::string eventHistoryJson =
      readNamedJsonBuffer(graphOutputJsonLen, writeGraphOutputJson, "event_history");
  const std::string graphInfoJson = readJsonBuffer(graphInfoJsonLen, writeGraphInfoJson);
  const std::filesystem::path eventsPath = outputDirectory / "graph-events.json";
  const std::filesystem::path framePath = outputDirectory / "graph-frame.json";
  const std::filesystem::path timelinePath = outputDirectory / "graph-timeline.json";
  const std::filesystem::path statsPath = outputDirectory / "graph-stats.json";
  const std::filesystem::path analysisNodesPath = outputDirectory / "graph-analysis-nodes.json";
  const std::filesystem::path eventHistoryPath = outputDirectory / "graph-event-history.json";
  const std::filesystem::path graphInfoPath = outputDirectory / "graph-info.json";

  std::ofstream eventsFile(eventsPath, std::ios::binary);
  eventsFile.write(eventsJson.data(), static_cast<std::streamsize>(eventsJson.size()));
  std::ofstream frameFile(framePath, std::ios::binary);
  frameFile.write(frameJson.data(), static_cast<std::streamsize>(frameJson.size()));
  std::ofstream timelineFile(timelinePath, std::ios::binary);
  timelineFile.write(timelineJson.data(), static_cast<std::streamsize>(timelineJson.size()));
  std::ofstream statsFile(statsPath, std::ios::binary);
  statsFile.write(statsJson.data(), static_cast<std::streamsize>(statsJson.size()));
  std::ofstream analysisNodesFile(analysisNodesPath, std::ios::binary);
  analysisNodesFile.write(
      analysisNodesJson.data(), static_cast<std::streamsize>(analysisNodesJson.size()));
  std::ofstream eventHistoryFile(eventHistoryPath, std::ios::binary);
  eventHistoryFile.write(
      eventHistoryJson.data(), static_cast<std::streamsize>(eventHistoryJson.size()));
  std::ofstream graphInfoFile(graphInfoPath, std::ios::binary);
  graphInfoFile.write(graphInfoJson.data(), static_cast<std::streamsize>(graphInfoJson.size()));

  if (!eventsFile || !frameFile || !timelineFile || !statsFile || !analysisNodesFile ||
      !eventHistoryFile || !graphInfoFile) {
    cvarManager->log("subtr-actor: failed to write graph JSON snapshots");
    return;
  }

  cvarManager->log(std::format(
      "subtr-actor: wrote graph JSON snapshots{}: {} ({} bytes), {} ({} bytes), "
      "{} ({} bytes), {} ({} bytes), {} ({} bytes), {} ({} bytes), {} ({} bytes)",
      shouldFinish ? " after finish" : "",
      eventsPath.string(),
      eventsJson.size(),
      framePath.string(),
      frameJson.size(),
      timelinePath.string(),
      timelineJson.size(),
      statsPath.string(),
      statsJson.size(),
      analysisNodesPath.string(),
      analysisNodesJson.size(),
      eventHistoryPath.string(),
      eventHistoryJson.size(),
      graphInfoPath.string(),
      graphInfoJson.size()));
}

void SubtrActorPlugin::dumpStatsModuleJson(std::vector<std::string> params) {
  if (!loaded || !engine) {
    cvarManager->log("subtr-actor: stats module dump requested before engine was loaded");
    return;
  }
  if (params.size() < 2) {
    cvarManager->log("subtr-actor: usage: subtr_actor_dump_stats_module <module_name> [finish]");
    return;
  }

  const std::string moduleName = params[1];
  const bool shouldFinish =
      std::find_if(params.begin() + 2, params.end(), [](const std::string &param) {
        return param == "finish" || param == "finalize";
      }) != params.end();
  if (shouldFinish) {
    if (!engineFinish) {
      cvarManager->log("subtr-actor: stats module dump requested finish but finish ABI is unavailable");
      return;
    }
    const int32_t finishResult = engineFinish(engine);
    if (finishResult != 0) {
      cvarManager->log(
          std::format("subtr-actor: graph finish failed before stats module dump: {}", finishResult));
      return;
    }
    drainPendingEvents();
  }

  const std::string moduleJson =
      readNamedJsonBuffer(statsModuleJsonLen, writeStatsModuleJson, moduleName);
  if (moduleJson.empty()) {
    cvarManager->log(std::format(
        "subtr-actor: stats module '{}' was unavailable or produced empty JSON", moduleName));
    return;
  }

  const std::filesystem::path outputDirectory =
      gameWrapper->GetDataFolder() / "subtr-actor";
  std::error_code error;
  std::filesystem::create_directories(outputDirectory, error);
  if (error) {
    cvarManager->log(std::format(
        "subtr-actor: failed to create stats module dump directory: {}", error.message()));
    return;
  }

  const std::filesystem::path modulePath =
      outputDirectory / std::format("graph-module-{}.json", safeModuleFileStem(moduleName));
  std::ofstream moduleFile(modulePath, std::ios::binary);
  moduleFile.write(moduleJson.data(), static_cast<std::streamsize>(moduleJson.size()));
  if (!moduleFile) {
    cvarManager->log("subtr-actor: failed to write stats module JSON snapshot");
    return;
  }

  cvarManager->log(std::format(
      "subtr-actor: wrote stats module '{}' JSON{}: {} ({} bytes)",
      moduleName,
      shouldFinish ? " after finish" : "",
      modulePath.string(),
      moduleJson.size()));
}

void SubtrActorPlugin::dumpStatsModuleFrameJson(std::vector<std::string> params) {
  if (!loaded || !engine) {
    cvarManager->log("subtr-actor: stats module frame dump requested before engine was loaded");
    return;
  }
  if (params.size() < 2) {
    cvarManager->log(
        "subtr-actor: usage: subtr_actor_dump_stats_module_frame <module_name> [finish]");
    return;
  }

  const std::string moduleName = params[1];
  const bool shouldFinish =
      std::find_if(params.begin() + 2, params.end(), [](const std::string &param) {
        return param == "finish" || param == "finalize";
      }) != params.end();
  if (shouldFinish) {
    if (!engineFinish) {
      cvarManager->log(
          "subtr-actor: stats module frame dump requested finish but finish ABI is unavailable");
      return;
    }
    const int32_t finishResult = engineFinish(engine);
    if (finishResult != 0) {
      cvarManager->log(std::format(
          "subtr-actor: graph finish failed before stats module frame dump: {}",
          finishResult));
      return;
    }
    drainPendingEvents();
  }

  const std::string moduleJson =
      readNamedJsonBuffer(statsModuleFrameJsonLen, writeStatsModuleFrameJson, moduleName);
  if (moduleJson.empty()) {
    cvarManager->log(std::format(
        "subtr-actor: stats module '{}' frame was unavailable or produced empty JSON",
        moduleName));
    return;
  }

  const std::filesystem::path outputDirectory =
      gameWrapper->GetDataFolder() / "subtr-actor";
  std::error_code error;
  std::filesystem::create_directories(outputDirectory, error);
  if (error) {
    cvarManager->log(std::format(
        "subtr-actor: failed to create stats module frame dump directory: {}",
        error.message()));
    return;
  }

  const std::filesystem::path modulePath =
      outputDirectory / std::format("graph-module-frame-{}.json", safeModuleFileStem(moduleName));
  std::ofstream moduleFile(modulePath, std::ios::binary);
  moduleFile.write(moduleJson.data(), static_cast<std::streamsize>(moduleJson.size()));
  if (!moduleFile) {
    cvarManager->log("subtr-actor: failed to write stats module frame JSON snapshot");
    return;
  }

  cvarManager->log(std::format(
      "subtr-actor: wrote stats module '{}' frame JSON{}: {} ({} bytes)",
      moduleName,
      shouldFinish ? " after finish" : "",
      modulePath.string(),
      moduleJson.size()));
}

void SubtrActorPlugin::dumpStatsModuleConfigJson(std::vector<std::string> params) {
  if (!loaded || !engine) {
    cvarManager->log("subtr-actor: stats module config dump requested before engine was loaded");
    return;
  }
  if (params.size() < 2) {
    cvarManager->log(
        "subtr-actor: usage: subtr_actor_dump_stats_module_config <module_name> [finish]");
    return;
  }

  const std::string moduleName = params[1];
  const bool shouldFinish =
      std::find_if(params.begin() + 2, params.end(), [](const std::string &param) {
        return param == "finish" || param == "finalize";
      }) != params.end();
  if (shouldFinish) {
    if (!engineFinish) {
      cvarManager->log(
          "subtr-actor: stats module config dump requested finish but finish ABI is unavailable");
      return;
    }
    const int32_t finishResult = engineFinish(engine);
    if (finishResult != 0) {
      cvarManager->log(std::format(
          "subtr-actor: graph finish failed before stats module config dump: {}",
          finishResult));
      return;
    }
    drainPendingEvents();
  }

  const std::string moduleJson =
      readNamedJsonBuffer(statsModuleConfigJsonLen, writeStatsModuleConfigJson, moduleName);
  if (moduleJson.empty()) {
    cvarManager->log(std::format(
        "subtr-actor: stats module '{}' config was unavailable or produced empty JSON",
        moduleName));
    return;
  }

  const std::filesystem::path outputDirectory =
      gameWrapper->GetDataFolder() / "subtr-actor";
  std::error_code error;
  std::filesystem::create_directories(outputDirectory, error);
  if (error) {
    cvarManager->log(std::format(
        "subtr-actor: failed to create stats module config dump directory: {}",
        error.message()));
    return;
  }

  const std::filesystem::path modulePath =
      outputDirectory / std::format("graph-module-config-{}.json", safeModuleFileStem(moduleName));
  std::ofstream moduleFile(modulePath, std::ios::binary);
  moduleFile.write(moduleJson.data(), static_cast<std::streamsize>(moduleJson.size()));
  if (!moduleFile) {
    cvarManager->log("subtr-actor: failed to write stats module config JSON snapshot");
    return;
  }

  cvarManager->log(std::format(
      "subtr-actor: wrote stats module '{}' config JSON{}: {} ({} bytes)",
      moduleName,
      shouldFinish ? " after finish" : "",
      modulePath.string(),
      moduleJson.size()));
}

void SubtrActorPlugin::dumpGraphOutputJson(std::vector<std::string> params) {
  if (!loaded || !engine) {
    cvarManager->log("subtr-actor: graph output dump requested before engine was loaded");
    return;
  }
  if (params.size() < 2) {
    cvarManager->log(std::format("subtr-actor: usage: {}", GRAPH_OUTPUT_USAGE));
    return;
  }

  const std::string outputName = params[1];
  const bool shouldFinish =
      std::find_if(params.begin() + 2, params.end(), [](const std::string &param) {
        return param == "finish" || param == "finalize";
      }) != params.end();
  if (shouldFinish) {
    if (!engineFinish) {
      cvarManager->log("subtr-actor: graph output dump requested finish but finish ABI is unavailable");
      return;
    }
    const int32_t finishResult = engineFinish(engine);
    if (finishResult != 0) {
      cvarManager->log(
          std::format("subtr-actor: graph finish failed before output dump: {}", finishResult));
      return;
    }
    drainPendingEvents();
  }

  const std::string outputJson =
      readNamedJsonBuffer(graphOutputJsonLen, writeGraphOutputJson, outputName);
  if (outputJson.empty()) {
    cvarManager->log(std::format(
        "subtr-actor: graph output '{}' was unavailable or produced empty JSON", outputName));
    return;
  }

  const std::filesystem::path outputDirectory =
      gameWrapper->GetDataFolder() / "subtr-actor";
  std::error_code error;
  std::filesystem::create_directories(outputDirectory, error);
  if (error) {
    cvarManager->log(std::format(
        "subtr-actor: failed to create graph output dump directory: {}", error.message()));
    return;
  }

  const std::filesystem::path outputPath =
      outputDirectory / std::format("graph-output-{}.json", safeModuleFileStem(outputName));
  std::ofstream outputFile(outputPath, std::ios::binary);
  outputFile.write(outputJson.data(), static_cast<std::streamsize>(outputJson.size()));
  if (!outputFile) {
    cvarManager->log("subtr-actor: failed to write graph output JSON snapshot");
    return;
  }

  cvarManager->log(std::format(
      "subtr-actor: wrote graph output '{}' JSON{}: {} ({} bytes)",
      outputName,
      shouldFinish ? " after finish" : "",
      outputPath.string(),
      outputJson.size()));
}

void SubtrActorPlugin::dumpAnalysisNodeJson(std::vector<std::string> params) {
  if (!loaded || !engine) {
    cvarManager->log("subtr-actor: analysis node dump requested before engine was loaded");
    return;
  }
  if (params.size() < 2) {
    cvarManager->log(
        "subtr-actor: usage: subtr_actor_dump_analysis_node <node_name> [finish]");
    return;
  }

  const std::string nodeName = params[1];
  const bool shouldFinish =
      std::find_if(params.begin() + 2, params.end(), [](const std::string &param) {
        return param == "finish" || param == "finalize";
      }) != params.end();
  if (shouldFinish) {
    if (!engineFinish) {
      cvarManager->log(
          "subtr-actor: analysis node dump requested finish but finish ABI is unavailable");
      return;
    }
    const int32_t finishResult = engineFinish(engine);
    if (finishResult != 0) {
      cvarManager->log(std::format(
          "subtr-actor: graph finish failed before analysis node dump: {}",
          finishResult));
      return;
    }
    drainPendingEvents();
  }

  const std::string nodeJson =
      readNamedJsonBuffer(analysisNodeJsonLen, writeAnalysisNodeJson, nodeName);
  if (nodeJson.empty()) {
    cvarManager->log(std::format(
        "subtr-actor: analysis node '{}' was unavailable or produced empty JSON",
        nodeName));
    return;
  }

  const std::filesystem::path outputDirectory =
      gameWrapper->GetDataFolder() / "subtr-actor";
  std::error_code error;
  std::filesystem::create_directories(outputDirectory, error);
  if (error) {
    cvarManager->log(std::format(
        "subtr-actor: failed to create analysis node dump directory: {}",
        error.message()));
    return;
  }

  const std::filesystem::path nodePath =
      outputDirectory / std::format("graph-node-{}.json", safeModuleFileStem(nodeName));
  std::ofstream nodeFile(nodePath, std::ios::binary);
  nodeFile.write(nodeJson.data(), static_cast<std::streamsize>(nodeJson.size()));
  if (!nodeFile) {
    cvarManager->log("subtr-actor: failed to write analysis node JSON snapshot");
    return;
  }

  cvarManager->log(std::format(
      "subtr-actor: wrote analysis node '{}' JSON{}: {} ({} bytes)",
      nodeName,
      shouldFinish ? " after finish" : "",
      nodePath.string(),
      nodeJson.size()));
}

void SubtrActorPlugin::verifyGraphRuntime(std::vector<std::string> params) {
  if (!loaded || !engine) {
    cvarManager->log("subtr-actor: graph verification requested before engine was loaded");
    return;
  }

  const bool shouldFinish =
      std::find_if(params.begin(), params.end(), [](const std::string &param) {
        return param == "finish" || param == "finalize";
      }) != params.end();
  const bool requireEventHistory = wantsRequiredEventHistory(params);
  const bool requireGraphEvents = wantsRequiredGraphEvents(params);
  if (shouldFinish) {
    if (!engineFinish) {
      cvarManager->log(
          "subtr-actor: graph verification requested finish but finish ABI is unavailable");
      return;
    }
    const int32_t finishResult = engineFinish(engine);
    if (finishResult != 0) {
      cvarManager->log(
          std::format("subtr-actor: graph finish failed before verification: {}", finishResult));
      return;
    }
    drainPendingEvents();
  }

  bool ok = true;
  const std::string graphInfoJson =
      readNamedJsonBuffer(graphOutputJsonLen, writeGraphOutputJson, "graph_info");
  std::vector<std::string> outputNames =
      parseJsonStringArrayProperty(graphInfoJson, "graph_output_names");
  if (outputNames.empty()) {
    if (!graphInfoJson.empty()) {
      ok = false;
      cvarManager->log(
          "subtr-actor: graph verification could not read graph output names from graph_info");
    }
    outputNames.assign(VERIFY_GRAPH_OUTPUTS.begin(), VERIFY_GRAPH_OUTPUTS.end());
  }
  bool missingRequiredGraphOutput = false;
  for (const char *outputName : VERIFY_GRAPH_OUTPUTS) {
    if (!containsString(outputNames, outputName)) {
      ok = false;
      missingRequiredGraphOutput = true;
      cvarManager->log(std::format(
          "subtr-actor: graph verification graph_info missing required graph output '{}'",
          outputName));
    }
  }
  if (!missingRequiredGraphOutput) {
    cvarManager->log(std::format(
        "subtr-actor: graph_info declares all {} required graph outputs",
        VERIFY_GRAPH_OUTPUTS.size()));
  }
  std::vector<std::string> graphEventFieldNames =
      parseJsonStringArrayProperty(graphInfoJson, "graph_event_field_names");
  if (graphEventFieldNames.empty()) {
    if (!graphInfoJson.empty()) {
      ok = false;
      cvarManager->log(
          "subtr-actor: graph verification could not read graph event field names from graph_info");
    }
    graphEventFieldNames = defaultGraphEventFields();
  }
  bool missingKnownGraphEventField = false;
  for (const char *fieldName : GRAPH_EVENT_FIELDS) {
    if (!containsString(graphEventFieldNames, fieldName)) {
      ok = false;
      missingKnownGraphEventField = true;
      cvarManager->log(std::format(
          "subtr-actor: graph verification graph_info missing required graph event field '{}'",
          fieldName));
    }
  }
  if (!missingKnownGraphEventField) {
    cvarManager->log(std::format(
        "subtr-actor: graph_info declares all {} known graph event fields",
        GRAPH_EVENT_FIELDS.size()));
  }
  std::vector<std::string> requiredGraphEventFieldNames =
      parseJsonStringArrayProperty(graphInfoJson, "required_graph_event_field_names");
  if (requiredGraphEventFieldNames.empty()) {
    if (!graphInfoJson.empty()) {
      ok = false;
      cvarManager->log(
          "subtr-actor: graph verification could not read required graph event field names from graph_info");
    }
    requiredGraphEventFieldNames = defaultRequiredGraphEventFields();
  }
  bool missingKnownRequiredGraphEventField = false;
  for (const char *fieldName : REQUIRED_GRAPH_EVENT_FIELDS) {
    if (!containsString(requiredGraphEventFieldNames, fieldName)) {
      ok = false;
      missingKnownRequiredGraphEventField = true;
      cvarManager->log(std::format(
          "subtr-actor: graph verification graph_info missing strict graph event field '{}'",
          fieldName));
    }
  }
  if (!missingKnownRequiredGraphEventField) {
    cvarManager->log(std::format(
        "subtr-actor: graph_info declares all {} strict graph event fields",
        REQUIRED_GRAPH_EVENT_FIELDS.size()));
  }
  bool requiredGraphEventFieldNotDeclared = false;
  for (const std::string &fieldName : requiredGraphEventFieldNames) {
    if (!containsString(graphEventFieldNames, fieldName)) {
      ok = false;
      requiredGraphEventFieldNotDeclared = true;
      cvarManager->log(std::format(
          "subtr-actor: graph verification required graph event field '{}' is not declared",
          fieldName));
    }
  }
  if (!requiredGraphEventFieldNotDeclared) {
    cvarManager->log(std::format(
        "subtr-actor: graph_info declares {} graph event fields and {} required graph event fields",
        graphEventFieldNames.size(),
        requiredGraphEventFieldNames.size()));
  }
  std::vector<std::string> eventHistoryFieldNames =
      parseJsonStringArrayProperty(graphInfoJson, "event_history_field_names");
  if (eventHistoryFieldNames.empty()) {
    if (!graphInfoJson.empty()) {
      ok = false;
      cvarManager->log(
          "subtr-actor: graph verification could not read event_history field names from graph_info");
    }
    eventHistoryFieldNames = defaultEventHistoryFields();
  }
  bool missingKnownEventHistoryField = false;
  for (const char *fieldName : FRAME_EVENTS_STATE_EVENT_FIELDS) {
    if (!containsString(eventHistoryFieldNames, fieldName)) {
      ok = false;
      missingKnownEventHistoryField = true;
      cvarManager->log(std::format(
          "subtr-actor: graph verification graph_info missing required event_history field '{}'",
          fieldName));
    }
  }
  if (!missingKnownEventHistoryField) {
    cvarManager->log(std::format(
        "subtr-actor: graph_info declares all {} known event_history fields",
        FRAME_EVENTS_STATE_EVENT_FIELDS.size()));
  }
  std::vector<std::string> requiredEventHistoryFieldNames =
      parseJsonStringArrayProperty(graphInfoJson, "required_event_history_field_names");
  if (requiredEventHistoryFieldNames.empty()) {
    if (!graphInfoJson.empty()) {
      ok = false;
      cvarManager->log(
          "subtr-actor: graph verification could not read required event_history field names from graph_info");
    }
    requiredEventHistoryFieldNames = defaultRequiredEventHistoryFields();
  }
  bool missingKnownRequiredEventHistoryField = false;
  for (const char *fieldName : REQUIRED_EVENT_HISTORY_FIELDS) {
    if (!containsString(requiredEventHistoryFieldNames, fieldName)) {
      ok = false;
      missingKnownRequiredEventHistoryField = true;
      cvarManager->log(std::format(
          "subtr-actor: graph verification graph_info missing strict cumulative event_history field '{}'",
          fieldName));
    }
  }
  if (!missingKnownRequiredEventHistoryField) {
    cvarManager->log(std::format(
        "subtr-actor: graph_info declares all {} strict cumulative event_history fields",
        REQUIRED_EVENT_HISTORY_FIELDS.size()));
  }
  bool requiredEventHistoryFieldNotDeclared = false;
  for (const std::string &fieldName : requiredEventHistoryFieldNames) {
    if (!containsString(eventHistoryFieldNames, fieldName)) {
      ok = false;
      requiredEventHistoryFieldNotDeclared = true;
      cvarManager->log(std::format(
          "subtr-actor: graph verification required event_history field '{}' is not declared",
          fieldName));
    }
  }
  if (!requiredEventHistoryFieldNotDeclared) {
    cvarManager->log(std::format(
        "subtr-actor: graph_info declares {} event_history fields and {} required cumulative event fields",
        eventHistoryFieldNames.size(),
        requiredEventHistoryFieldNames.size()));
  }

  std::string analysisNodesJson;
  std::string graphEventsJson;
  std::string eventHistoryJson;
  for (const std::string &outputName : outputNames) {
    const std::string outputJson =
        readNamedJsonBuffer(graphOutputJsonLen, writeGraphOutputJson, outputName);
    if (outputJson.empty()) {
      ok = false;
      cvarManager->log(std::format(
          "subtr-actor: graph verification missing graph output '{}'", outputName));
      continue;
    }
    cvarManager->log(std::format(
        "subtr-actor: graph output '{}' callable ({} bytes)",
        outputName,
        outputJson.size()));
    std::string fixedOutputJson;
    if (outputName == "events") {
      fixedOutputJson = readJsonBuffer(eventsJsonLen, writeEventsJson);
    } else if (outputName == "frame") {
      fixedOutputJson = readJsonBuffer(frameJsonLen, writeFrameJson);
    } else if (outputName == "timeline") {
      fixedOutputJson = readJsonBuffer(timelineJsonLen, writeTimelineJson);
    } else if (outputName == "stats") {
      fixedOutputJson = readJsonBuffer(statsJsonLen, writeStatsJson);
    } else if (outputName == "graph_info") {
      fixedOutputJson = readJsonBuffer(graphInfoJsonLen, writeGraphInfoJson);
    }
    if (!fixedOutputJson.empty()) {
      if (fixedOutputJson != outputJson) {
        ok = false;
        cvarManager->log(std::format(
            "subtr-actor: graph verification fixed ABI output '{}' differs from named output",
            outputName));
      } else {
        cvarManager->log(std::format(
            "subtr-actor: graph output '{}' matches fixed ABI",
            outputName));
      }
    }
    if (outputName == "events") {
      graphEventsJson = outputJson;
    } else if (outputName == "analysis_nodes") {
      analysisNodesJson = outputJson;
    } else if (outputName == "event_history") {
      eventHistoryJson = outputJson;
    }
  }
  if (!outputNames.empty()) {
    cvarManager->log(std::format(
        "subtr-actor: verified {} graph outputs by name",
        outputNames.size()));
  }

  std::vector<std::string> graphEventKeys = parseJsonObjectKeys(graphEventsJson);
  if (graphEventKeys.empty()) {
    ok = false;
    cvarManager->log(
        "subtr-actor: graph verification could not inspect events graph output fields");
  } else {
    std::sort(graphEventKeys.begin(), graphEventKeys.end());
    bool missingGraphEventField = false;
    bool missingRequiredGraphEvent = false;
    for (const std::string &fieldName : graphEventFieldNames) {
      if (!std::binary_search(graphEventKeys.begin(), graphEventKeys.end(), fieldName)) {
        ok = false;
        missingGraphEventField = true;
        if (requireGraphEvents && containsString(requiredGraphEventFieldNames, fieldName)) {
          missingRequiredGraphEvent = true;
        }
        cvarManager->log(std::format(
            "subtr-actor: graph verification events output missing graph event field '{}'",
            fieldName));
        continue;
      }
      const auto eventCount = parseJsonArrayPropertyElementCount(graphEventsJson, fieldName);
      if (!eventCount) {
        ok = false;
        if (requireGraphEvents && containsString(requiredGraphEventFieldNames, fieldName)) {
          missingRequiredGraphEvent = true;
        }
        cvarManager->log(std::format(
            "subtr-actor: graph verification events output field '{}' is not an array",
            fieldName));
        continue;
      }
      cvarManager->log(std::format(
          "subtr-actor: events output field '{}' has {} entries",
          fieldName,
          *eventCount));
      if (requireGraphEvents && containsString(requiredGraphEventFieldNames, fieldName) &&
          *eventCount == 0) {
        ok = false;
        missingRequiredGraphEvent = true;
        cvarManager->log(std::format(
            "subtr-actor: graph verification events required graph event field '{}' has no entries",
            fieldName));
      }
    }
    if (!missingGraphEventField) {
      cvarManager->log(std::format(
          "subtr-actor: events output exposes {} graph event fields",
          graphEventFieldNames.size()));
    }
    if (requireGraphEvents && !requiredGraphEventFieldNotDeclared &&
        !missingRequiredGraphEvent && !missingGraphEventField) {
      cvarManager->log(
          "subtr-actor: events required graph event fields are nonzero");
    }
  }

  const std::vector<std::string> moduleNames =
      parseJsonStringArrayProperty(graphInfoJson, "builtin_stats_module_names");
  if (moduleNames.empty()) {
    ok = false;
    cvarManager->log(
        "subtr-actor: graph verification could not read builtin stats module names from graph_info");
  }
  for (const std::string &moduleName : moduleNames) {
    const std::string moduleJson =
        readNamedJsonBuffer(statsModuleJsonLen, writeStatsModuleJson, moduleName);
    const std::string frameJson =
        readNamedJsonBuffer(statsModuleFrameJsonLen, writeStatsModuleFrameJson, moduleName);
    const std::string configJson =
        readNamedJsonBuffer(statsModuleConfigJsonLen, writeStatsModuleConfigJson, moduleName);
    if (moduleJson.empty() || frameJson.empty() || configJson.empty()) {
      ok = false;
      cvarManager->log(std::format(
          "subtr-actor: graph verification missing stats module '{}' output: module={} frame={} config={}",
          moduleName,
          moduleJson.size(),
          frameJson.size(),
          configJson.size()));
      continue;
    }
    cvarManager->log(std::format(
        "subtr-actor: stats module '{}' callable (module={} frame={} config={} bytes)",
        moduleName,
        moduleJson.size(),
        frameJson.size(),
        configJson.size()));
  }
  if (!moduleNames.empty()) {
    cvarManager->log(std::format(
        "subtr-actor: verified {} builtin stats modules by name",
        moduleNames.size()));
  }

  const std::string nodeNamesJson =
      readJsonBuffer(analysisNodeNamesJsonLen, writeAnalysisNodeNamesJson);
  const std::vector<std::string> nodeNames = parseJsonStringArray(nodeNamesJson);
  const std::vector<std::string> graphInfoNodeNames =
      parseJsonStringArrayProperty(graphInfoJson, "callable_analysis_node_names");
  const std::vector<std::string> resolvedGraphNodeNames =
      parseJsonStringArrayProperty(graphInfoJson, "node_names");
  if (nodeNames.empty()) {
    ok = false;
    cvarManager->log(
        "subtr-actor: graph verification could not read callable analysis node names");
  }
  if (graphInfoNodeNames.empty()) {
    ok = false;
    cvarManager->log(
        "subtr-actor: graph verification could not read callable analysis node names from graph_info");
  } else if (!nodeNames.empty() && graphInfoNodeNames != nodeNames) {
    ok = false;
    cvarManager->log(std::format(
        "subtr-actor: graph verification callable analysis node registry mismatch: graph_info={} names_abi={}",
        graphInfoNodeNames.size(),
        nodeNames.size()));
  } else if (!nodeNames.empty()) {
    cvarManager->log(
        "subtr-actor: callable analysis node registry matches graph_info");
  }

  if (resolvedGraphNodeNames.empty()) {
    ok = false;
    cvarManager->log(
        "subtr-actor: graph verification could not read resolved graph node names from graph_info");
  } else if (!nodeNames.empty()) {
    std::vector<std::string> sortedNodeNames = nodeNames;
    std::sort(sortedNodeNames.begin(), sortedNodeNames.end());
    bool missingResolvedNode = false;
    for (const std::string &resolvedNodeName : resolvedGraphNodeNames) {
      if (!std::binary_search(
              sortedNodeNames.begin(), sortedNodeNames.end(), resolvedNodeName)) {
        ok = false;
        missingResolvedNode = true;
        cvarManager->log(std::format(
            "subtr-actor: graph verification resolved node '{}' is not callable by name",
            resolvedNodeName));
      }
    }
    if (!missingResolvedNode) {
      cvarManager->log(std::format(
          "subtr-actor: all {} resolved analysis graph nodes are callable by name",
          resolvedGraphNodeNames.size()));
    }
  }

  if (analysisNodesJson.empty()) {
    ok = false;
    cvarManager->log(
        "subtr-actor: graph verification could not inspect analysis_nodes output");
  } else if (!nodeNames.empty()) {
    std::vector<std::string> analysisNodeKeys = parseJsonObjectKeys(analysisNodesJson);
    if (analysisNodeKeys.empty()) {
      ok = false;
      cvarManager->log(
          "subtr-actor: graph verification could not parse analysis_nodes output keys");
    } else {
      std::vector<std::string> sortedNodeNames = nodeNames;
      std::sort(sortedNodeNames.begin(), sortedNodeNames.end());
      std::sort(analysisNodeKeys.begin(), analysisNodeKeys.end());
      bool nodeSetMismatch = false;
      for (const std::string &nodeName : nodeNames) {
        if (!std::binary_search(analysisNodeKeys.begin(), analysisNodeKeys.end(), nodeName)) {
          ok = false;
          nodeSetMismatch = true;
          cvarManager->log(std::format(
              "subtr-actor: graph verification analysis_nodes output missing callable node '{}'",
              nodeName));
        }
      }
      for (const std::string &nodeName : analysisNodeKeys) {
        if (!std::binary_search(sortedNodeNames.begin(), sortedNodeNames.end(), nodeName)) {
          ok = false;
          nodeSetMismatch = true;
          cvarManager->log(std::format(
              "subtr-actor: graph verification analysis_nodes output has unexpected node '{}'",
              nodeName));
        }
      }
      if (!nodeSetMismatch) {
        cvarManager->log(std::format(
            "subtr-actor: analysis_nodes output contains {} callable analysis nodes exactly",
            nodeNames.size()));
      }
    }
  }

  for (const std::string &nodeName : nodeNames) {
    const std::string nodeJson =
        readNamedJsonBuffer(analysisNodeJsonLen, writeAnalysisNodeJson, nodeName);
    if (nodeJson.empty()) {
      ok = false;
      cvarManager->log(std::format(
          "subtr-actor: graph verification missing analysis node '{}'", nodeName));
      continue;
    }
    cvarManager->log(std::format(
        "subtr-actor: analysis node '{}' callable ({} bytes)",
        nodeName,
        nodeJson.size()));
  }
  if (!nodeNames.empty()) {
    cvarManager->log(std::format(
        "subtr-actor: verified {} callable analysis nodes by name",
        nodeNames.size()));
  }

  const std::string frameEventsJson =
      readNamedJsonBuffer(analysisNodeJsonLen, writeAnalysisNodeJson, FRAME_EVENTS_STATE_NODE);
  std::vector<std::string> frameEventKeys = parseJsonObjectKeys(frameEventsJson);
  if (frameEventKeys.empty()) {
    ok = false;
    cvarManager->log(
        "subtr-actor: graph verification could not inspect frame_events_state event fields");
  } else {
    std::sort(frameEventKeys.begin(), frameEventKeys.end());
    bool missingEventField = false;
    for (const std::string &fieldName : eventHistoryFieldNames) {
      if (!std::binary_search(frameEventKeys.begin(), frameEventKeys.end(), fieldName)) {
        ok = false;
        missingEventField = true;
        cvarManager->log(std::format(
            "subtr-actor: graph verification frame_events_state missing event field '{}'",
            fieldName));
        continue;
      }
      const auto eventCount = parseJsonArrayPropertyElementCount(frameEventsJson, fieldName);
      if (!eventCount) {
        ok = false;
        cvarManager->log(std::format(
            "subtr-actor: graph verification frame_events_state event field '{}' is not an array",
            fieldName));
        continue;
      }
      cvarManager->log(std::format(
          "subtr-actor: frame_events_state event field '{}' has {} entries",
          fieldName,
          *eventCount));
    }
    if (!missingEventField) {
      cvarManager->log(std::format(
          "subtr-actor: frame_events_state exposes {} live event fields",
          eventHistoryFieldNames.size()));
    }
  }

  std::vector<std::string> eventHistoryKeys = parseJsonObjectKeys(eventHistoryJson);
  if (eventHistoryKeys.empty()) {
    ok = false;
    cvarManager->log(
        "subtr-actor: graph verification could not inspect event_history event fields");
  } else {
    std::sort(eventHistoryKeys.begin(), eventHistoryKeys.end());
    bool missingEventHistoryField = false;
    bool missingRequiredEventHistory = false;
    for (const std::string &fieldName : eventHistoryFieldNames) {
      if (!std::binary_search(eventHistoryKeys.begin(), eventHistoryKeys.end(), fieldName)) {
        ok = false;
        missingEventHistoryField = true;
        if (requireEventHistory && containsString(requiredEventHistoryFieldNames, fieldName)) {
          missingRequiredEventHistory = true;
        }
        cvarManager->log(std::format(
            "subtr-actor: graph verification event_history missing event field '{}'",
            fieldName));
        continue;
      }
      const auto eventCount = parseJsonArrayPropertyElementCount(eventHistoryJson, fieldName);
      if (!eventCount) {
        ok = false;
        if (requireEventHistory && containsString(requiredEventHistoryFieldNames, fieldName)) {
          missingRequiredEventHistory = true;
        }
        cvarManager->log(std::format(
            "subtr-actor: graph verification event_history event field '{}' is not an array",
            fieldName));
        continue;
      }
      cvarManager->log(std::format(
          "subtr-actor: event_history event field '{}' has {} cumulative entries",
          fieldName,
          *eventCount));
      if (requireEventHistory && containsString(requiredEventHistoryFieldNames, fieldName) &&
          *eventCount == 0) {
        ok = false;
        missingRequiredEventHistory = true;
        cvarManager->log(std::format(
            "subtr-actor: graph verification event_history required event field '{}' has no cumulative entries",
            fieldName));
      }
    }
    if (!missingEventHistoryField) {
      cvarManager->log(std::format(
          "subtr-actor: event_history exposes {} cumulative live event fields",
          eventHistoryFieldNames.size()));
    }
    if (requireEventHistory && !requiredEventHistoryFieldNotDeclared &&
        !missingRequiredEventHistory && !missingEventHistoryField) {
      cvarManager->log(
          "subtr-actor: event_history required cumulative event fields are nonzero");
    }
  }

  cvarManager->log(ok
                       ? "subtr-actor: graph verification passed"
                       : "subtr-actor: graph verification failed; enter gameplay/replay and try again");
}

void SubtrActorPlugin::selfTestGraphRuntime(std::vector<std::string> params) {
  if (!loaded || !engineCreate || !engineDestroy || !processFrame || !engineFinish ||
      !graphOutputJsonLen || !writeGraphOutputJson) {
    cvarManager->log("subtr-actor: graph self-test requested before ABI was loaded");
    return;
  }
  const bool shouldDump =
      std::find_if(params.begin(), params.end(), [](const std::string &param) {
        return param == "dump" || param == "write_dump" || param == "write-dump";
      }) != params.end();

  SaEngine *selfTestEngine = engineCreate();
  if (!selfTestEngine) {
    cvarManager->log("subtr-actor: graph self-test failed to create temporary engine");
    return;
  }

  std::array<SaPlayerFrame, 2> players{
      syntheticPlayer(0, "self-test-blue", 1, 0.0f, 0.0f, 92.75f),
      syntheticPlayer(1, "self-test-orange", 0, 120.0f, 0.0f, 92.75f),
  };
  std::array<SaTouchEvent, 1> touches{SaTouchEvent{
      syntheticTiming(1, 0.1f),
      0,
      1,
      1,
      12.0f,
      1,
  }};
  std::array<SaDodgeRefreshedEvent, 1> dodgeRefreshes{SaDodgeRefreshedEvent{
      syntheticTiming(1, 0.1f),
      0,
      1,
      1,
  }};
  std::array<SaBoostPadEvent, 1> boostPadEvents{SaBoostPadEvent{
      syntheticTiming(1, 0.1f),
      34,
      SaBoostPadEventKindPickedUp,
      1,
      0,
      1,
  }};
  std::array<SaGoalEvent, 1> goals{SaGoalEvent{
      syntheticTiming(1, 0.1f),
      1,
      0,
      1,
      1,
      1,
      0,
      1,
  }};
  const SaRigidBody shotBall = syntheticRigidBody(300.0f, 100.0f, 120.0f, 1000.0f, 500.0f, 100.0f);
  const SaRigidBody shotPlayer = syntheticRigidBody(240.0f, 90.0f, 92.75f, 800.0f, 300.0f, 0.0f);
  std::array<SaPlayerStatEvent, 3> playerStatEvents{
      SaPlayerStatEvent{
          syntheticTiming(1, 0.1f),
          0,
          1,
          SaPlayerStatEventKindShot,
          1,
          shotBall,
          1,
          shotPlayer,
      },
      SaPlayerStatEvent{
          syntheticTiming(1, 0.1f),
          1,
          0,
          SaPlayerStatEventKindSave,
          0,
          SaRigidBody{},
          0,
          SaRigidBody{},
      },
      SaPlayerStatEvent{
          syntheticTiming(1, 0.1f),
          0,
          1,
          SaPlayerStatEventKindAssist,
          0,
          SaRigidBody{},
          0,
          SaRigidBody{},
      },
  };
  std::array<SaDemolishEvent, 1> demolishes{SaDemolishEvent{
      syntheticTiming(1, 0.1f),
      0,
      1,
      SaVec3{2300.0f, 0.0f, 0.0f},
      SaVec3{0.0f, 0.0f, 0.0f},
      SaVec3{120.0f, 0.0f, 92.75f},
      DEMO_ACTIVE_DURATION_SECONDS,
  }};

  std::array<SaLiveFrame, 3> frames{};
  for (size_t index = 0; index < frames.size(); index += 1) {
    const uint64_t frameNumber = static_cast<uint64_t>(index + 1);
    SaLiveFrame &frame = frames[index];
    frame.frame_number = frameNumber;
    frame.time = 0.1f * static_cast<float>(frameNumber);
    frame.dt = index == 0 ? 0.0f : 0.1f;
    frame.seconds_remaining = 300;
    frame.has_seconds_remaining = 1;
    frame.ball_has_been_hit = 1;
    frame.has_ball_has_been_hit = 1;
    frame.team_zero_score = 1;
    frame.has_team_zero_score = 1;
    frame.team_one_score = 0;
    frame.has_team_one_score = 1;
    frame.possession_team_is_team_0 = 1;
    frame.has_possession_team = 1;
    frame.scored_on_team_is_team_0 = 0;
    frame.has_scored_on_team = 1;
    frame.live_play = 1;
    frame.has_live_play = 1;
    frame.has_ball = 1;
    frame.ball = syntheticRigidBody(25.0f * static_cast<float>(frameNumber), 0.0f, 120.0f);
    frame.players = players.data();
    frame.player_count = players.size();
  }
  frames[0].touches = touches.data();
  frames[0].touch_count = touches.size();
  frames[0].dodge_refreshes = dodgeRefreshes.data();
  frames[0].dodge_refresh_count = dodgeRefreshes.size();
  frames[0].boost_pad_events = boostPadEvents.data();
  frames[0].boost_pad_event_count = boostPadEvents.size();
  frames[0].goals = goals.data();
  frames[0].goal_count = goals.size();
  frames[0].player_stat_events = playerStatEvents.data();
  frames[0].player_stat_event_count = playerStatEvents.size();
  frames[0].demolishes = demolishes.data();
  frames[0].demolish_count = demolishes.size();

  SaEngine *liveEngine = engine;
  const auto liveMessages = messages;
  engine = selfTestEngine;
  bool processed = true;
  for (const SaLiveFrame &frame : frames) {
    const int32_t result = processFrame(engine, &frame);
    if (result != 0) {
      processed = false;
      cvarManager->log(std::format(
          "subtr-actor: graph self-test frame {} failed: {}",
          frame.frame_number,
          result));
      break;
    }
  }

  if (processed) {
    const std::string eventHistoryJson =
        readNamedJsonBuffer(graphOutputJsonLen, writeGraphOutputJson, "event_history");
    const auto activeDemoCount =
        parseJsonArrayPropertyElementCount(eventHistoryJson, "active_demos");
    if (!activeDemoCount || *activeDemoCount == 0) {
      processed = false;
      cvarManager->log(
          "subtr-actor: graph self-test failed to derive active_demos from demolish event");
    } else {
      cvarManager->log(std::format(
          "subtr-actor: graph self-test derived active_demos from demolish event ({} entries)",
          *activeDemoCount));
    }
  }

  if (processed) {
    cvarManager->log(
        "subtr-actor: graph self-test fed every required event family");
    verifyGraphRuntime({"finish", "require_event_history", "require_graph_events"});
    if (shouldDump) {
      cvarManager->log("subtr-actor: graph self-test writing synthetic graph dump");
      dumpGraphJson({"subtr_actor_dump_graph", "finish"});
    }
  }
  messages = liveMessages;
  engine = liveEngine;
  engineDestroy(selfTestEngine);
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
