#include "SubtrActorPlugin.h"

#include <algorithm>
#include <array>
#include <cctype>
#include <cmath>
#include <cstddef>
#include <cstdlib>
#include <fstream>
#include <format>
#include <initializer_list>
#include <iterator>
#include <limits>
#include <sstream>
#include <type_traits>

#include "imgui/imgui.h"

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
constexpr std::array<const char *, 41> GRAPH_EVENT_FIELDS{
    "timeline",
    "mechanics",
    "goal_context",
    "core_player",
    "core_team",
    "possession",
    "pressure",
    "territorial_pressure",
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
constexpr size_t MAX_RECENT_UI_EVENTS = 200;

int moduleAnchor = 0;

struct EventFilterOption {
  const char *value;
  const char *label;
};

struct JsonFieldSummary {
  std::string label;
  std::string value;
};

struct UiStatDefinition {
  const char *id;
  const char *label;
  const char *category;
  bool player;
  bool team;
  bool event;
};

struct UiStatDefinitionMatch {
  const UiStatDefinition *definition = nullptr;
  double score = 0.0;
  size_t index = 0;
};

struct UiStatIdAlias {
  const char *external_id;
  const char *local_id;
};

constexpr std::array<EventFilterOption, 30> EVENT_FILTER_OPTIONS{{
    {"all", "All events"},
    {"mechanics", "All mechanics"},
    {"team", "Team events"},
    {"goal_context", "Goal context"},
    {"touch", "Touch"},
    {"touch_ball_movement", "Touch movement"},
    {"dodge_reset", "Dodge refresh"},
    {"boost_pickup", "Boost pickup"},
    {"boost_ledger", "Boost ledger"},
    {"boost_state", "Boost state"},
    {"speed_flip", "Speed flip"},
    {"half_flip", "Half flip"},
    {"wavedash", "Wavedash"},
    {"ball_carry", "Ball carry"},
    {"air_dribble", "Air dribble"},
    {"ceiling_shot", "Ceiling shot"},
    {"wall_aerial", "Wall aerial"},
    {"wall_aerial_shot", "Wall aerial shot"},
    {"center", "Center"},
    {"flip_reset", "Flip reset"},
    {"double_tap", "Double tap"},
    {"flick", "Flick"},
    {"musty_flick", "Musty flick"},
    {"one_timer", "One timer"},
    {"pass", "Pass"},
    {"half_volley", "Half volley"},
    {"whiff", "Whiff"},
    {"bump", "Bump"},
    {"demo", "Demo"},
    {"goal", "Goal"},
}};

constexpr std::array<const char *, 21> MECHANIC_FILTER_TOKENS{{
    "speed_flip",
    "half_flip",
    "wavedash",
    "ball_carry",
    "air_dribble",
    "ceiling_shot",
    "wall_aerial",
    "wall_aerial_shot",
    "center",
    "flip_reset",
    "double_tap",
    "flick",
    "musty_flick",
    "one_timer",
    "pass",
    "half_volley",
    "whiff",
    "bump",
    "demo",
    "goal",
    "dodge_reset",
}};

constexpr std::array<UiStatDefinition, 15> UI_STAT_DEFINITIONS{{
    {"score", "Score", "Core", true, true, false},
    {"goals", "Goals", "Core", true, true, false},
    {"assists", "Assists", "Core", true, true, false},
    {"saves", "Saves", "Core", true, true, false},
    {"shots", "Shots", "Core", true, true, false},
    {"boost", "Boost", "Resources", true, false, false},
    {"average_boost", "Average boost", "Resources", false, true, false},
    {"players", "Players", "Team", false, true, false},
    {"recent_events", "Recent events", "Events", true, true, true},
    {"goal", "Goals detected", "Events", false, false, true},
    {"shot", "Shots detected", "Events", false, false, true},
    {"save", "Saves detected", "Events", false, false, true},
    {"assist", "Assists detected", "Events", false, false, true},
    {"demo", "Demos detected", "Events", false, false, true},
    {"flip_reset", "Flip resets detected", "Events", false, false, true},
}};

constexpr std::array<UiStatIdAlias, 20> UI_STAT_ID_ALIASES{{
    {"player:core.score", "score"},
    {"player.core.score", "score"},
    {"team:core.score", "score"},
    {"team.core.score", "score"},
    {"player:core.goals", "goals"},
    {"player.core.goals", "goals"},
    {"team:core.goals", "goals"},
    {"team.core.goals", "goals"},
    {"player:core.assists", "assists"},
    {"player.core.assists", "assists"},
    {"team:core.assists", "assists"},
    {"team.core.assists", "assists"},
    {"player:core.saves", "saves"},
    {"player.core.saves", "saves"},
    {"team:core.saves", "saves"},
    {"team.core.saves", "saves"},
    {"player:core.shots", "shots"},
    {"player.core.shots", "shots"},
    {"team:core.shots", "shots"},
    {"team.core.shots", "shots"},
}};

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

std::optional<uint32_t> parseUnsignedIntegerString(std::string_view value) {
  if (value.empty() ||
      !std::all_of(value.begin(), value.end(), [](char ch) {
        return std::isdigit(static_cast<unsigned char>(ch)) != 0;
      })) {
    return std::nullopt;
  }
  try {
    const unsigned long long parsed = std::stoull(std::string{value});
    if (parsed > std::numeric_limits<uint32_t>::max()) {
      return std::nullopt;
    }
    return static_cast<uint32_t>(parsed);
  } catch (const std::exception &) {
    return std::nullopt;
  }
}

double recordingPlaybackRateFromIndex(int index) {
  constexpr std::array<double, 4> rates{{0.5, 1.0, 1.5, 2.0}};
  return rates[static_cast<size_t>(std::clamp(index, 0, static_cast<int>(rates.size()) - 1))];
}

int recordingPlaybackRateIndexForValue(double value) {
  constexpr std::array<double, 4> rates{{0.5, 1.0, 1.5, 2.0}};
  const auto closest = std::min_element(
      rates.begin(),
      rates.end(),
      [value](double left, double right) {
        return std::abs(left - value) < std::abs(right - value);
      });
  return static_cast<int>(std::distance(rates.begin(), closest));
}

ImVec2 mapWindowPositionToViewport(
    float x,
    float y,
    float width,
    float height,
    float sourceViewportWidth,
    float sourceViewportHeight) {
  const ImVec2 displaySize = ImGui::GetIO().DisplaySize;
  if (displaySize.x <= 0.0f || displaySize.y <= 0.0f) {
    return ImVec2{x, y};
  }

  constexpr float margin = 8.0f;
  const float scaleX = sourceViewportWidth > 0.0f ? displaySize.x / sourceViewportWidth : 1.0f;
  const float scaleY = sourceViewportHeight > 0.0f ? displaySize.y / sourceViewportHeight : 1.0f;
  const float clampedWidth = std::max(120.0f, width);
  const float clampedHeight = std::max(80.0f, height);
  const float maxX = std::max(margin, displaySize.x - clampedWidth);
  const float maxY = std::max(margin, displaySize.y - clampedHeight);
  return ImVec2{
      std::clamp(x * scaleX, margin, maxX),
      std::clamp(y * scaleY, margin, maxY),
  };
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

std::optional<size_t> parseJsonArrayElementCountAt(const std::string &json, size_t offset) {
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

std::string summarizeJsonValueAt(const std::string &json, size_t offset) {
  skipJsonWhitespace(json, offset);
  if (offset >= json.size()) {
    return "--";
  }

  if (json[offset] == '"') {
    auto value = parseJsonString(json, offset);
    if (!value) {
      return "--";
    }
    return value->size() > 96 ? value->substr(0, 93) + "..." : *value;
  }

  if (json[offset] == '[') {
    const auto count = parseJsonArrayElementCountAt(json, offset);
    return count ? std::format("{} item{}", *count, *count == 1 ? "" : "s") : "array";
  }

  if (json[offset] == '{') {
    return "object";
  }

  const size_t start = offset;
  if (!skipJsonValue(json, offset)) {
    return "--";
  }
  return json.substr(start, offset - start);
}

void collectJsonFieldSummaries(
    const std::string &json,
    std::string_view prefix,
    std::vector<JsonFieldSummary> &out,
    size_t maxFields,
    int maxDepth) {
  if (out.size() >= maxFields || maxDepth < 0) {
    return;
  }

  size_t offset = 0;
  skipJsonWhitespace(json, offset);
  if (offset >= json.size() || json[offset] != '{') {
    return;
  }
  ++offset;
  skipJsonWhitespace(json, offset);

  while (offset < json.size() && out.size() < maxFields) {
    if (json[offset] == '}') {
      return;
    }

    auto key = parseJsonString(json, offset);
    if (!key) {
      return;
    }
    skipJsonWhitespace(json, offset);
    if (offset >= json.size() || json[offset] != ':') {
      return;
    }
    ++offset;
    skipJsonWhitespace(json, offset);

    const std::string label =
        prefix.empty() ? *key : std::format("{}.{}", prefix, *key);
    const size_t valueStart = offset;
    if (offset < json.size() && json[offset] == '{' && maxDepth > 0) {
      size_t end = offset;
      if (!skipJsonValue(json, end)) {
        return;
      }
      collectJsonFieldSummaries(
          json.substr(valueStart, end - valueStart),
          label,
          out,
          maxFields,
          maxDepth - 1);
      offset = end;
    } else {
      out.push_back(JsonFieldSummary{label, summarizeJsonValueAt(json, offset)});
      if (!skipJsonValue(json, offset)) {
        return;
      }
    }

    skipJsonWhitespace(json, offset);
    if (offset < json.size() && json[offset] == ',') {
      ++offset;
      skipJsonWhitespace(json, offset);
      continue;
    }
    if (offset < json.size() && json[offset] == '}') {
      return;
    }
    return;
  }
}

std::string escapeJsonString(std::string_view value) {
  std::string escaped;
  escaped.reserve(value.size() + 8);
  for (const char ch : value) {
    switch (ch) {
    case '"':
      escaped += "\\\"";
      break;
    case '\\':
      escaped += "\\\\";
      break;
    case '\b':
      escaped += "\\b";
      break;
    case '\f':
      escaped += "\\f";
      break;
    case '\n':
      escaped += "\\n";
      break;
    case '\r':
      escaped += "\\r";
      break;
    case '\t':
      escaped += "\\t";
      break;
    default:
      escaped.push_back(ch);
      break;
    }
  }
  return escaped;
}

std::string clippedDisplayText(std::string value, size_t maxBytes = 24000) {
  if (value.size() <= maxBytes) {
    return value;
  }
  const size_t originalSize = value.size();
  value.resize(maxBytes);
  value += std::format(
      "\n\n... truncated {} of {} bytes ...",
      originalSize - maxBytes,
      originalSize);
  return value;
}

std::string formatByteSize(size_t bytes) {
  constexpr double KIB = 1024.0;
  constexpr double MIB = KIB * 1024.0;
  if (bytes >= static_cast<size_t>(MIB)) {
    return std::format("{:.1f} MiB", static_cast<double>(bytes) / MIB);
  }
  if (bytes >= static_cast<size_t>(KIB)) {
    return std::format("{:.1f} KiB", static_cast<double>(bytes) / KIB);
  }
  return std::format("{} B", bytes);
}

bool findJsonPropertyValueOffset(
    const std::string &json,
    const std::string &propertyName,
    size_t &offset) {
  const std::string needle = std::format("\"{}\"", propertyName);
  offset = json.find(needle);
  if (offset == std::string::npos) {
    return false;
  }
  offset += needle.size();
  skipJsonWhitespace(json, offset);
  if (offset >= json.size() || json[offset] != ':') {
    return false;
  }
  ++offset;
  skipJsonWhitespace(json, offset);
  return offset < json.size();
}

bool jsonPropertyExists(const std::string &json, const std::string &propertyName) {
  size_t offset = 0;
  return findJsonPropertyValueOffset(json, propertyName, offset);
}

std::optional<std::string> parseJsonStringProperty(
    const std::string &json,
    const std::string &propertyName) {
  size_t offset = 0;
  if (!findJsonPropertyValueOffset(json, propertyName, offset)) {
    return std::nullopt;
  }
  return parseJsonString(json, offset);
}

std::optional<bool> parseJsonBoolProperty(
    const std::string &json,
    const std::string &propertyName) {
  size_t offset = 0;
  if (!findJsonPropertyValueOffset(json, propertyName, offset)) {
    return std::nullopt;
  }
  if (json.compare(offset, 4, "true") == 0) {
    return true;
  }
  if (json.compare(offset, 5, "false") == 0) {
    return false;
  }
  return std::nullopt;
}

std::optional<double> parseJsonNumberProperty(
    const std::string &json,
    const std::string &propertyName) {
  size_t offset = 0;
  if (!findJsonPropertyValueOffset(json, propertyName, offset)) {
    return std::nullopt;
  }
  const size_t start = offset;
  if (offset < json.size() && json[offset] == '-') {
    ++offset;
  }
  while (offset < json.size() &&
         std::isdigit(static_cast<unsigned char>(json[offset])) != 0) {
    ++offset;
  }
  if (offset < json.size() && json[offset] == '.') {
    ++offset;
    while (offset < json.size() &&
           std::isdigit(static_cast<unsigned char>(json[offset])) != 0) {
      ++offset;
    }
  }
  if (offset == start) {
    return std::nullopt;
  }
  try {
    return std::stod(json.substr(start, offset - start));
  } catch (...) {
    return std::nullopt;
  }
}

std::vector<std::string> parseJsonObjectArrayProperty(
    const std::string &json,
    const std::string &propertyName) {
  std::vector<std::string> objects;
  size_t offset = 0;
  if (!findJsonPropertyValueOffset(json, propertyName, offset) || json[offset] != '[') {
    return objects;
  }
  ++offset;
  while (offset < json.size()) {
    skipJsonWhitespace(json, offset);
    if (offset < json.size() && json[offset] == ']') {
      break;
    }
    if (offset >= json.size() || json[offset] != '{') {
      return {};
    }
    const size_t start = offset;
    size_t end = offset;
    if (!skipJsonValue(json, end)) {
      return {};
    }
    objects.push_back(json.substr(start, end - start));
    offset = end;
    skipJsonWhitespace(json, offset);
    if (offset < json.size() && json[offset] == ',') {
      ++offset;
      continue;
    }
    if (offset < json.size() && json[offset] == ']') {
      break;
    }
  }
  return objects;
}

std::optional<std::string> parseJsonObjectProperty(
    const std::string &json,
    const std::string &propertyName) {
  size_t offset = 0;
  if (!findJsonPropertyValueOffset(json, propertyName, offset) || json[offset] != '{') {
    return std::nullopt;
  }
  const size_t start = offset;
  size_t end = offset;
  if (!skipJsonValue(json, end)) {
    return std::nullopt;
  }
  return json.substr(start, end - start);
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

std::string normalizeEventFilterToken(std::string_view token) {
  std::string normalized;
  normalized.reserve(token.size());
  bool previousSeparator = false;
  for (const char ch : token) {
    const unsigned char value = static_cast<unsigned char>(ch);
    if (std::isalnum(value) != 0) {
      normalized.push_back(static_cast<char>(std::tolower(value)));
      previousSeparator = false;
      continue;
    }
    if ((ch == '_' || ch == '-' || std::isspace(value) != 0) && !previousSeparator &&
        !normalized.empty()) {
      normalized.push_back('_');
      previousSeparator = true;
    }
  }
  if (!normalized.empty() && normalized.back() == '_') {
    normalized.pop_back();
  }
  return normalized;
}

bool isKnownEventFilterToken(std::string_view token) {
  return std::any_of(
      EVENT_FILTER_OPTIONS.begin(),
      EVENT_FILTER_OPTIONS.end(),
      [token](const EventFilterOption &option) { return std::string_view{option.value} == token; });
}

bool isMechanicFilterToken(std::string_view token) {
  return std::any_of(
      MECHANIC_FILTER_TOKENS.begin(),
      MECHANIC_FILTER_TOKENS.end(),
      [token](const char *option) { return token == std::string_view{option}; });
}

void appendUniqueFilterToken(std::vector<std::string> &tokens, std::string_view token) {
  if (token.empty() || token == "all" || containsString(tokens, token)) {
    return;
  }
  tokens.emplace_back(token);
}

std::string mechanicToken(SaMechanicKind kind) {
  return normalizeEventFilterToken(mechanicLabel(kind));
}

bool eventFilterAllows(
    std::string_view rawFilter,
    std::string_view category,
    std::string_view type) {
  const std::string normalizedCategory = normalizeEventFilterToken(category);
  const std::string normalizedType = normalizeEventFilterToken(type);
  std::string token;
  bool sawToken = false;

  auto flushToken = [&]() {
    if (token.empty()) {
      return false;
    }
    sawToken = true;
    const bool allowed = token == "all" || token == normalizedCategory ||
                         token == normalizedType ||
                         token == std::format("{}_{}", normalizedCategory, normalizedType);
    token.clear();
    return allowed;
  };

  for (const char ch : rawFilter) {
    const unsigned char value = static_cast<unsigned char>(ch);
    if (ch == ',' || ch == ';' || ch == '|' || std::isspace(value) != 0) {
      if (flushToken()) {
        return true;
      }
      continue;
    }
    if (std::isalnum(value) != 0) {
      token.push_back(static_cast<char>(std::tolower(value)));
    } else if ((ch == '_' || ch == '-') && !token.empty() && token.back() != '_') {
      token.push_back('_');
    }
  }

  if (flushToken()) {
    return true;
  }
  return !sawToken;
}

std::vector<std::string> eventFilterTokens(std::string_view rawFilter) {
  std::vector<std::string> tokens;
  std::string token;

  auto flushToken = [&]() {
    if (!token.empty()) {
      tokens.push_back(token);
      token.clear();
    }
  };

  for (const char ch : rawFilter) {
    const unsigned char value = static_cast<unsigned char>(ch);
    if (ch == ',' || ch == ';' || ch == '|' || std::isspace(value) != 0) {
      flushToken();
      continue;
    }
    if (std::isalnum(value) != 0) {
      token.push_back(static_cast<char>(std::tolower(value)));
    } else if ((ch == '_' || ch == '-') && !token.empty() && token.back() != '_') {
      token.push_back('_');
    }
  }
  flushToken();
  return tokens;
}

bool allEventSourcesSelected(std::string_view rawFilter) {
  const std::vector<std::string> tokens = eventFilterTokens(rawFilter);
  return tokens.empty() || containsString(tokens, "all");
}

std::vector<std::string> selectedEventSourceTokens(std::string_view rawFilter) {
  std::vector<std::string> tokens;
  if (allEventSourcesSelected(rawFilter)) {
    for (const EventFilterOption &option : EVENT_FILTER_OPTIONS) {
      if (std::string_view{option.value} != "all") {
        tokens.emplace_back(option.value);
      }
    }
    return tokens;
  }

  for (const std::string &token : eventFilterTokens(rawFilter)) {
    if (token != "all" && token != "none" && !containsString(tokens, token)) {
      tokens.push_back(token);
    }
  }
  return tokens;
}

std::string eventFilterFromSelectedSources(const std::vector<std::string> &tokens) {
  if (tokens.empty()) {
    return "none";
  }
  if (tokens.size() + 1 >= EVENT_FILTER_OPTIONS.size()) {
    return "all";
  }

  std::string filter;
  for (const std::string &token : tokens) {
    if (!filter.empty()) {
      filter += ",";
    }
    filter += token;
  }
  return filter;
}

const char *eventFilterLabel(std::string_view value) {
  const std::string normalized = normalizeEventFilterToken(value);
  for (const EventFilterOption &option : EVENT_FILTER_OPTIONS) {
    if (normalized == option.value) {
      return option.label;
    }
  }
  return value.empty() ? "All events" : "Custom filter";
}

std::string eventFilterPreview(std::string_view rawFilter) {
  const std::vector<std::string> selected = selectedEventSourceTokens(rawFilter);
  if (allEventSourcesSelected(rawFilter)) {
    return "All events";
  }
  if (selected.empty()) {
    return "No events";
  }
  if (selected.size() == 1) {
    return eventFilterLabel(selected.front());
  }
  return std::format("{} event types", selected.size());
}

const UiStatDefinition *localUiStatDefinition(std::string_view statId) {
  const auto found = std::find_if(
      UI_STAT_DEFINITIONS.begin(),
      UI_STAT_DEFINITIONS.end(),
      [statId](const UiStatDefinition &definition) { return definition.id == statId; });
  return found == UI_STAT_DEFINITIONS.end() ? nullptr : &*found;
}

std::string normalizeUiStatId(std::string_view statId) {
  if (localUiStatDefinition(statId)) {
    return std::string{statId};
  }
  for (const UiStatIdAlias &alias : UI_STAT_ID_ALIASES) {
    if (statId == alias.external_id) {
      return alias.local_id;
    }
  }
  return std::string{statId};
}

const UiStatDefinition *uiStatDefinition(std::string_view statId) {
  const std::string normalized = normalizeUiStatId(statId);
  return localUiStatDefinition(normalized);
}

const char *uiStatLabel(std::string_view statId) {
  if (const UiStatDefinition *definition = uiStatDefinition(statId)) {
    return definition->label;
  }
  return "Stat";
}

std::optional<std::string_view> coreStatsPlayerField(std::string_view localStatId) {
  if (
      localStatId == "score" ||
      localStatId == "goals" ||
      localStatId == "assists" ||
      localStatId == "saves" ||
      localStatId == "shots") {
    return localStatId;
  }
  return std::nullopt;
}

std::string normalizeStatSearchText(std::string_view value) {
  std::string normalized;
  normalized.reserve(value.size());
  bool previousWasSpace = true;
  for (const char ch : value) {
    const unsigned char byte = static_cast<unsigned char>(ch);
    if (std::isalnum(byte) != 0) {
      normalized.push_back(static_cast<char>(std::tolower(byte)));
      previousWasSpace = false;
      continue;
    }
    if ((ch == '_' || ch == '/' || ch == '.' || ch == '-' || std::isspace(byte) != 0) &&
        !previousWasSpace) {
      normalized.push_back(' ');
      previousWasSpace = true;
    }
  }
  if (!normalized.empty() && normalized.back() == ' ') {
    normalized.pop_back();
  }
  return normalized;
}

std::vector<std::string_view> statSearchTokens(std::string_view query) {
  static thread_local std::string normalized;
  normalized = normalizeStatSearchText(query);
  std::vector<std::string_view> tokens;
  size_t offset = 0;
  while (offset < normalized.size()) {
    const size_t end = normalized.find(' ', offset);
    tokens.emplace_back(
        normalized.data() + offset,
        (end == std::string::npos ? normalized.size() : end) - offset);
    if (end == std::string::npos) {
      break;
    }
    offset = end + 1;
  }
  return tokens;
}

std::optional<double> statDefinitionSearchScore(
    const UiStatDefinition &definition,
    std::string_view query) {
  const std::vector<std::string_view> tokens = statSearchTokens(query);
  if (tokens.empty()) {
    return 0.0;
  }

  const std::string searchText = normalizeStatSearchText(std::format(
      "{} {} {}",
      definition.category,
      definition.label,
      definition.id));
  double total = 0.0;
  for (std::string_view token : tokens) {
    const size_t index = searchText.find(token);
    if (index == std::string::npos) {
      return std::nullopt;
    }
    total += static_cast<double>(index);
  }
  return total + static_cast<double>(searchText.size()) / 1000.0;
}

ImVec4 toImVec4(LinearColor color) {
  return ImVec4{
      color.R / 255.0f,
      color.G / 255.0f,
      color.B / 255.0f,
      color.A / 255.0f,
  };
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

std::string SubtrActorPlugin::webUiStatIdForWindow(
    const UiStatsWindow &window,
    const UiStatsWindow::Entry &entry) const {
  const std::string localStatId = normalizeUiStatId(entry.stat_id);
  const auto coreField = coreStatsPlayerField(localStatId);
  if (!coreField) {
    return localStatId;
  }

  const UiStatDefinition *definition = localUiStatDefinition(localStatId);
  if (!definition) {
    return localStatId;
  }

  bool preferTeamScope = false;
  switch (window.kind) {
  case UiStatsWindowKind::Team:
  case UiStatsWindowKind::AllTeams:
    preferTeamScope = true;
    break;
  case UiStatsWindowKind::AdHoc:
    preferTeamScope = entry.target_id == "blue" || entry.target_id == "orange";
    break;
  default:
    preferTeamScope = false;
    break;
  }

  if (preferTeamScope && definition->team) {
    return std::format("team:core.{}", *coreField);
  }
  if (definition->player) {
    return std::format("player:core.{}", *coreField);
  }
  if (definition->team) {
    return std::format("team:core.{}", *coreField);
  }
  return localStatId;
}

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
      "subtr_actor_ui_enabled",
      "1",
      "Enable the interactive subtr-actor BakkesMod window interface.",
      true,
      true,
      0,
      true,
      1);
  cvarManager->registerCvar(
      "subtr_actor_overlay_x",
      "64",
      "Subtr-actor overlay panel X position.",
      true,
      true,
      0,
      true,
      10000);
  cvarManager->registerCvar(
      "subtr_actor_overlay_y",
      "240",
      "Subtr-actor overlay panel Y position.",
      true,
      true,
      0,
      true,
      10000);
  cvarManager->registerCvar(
      "subtr_actor_overlay_scale",
      "1",
      "Subtr-actor overlay text scale.",
      true,
      true,
      0.5,
      true,
      3);
  cvarManager->registerCvar(
      "subtr_actor_overlay_message_seconds",
      "3",
      "Seconds to keep subtr-actor event messages visible.",
      true,
      true,
      0.5,
      true,
      30);
  cvarManager->registerCvar(
      "subtr_actor_overlay_max_messages",
      "8",
      "Maximum subtr-actor event messages to keep on screen.",
      true,
      true,
      1,
      true,
      30);
  cvarManager->registerCvar(
      "subtr_actor_overlay_mechanics_enabled",
      "1",
      "Show individual mechanic events in the subtr-actor overlay.",
      true,
      true,
      0,
      true,
      1);
  cvarManager->registerCvar(
      "subtr_actor_overlay_team_events_enabled",
      "1",
      "Show team-level events in the subtr-actor overlay.",
      true,
      true,
      0,
      true,
      1);
  cvarManager->registerCvar(
      "subtr_actor_overlay_goal_context_enabled",
      "1",
      "Show goal-context events in the subtr-actor overlay.",
      true,
      true,
      0,
      true,
      1);
  cvarManager->registerCvar(
      "subtr_actor_overlay_event_types",
      "all",
      "Comma-separated overlay filter: all, mechanics, team, goal_context, or mechanic "
      "tokens like speed_flip,wavedash,half_flip.",
      true);
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

  loadUiConfig();

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
  cvarManager->registerNotifier(
      "subtr_actor_overlay_options",
      [this](std::vector<std::string>) {
        cvarManager->log(
            "subtr-actor overlay filters: all, mechanics, team, goal_context, speed_flip, "
            "half_flip, wavedash, ball_carry, air_dribble, ceiling_shot, wall_aerial, "
            "wall_aerial_shot, center, flip_reset, double_tap, flick, musty_flick, "
            "one_timer, pass, half_volley, whiff, bump, backboard, boost_pickup, demo, "
            "shot, save, assist, goal");
      },
      "Logs supported values for subtr_actor_overlay_event_types.",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "subtr_actor_open_ui",
      [this](std::vector<std::string>) {
        uiWindowOpen = true;
        uiLauncherOpen = true;
        launcherPlacement.pending_focus = true;
      },
      "Opens the subtr-actor in-game launcher window.",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "subtr_actor_toggle_ui",
      [this](std::vector<std::string>) {
        uiWindowOpen = true;
        uiLauncherOpen = !uiLauncherOpen;
        if (uiLauncherOpen) {
          launcherPlacement.pending_focus = true;
        }
      },
      "Toggles the subtr-actor in-game launcher window.",
      PERMISSION_ALL);
  hookGameEvents();

  cvarManager->log("subtr-actor: mechanic overlay loaded");
}

void SubtrActorPlugin::onUnload() {
  saveUiConfig();
  if (liveTickCancelled) {
    *liveTickCancelled = true;
  }
  gameWrapper->UnregisterDrawables();
  unhookGameEvents();
  unloadRustLibrary();
}

void SubtrActorPlugin::SetImGuiContext(uintptr_t ctx) {
  imguiContext = ctx;
  ImGui::SetCurrentContext(reinterpret_cast<ImGuiContext *>(ctx));
}

std::string SubtrActorPlugin::GetMenuName() {
  return "subtr-actor";
}

std::string SubtrActorPlugin::GetMenuTitle() {
  return "subtr-actor";
}

std::string SubtrActorPlugin::GetPluginName() {
  return "subtr-actor";
}

bool SubtrActorPlugin::ShouldBlockInput() {
  return true;
}

bool SubtrActorPlugin::IsActiveOverlay() {
  return false;
}

void SubtrActorPlugin::OnOpen() {
  uiWindowOpen = true;
  uiLauncherOpen = true;
  launcherPlacement.pending_focus = true;
}

void SubtrActorPlugin::OnClose() {
  uiWindowOpen = false;
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

bool SubtrActorPlugin::uiEnabled() {
  return cvarBool("subtr_actor_ui_enabled", true);
}

bool SubtrActorPlugin::cvarBool(const char *name, bool defaultValue) const {
  auto cvar = cvarManager->getCvar(name);
  return static_cast<bool>(cvar) ? cvar.getBoolValue() : defaultValue;
}

void SubtrActorPlugin::setCvarBool(const char *name, bool value) {
  auto cvar = cvarManager->getCvar(name);
  if (static_cast<bool>(cvar)) {
    cvar.setValue(value ? 1 : 0);
  }
}

std::string SubtrActorPlugin::cvarString(const char *name, std::string_view defaultValue) const {
  auto cvar = cvarManager->getCvar(name);
  return static_cast<bool>(cvar) ? cvar.getStringValue() : std::string(defaultValue);
}

void SubtrActorPlugin::setCvarString(const char *name, std::string_view value) {
  auto cvar = cvarManager->getCvar(name);
  if (static_cast<bool>(cvar)) {
    cvar.setValue(std::string(value));
  }
}

std::filesystem::path SubtrActorPlugin::uiConfigPath() const {
  if (gameWrapper) {
    return gameWrapper->GetDataFolder() / "subtr-actor" / "ui-config.json";
  }
  const auto moduleDirectory = currentModuleDirectory();
  return moduleDirectory.empty() ? std::filesystem::path{"ui-config.json"}
                                 : moduleDirectory / "ui-config.json";
}

void SubtrActorPlugin::loadUiConfig() {
  const auto path = uiConfigPath();
  std::ifstream file(path, std::ios::binary);
  if (!file) {
    return;
  }

  const std::string json((std::istreambuf_iterator<char>(file)), std::istreambuf_iterator<char>());
  applyUiConfigJson(json, path.string());
}

void SubtrActorPlugin::applyUiConfigJson(
    const std::string &json,
    std::string_view sourceLabel) {
  const size_t firstJsonByte = json.find_first_not_of(" \t\r\n");
  if (firstJsonByte == std::string::npos || json[firstJsonByte] != '{' ||
      json.find("\"version\"") == std::string::npos) {
    cvarManager->log(std::format("subtr-actor: ignored invalid UI config from {}", sourceLabel));
    return;
  }

  auto loadPlacementObject = [this](
                                 const std::string &object,
                                 UiWindowPlacement &out,
                                 bool *visible = nullptr) {
    if (visible != nullptr) {
      *visible = parseJsonBoolProperty(object, "visible").value_or(*visible);
    }
    out.has_placement = parseJsonBoolProperty(object, "has_placement").value_or(true);
    out.pending_apply_placement = out.has_placement;
    out.x = static_cast<float>(parseJsonNumberProperty(object, "x").value_or(out.x));
    out.y = static_cast<float>(parseJsonNumberProperty(object, "y").value_or(out.y));
    out.width = static_cast<float>(parseJsonNumberProperty(object, "width").value_or(out.width));
    out.height =
        static_cast<float>(parseJsonNumberProperty(object, "height").value_or(out.height));
    out.viewport_width = static_cast<float>(
        parseJsonNumberProperty(object, "viewport_width").value_or(out.viewport_width));
    out.viewport_height = static_cast<float>(
        parseJsonNumberProperty(object, "viewport_height").value_or(out.viewport_height));
    if (const auto viewport = parseJsonObjectProperty(object, "viewport")) {
      out.viewport_width = static_cast<float>(
          parseJsonNumberProperty(*viewport, "width").value_or(out.viewport_width));
      out.viewport_height = static_cast<float>(
          parseJsonNumberProperty(*viewport, "height").value_or(out.viewport_height));
    }
    out.z_index = static_cast<int>(parseJsonNumberProperty(object, "zIndex").value_or(out.z_index));
    nextUiWindowZIndex = std::max(nextUiWindowZIndex, out.z_index + 1);
  };
  auto loadPlacement = [&loadPlacementObject](
                           const std::string &parent,
                           const char *name,
                           UiWindowPlacement &out,
                           bool *visible = nullptr) {
    const auto object = parseJsonObjectProperty(parent, name);
    if (!object) {
      return;
    }
    loadPlacementObject(*object, out, visible);
  };

  uiLauncherOpen = parseJsonBoolProperty(json, "launcher_open").value_or(uiLauncherOpen);
  uiScoreboardOpen = parseJsonBoolProperty(json, "scoreboard_open").value_or(uiScoreboardOpen);
  uiEventsOpen = parseJsonBoolProperty(json, "events_open").value_or(uiEventsOpen);
  uiStatusOpen = parseJsonBoolProperty(json, "status_open").value_or(uiStatusOpen);
  uiCameraOpen = parseJsonBoolProperty(json, "camera_open").value_or(uiCameraOpen);
  uiPlaybackControlsOpen =
      parseJsonBoolProperty(json, "playback_controls_open").value_or(uiPlaybackControlsOpen);
  uiRecordingOpen =
      parseJsonBoolProperty(json, "recording_open").value_or(uiRecordingOpen);
  uiGraphInspectorOpen =
      parseJsonBoolProperty(json, "graph_inspector_open").value_or(uiGraphInspectorOpen);
  uiEventPlaylistOpen =
      parseJsonBoolProperty(json, "event_playlist_open").value_or(uiEventPlaylistOpen);
  uiMechanicsReviewOpen =
      parseJsonBoolProperty(json, "mechanics_review_open").value_or(uiMechanicsReviewOpen);
  uiReplayLoadingOpen =
      parseJsonBoolProperty(json, "replay_loading_open").value_or(uiReplayLoadingOpen);
  uiModuleControlsOpen =
      parseJsonBoolProperty(json, "module_controls_open").value_or(uiModuleControlsOpen);
  uiTouchControlsOpen =
      parseJsonBoolProperty(json, "touch_controls_open").value_or(uiTouchControlsOpen);
  uiBoostPickupControlsOpen =
      parseJsonBoolProperty(json, "boost_pickup_controls_open").value_or(uiBoostPickupControlsOpen);
  eventPlaylistMechanicsEnabled = parseJsonBoolProperty(json, "event_playlist_mechanics_enabled")
                                      .value_or(eventPlaylistMechanicsEnabled);
  eventPlaylistTeamEventsEnabled = parseJsonBoolProperty(json, "event_playlist_team_enabled")
                                       .value_or(eventPlaylistTeamEventsEnabled);
  eventPlaylistGoalContextEnabled =
      parseJsonBoolProperty(json, "event_playlist_goal_context_enabled")
          .value_or(eventPlaylistGoalContextEnabled);
  eventPlaylistAutoFollow =
      parseJsonBoolProperty(json, "event_playlist_auto_follow").value_or(eventPlaylistAutoFollow);
  if (const auto overlays = parseJsonObjectProperty(json, "overlays")) {
    const std::vector<std::string> timelineEvents =
        parseJsonStringArrayProperty(*overlays, "timelineEvents");
    const bool hasTimelineEvents = jsonPropertyExists(*overlays, "timelineEvents");
    if (hasTimelineEvents) {
      eventPlaylistMechanicsEnabled = containsString(timelineEvents, "mechanics");
      eventPlaylistTeamEventsEnabled = containsString(timelineEvents, "team");
      eventPlaylistGoalContextEnabled = containsString(timelineEvents, "goal_context");
    }

    const std::vector<std::string> mechanicFilters =
        parseJsonStringArrayProperty(*overlays, "mechanics");
    const bool hasMechanicFilters = jsonPropertyExists(*overlays, "mechanics");
    if (hasTimelineEvents || hasMechanicFilters) {
      std::vector<std::string> selectedFilters;
      for (const std::string &id : timelineEvents) {
        const std::string token = normalizeEventFilterToken(id);
        if (token == "mechanics" && hasMechanicFilters && !mechanicFilters.empty()) {
          continue;
        }
        if (isKnownEventFilterToken(token)) {
          appendUniqueFilterToken(selectedFilters, token);
        }
      }
      for (const std::string &id : mechanicFilters) {
        const std::string token = normalizeEventFilterToken(id);
        if (!token.empty()) {
          appendUniqueFilterToken(selectedFilters, token);
        }
      }
      if (hasMechanicFilters && !mechanicFilters.empty()) {
        eventPlaylistMechanicsEnabled = true;
      }
      setCvarString(
          "subtr_actor_overlay_event_types",
          eventFilterFromSelectedSources(selectedFilters));
    }

    const std::vector<std::string> timelineRanges =
        parseJsonStringArrayProperty(*overlays, "timelineRanges");
    if (jsonPropertyExists(*overlays, "timelineRanges")) {
      timelineRangeBoostEnabled = containsString(timelineRanges, "boost");
      timelineRangePossessionEnabled = containsString(timelineRanges, "possession");
      timelineRangePressureEnabled = containsString(timelineRanges, "pressure");
      timelineRangeRushEnabled = containsString(timelineRanges, "rush");
      timelineRangeAbsolutePositioningEnabled =
          containsString(timelineRanges, "absolute-positioning");
    }

    const std::vector<std::string> renderEffects =
        parseJsonStringArrayProperty(*overlays, "renderEffects");
    if (overlays->find("\"renderEffects\"") != std::string::npos) {
      const bool anyRenderEffect = !renderEffects.empty();
      renderEffectCeilingShotEnabled = containsString(renderEffects, "ceiling-shot");
      renderEffectFiftyFiftyEnabled = containsString(renderEffects, "fifty-fifty");
      renderEffectPressureEnabled = containsString(renderEffects, "pressure");
      renderEffectRelativePositioningEnabled =
          containsString(renderEffects, "relative-positioning");
      renderEffectAbsolutePositioningEnabled =
          containsString(renderEffects, "absolute-positioning");
      renderEffectSpeedFlipEnabled = containsString(renderEffects, "speed-flip");
      renderEffectTouchEnabled = containsString(renderEffects, "touch");
      setCvarBool("subtr_actor_overlay_enabled", anyRenderEffect);
      setCvarBool(
          "subtr_actor_overlay_mechanics_enabled",
          containsString(renderEffects, "mechanics") ||
              renderEffectCeilingShotEnabled ||
              renderEffectFiftyFiftyEnabled ||
              renderEffectRelativePositioningEnabled ||
              renderEffectAbsolutePositioningEnabled ||
              renderEffectSpeedFlipEnabled ||
              renderEffectTouchEnabled);
      setCvarBool(
          "subtr_actor_overlay_team_events_enabled",
          containsString(renderEffects, "team") || renderEffectPressureEnabled);
      setCvarBool(
          "subtr_actor_overlay_goal_context_enabled",
          containsString(renderEffects, "goal_context"));
    }

    const std::optional<bool> boostPads = parseJsonBoolProperty(*overlays, "boostPads");
    if (boostPads) {
      boostPickupPadBig = *boostPads;
      boostPickupPadSmall = *boostPads;
      boostPickupPadAmbiguous = *boostPads;
    }
    boostPickupAnimationEnabled = parseJsonBoolProperty(*overlays, "boostPickupAnimation")
                                      .value_or(boostPickupAnimationEnabled);
  }
  cameraViewMode = static_cast<int>(
      std::clamp(parseJsonNumberProperty(json, "camera_view_mode").value_or(0.0), 0.0, 3.0));
  cameraFreePreset = static_cast<int>(
      std::clamp(parseJsonNumberProperty(json, "camera_free_preset").value_or(0.0), 0.0, 1.0));
  cameraSelectedPlayerIndex = static_cast<uint32_t>(
      std::max(0.0, parseJsonNumberProperty(json, "camera_selected_player_index").value_or(0.0)));
  cameraDistanceScale = static_cast<float>(std::clamp(
      parseJsonNumberProperty(json, "camera_distance_scale").value_or(1.0),
      0.75,
      4.0));
  cameraCustomSettingsEnabled =
      parseJsonBoolProperty(json, "camera_custom_settings_enabled")
          .value_or(cameraCustomSettingsEnabled);
  cameraBallCamEnabled =
      parseJsonBoolProperty(json, "camera_ball_cam_enabled").value_or(cameraBallCamEnabled);
  cameraCustomFov = static_cast<float>(
      std::clamp(parseJsonNumberProperty(json, "camera_custom_fov").value_or(110.0), 60.0, 130.0));
  cameraCustomHeight = static_cast<float>(std::clamp(
      parseJsonNumberProperty(json, "camera_custom_height").value_or(100.0),
      40.0,
      250.0));
  cameraCustomPitch = static_cast<float>(std::clamp(
      parseJsonNumberProperty(json, "camera_custom_pitch").value_or(-4.0),
      -30.0,
      30.0));
  cameraCustomDistance = static_cast<float>(std::clamp(
      parseJsonNumberProperty(json, "camera_custom_distance").value_or(270.0),
      100.0,
      500.0));
  cameraCustomStiffness = static_cast<float>(std::clamp(
      parseJsonNumberProperty(json, "camera_custom_stiffness").value_or(0.0),
      0.0,
      1.0));
  cameraCustomSwivelSpeed = static_cast<float>(std::clamp(
      parseJsonNumberProperty(json, "camera_custom_swivel_speed").value_or(1.0),
      1.0,
      10.0));
  cameraCustomTransitionSpeed = static_cast<float>(std::clamp(
      parseJsonNumberProperty(json, "camera_custom_transition_speed").value_or(1.0),
      0.5,
      2.0));
  recordingFps = static_cast<int>(
      std::clamp(parseJsonNumberProperty(json, "recording_fps").value_or(60.0), 1.0, 120.0));
  recordingPlaybackRateIndex = static_cast<int>(std::clamp(
      parseJsonNumberProperty(json, "recording_playback_rate_index").value_or(1.0),
      0.0,
      3.0));
  recordingFinishBeforeDump =
      parseJsonBoolProperty(json, "recording_finish_before_dump")
          .value_or(recordingFinishBeforeDump);

  if (const auto camera = parseJsonObjectProperty(json, "camera")) {
    const std::optional<std::string> mode = parseJsonStringProperty(*camera, "mode");
    if (mode == "follow") {
      cameraViewMode = 1;
    } else if (mode == "free") {
      cameraViewMode = 0;
    }
    const std::optional<std::string> freePreset =
        parseJsonStringProperty(*camera, "freePreset");
    if (freePreset == "overhead") {
      cameraFreePreset = 0;
      if (cameraViewMode != 1) {
        cameraViewMode = 2;
      }
    } else if (freePreset == "side") {
      cameraFreePreset = 1;
      if (cameraViewMode != 1) {
        cameraViewMode = 3;
      }
    }
    if (const auto attachedPlayerId = parseJsonStringProperty(*camera, "attachedPlayerId")) {
      if (const auto parsedPlayerIndex = parseUnsignedIntegerString(*attachedPlayerId)) {
        cameraSelectedPlayerIndex = *parsedPlayerIndex;
      }
    }
    cameraDistanceScale = static_cast<float>(std::clamp(
        parseJsonNumberProperty(*camera, "distanceScale").value_or(cameraDistanceScale),
        0.75,
        4.0));
    cameraBallCamEnabled =
        parseJsonBoolProperty(*camera, "ballCam").value_or(cameraBallCamEnabled);
    if (const auto customSettings = parseJsonObjectProperty(*camera, "customSettings")) {
      cameraCustomSettingsEnabled = true;
      cameraCustomFov = static_cast<float>(std::clamp(
          parseJsonNumberProperty(*customSettings, "fov").value_or(cameraCustomFov),
          60.0,
          130.0));
      cameraCustomHeight = static_cast<float>(std::clamp(
          parseJsonNumberProperty(*customSettings, "height").value_or(cameraCustomHeight),
          40.0,
          250.0));
      cameraCustomPitch = static_cast<float>(std::clamp(
          parseJsonNumberProperty(*customSettings, "pitch").value_or(cameraCustomPitch),
          -30.0,
          30.0));
      cameraCustomDistance = static_cast<float>(std::clamp(
          parseJsonNumberProperty(*customSettings, "distance").value_or(cameraCustomDistance),
          100.0,
          500.0));
      cameraCustomStiffness = static_cast<float>(std::clamp(
          parseJsonNumberProperty(*customSettings, "stiffness").value_or(cameraCustomStiffness),
          0.0,
          1.0));
      cameraCustomSwivelSpeed = static_cast<float>(std::clamp(
          parseJsonNumberProperty(*customSettings, "swivelSpeed")
              .value_or(cameraCustomSwivelSpeed),
          1.0,
          10.0));
      cameraCustomTransitionSpeed = static_cast<float>(std::clamp(
          parseJsonNumberProperty(*customSettings, "transitionSpeed")
              .value_or(cameraCustomTransitionSpeed),
          0.5,
          2.0));
    } else if (camera->find("\"customSettings\":null") != std::string::npos) {
      cameraCustomSettingsEnabled = false;
    }
  }
  if (const auto recording = parseJsonObjectProperty(json, "recording")) {
    recordingFps = static_cast<int>(
        std::clamp(parseJsonNumberProperty(*recording, "fps").value_or(recordingFps), 1.0, 120.0));
    if (const auto playbackRate = parseJsonNumberProperty(*recording, "playbackRate")) {
      recordingPlaybackRateIndex = recordingPlaybackRateIndexForValue(*playbackRate);
    }
  }
  if (const auto playback = parseJsonObjectProperty(json, "playback")) {
    playbackCurrentTime = static_cast<float>(std::max(
        0.0,
        parseJsonNumberProperty(*playback, "currentTime").value_or(playbackCurrentTime)));
    playbackPlaying = parseJsonBoolProperty(*playback, "playing").value_or(playbackPlaying);
    playbackRate = static_cast<float>(
        std::clamp(parseJsonNumberProperty(*playback, "rate").value_or(playbackRate), 0.1, 4.0));
    playbackSkipPostGoalTransitions =
        parseJsonBoolProperty(*playback, "skipPostGoalTransitions")
            .value_or(playbackSkipPostGoalTransitions);
    playbackSkipKickoffs =
        parseJsonBoolProperty(*playback, "skipKickoffs").value_or(playbackSkipKickoffs);
  }
  touchControlsMode = static_cast<int>(
      std::clamp(
          parseJsonNumberProperty(json, "touch_controls_mode").value_or(touchControlsMode),
          0.0,
          1.0));
  touchMarkerDecaySeconds = static_cast<float>(std::clamp(
      parseJsonNumberProperty(json, "touch_marker_decay_seconds").value_or(touchMarkerDecaySeconds),
      1.0,
      10.0));
  touchBreakdownKind =
      parseJsonBoolProperty(json, "touch_breakdown_kind").value_or(touchBreakdownKind);
  touchBreakdownHeight =
      parseJsonBoolProperty(json, "touch_breakdown_height").value_or(touchBreakdownHeight);
  touchBreakdownSurface =
      parseJsonBoolProperty(json, "touch_breakdown_surface").value_or(touchBreakdownSurface);
  touchBreakdownDodge =
      parseJsonBoolProperty(json, "touch_breakdown_dodge").value_or(touchBreakdownDodge);
  movementBreakdownSpeed =
      parseJsonBoolProperty(json, "movement_breakdown_speed").value_or(movementBreakdownSpeed);
  movementBreakdownHeight =
      parseJsonBoolProperty(json, "movement_breakdown_height").value_or(movementBreakdownHeight);
  possessionBreakdownState = parseJsonBoolProperty(json, "possession_breakdown_state")
                                 .value_or(possessionBreakdownState);
  possessionBreakdownThird = parseJsonBoolProperty(json, "possession_breakdown_third")
                                 .value_or(possessionBreakdownThird);
  boostPickupPadBig =
      parseJsonBoolProperty(json, "boost_pickup_pad_big").value_or(boostPickupPadBig);
  boostPickupPadSmall =
      parseJsonBoolProperty(json, "boost_pickup_pad_small").value_or(boostPickupPadSmall);
  boostPickupPadAmbiguous =
      parseJsonBoolProperty(json, "boost_pickup_pad_ambiguous").value_or(boostPickupPadAmbiguous);
  boostPickupAnimationEnabled = parseJsonBoolProperty(json, "boost_pickup_animation_enabled")
                                    .value_or(boostPickupAnimationEnabled);
  boostPickupActivityActive = parseJsonBoolProperty(json, "boost_pickup_activity_active")
                                  .value_or(boostPickupActivityActive);
  boostPickupActivityInactive = parseJsonBoolProperty(json, "boost_pickup_activity_inactive")
                                    .value_or(boostPickupActivityInactive);
  boostPickupActivityUnknown = parseJsonBoolProperty(json, "boost_pickup_activity_unknown")
                                   .value_or(boostPickupActivityUnknown);
  boostPickupFieldOwn =
      parseJsonBoolProperty(json, "boost_pickup_field_own").value_or(boostPickupFieldOwn);
  boostPickupFieldOpponent = parseJsonBoolProperty(json, "boost_pickup_field_opponent")
                                 .value_or(boostPickupFieldOpponent);
  boostPickupFieldUnknown = parseJsonBoolProperty(json, "boost_pickup_field_unknown")
                                .value_or(boostPickupFieldUnknown);
  if (const auto moduleConfigs = parseJsonObjectProperty(json, "moduleConfigs")) {
    std::optional<std::string> boostConfig = parseJsonObjectProperty(*moduleConfigs, "boost");
    if (!boostConfig) {
      boostConfig = parseJsonObjectProperty(*moduleConfigs, "boost-pickup-animation");
    }
    if (boostConfig) {
      const std::vector<std::string> padTypes =
          parseJsonStringArrayProperty(*boostConfig, "padTypes");
      if (jsonPropertyExists(*boostConfig, "padTypes")) {
        boostPickupPadBig = containsString(padTypes, "big");
        boostPickupPadSmall = containsString(padTypes, "small");
        boostPickupPadAmbiguous = containsString(padTypes, "ambiguous");
      }
      const std::vector<std::string> activities =
          parseJsonStringArrayProperty(*boostConfig, "activities");
      if (jsonPropertyExists(*boostConfig, "activities")) {
        boostPickupActivityActive = containsString(activities, "active");
        boostPickupActivityInactive = containsString(activities, "inactive");
        boostPickupActivityUnknown = containsString(activities, "unknown");
      }
      const std::vector<std::string> fieldHalves =
          parseJsonStringArrayProperty(*boostConfig, "fieldHalves");
      if (jsonPropertyExists(*boostConfig, "fieldHalves")) {
        boostPickupFieldOwn = containsString(fieldHalves, "own");
        boostPickupFieldOpponent = containsString(fieldHalves, "opponent");
        boostPickupFieldUnknown = containsString(fieldHalves, "unknown");
      }
    }
    if (const auto touchConfig = parseJsonObjectProperty(*moduleConfigs, "touch")) {
      touchMarkerDecaySeconds = static_cast<float>(std::clamp(
          parseJsonNumberProperty(*touchConfig, "decaySeconds").value_or(touchMarkerDecaySeconds),
          1.0,
          10.0));
      if (const auto overlayMode = parseJsonStringProperty(*touchConfig, "overlayMode")) {
        if (*overlayMode == "markers") {
          touchControlsMode = 0;
        } else if (*overlayMode == "advancement") {
          touchControlsMode = 1;
        }
      }
      const std::vector<std::string> breakdownClasses =
          parseJsonStringArrayProperty(*touchConfig, "breakdownClasses");
      if (jsonPropertyExists(*touchConfig, "breakdownClasses")) {
        touchBreakdownKind = containsString(breakdownClasses, "kind");
        touchBreakdownHeight = containsString(breakdownClasses, "height_band");
        touchBreakdownSurface = containsString(breakdownClasses, "surface");
        touchBreakdownDodge = containsString(breakdownClasses, "dodge_state");
      }
    }
    if (const auto movementConfig = parseJsonObjectProperty(*moduleConfigs, "movement")) {
      const std::vector<std::string> breakdownClasses =
          parseJsonStringArrayProperty(*movementConfig, "breakdownClasses");
      if (jsonPropertyExists(*movementConfig, "breakdownClasses")) {
        movementBreakdownSpeed = containsString(breakdownClasses, "speed_band");
        movementBreakdownHeight = containsString(breakdownClasses, "height_band");
      }
    }
    if (const auto possessionConfig = parseJsonObjectProperty(*moduleConfigs, "possession")) {
      const std::vector<std::string> breakdownClasses =
          parseJsonStringArrayProperty(*possessionConfig, "breakdownClasses");
      if (jsonPropertyExists(*possessionConfig, "breakdownClasses")) {
        possessionBreakdownState = containsString(breakdownClasses, "possession_state");
        possessionBreakdownThird = containsString(breakdownClasses, "field_third");
      }
    }
  }
  graphInspectorView = static_cast<int>(
      std::max(0.0, parseJsonNumberProperty(json, "graph_inspector_view").value_or(0.0)));
  mechanicsReviewIndex = static_cast<int>(
      std::max(0.0, parseJsonNumberProperty(json, "mechanics_review_index").value_or(0.0)));
  mechanicsReviewClipLeadSeconds = static_cast<float>(std::clamp(
      parseJsonNumberProperty(json, "mechanics_review_clip_lead_seconds")
          .value_or(mechanicsReviewClipLeadSeconds),
      0.0,
      10.0));
  mechanicsReviewClipTrailSeconds = static_cast<float>(std::clamp(
      parseJsonNumberProperty(json, "mechanics_review_clip_trail_seconds")
          .value_or(mechanicsReviewClipTrailSeconds),
      0.0,
      10.0));
  selectedGraphOutput = parseJsonStringProperty(json, "selected_graph_output").value_or("");
  selectedAnalysisNode = parseJsonStringProperty(json, "selected_analysis_node").value_or("");
  graphInspectorNodeQuery =
      parseJsonStringProperty(json, "graph_inspector_node_query").value_or("");

  if (const auto placements = parseJsonObjectProperty(json, "placements")) {
    loadPlacement(*placements, "launcher", launcherPlacement, &uiLauncherOpen);
    loadPlacement(*placements, "scoreboard", scoreboardPlacement, &uiScoreboardOpen);
    loadPlacement(*placements, "events", eventsPlacement, &uiEventsOpen);
    loadPlacement(*placements, "status", statusPlacement, &uiStatusOpen);
    loadPlacement(*placements, "camera", cameraPlacement, &uiCameraOpen);
    loadPlacement(
        *placements,
        "playback_controls",
        playbackControlsPlacement,
        &uiPlaybackControlsOpen);
    loadPlacement(*placements, "recording", recordingPlacement, &uiRecordingOpen);
    loadPlacement(*placements, "graph_inspector", graphInspectorPlacement, &uiGraphInspectorOpen);
    loadPlacement(*placements, "event_playlist", eventPlaylistPlacement, &uiEventPlaylistOpen);
    loadPlacement(
        *placements,
        "mechanics_review",
        mechanicsReviewPlacement,
        &uiMechanicsReviewOpen);
    loadPlacement(*placements, "replay_loading", replayLoadingPlacement, &uiReplayLoadingOpen);
    loadPlacement(*placements, "module_controls", moduleControlsPlacement, &uiModuleControlsOpen);
    loadPlacement(*placements, "touch_controls", touchControlsPlacement, &uiTouchControlsOpen);
    loadPlacement(
        *placements,
        "boost_pickup_controls",
        boostPickupControlsPlacement,
        &uiBoostPickupControlsOpen);
  }
  for (const std::string &object : parseJsonObjectArrayProperty(json, "singletonWindows")) {
    const std::string id = parseJsonStringProperty(object, "id").value_or("");
    const auto placement = parseJsonObjectProperty(object, "placement");
    if (!placement) {
      continue;
    }
    if (id == "camera") {
      loadPlacementObject(*placement, cameraPlacement, &uiCameraOpen);
    } else if (id == "scoreboard") {
      loadPlacementObject(*placement, scoreboardPlacement, &uiScoreboardOpen);
    } else if (id == "playback") {
      loadPlacementObject(*placement, playbackControlsPlacement, &uiPlaybackControlsOpen);
    } else if (id == "recording") {
      loadPlacementObject(*placement, recordingPlacement, &uiRecordingOpen);
    } else if (id == "mechanics") {
      loadPlacementObject(*placement, eventsPlacement, &uiEventsOpen);
    } else if (id == "event-playlist") {
      loadPlacementObject(*placement, eventPlaylistPlacement, &uiEventPlaylistOpen);
    } else if (id == "mechanics-review") {
      loadPlacementObject(*placement, mechanicsReviewPlacement, &uiMechanicsReviewOpen);
    } else if (id == "replay-loading") {
      loadPlacementObject(*placement, replayLoadingPlacement, &uiReplayLoadingOpen);
    } else if (id == "boost-pickups") {
      loadPlacementObject(*placement, boostPickupControlsPlacement, &uiBoostPickupControlsOpen);
    } else if (id == "touch-controls") {
      loadPlacementObject(*placement, touchControlsPlacement, &uiTouchControlsOpen);
    }
  }

  uiStatsWindows.clear();
  nextUiStatsWindowId = 1;
  std::vector<std::string> statsWindowObjects =
      parseJsonObjectArrayProperty(json, "stats_windows");
  if (statsWindowObjects.empty()) {
    statsWindowObjects = parseJsonObjectArrayProperty(json, "statsWindows");
  }
  for (const std::string &object : statsWindowObjects) {
    const auto kind = parseJsonStringProperty(object, "kind");
    if (!kind) {
      continue;
    }

    UiStatsWindow window{};
    if (*kind == "player") {
      window.kind = UiStatsWindowKind::Player;
    } else if (*kind == "team") {
      window.kind = UiStatsWindowKind::Team;
    } else if (*kind == "all-players") {
      window.kind = UiStatsWindowKind::AllPlayers;
    } else if (*kind == "all-teams") {
      window.kind = UiStatsWindowKind::AllTeams;
    } else if (*kind == "goals-overview") {
      window.kind = UiStatsWindowKind::GoalsOverview;
    } else if (*kind == "ad-hoc") {
      window.kind = UiStatsWindowKind::AdHoc;
    } else if (*kind == "stats-module") {
      window.kind = UiStatsWindowKind::StatsModule;
    } else {
      continue;
    }

    window.id = static_cast<uint32_t>(parseJsonNumberProperty(object, "id").value_or(0.0));
    if (const auto idString = parseJsonStringProperty(object, "id")) {
      const size_t digitOffset = idString->find_first_of("0123456789");
      if (digitOffset != std::string::npos) {
        try {
          window.id = static_cast<uint32_t>(std::stoul(idString->substr(digitOffset)));
        } catch (const std::exception &) {
          window.id = 0;
        }
      }
    }
    if (window.id == 0) {
      window.id = nextUiStatsWindowId;
    }
    nextUiStatsWindowId = std::max(nextUiStatsWindowId, window.id + 1);
    window.open = parseJsonBoolProperty(object, "open").value_or(true);
    window.open = parseJsonBoolProperty(object, "visible").value_or(window.open);
    if (const auto placement = parseJsonObjectProperty(object, "placement")) {
      window.open = parseJsonBoolProperty(*placement, "visible").value_or(window.open);
      window.has_placement = true;
      window.pending_apply_placement = true;
      window.x = static_cast<float>(parseJsonNumberProperty(*placement, "x").value_or(window.x));
      window.y = static_cast<float>(parseJsonNumberProperty(*placement, "y").value_or(window.y));
      window.width =
          static_cast<float>(parseJsonNumberProperty(*placement, "width").value_or(window.width));
      window.height =
          static_cast<float>(parseJsonNumberProperty(*placement, "height").value_or(window.height));
      window.viewport_width = static_cast<float>(
          parseJsonNumberProperty(*placement, "viewport_width").value_or(window.viewport_width));
      window.viewport_height = static_cast<float>(
          parseJsonNumberProperty(*placement, "viewport_height").value_or(window.viewport_height));
      if (const auto viewport = parseJsonObjectProperty(*placement, "viewport")) {
        window.viewport_width = static_cast<float>(
            parseJsonNumberProperty(*viewport, "width").value_or(window.viewport_width));
        window.viewport_height = static_cast<float>(
            parseJsonNumberProperty(*viewport, "height").value_or(window.viewport_height));
      }
      window.z_index =
          static_cast<int>(parseJsonNumberProperty(*placement, "zIndex").value_or(window.z_index));
    }
    window.selected_player_index = static_cast<uint32_t>(
        std::max(0.0, parseJsonNumberProperty(object, "selected_player_index").value_or(0.0)));
    if (const auto playerId = parseJsonStringProperty(object, "playerId")) {
      if (const auto parsedPlayerIndex = parseUnsignedIntegerString(*playerId)) {
        window.selected_player_index = *parsedPlayerIndex;
      }
    }
    window.selected_team_is_team_0 =
        parseJsonBoolProperty(object, "selected_team_is_team_0").value_or(true) ? 1 : 0;
    if (const auto team = parseJsonStringProperty(object, "team")) {
      if (*team == "blue") {
        window.selected_team_is_team_0 = 1;
      } else if (*team == "orange") {
        window.selected_team_is_team_0 = 0;
      }
    }
    window.module_name = parseJsonStringProperty(object, "module_name").value_or("");
    window.module_view = static_cast<int>(
        std::max(0.0, parseJsonNumberProperty(object, "module_view").value_or(0.0)));
    window.picker_query = parseJsonStringProperty(object, "picker_query").value_or("");
    const bool hasEntriesProperty = object.find("\"entries\"") != std::string::npos;
    for (const std::string &statId : parseJsonStringArrayProperty(object, "entries")) {
      window.entries.push_back(UiStatsWindow::Entry{normalizeUiStatId(statId), ""});
    }
    for (const std::string &entryObject : parseJsonObjectArrayProperty(object, "entries")) {
      std::string statId = parseJsonStringProperty(entryObject, "stat_id").value_or("");
      if (statId.empty()) {
        statId = parseJsonStringProperty(entryObject, "statId").value_or("");
      }
      if (statId.empty()) {
        continue;
      }
      statId = normalizeUiStatId(statId);
      std::string targetId = parseJsonStringProperty(entryObject, "target_id").value_or("");
      if (targetId.empty()) {
        targetId = parseJsonStringProperty(entryObject, "targetId").value_or("");
      }
      window.entries.push_back(UiStatsWindow::Entry{statId, targetId});
    }
    if (!hasEntriesProperty && window.entries.empty() &&
        window.kind != UiStatsWindowKind::StatsModule) {
      initializeStatsWindowEntries(window);
    }
    window.has_placement =
        parseJsonBoolProperty(object, "has_placement").value_or(window.has_placement);
    window.pending_apply_placement = window.has_placement;
    window.x = static_cast<float>(parseJsonNumberProperty(object, "x").value_or(window.x));
    window.y = static_cast<float>(parseJsonNumberProperty(object, "y").value_or(window.y));
    window.width =
        static_cast<float>(parseJsonNumberProperty(object, "width").value_or(window.width));
    window.height =
        static_cast<float>(parseJsonNumberProperty(object, "height").value_or(window.height));
    window.viewport_width = static_cast<float>(
        parseJsonNumberProperty(object, "viewport_width").value_or(window.viewport_width));
    window.viewport_height = static_cast<float>(
        parseJsonNumberProperty(object, "viewport_height").value_or(window.viewport_height));
    window.z_index =
        static_cast<int>(parseJsonNumberProperty(object, "zIndex").value_or(window.z_index));
    nextUiWindowZIndex = std::max(nextUiWindowZIndex, window.z_index + 1);
    uiStatsWindows.push_back(std::move(window));
  }

  if (!uiStatsWindows.empty()) {
    cvarManager->log(std::format(
        "subtr-actor: loaded {} UI stats windows from {}",
        uiStatsWindows.size(),
        sourceLabel));
  }
  focusTopLoadedWindow();
  lastSavedUiConfigJson = uiConfigJson();
  nextUiConfigAutosave = std::chrono::steady_clock::now() + std::chrono::seconds(2);
}

std::string SubtrActorPlugin::uiConfigJson() const {
  auto kindValue = [](UiStatsWindowKind kind) {
    switch (kind) {
    case UiStatsWindowKind::Player:
      return "player";
    case UiStatsWindowKind::Team:
      return "team";
    case UiStatsWindowKind::AllPlayers:
      return "all-players";
    case UiStatsWindowKind::AllTeams:
      return "all-teams";
    case UiStatsWindowKind::GoalsOverview:
      return "goals-overview";
    case UiStatsWindowKind::AdHoc:
      return "ad-hoc";
    case UiStatsWindowKind::StatsModule:
      return "stats-module";
    default:
      return "player";
    }
  };

  auto writePlacement = [](
                            std::ostream &out,
                            const UiWindowPlacement &placement,
                            bool visible) {
    out << "{\"has_placement\":" << (placement.has_placement ? "true" : "false")
        << ",\"visible\":" << (visible ? "true" : "false")
        << ",\"x\":" << placement.x << ",\"y\":" << placement.y
        << ",\"width\":" << placement.width << ",\"height\":" << placement.height
        << ",\"viewport_width\":" << placement.viewport_width
        << ",\"viewport\":{\"width\":" << placement.viewport_width
        << ",\"height\":" << placement.viewport_height << "}"
        << ",\"viewport_height\":" << placement.viewport_height
        << ",\"zIndex\":" << placement.z_index << "}";
  };
  auto writeEnabledStringArray =
      [](std::ostream &out, std::initializer_list<std::pair<const char *, bool>> values) {
        out << "[";
        bool wroteValue = false;
        for (const auto &[value, enabled] : values) {
          if (!enabled) {
            continue;
          }
          if (wroteValue) {
            out << ",";
          }
          out << "\"" << value << "\"";
          wroteValue = true;
        }
        out << "]";
      };

  std::ostringstream file;
  file << "{\n";
  file << "  \"version\": 1,\n";
  file << "  \"launcher_open\": " << (uiLauncherOpen ? "true" : "false") << ",\n";
  file << "  \"scoreboard_open\": " << (uiScoreboardOpen ? "true" : "false") << ",\n";
  file << "  \"events_open\": " << (uiEventsOpen ? "true" : "false") << ",\n";
  file << "  \"status_open\": " << (uiStatusOpen ? "true" : "false") << ",\n";
  file << "  \"camera_open\": " << (uiCameraOpen ? "true" : "false") << ",\n";
  file << "  \"playback_controls_open\": "
       << (uiPlaybackControlsOpen ? "true" : "false") << ",\n";
  file << "  \"recording_open\": " << (uiRecordingOpen ? "true" : "false") << ",\n";
  file << "  \"graph_inspector_open\": " << (uiGraphInspectorOpen ? "true" : "false")
       << ",\n";
  file << "  \"event_playlist_open\": " << (uiEventPlaylistOpen ? "true" : "false")
       << ",\n";
  file << "  \"mechanics_review_open\": " << (uiMechanicsReviewOpen ? "true" : "false")
       << ",\n";
  file << "  \"replay_loading_open\": " << (uiReplayLoadingOpen ? "true" : "false")
       << ",\n";
  file << "  \"module_controls_open\": " << (uiModuleControlsOpen ? "true" : "false")
       << ",\n";
  file << "  \"touch_controls_open\": " << (uiTouchControlsOpen ? "true" : "false")
       << ",\n";
  file << "  \"boost_pickup_controls_open\": "
       << (uiBoostPickupControlsOpen ? "true" : "false") << ",\n";
  file << "  \"event_playlist_mechanics_enabled\": "
       << (eventPlaylistMechanicsEnabled ? "true" : "false") << ",\n";
  file << "  \"event_playlist_team_enabled\": "
       << (eventPlaylistTeamEventsEnabled ? "true" : "false") << ",\n";
  file << "  \"event_playlist_goal_context_enabled\": "
       << (eventPlaylistGoalContextEnabled ? "true" : "false") << ",\n";
  file << "  \"event_playlist_auto_follow\": "
       << (eventPlaylistAutoFollow ? "true" : "false") << ",\n";
  file << "  \"overlays\": {\n";
  const std::string currentEventFilter = cvarString("subtr_actor_overlay_event_types", "all");
  const std::vector<std::string> currentEventFilterTokens =
      selectedEventSourceTokens(currentEventFilter);
  file << "    \"timelineEvents\": [";
  bool wroteOverlayValue = false;
  auto writeOverlayId = [&](const char *id, bool enabled) {
    if (!enabled) {
      return;
    }
    if (wroteOverlayValue) {
      file << ",";
    }
    file << "\"" << id << "\"";
    wroteOverlayValue = true;
  };
  writeOverlayId("mechanics", eventPlaylistMechanicsEnabled);
  writeOverlayId("team", eventPlaylistTeamEventsEnabled);
  writeOverlayId("goal_context", eventPlaylistGoalContextEnabled);
  if (!allEventSourcesSelected(currentEventFilter)) {
    for (const std::string &token : currentEventFilterTokens) {
      if (token == "mechanics" || token == "team" || token == "goal_context" ||
          isMechanicFilterToken(token)) {
        continue;
      }
      writeOverlayId(token.c_str(), true);
    }
  }
  file << "],\n";
  file << "    \"timelineRanges\": [";
  wroteOverlayValue = false;
  writeOverlayId("boost", timelineRangeBoostEnabled);
  writeOverlayId("possession", timelineRangePossessionEnabled);
  writeOverlayId("pressure", timelineRangePressureEnabled);
  writeOverlayId("rush", timelineRangeRushEnabled);
  writeOverlayId("absolute-positioning", timelineRangeAbsolutePositioningEnabled);
  file << "],\n";
  file << "    \"mechanics\": [";
  const bool allMechanicsSelected =
      allEventSourcesSelected(currentEventFilter) ||
      containsString(currentEventFilterTokens, "mechanics");
  bool wroteMechanicFilter = false;
  if (!allMechanicsSelected) {
    for (const std::string &token : currentEventFilterTokens) {
      if (!isMechanicFilterToken(token)) {
        continue;
      }
      if (wroteMechanicFilter) {
        file << ",";
      }
      file << "\"" << escapeJsonString(token) << "\"";
      wroteMechanicFilter = true;
    }
  }
  file << "],\n";
  file << "    \"renderEffects\": [";
  wroteOverlayValue = false;
  const bool hudOverlayEnabled = cvarBool("subtr_actor_overlay_enabled", true);
  writeOverlayId(
      "mechanics",
      hudOverlayEnabled && cvarBool("subtr_actor_overlay_mechanics_enabled", true));
  writeOverlayId(
      "team",
      hudOverlayEnabled && cvarBool("subtr_actor_overlay_team_events_enabled", true));
  writeOverlayId(
      "goal_context",
      hudOverlayEnabled && cvarBool("subtr_actor_overlay_goal_context_enabled", true));
  writeOverlayId("ceiling-shot", hudOverlayEnabled && renderEffectCeilingShotEnabled);
  writeOverlayId("fifty-fifty", hudOverlayEnabled && renderEffectFiftyFiftyEnabled);
  writeOverlayId("pressure", hudOverlayEnabled && renderEffectPressureEnabled);
  writeOverlayId(
      "relative-positioning",
      hudOverlayEnabled && renderEffectRelativePositioningEnabled);
  writeOverlayId(
      "absolute-positioning",
      hudOverlayEnabled && renderEffectAbsolutePositioningEnabled);
  writeOverlayId("speed-flip", hudOverlayEnabled && renderEffectSpeedFlipEnabled);
  writeOverlayId("touch", hudOverlayEnabled && renderEffectTouchEnabled);
  file << "],\n";
  file << "    \"followedPlayerHud\": false,\n";
  file << "    \"boostPads\": "
       << (boostPickupPadBig || boostPickupPadSmall || boostPickupPadAmbiguous ? "true" : "false")
       << ",\n";
  file << "    \"boostPickupAnimation\": "
       << (boostPickupAnimationEnabled ? "true" : "false") << "\n";
  file << "  },\n";
  const bool hasReplayServerForPlayback = gameWrapper && gameWrapper->IsInReplay() &&
                                          !gameWrapper->GetGameEventAsReplay().IsNull();
  const float currentPlaybackTime =
      hasReplayServerForPlayback ? gameWrapper->GetGameEventAsReplay().GetReplayTimeElapsed()
                                 : playbackCurrentTime;
  file << "  \"playback\": {";
  file << "\"currentTime\":" << currentPlaybackTime
       << ",\"playing\":" << (playbackPlaying ? "true" : "false")
       << ",\"rate\":" << playbackRate
       << ",\"skipPostGoalTransitions\":"
       << (playbackSkipPostGoalTransitions ? "true" : "false")
       << ",\"skipKickoffs\":" << (playbackSkipKickoffs ? "true" : "false")
       << "},\n";
  file << "  \"camera\": {";
  file << "\"mode\":\"" << (cameraViewMode == 1 ? "follow" : "free") << "\"";
  file << ",\"freePreset\":";
  if (cameraFreePreset == 0 || cameraViewMode == 2) {
    file << "\"overhead\"";
  } else if (cameraFreePreset == 1 || cameraViewMode == 3) {
    file << "\"side\"";
  } else {
    file << "null";
  }
  file << ",\"attachedPlayerId\":";
  if (cameraViewMode == 1) {
    file << "\"" << cameraSelectedPlayerIndex << "\"";
  } else {
    file << "null";
  }
  file << ",\"distanceScale\":" << cameraDistanceScale
       << ",\"ballCam\":" << (cameraBallCamEnabled ? "true" : "false")
       << ",\"customSettings\":";
  if (cameraCustomSettingsEnabled) {
    file << "{\"fov\":" << cameraCustomFov << ",\"height\":" << cameraCustomHeight
         << ",\"pitch\":" << cameraCustomPitch << ",\"distance\":" << cameraCustomDistance
         << ",\"stiffness\":" << cameraCustomStiffness
         << ",\"swivelSpeed\":" << cameraCustomSwivelSpeed
         << ",\"transitionSpeed\":" << cameraCustomTransitionSpeed << "}";
  } else {
    file << "null";
  }
  file << "},\n";
  file << "  \"recording\": {\"fps\":" << recordingFps
       << ",\"playbackRate\":" << recordingPlaybackRateFromIndex(recordingPlaybackRateIndex)
       << "},\n";
  file << "  \"moduleConfigs\": {\n";
  file << "    \"boost\": {\"padTypes\":";
  writeEnabledStringArray(
      file,
      {{"big", boostPickupPadBig},
       {"small", boostPickupPadSmall},
       {"ambiguous", boostPickupPadAmbiguous}});
  file << ",\"comparisons\":[\"both\"],\"activities\":";
  writeEnabledStringArray(
      file,
      {{"active", boostPickupActivityActive},
       {"inactive", boostPickupActivityInactive},
       {"unknown", boostPickupActivityUnknown}});
  file << ",\"fieldHalves\":";
  writeEnabledStringArray(
      file,
      {{"own", boostPickupFieldOwn},
       {"opponent", boostPickupFieldOpponent},
       {"unknown", boostPickupFieldUnknown}});
  file << ",\"playerIds\":null},\n";
  file << "    \"touch\": {\"decaySeconds\":" << touchMarkerDecaySeconds
       << ",\"overlayMode\":\"" << (touchControlsMode == 0 ? "markers" : "advancement")
       << "\",\"breakdownClasses\":";
  writeEnabledStringArray(
      file,
      {{"kind", touchBreakdownKind},
       {"height_band", touchBreakdownHeight},
       {"surface", touchBreakdownSurface},
       {"dodge_state", touchBreakdownDodge}});
  file << "},\n";
  file << "    \"movement\": {\"breakdownClasses\":";
  writeEnabledStringArray(
      file,
      {{"speed_band", movementBreakdownSpeed},
       {"height_band", movementBreakdownHeight}});
  file << "},\n";
  file << "    \"possession\": {\"breakdownClasses\":";
  writeEnabledStringArray(
      file,
      {{"possession_state", possessionBreakdownState},
       {"field_third", possessionBreakdownThird}});
  file << "}\n";
  file << "  },\n";
  file << "  \"camera_view_mode\": " << cameraViewMode << ",\n";
  file << "  \"camera_free_preset\": " << cameraFreePreset << ",\n";
  file << "  \"camera_selected_player_index\": " << cameraSelectedPlayerIndex << ",\n";
  file << "  \"camera_distance_scale\": " << cameraDistanceScale << ",\n";
  file << "  \"camera_custom_settings_enabled\": "
       << (cameraCustomSettingsEnabled ? "true" : "false") << ",\n";
  file << "  \"camera_ball_cam_enabled\": " << (cameraBallCamEnabled ? "true" : "false")
       << ",\n";
  file << "  \"camera_custom_fov\": " << cameraCustomFov << ",\n";
  file << "  \"camera_custom_height\": " << cameraCustomHeight << ",\n";
  file << "  \"camera_custom_pitch\": " << cameraCustomPitch << ",\n";
  file << "  \"camera_custom_distance\": " << cameraCustomDistance << ",\n";
  file << "  \"camera_custom_stiffness\": " << cameraCustomStiffness << ",\n";
  file << "  \"camera_custom_swivel_speed\": " << cameraCustomSwivelSpeed << ",\n";
  file << "  \"camera_custom_transition_speed\": " << cameraCustomTransitionSpeed << ",\n";
  file << "  \"recording_fps\": " << recordingFps << ",\n";
  file << "  \"recording_playback_rate_index\": " << recordingPlaybackRateIndex << ",\n";
  file << "  \"recording_finish_before_dump\": "
       << (recordingFinishBeforeDump ? "true" : "false") << ",\n";
  file << "  \"touch_controls_mode\": " << touchControlsMode << ",\n";
  file << "  \"touch_marker_decay_seconds\": " << touchMarkerDecaySeconds << ",\n";
  file << "  \"touch_breakdown_kind\": " << (touchBreakdownKind ? "true" : "false")
       << ",\n";
  file << "  \"touch_breakdown_height\": " << (touchBreakdownHeight ? "true" : "false")
       << ",\n";
  file << "  \"touch_breakdown_surface\": " << (touchBreakdownSurface ? "true" : "false")
       << ",\n";
  file << "  \"touch_breakdown_dodge\": " << (touchBreakdownDodge ? "true" : "false")
       << ",\n";
  file << "  \"movement_breakdown_speed\": " << (movementBreakdownSpeed ? "true" : "false")
       << ",\n";
  file << "  \"movement_breakdown_height\": " << (movementBreakdownHeight ? "true" : "false")
       << ",\n";
  file << "  \"possession_breakdown_state\": " << (possessionBreakdownState ? "true" : "false")
       << ",\n";
  file << "  \"possession_breakdown_third\": " << (possessionBreakdownThird ? "true" : "false")
       << ",\n";
  file << "  \"boost_pickup_pad_big\": " << (boostPickupPadBig ? "true" : "false")
       << ",\n";
  file << "  \"boost_pickup_pad_small\": " << (boostPickupPadSmall ? "true" : "false")
       << ",\n";
  file << "  \"boost_pickup_pad_ambiguous\": "
       << (boostPickupPadAmbiguous ? "true" : "false") << ",\n";
  file << "  \"boost_pickup_animation_enabled\": "
       << (boostPickupAnimationEnabled ? "true" : "false") << ",\n";
  file << "  \"boost_pickup_activity_active\": "
       << (boostPickupActivityActive ? "true" : "false") << ",\n";
  file << "  \"boost_pickup_activity_inactive\": "
       << (boostPickupActivityInactive ? "true" : "false") << ",\n";
  file << "  \"boost_pickup_activity_unknown\": "
       << (boostPickupActivityUnknown ? "true" : "false") << ",\n";
  file << "  \"boost_pickup_field_own\": " << (boostPickupFieldOwn ? "true" : "false")
       << ",\n";
  file << "  \"boost_pickup_field_opponent\": "
       << (boostPickupFieldOpponent ? "true" : "false") << ",\n";
  file << "  \"boost_pickup_field_unknown\": "
       << (boostPickupFieldUnknown ? "true" : "false") << ",\n";
  file << "  \"graph_inspector_view\": " << graphInspectorView << ",\n";
  file << "  \"mechanics_review_index\": " << mechanicsReviewIndex << ",\n";
  file << "  \"mechanics_review_clip_lead_seconds\": " << mechanicsReviewClipLeadSeconds << ",\n";
  file << "  \"mechanics_review_clip_trail_seconds\": " << mechanicsReviewClipTrailSeconds << ",\n";
  file << "  \"selected_graph_output\": \"" << escapeJsonString(selectedGraphOutput)
       << "\",\n";
  file << "  \"selected_analysis_node\": \"" << escapeJsonString(selectedAnalysisNode)
       << "\",\n";
  file << "  \"graph_inspector_node_query\": \""
       << escapeJsonString(graphInspectorNodeQuery) << "\",\n";
  file << "  \"placements\": {\n";
  file << "    \"launcher\": ";
  writePlacement(file, launcherPlacement, uiLauncherOpen);
  file << ",\n    \"scoreboard\": ";
  writePlacement(file, scoreboardPlacement, uiScoreboardOpen);
  file << ",\n    \"events\": ";
  writePlacement(file, eventsPlacement, uiEventsOpen);
  file << ",\n    \"status\": ";
  writePlacement(file, statusPlacement, uiStatusOpen);
  file << ",\n    \"camera\": ";
  writePlacement(file, cameraPlacement, uiCameraOpen);
  file << ",\n    \"playback_controls\": ";
  writePlacement(file, playbackControlsPlacement, uiPlaybackControlsOpen);
  file << ",\n    \"recording\": ";
  writePlacement(file, recordingPlacement, uiRecordingOpen);
  file << ",\n    \"graph_inspector\": ";
  writePlacement(file, graphInspectorPlacement, uiGraphInspectorOpen);
  file << ",\n    \"event_playlist\": ";
  writePlacement(file, eventPlaylistPlacement, uiEventPlaylistOpen);
  file << ",\n    \"mechanics_review\": ";
  writePlacement(file, mechanicsReviewPlacement, uiMechanicsReviewOpen);
  file << ",\n    \"replay_loading\": ";
  writePlacement(file, replayLoadingPlacement, uiReplayLoadingOpen);
  file << ",\n    \"module_controls\": ";
  writePlacement(file, moduleControlsPlacement, uiModuleControlsOpen);
  file << ",\n    \"touch_controls\": ";
  writePlacement(file, touchControlsPlacement, uiTouchControlsOpen);
  file << ",\n    \"boost_pickup_controls\": ";
  writePlacement(file, boostPickupControlsPlacement, uiBoostPickupControlsOpen);
  file << "\n  },\n";
  file << "  \"singletonWindows\": [\n";
  auto writeSingletonWindow = [&](
                                  const char *id,
                                  const UiWindowPlacement &placement,
                                  bool visible,
                                  bool last) {
    file << "    {\"id\":\"" << id << "\",\"placement\":";
    writePlacement(file, placement, visible);
    file << "}";
    if (!last) {
      file << ",";
    }
    file << "\n";
  };
  writeSingletonWindow("camera", cameraPlacement, uiCameraOpen, false);
  writeSingletonWindow("scoreboard", scoreboardPlacement, uiScoreboardOpen, false);
  writeSingletonWindow("playback", playbackControlsPlacement, uiPlaybackControlsOpen, false);
  writeSingletonWindow("recording", recordingPlacement, uiRecordingOpen, false);
  writeSingletonWindow("mechanics", eventsPlacement, uiEventsOpen, false);
  writeSingletonWindow("event-playlist", eventPlaylistPlacement, uiEventPlaylistOpen, false);
  writeSingletonWindow("mechanics-review", mechanicsReviewPlacement, uiMechanicsReviewOpen, false);
  writeSingletonWindow("replay-loading", replayLoadingPlacement, uiReplayLoadingOpen, false);
  writeSingletonWindow(
      "boost-pickups",
      boostPickupControlsPlacement,
      uiBoostPickupControlsOpen,
      false);
  writeSingletonWindow("touch-controls", touchControlsPlacement, uiTouchControlsOpen, true);
  file << "  ],\n";
  file << "  \"stats_windows\": [\n";
  for (size_t i = 0; i < uiStatsWindows.size(); i += 1) {
    const UiStatsWindow &window = uiStatsWindows[i];
    file << "    {\"id\":" << window.id << ",\"kind\":\"" << kindValue(window.kind)
         << "\",\"open\":" << (window.open ? "true" : "false")
         << ",\"visible\":" << (window.open ? "true" : "false")
         << ",\"placement\":{\"x\":" << window.x << ",\"y\":" << window.y
         << ",\"viewport\":{\"width\":" << window.viewport_width
         << ",\"height\":" << window.viewport_height << "}"
         << ",\"zIndex\":" << window.z_index
         << ",\"visible\":" << (window.open ? "true" : "false") << "}"
         << ",\"selected_player_index\":" << window.selected_player_index
         << ",\"selected_team_is_team_0\":"
         << (window.selected_team_is_team_0 != 0 ? "true" : "false")
         << ",\"module_name\":\"" << escapeJsonString(window.module_name) << "\""
         << ",\"module_view\":" << window.module_view
         << ",\"picker_query\":\"" << escapeJsonString(window.picker_query) << "\""
         << ",\"has_placement\":" << (window.has_placement ? "true" : "false")
         << ",\"x\":" << window.x << ",\"y\":" << window.y
         << ",\"width\":" << window.width << ",\"height\":" << window.height
         << ",\"viewport_width\":" << window.viewport_width
         << ",\"viewport_height\":" << window.viewport_height
         << ",\"zIndex\":" << window.z_index
         << ",\"entries\":[";
    for (size_t j = 0; j < window.entries.size(); j += 1) {
      if (j != 0) {
        file << ",";
      }
      const UiStatsWindow::Entry &entry = window.entries[j];
      file << "{\"stat_id\":\"" << escapeJsonString(entry.stat_id) << "\""
           << ",\"target_id\":\"" << escapeJsonString(entry.target_id) << "\"}";
    }
    file << "]}";
    if (i + 1 != uiStatsWindows.size()) {
      file << ",";
    }
    file << "\n";
  }
  file << "  ],\n";
  file << "  \"statsWindows\": [\n";
  bool wroteWebStatsWindow = false;
  for (size_t i = 0; i < uiStatsWindows.size(); i += 1) {
    const UiStatsWindow &window = uiStatsWindows[i];
    if (window.kind == UiStatsWindowKind::StatsModule) {
      continue;
    }
    if (wroteWebStatsWindow) {
      file << ",\n";
    }
    file << "    {\"id\":\"stats-" << window.id << "\",\"kind\":\"" << kindValue(window.kind)
         << "\",\"placement\":{\"x\":" << window.x << ",\"y\":" << window.y
         << ",\"viewport\":{\"width\":" << window.viewport_width
         << ",\"height\":" << window.viewport_height << "}"
         << ",\"zIndex\":" << window.z_index
         << ",\"visible\":" << (window.open ? "true" : "false") << "}"
         << ",\"playerId\":\"" << window.selected_player_index << "\""
         << ",\"team\":\"" << (window.selected_team_is_team_0 != 0 ? "blue" : "orange")
         << "\",\"entries\":[";
    for (size_t j = 0; j < window.entries.size(); j += 1) {
      if (j != 0) {
        file << ",";
      }
      const UiStatsWindow::Entry &entry = window.entries[j];
      file << "{\"statId\":\"" << escapeJsonString(webUiStatIdForWindow(window, entry)) << "\"";
      if (!entry.target_id.empty()) {
        file << ",\"targetId\":\"" << escapeJsonString(entry.target_id) << "\"";
      }
      file << "}";
    }
    file << "]}";
    wroteWebStatsWindow = true;
  }
  if (wroteWebStatsWindow) {
    file << "\n";
  }
  file << "  ]\n";
  file << "}\n";
  return file.str();
}

void SubtrActorPlugin::saveUiConfig() {
  const auto path = uiConfigPath();
  std::error_code error;
  std::filesystem::create_directories(path.parent_path(), error);
  if (error) {
    cvarManager->log(
        std::format("subtr-actor: failed to create UI config directory: {}", error.message()));
    return;
  }

  std::ofstream file(path, std::ios::binary);
  if (!file) {
    cvarManager->log(std::format("subtr-actor: failed to write UI config {}", path.string()));
    return;
  }

  const std::string json = uiConfigJson();
  file << json;
  lastSavedUiConfigJson = json;
  nextUiConfigAutosave = std::chrono::steady_clock::now() + std::chrono::seconds(2);
}

void SubtrActorPlugin::maybeAutosaveUiConfig() {
  const auto now = std::chrono::steady_clock::now();
  if (now < nextUiConfigAutosave) {
    return;
  }
  nextUiConfigAutosave = now + std::chrono::seconds(2);

  const std::string json = uiConfigJson();
  if (json == lastSavedUiConfigJson) {
    return;
  }
  saveUiConfig();
}

float SubtrActorPlugin::sampleIntervalSeconds() {
  auto intervalCvar = cvarManager->getCvar("subtr_actor_sample_interval_ms");
  const float intervalMs =
      std::clamp(static_cast<bool>(intervalCvar) ? intervalCvar.getFloatValue() : 50.0f,
                 1.0f,
                 1000.0f);
  return intervalMs / 1000.0f;
}

int SubtrActorPlugin::overlayX() {
  auto cvar = cvarManager->getCvar("subtr_actor_overlay_x");
  return std::clamp(static_cast<bool>(cvar) ? cvar.getIntValue() : 64, 0, 10000);
}

int SubtrActorPlugin::overlayY() {
  auto cvar = cvarManager->getCvar("subtr_actor_overlay_y");
  return std::clamp(static_cast<bool>(cvar) ? cvar.getIntValue() : 240, 0, 10000);
}

float SubtrActorPlugin::overlayScale() {
  auto cvar = cvarManager->getCvar("subtr_actor_overlay_scale");
  return std::clamp(static_cast<bool>(cvar) ? cvar.getFloatValue() : 1.0f, 0.5f, 3.0f);
}

float SubtrActorPlugin::overlayMessageSeconds() {
  auto cvar = cvarManager->getCvar("subtr_actor_overlay_message_seconds");
  return std::clamp(static_cast<bool>(cvar) ? cvar.getFloatValue() : 3.0f, 0.5f, 30.0f);
}

int SubtrActorPlugin::overlayMaxMessages() {
  auto cvar = cvarManager->getCvar("subtr_actor_overlay_max_messages");
  return std::clamp(static_cast<bool>(cvar) ? cvar.getIntValue() : 8, 1, 30);
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
  if (!sampledPlayerNames.back().empty()) {
    playerNamesByIndex[playerIndex] = sampledPlayerNames.back();
  }
  playerTeamsByIndex[playerIndex] = player.is_team_0;
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
    playerNamesByIndex.try_emplace(playerIndex, std::format("Player {}", playerIndex + 1));
    playerTeamsByIndex.try_emplace(playerIndex, player.is_team_0);
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
  playerNamesByIndex.clear();
  playerTeamsByIndex.clear();
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
  recentUiEvents.clear();
  mechanicsReviewDecisions.clear();
  mechanicsReviewIndex = 0;
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

bool SubtrActorPlugin::overlayCategoryEnabled(std::string_view category) {
  const std::string normalizedCategory = normalizeEventFilterToken(category);
  if (normalizedCategory == "mechanics") {
    auto cvar = cvarManager->getCvar("subtr_actor_overlay_mechanics_enabled");
    if (static_cast<bool>(cvar) && !cvar.getBoolValue()) {
      return false;
    }
  } else if (normalizedCategory == "team") {
    auto cvar = cvarManager->getCvar("subtr_actor_overlay_team_events_enabled");
    if (static_cast<bool>(cvar) && !cvar.getBoolValue()) {
      return false;
    }
  } else if (normalizedCategory == "goal_context") {
    auto cvar = cvarManager->getCvar("subtr_actor_overlay_goal_context_enabled");
    if (static_cast<bool>(cvar) && !cvar.getBoolValue()) {
      return false;
    }
  }

  auto filterCvar = cvarManager->getCvar("subtr_actor_overlay_event_types");
  const std::string filter =
      static_cast<bool>(filterCvar) ? filterCvar.getStringValue() : "all";
  return eventFilterAllows(filter, normalizedCategory, normalizedCategory);
}

bool SubtrActorPlugin::overlayMechanicEnabled(SaMechanicKind kind) {
  auto cvar = cvarManager->getCvar("subtr_actor_overlay_mechanics_enabled");
  if (static_cast<bool>(cvar) && !cvar.getBoolValue()) {
    return false;
  }
  auto filterCvar = cvarManager->getCvar("subtr_actor_overlay_event_types");
  const std::string filter =
      static_cast<bool>(filterCvar) ? filterCvar.getStringValue() : "all";
  return eventFilterAllows(filter, "mechanics", mechanicToken(kind));
}

std::string SubtrActorPlugin::teamLabel(uint8_t isTeam0) const {
  return isTeam0 != 0 ? "Blue" : "Orange";
}

std::string SubtrActorPlugin::playerLabel(uint32_t playerIndex, uint8_t isTeam0) const {
  const auto name = playerNamesByIndex.find(playerIndex);
  if (name != playerNamesByIndex.end() && !name->second.empty()) {
    return name->second;
  }
  const auto team = playerTeamsByIndex.find(playerIndex);
  const uint8_t labelTeam = team == playerTeamsByIndex.end() ? isTeam0 : team->second;
  return std::format("{} #{}", teamLabel(labelTeam), playerIndex + 1);
}

void SubtrActorPlugin::appendUiEvent(UiEventRecord event) {
  recentUiEvents.push_front(std::move(event));
  while (recentUiEvents.size() > MAX_RECENT_UI_EVENTS) {
    mechanicsReviewDecisions.erase(mechanicsReviewKey(recentUiEvents.back()));
    recentUiEvents.pop_back();
  }
}

bool SubtrActorPlugin::uiEventVisible(const UiEventRecord &event) {
  if (event.category == "mechanics" &&
      !cvarBool("subtr_actor_overlay_mechanics_enabled", true)) {
    return false;
  }
  if (event.category == "team" && !cvarBool("subtr_actor_overlay_team_events_enabled", true)) {
    return false;
  }
  if (event.category == "goal_context" &&
      !cvarBool("subtr_actor_overlay_goal_context_enabled", true)) {
    return false;
  }
  return eventFilterAllows(cvarString("subtr_actor_overlay_event_types", "all"), event.category, event.type);
}

void SubtrActorPlugin::pushEventMessage(const SaMechanicEvent &event) {
  const bool isBlue = event.is_team_0 != 0;
  const std::string action = event.confidence < 0.999f
                                 ? std::format(
                                       "{} ({:.0f}%)",
                                       mechanicLabel(event.kind),
                                       event.confidence * 100.0f)
                                 : mechanicLabel(event.kind);
  const std::string label =
      std::format("{}: {}", playerLabel(event.player_index, event.is_team_0), action);
  appendUiEvent(UiEventRecord{
      "mechanics",
      mechanicToken(event.kind),
      playerLabel(event.player_index, event.is_team_0),
      mechanicLabel(event.kind),
      event.confidence < 0.999f ? std::format("{:.0f}% confidence", event.confidence * 100.0f)
                                : "high confidence",
      isBlue ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255},
      event.frame_number,
      event.time,
  });

  if (!overlayMechanicEnabled(event.kind)) {
    return;
  }

  OverlayMessage message{
      label,
      isBlue ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255},
      std::chrono::steady_clock::now() +
          std::chrono::duration_cast<std::chrono::steady_clock::duration>(
              std::chrono::duration<float>(overlayMessageSeconds())),
  };
  messages.push_back(message);
  while (messages.size() > static_cast<size_t>(overlayMaxMessages())) {
    messages.pop_front();
  }
}

void SubtrActorPlugin::pushTeamEventMessage(const SaTeamEvent &event) {
  const bool isBlue = event.is_team_0 != 0;
  const std::string action = event.confidence < 0.999f
                                 ? std::format(
                                       "{} ({:.0f}%)",
                                       teamEventLabel(event),
                                       event.confidence * 100.0f)
                                 : teamEventLabel(event);
  const std::string label = std::format("{}: {}", teamLabel(event.is_team_0), action);
  appendUiEvent(UiEventRecord{
      "team",
      "rush",
      teamLabel(event.is_team_0),
      teamEventLabel(event),
      std::format("{:.1f}s - {:.1f}s", event.start_time, event.end_time),
      isBlue ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255},
      event.start_frame,
      event.start_time,
  });

  if (!overlayCategoryEnabled("team")) {
    return;
  }

  OverlayMessage message{
      label,
      isBlue ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255},
      std::chrono::steady_clock::now() +
          std::chrono::duration_cast<std::chrono::steady_clock::duration>(
              std::chrono::duration<float>(overlayMessageSeconds())),
  };
  messages.push_back(message);
  while (messages.size() > static_cast<size_t>(overlayMaxMessages())) {
    messages.pop_front();
  }
}

void SubtrActorPlugin::pushGoalContextEventMessage(const SaGoalContextEvent &event) {
  const bool isBlue = event.scoring_team_is_team_0 != 0;
  const std::string actor =
      event.has_scorer != 0
          ? playerLabel(event.scorer_index, event.scoring_team_is_team_0)
          : teamLabel(event.scoring_team_is_team_0);
  appendUiEvent(UiEventRecord{
      "goal_context",
      "goal_context",
      actor,
      goalContextLabel(event),
      teamLabel(event.scoring_team_is_team_0),
      isBlue ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255},
      event.frame_number,
      event.time,
  });

  if (!overlayCategoryEnabled("goal_context")) {
    return;
  }

  OverlayMessage message{
      std::format("{}: {}", actor, goalContextLabel(event)),
      isBlue ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255},
      std::chrono::steady_clock::now() +
          std::chrono::duration_cast<std::chrono::steady_clock::duration>(
              std::chrono::duration<float>(overlayMessageSeconds())),
  };
  messages.push_back(message);
  while (messages.size() > static_cast<size_t>(overlayMaxMessages())) {
    messages.pop_front();
  }
}

void SubtrActorPlugin::Render() {
  if (imguiContext != 0) {
    ImGui::SetCurrentContext(reinterpret_cast<ImGuiContext *>(imguiContext));
  }

  renderLauncherWindow();
  if (!uiEnabled()) {
    return;
  }

  renderScoreboardWindow();
  renderEventsWindow();
  renderStatusWindow();
  renderCameraWindow();
  renderPlaybackControlsWindow();
  renderRecordingWindow();
  renderGraphInspectorWindow();
  renderEventPlaylistWindow();
  renderMechanicsReviewWindow();
  renderReplayLoadingWindow();
  renderModuleControlsWindow();
  renderTouchControlsWindow();
  renderBoostPickupControlsWindow();
  renderStatsWindows();
  maybeAutosaveUiConfig();
}

void SubtrActorPlugin::RenderSettings() {
  if (imguiContext != 0) {
    ImGui::SetCurrentContext(reinterpret_cast<ImGuiContext *>(imguiContext));
  }
  renderSharedSettingsControls();
}

void SubtrActorPlugin::renderSharedSettingsControls() {
  auto checkboxCvar = [this](const char *label, const char *name, bool defaultValue) {
    bool value = cvarBool(name, defaultValue);
    if (ImGui::Checkbox(label, &value)) {
      setCvarBool(name, value);
    }
  };

  checkboxCvar("Interactive in-game UI", "subtr_actor_ui_enabled", true);
  checkboxCvar("Live analysis graph", "subtr_actor_enabled", false);
  checkboxCvar("Canvas HUD overlay", "subtr_actor_overlay_enabled", true);
  checkboxCvar("Canvas status line", "subtr_actor_status_overlay_enabled", true);
  checkboxCvar("Replay annotations", "subtr_actor_replay_annotations_enabled", true);

  ImGui::Separator();
  ImGui::Text("Event visibility");
  checkboxCvar("Mechanics", "subtr_actor_overlay_mechanics_enabled", true);
  checkboxCvar("Team events", "subtr_actor_overlay_team_events_enabled", true);
  checkboxCvar("Goal context", "subtr_actor_overlay_goal_context_enabled", true);

  renderEventFilterCombo("Event filter");

  auto intervalCvar = cvarManager->getCvar("subtr_actor_sample_interval_ms");
  int intervalMs = static_cast<bool>(intervalCvar) ? intervalCvar.getIntValue() : 8;
  if (ImGui::SliderInt("Sample interval ms", &intervalMs, 1, 1000) &&
      static_cast<bool>(intervalCvar)) {
    intervalCvar.setValue(intervalMs);
  }

  auto maxMessagesCvar = cvarManager->getCvar("subtr_actor_overlay_max_messages");
  int maxMessages = static_cast<bool>(maxMessagesCvar) ? maxMessagesCvar.getIntValue() : 8;
  if (ImGui::SliderInt("HUD message count", &maxMessages, 1, 30) &&
      static_cast<bool>(maxMessagesCvar)) {
    maxMessagesCvar.setValue(maxMessages);
  }
}

bool SubtrActorPlugin::renderEventFilterCombo(const char *label) {
  std::string currentFilter = cvarString("subtr_actor_overlay_event_types", "all");
  std::vector<std::string> selected = selectedEventSourceTokens(currentFilter);
  std::string preview = eventFilterPreview(currentFilter);

  bool changed = false;
  auto applySelection = [&]() {
    setCvarString("subtr_actor_overlay_event_types", eventFilterFromSelectedSources(selected));
    currentFilter = cvarString("subtr_actor_overlay_event_types", "all");
    selected = selectedEventSourceTokens(currentFilter);
    preview = eventFilterPreview(currentFilter);
    changed = true;
  };

  if (!ImGui::BeginCombo(label, preview.c_str())) {
    return false;
  }

  if (ImGui::SmallButton("All")) {
    selected = selectedEventSourceTokens("all");
    applySelection();
  }
  ImGui::SameLine();
  if (ImGui::SmallButton("None")) {
    selected.clear();
    applySelection();
  }
  ImGui::SameLine();
  ImGui::TextDisabled("%s", preview.c_str());
  ImGui::Separator();

  ImGui::Columns(2, std::format("{}-event-filter-columns", label).c_str(), false);
  for (const EventFilterOption &option : EVENT_FILTER_OPTIONS) {
    if (std::string_view{option.value} == "all") {
      continue;
    }

    ImGui::PushID(option.value);
    bool enabled = containsString(selected, option.value);
    if (ImGui::Checkbox(option.label, &enabled)) {
      if (enabled && !containsString(selected, option.value)) {
        selected.emplace_back(option.value);
      } else if (!enabled) {
        selected.erase(
            std::remove(selected.begin(), selected.end(), std::string{option.value}),
            selected.end());
      }
      applySelection();
    }
    ImGui::PopID();
    ImGui::NextColumn();
  }
  ImGui::Columns(1);
  ImGui::EndCombo();
  return changed;
}

void SubtrActorPlugin::applyWindowPlacement(
    UiWindowPlacement &placement,
    float x,
    float y,
    float width,
    float height) {
  auto applyFocus = [&]() {
    if (placement.pending_focus) {
      ImGui::SetNextWindowFocus();
      placement.z_index = nextUiWindowZIndex++;
      placement.pending_focus = false;
    }
  };

  if (placement.has_placement) {
    const ImGuiCond condition =
        placement.pending_apply_placement ? ImGuiCond_Always : ImGuiCond_FirstUseEver;
    const ImVec2 size{
        std::max(120.0f, placement.width),
        std::max(80.0f, placement.height),
    };
    const ImVec2 position = mapWindowPositionToViewport(
        placement.x,
        placement.y,
        size.x,
        size.y,
        placement.viewport_width,
        placement.viewport_height);
    ImGui::SetNextWindowPos(position, condition);
    ImGui::SetNextWindowSize(size, condition);
    placement.x = position.x;
    placement.y = position.y;
    placement.width = size.x;
    placement.height = size.y;
    placement.pending_apply_placement = false;
    applyFocus();
    return;
  }
  const ImVec2 defaultPosition = mapWindowPositionToViewport(x, y, width, height, 0.0f, 0.0f);
  ImGui::SetNextWindowPos(defaultPosition, ImGuiCond_FirstUseEver);
  ImGui::SetNextWindowSize(ImVec2{width, height}, ImGuiCond_FirstUseEver);
  applyFocus();
}

void SubtrActorPlugin::captureWindowPlacement(UiWindowPlacement &placement) {
  const ImVec2 position = ImGui::GetWindowPos();
  const ImVec2 size = ImGui::GetWindowSize();
  placement.has_placement = true;
  placement.x = position.x;
  placement.y = position.y;
  placement.width = size.x;
  placement.height = size.y;
  const ImVec2 displaySize = ImGui::GetIO().DisplaySize;
  placement.viewport_width = displaySize.x;
  placement.viewport_height = displaySize.y;
  if (ImGui::IsWindowFocused(ImGuiFocusedFlags_RootAndChildWindows) &&
      ImGui::IsMouseClicked(ImGuiMouseButton_Left)) {
    placement.z_index = std::max(placement.z_index, nextUiWindowZIndex++);
  }
}

bool SubtrActorPlugin::renderSingletonWindowHeader(const char *label, bool &open) {
  if (ImGui::SmallButton(std::format("Hide##singleton-window-hide-{}", label).c_str())) {
    open = false;
    return true;
  }
  ImGui::SameLine();
  ImGui::TextDisabled("%s", label);
  ImGui::Separator();
  return false;
}

void SubtrActorPlugin::applyStatsWindowPlacement(UiStatsWindow &window) {
  if (window.has_placement) {
    const ImGuiCond condition =
        window.pending_apply_placement ? ImGuiCond_Always : ImGuiCond_FirstUseEver;
    const ImVec2 size{
        std::max(180.0f, window.width),
        std::max(120.0f, window.height),
    };
    const ImVec2 position = mapWindowPositionToViewport(
        window.x,
        window.y,
        size.x,
        size.y,
        window.viewport_width,
        window.viewport_height);
    ImGui::SetNextWindowPos(position, condition);
    ImGui::SetNextWindowSize(size, condition);
    window.x = position.x;
    window.y = position.y;
    window.width = size.x;
    window.height = size.y;
    window.pending_apply_placement = false;
    return;
  }
  const float offset = static_cast<float>((window.id - 1) * 24);
  const float width = window.kind == UiStatsWindowKind::StatsModule ? 680.0f : 540.0f;
  const float height = window.kind == UiStatsWindowKind::StatsModule ? 460.0f : 330.0f;
  ImGui::SetNextWindowPos(
      mapWindowPositionToViewport(96.0f + offset, 96.0f + offset, width, height, 0.0f, 0.0f),
      ImGuiCond_FirstUseEver);
  ImGui::SetNextWindowSize(ImVec2{width, height}, ImGuiCond_FirstUseEver);
}

void SubtrActorPlugin::captureStatsWindowPlacement(UiStatsWindow &window) {
  const ImVec2 position = ImGui::GetWindowPos();
  const ImVec2 size = ImGui::GetWindowSize();
  window.has_placement = true;
  window.x = position.x;
  window.y = position.y;
  window.width = size.x;
  window.height = size.y;
  const ImVec2 displaySize = ImGui::GetIO().DisplaySize;
  window.viewport_width = displaySize.x;
  window.viewport_height = displaySize.y;
  if (ImGui::IsWindowFocused(ImGuiFocusedFlags_RootAndChildWindows) &&
      ImGui::IsMouseClicked(ImGuiMouseButton_Left)) {
    window.z_index = std::max(window.z_index, nextUiWindowZIndex++);
  }
}

bool SubtrActorPlugin::renderModuleSummaryToggle(
    const char *label,
    bool active,
    const char *idSuffix) {
  const std::string buttonLabel = std::format("{}##{}-{}", label, idSuffix, label);
  if (active) {
    ImGui::PushStyleColor(ImGuiCol_Button, ImVec4{0.16f, 0.35f, 0.28f, 1.0f});
    ImGui::PushStyleColor(ImGuiCol_ButtonHovered, ImVec4{0.20f, 0.45f, 0.36f, 1.0f});
  }
  const bool clicked = ImGui::Button(buttonLabel.c_str(), ImVec2{190.0f, 0.0f});
  if (active) {
    ImGui::PopStyleColor(2);
  }
  ImGui::SameLine();
  ImGui::TextDisabled("%s", active ? "On" : "Off");
  return clicked;
}

void SubtrActorPlugin::renderCvarModuleSummaryToggle(
    const char *label,
    const char *name,
    bool defaultValue,
    const char *idSuffix) {
  const bool active = cvarBool(name, defaultValue);
  if (renderModuleSummaryToggle(label, active, idSuffix)) {
    setCvarBool(name, !active);
  }
}

void SubtrActorPlugin::renderBoolModuleSummaryToggle(
    const char *label,
    bool &active,
    const char *idSuffix) {
  if (renderModuleSummaryToggle(label, active, idSuffix)) {
    active = !active;
  }
}

void SubtrActorPlugin::renderModuleSummaryControls(const char *idSuffix) {
  if (ImGui::TreeNode(std::format("Timeline visualizations##{}-timeline", idSuffix).c_str())) {
    renderBoolModuleSummaryToggle("Mechanics playlist", eventPlaylistMechanicsEnabled, idSuffix);
    renderBoolModuleSummaryToggle("Team event playlist", eventPlaylistTeamEventsEnabled, idSuffix);
    renderBoolModuleSummaryToggle(
        "Goal context playlist",
        eventPlaylistGoalContextEnabled,
        idSuffix);
    renderBoolModuleSummaryToggle("Boost pickup timeline", timelineRangeBoostEnabled, idSuffix);
    renderBoolModuleSummaryToggle(
        "Possession timeline",
        timelineRangePossessionEnabled,
        idSuffix);
    renderBoolModuleSummaryToggle(
        "Half control timeline",
        timelineRangePressureEnabled,
        idSuffix);
    renderBoolModuleSummaryToggle("Rush timeline", timelineRangeRushEnabled, idSuffix);
    renderBoolModuleSummaryToggle(
        "Position zones timeline",
        timelineRangeAbsolutePositioningEnabled,
        idSuffix);
    renderBoolModuleSummaryToggle("Playlist follow", eventPlaylistAutoFollow, idSuffix);
    ImGui::TreePop();
  }

  if (ImGui::TreeNode(std::format("In-game visualizations##{}-ingame", idSuffix).c_str())) {
    renderCvarModuleSummaryToggle(
        "Canvas HUD overlay",
        "subtr_actor_overlay_enabled",
        true,
        idSuffix);
    renderCvarModuleSummaryToggle(
        "Canvas status line",
        "subtr_actor_status_overlay_enabled",
        true,
        idSuffix);
    renderCvarModuleSummaryToggle(
        "HUD mechanics",
        "subtr_actor_overlay_mechanics_enabled",
        true,
        idSuffix);
    renderCvarModuleSummaryToggle(
        "HUD team events",
        "subtr_actor_overlay_team_events_enabled",
        true,
        idSuffix);
    renderCvarModuleSummaryToggle(
        "HUD goal context",
        "subtr_actor_overlay_goal_context_enabled",
        true,
        idSuffix);
    renderBoolModuleSummaryToggle("Ceiling shot labels", renderEffectCeilingShotEnabled, idSuffix);
    renderBoolModuleSummaryToggle("50/50 labels", renderEffectFiftyFiftyEnabled, idSuffix);
    renderBoolModuleSummaryToggle("Half control", renderEffectPressureEnabled, idSuffix);
    renderBoolModuleSummaryToggle("Player roles", renderEffectRelativePositioningEnabled, idSuffix);
    renderBoolModuleSummaryToggle(
        "Position zones",
        renderEffectAbsolutePositioningEnabled,
        idSuffix);
    renderBoolModuleSummaryToggle("Speed flip labels", renderEffectSpeedFlipEnabled, idSuffix);
    renderBoolModuleSummaryToggle("Touch labels", renderEffectTouchEnabled, idSuffix);
    renderBoolModuleSummaryToggle(
        "Boost pickup animation",
        boostPickupAnimationEnabled,
        idSuffix);
    const bool boostPadsEnabled =
        boostPickupPadBig || boostPickupPadSmall || boostPickupPadAmbiguous;
    if (renderModuleSummaryToggle("Boost pad locations", boostPadsEnabled, idSuffix)) {
      const bool next = !boostPadsEnabled;
      boostPickupPadBig = next;
      boostPickupPadSmall = next;
      boostPickupPadAmbiguous = next;
    }
    ImGui::TreePop();
  }
}

void SubtrActorPlugin::renderLauncherWindow() {
  if (!uiLauncherOpen) {
    return;
  }
  applyWindowPlacement(launcherPlacement, 16.0f, 68.0f, 340.0f, 430.0f);
  if (!ImGui::Begin("subtr-actor", &uiLauncherOpen)) {
    ImGui::End();
    return;
  }
  captureWindowPlacement(launcherPlacement);

  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "ACTIONS");
  if (ImGui::Button("Load Replay...")) {
    uiReplayLoadingOpen = true;
    replayLoadingPlacement.pending_focus = true;
    resetReplayAnnotations();
    tickReplayAnnotations();
    uiLauncherOpen = false;
  }
  ImGui::SameLine();
  bool liveAnalysis = liveProcessingEnabled();
  if (ImGui::Button(liveAnalysis ? "Stop analysis" : "Start analysis")) {
    setCvarBool("subtr_actor_enabled", !liveAnalysis);
  }
  if (ImGui::Button("Verify graph")) {
    uiGraphInspectorOpen = true;
    graphInspectorPlacement.pending_focus = true;
    verifyGraphRuntime({"subtr_actor_verify_graph"});
    uiLauncherOpen = false;
  }
  ImGui::SameLine();
  if (ImGui::Button("Open modules")) {
    uiModuleControlsOpen = true;
    moduleControlsPlacement.pending_focus = true;
    uiLauncherOpen = false;
  }
  ImGui::SameLine();
  if (ImGui::Button("Close launcher")) {
    uiLauncherOpen = false;
  }

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "WINDOWS");
  struct LauncherWindowToggle {
    const char *label;
    bool *open;
    UiWindowPlacement *placement;
  };
  auto renderLauncherWindowToggle = [&](LauncherWindowToggle &window) {
    ImGui::PushID(window.label);
    if (ImGui::Button(window.label, ImVec2{170.0f, 0.0f})) {
      *window.open = !*window.open;
      if (*window.open) {
        window.placement->pending_focus = true;
      }
      uiLauncherOpen = false;
    }
    ImGui::SameLine();
    ImGui::TextDisabled("%s", *window.open ? "Shown" : "Hidden");
    ImGui::PopID();
  };
  auto renderStatsWindowCreateButton = [&](const char *label, UiStatsWindowKind kind) {
    if (ImGui::Button(label, ImVec2{170.0f, 0.0f})) {
      createStatsWindow(kind);
      uiLauncherOpen = false;
    }
    ImGui::SameLine();
    ImGui::TextDisabled("New");
  };

  std::array<LauncherWindowToggle, 10> webLauncherWindows{{
      {"Camera", &uiCameraOpen, &cameraPlacement},
      {"Scoreboard", &uiScoreboardOpen, &scoreboardPlacement},
      {"Playback controls", &uiPlaybackControlsOpen, &playbackControlsPlacement},
      {"Recording", &uiRecordingOpen, &recordingPlacement},
      {"Events", &uiEventsOpen, &eventsPlacement},
      {"Event playlist", &uiEventPlaylistOpen, &eventPlaylistPlacement},
      {"Mechanics review", &uiMechanicsReviewOpen, &mechanicsReviewPlacement},
      {"Replay loading", &uiReplayLoadingOpen, &replayLoadingPlacement},
      {"Boost pickup filters", &uiBoostPickupControlsOpen, &boostPickupControlsPlacement},
      {"Touch controls", &uiTouchControlsOpen, &touchControlsPlacement},
  }};
  for (LauncherWindowToggle &window : webLauncherWindows) {
    renderLauncherWindowToggle(window);
  }
  renderStatsWindowCreateButton("New player stats", UiStatsWindowKind::Player);
  renderStatsWindowCreateButton("New team stats", UiStatsWindowKind::Team);
  renderStatsWindowCreateButton("New all players stats", UiStatsWindowKind::AllPlayers);
  renderStatsWindowCreateButton("New all teams stats", UiStatsWindowKind::AllTeams);
  renderStatsWindowCreateButton("New goal labels", UiStatsWindowKind::GoalsOverview);
  renderStatsWindowCreateButton("New ad hoc stats", UiStatsWindowKind::AdHoc);

  if (ImGui::TreeNode("Plugin tools##launcher-plugin-tools")) {
    std::array<LauncherWindowToggle, 3> pluginToolWindows{{
        {"Status", &uiStatusOpen, &statusPlacement},
        {"Graph inspector", &uiGraphInspectorOpen, &graphInspectorPlacement},
        {"Module controls", &uiModuleControlsOpen, &moduleControlsPlacement},
    }};
    for (LauncherWindowToggle &window : pluginToolWindows) {
      renderLauncherWindowToggle(window);
    }
    ImGui::TreePop();
  }
  renderSingletonWindowManager();

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "VISUALIZATIONS");
  renderModuleSummaryControls("launcher-module-summary");

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "LAYOUT");
  if (ImGui::Button("Default workspace")) {
    applyDefaultUiWorkspace();
  }
  ImGui::SameLine();
  if (ImGui::Button("Review workspace")) {
    applyReplayReviewUiWorkspace();
  }
  if (ImGui::Button("Debug workspace")) {
    applyGraphDebugUiWorkspace();
  }
  ImGui::SameLine();
  if (ImGui::Button("Recording workspace")) {
    applyRecordingUiWorkspace();
  }
  if (ImGui::Button("Reset positions")) {
    resetWindowPlacements();
  }
  ImGui::SameLine();
  if (ImGui::Button("Default stats windows")) {
    resetDefaultStatsWindows();
  }
  ImGui::SameLine();
  if (ImGui::Button("Hide side windows")) {
    uiEventsOpen = false;
    uiEventPlaylistOpen = false;
    uiStatusOpen = false;
    uiCameraOpen = false;
    uiPlaybackControlsOpen = false;
    uiRecordingOpen = false;
    uiGraphInspectorOpen = false;
    uiMechanicsReviewOpen = false;
    uiReplayLoadingOpen = false;
    uiTouchControlsOpen = false;
    uiBoostPickupControlsOpen = false;
  }
  if (ImGui::Button("Save layout")) {
    saveUiConfig();
  }
  ImGui::SameLine();
  if (ImGui::Button("Reload layout")) {
    loadUiConfig();
  }
  if (ImGui::Button("Copy layout JSON")) {
    const std::string json = uiConfigJson();
    ImGui::SetClipboardText(json.c_str());
    cvarManager->log(std::format("subtr-actor: copied {} UI config bytes", json.size()));
  }
  ImGui::SameLine();
  if (ImGui::Button("Paste layout JSON")) {
    const char *clipboardText = ImGui::GetClipboardText();
    if (clipboardText == nullptr || clipboardText[0] == '\0') {
      cvarManager->log("subtr-actor: clipboard does not contain UI config JSON");
    } else {
      applyUiConfigJson(clipboardText, "clipboard");
    }
  }

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "STATS WINDOWS");
  const size_t visibleStatsWindows = static_cast<size_t>(std::count_if(
      uiStatsWindows.begin(),
      uiStatsWindows.end(),
      [](const UiStatsWindow &window) { return window.open; }));
  ImGui::Text(
      "%zu visible / %zu stats windows",
      visibleStatsWindows,
      uiStatsWindows.size());
  renderStatsWindowManager();

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "GRAPH MODULES");
  const std::vector<std::string> &moduleNames = statsModuleNames();
  if (moduleNames.empty()) {
    ImGui::TextWrapped("Start live analysis to list graph-backed stats modules.");
  } else {
    ImGui::BeginChild("module-summary", ImVec2{0.0f, 150.0f}, true);
    for (const std::string &moduleName : moduleNames) {
      if (ImGui::SmallButton(std::format("Open##module-{}", moduleName).c_str())) {
        createStatsModuleWindow(moduleName);
        uiLauncherOpen = false;
      }
      ImGui::SameLine();
      ImGui::Text("%s", moduleName.c_str());
    }
    ImGui::EndChild();
  }

  ImGui::Separator();
  renderSharedSettingsControls();
  ImGui::End();
}

void SubtrActorPlugin::renderScoreboardWindow() {
  if (!uiScoreboardOpen) {
    return;
  }
  applyWindowPlacement(scoreboardPlacement, 760.0f, 18.0f, 210.0f, 78.0f);
  if (!ImGui::Begin("Scoreboard##subtr-actor", &uiScoreboardOpen)) {
    ImGui::End();
    return;
  }
  captureWindowPlacement(scoreboardPlacement);
  if (renderSingletonWindowHeader("Scoreboard", uiScoreboardOpen)) {
    ImGui::End();
    return;
  }

  if (lastTeamScores) {
    ImGui::TextColored(ImVec4{0.31f, 0.75f, 1.0f, 1.0f}, "%d", lastTeamScores->first);
    ImGui::SameLine();
    ImGui::Text("Blue  :  Orange");
    ImGui::SameLine();
    ImGui::TextColored(ImVec4{1.0f, 0.69f, 0.31f, 1.0f}, "%d", lastTeamScores->second);
  } else {
    ImGui::Text("Waiting for score data");
  }
  ImGui::End();
}

void SubtrActorPlugin::renderEventsWindow() {
  if (!uiEventsOpen) {
    return;
  }
  applyWindowPlacement(eventsPlacement, 16.0f, 505.0f, 520.0f, 360.0f);
  if (!ImGui::Begin("Events##subtr-actor", &uiEventsOpen)) {
    ImGui::End();
    return;
  }
  captureWindowPlacement(eventsPlacement);
  if (renderSingletonWindowHeader("Events", uiEventsOpen)) {
    ImGui::End();
    return;
  }

  renderEventFilterCombo("Filter");
  ImGui::SameLine();
  if (ImGui::Button("Clear")) {
    recentUiEvents.clear();
    mechanicsReviewDecisions.clear();
    mechanicsReviewIndex = 0;
  }

  renderEventSourceControls();

  size_t visibleCount = 0;
  for (const UiEventRecord &event : recentUiEvents) {
    if (uiEventVisible(event)) {
      visibleCount += 1;
    }
  }
  ImGui::Text("%zu visible / %zu recent", visibleCount, recentUiEvents.size());
  ImGui::Separator();

  ImGui::BeginChild("event-list", ImVec2{0.0f, 0.0f}, true);
  ImGui::Columns(4, "event-columns", true);
  ImGui::Text("Time");
  ImGui::NextColumn();
  ImGui::Text("Actor");
  ImGui::NextColumn();
  ImGui::Text("Event");
  ImGui::NextColumn();
  ImGui::Text("Details");
  ImGui::NextColumn();
  ImGui::Separator();

  for (const UiEventRecord &event : recentUiEvents) {
    if (!uiEventVisible(event)) {
      continue;
    }
    ImGui::Text("%.2fs", event.time);
    ImGui::NextColumn();
    ImGui::TextColored(toImVec4(event.color), "%s", event.actor.c_str());
    ImGui::NextColumn();
    ImGui::TextWrapped("%s", event.label.c_str());
    ImGui::NextColumn();
    ImGui::TextWrapped("%s", event.details.c_str());
    ImGui::NextColumn();
  }

  ImGui::Columns(1);
  ImGui::EndChild();
  ImGui::End();
}

void SubtrActorPlugin::renderEventSourceControls() {
  ImGui::SetNextItemOpen(true, ImGuiCond_FirstUseEver);
  if (!ImGui::TreeNode("Event sources##event-source-controls")) {
    return;
  }

  std::vector<std::string> selected =
      selectedEventSourceTokens(cvarString("subtr_actor_overlay_event_types", "all"));
  auto applySelection = [&]() {
    setCvarString("subtr_actor_overlay_event_types", eventFilterFromSelectedSources(selected));
  };

  if (ImGui::SmallButton("All##event-sources")) {
    selected = selectedEventSourceTokens("all");
    applySelection();
  }
  ImGui::SameLine();
  if (ImGui::SmallButton("None##event-sources")) {
    selected.clear();
    applySelection();
  }
  ImGui::SameLine();
  const std::string preview =
      eventFilterPreview(cvarString("subtr_actor_overlay_event_types", "all"));
  ImGui::TextDisabled("%s", preview.c_str());

  ImGui::BeginChild("event-source-list", ImVec2{0.0f, 145.0f}, true);
  ImGui::Columns(2, "event-source-columns", false);
  for (const EventFilterOption &option : EVENT_FILTER_OPTIONS) {
    if (std::string_view{option.value} == "all") {
      continue;
    }

    size_t count = 0;
    for (const UiEventRecord &event : recentUiEvents) {
      if (eventFilterAllows(option.value, event.category, event.type)) {
        count += 1;
      }
    }

    ImGui::PushID(option.value);
    bool enabled = containsString(selected, option.value);
    const std::string label = std::format("{} ({})", option.label, count);
    if (ImGui::Checkbox(label.c_str(), &enabled)) {
      if (enabled && !containsString(selected, option.value)) {
        selected.emplace_back(option.value);
      } else if (!enabled) {
        selected.erase(
            std::remove(selected.begin(), selected.end(), std::string{option.value}),
            selected.end());
      }
      applySelection();
    }
    ImGui::PopID();
    ImGui::NextColumn();
  }
  ImGui::Columns(1);
  ImGui::EndChild();
  ImGui::TreePop();
}

bool SubtrActorPlugin::eventPlaylistSourceEnabled(const UiEventRecord &event) const {
  if (event.category == "mechanics") {
    return eventPlaylistMechanicsEnabled;
  }
  if (event.category == "team") {
    return eventPlaylistTeamEventsEnabled;
  }
  if (event.category == "goal_context" || event.type == "goal") {
    return eventPlaylistGoalContextEnabled;
  }
  return true;
}

std::string SubtrActorPlugin::mechanicsReviewKey(const UiEventRecord &event) const {
  return std::format(
      "{}:{}:{}:{}",
      event.category,
      event.type,
      event.frame_number,
      event.actor);
}

const char *SubtrActorPlugin::mechanicsReviewDecisionLabel(const UiEventRecord &event) const {
  const auto decision = mechanicsReviewDecisions.find(mechanicsReviewKey(event));
  if (decision == mechanicsReviewDecisions.end()) {
    return "unreviewed";
  }
  if (decision->second == 1) {
    return "confirmed";
  }
  if (decision->second == 2) {
    return "rejected";
  }
  if (decision->second == 3) {
    return "uncertain";
  }
  return "unreviewed";
}

void SubtrActorPlugin::renderEventPlaylistWindow() {
  if (!uiEventPlaylistOpen) {
    return;
  }

  applyWindowPlacement(eventPlaylistPlacement, 545.0f, 505.0f, 430.0f, 430.0f);
  if (!ImGui::Begin("Event playlist##subtr-actor", &uiEventPlaylistOpen)) {
    ImGui::End();
    return;
  }
  captureWindowPlacement(eventPlaylistPlacement);
  if (renderSingletonWindowHeader("Event playlist", uiEventPlaylistOpen)) {
    ImGui::End();
    return;
  }

  size_t selectedCount = 0;
  size_t visibleCount = 0;
  size_t mechanicsSourceCount = 0;
  size_t teamSourceCount = 0;
  size_t goalContextSourceCount = 0;
  for (const UiEventRecord &event : recentUiEvents) {
    if (event.category == "mechanics") {
      mechanicsSourceCount += 1;
    } else if (event.category == "team") {
      teamSourceCount += 1;
    } else if (event.category == "goal_context" || event.type == "goal") {
      goalContextSourceCount += 1;
    }
    if (eventPlaylistSourceEnabled(event)) {
      selectedCount += 1;
      if (uiEventVisible(event)) {
        visibleCount += 1;
      }
    }
  }

  const bool allSourcesEnabled = eventPlaylistMechanicsEnabled && eventPlaylistTeamEventsEnabled &&
                                 eventPlaylistGoalContextEnabled;
  const bool noSourcesEnabled = !eventPlaylistMechanicsEnabled && !eventPlaylistTeamEventsEnabled &&
                                !eventPlaylistGoalContextEnabled;
  if (renderModuleSummaryToggle(
          std::format("All events ({})", recentUiEvents.size()).c_str(),
          allSourcesEnabled,
          "event-playlist-sources")) {
    eventPlaylistMechanicsEnabled = true;
    eventPlaylistTeamEventsEnabled = true;
    eventPlaylistGoalContextEnabled = true;
  }
  if (renderModuleSummaryToggle("No events", noSourcesEnabled, "event-playlist-sources")) {
    eventPlaylistMechanicsEnabled = false;
    eventPlaylistTeamEventsEnabled = false;
    eventPlaylistGoalContextEnabled = false;
  }
  ImGui::Checkbox("Follow", &eventPlaylistAutoFollow);

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "SOURCES");
  renderBoolModuleSummaryToggle(
      std::format("Mechanics ({})", mechanicsSourceCount).c_str(),
      eventPlaylistMechanicsEnabled,
      "event-playlist-sources");
  renderBoolModuleSummaryToggle(
      std::format("Team ({})", teamSourceCount).c_str(),
      eventPlaylistTeamEventsEnabled,
      "event-playlist-sources");
  renderBoolModuleSummaryToggle(
      std::format("Goal context ({})", goalContextSourceCount).c_str(),
      eventPlaylistGoalContextEnabled,
      "event-playlist-sources");

  renderEventFilterCombo("Event filter");

  ImGui::Text(
      "%zu visible / %zu selected / %zu recent",
      visibleCount,
      selectedCount,
      recentUiEvents.size());
  ImGui::Separator();

  ImGui::BeginChild("event-playlist-list", ImVec2{0.0f, 0.0f}, true);
  bool renderedAny = false;
  for (const UiEventRecord &event : recentUiEvents) {
    if (!eventPlaylistSourceEnabled(event) || !uiEventVisible(event)) {
      continue;
    }
    renderedAny = true;

    ImGui::PushID(static_cast<int>(event.frame_number));
    const ImVec4 color = toImVec4(event.color);
    ImGui::TextColored(color, "%.2fs", event.time);
    ImGui::SameLine();
    ImGui::TextColored(color, "%s", event.actor.c_str());
    ImGui::SameLine();
    ImGui::TextWrapped("%s", event.label.c_str());
    if (!event.details.empty()) {
      ImGui::TextDisabled("%s", event.details.c_str());
    }
    ImGui::TextDisabled("%s / %s", event.category.c_str(), event.type.c_str());
    ImGui::Separator();
    ImGui::PopID();
  }
  if (!renderedAny) {
    ImGui::TextWrapped("No events match the selected playlist sources.");
  } else if (eventPlaylistAutoFollow) {
    ImGui::SetScrollHereY(1.0f);
  }
  ImGui::EndChild();
  ImGui::End();
}

void SubtrActorPlugin::renderMechanicsReviewWindow() {
  if (!uiMechanicsReviewOpen) {
    return;
  }

  applyWindowPlacement(mechanicsReviewPlacement, 980.0f, 160.0f, 500.0f, 560.0f);
  if (!ImGui::Begin("Mechanics review##subtr-actor", &uiMechanicsReviewOpen)) {
    ImGui::End();
    return;
  }
  captureWindowPlacement(mechanicsReviewPlacement);
  if (renderSingletonWindowHeader("Mechanics review", uiMechanicsReviewOpen)) {
    ImGui::End();
    return;
  }

  std::vector<size_t> candidates;
  candidates.reserve(recentUiEvents.size());
  for (size_t index = 0; index < recentUiEvents.size(); index += 1) {
    const UiEventRecord &event = recentUiEvents[index];
    if (eventPlaylistSourceEnabled(event) && uiEventVisible(event)) {
      candidates.push_back(index);
    }
  }
  if (candidates.empty()) {
    mechanicsReviewIndex = 0;
  } else {
    mechanicsReviewIndex = std::clamp(
        mechanicsReviewIndex,
        0,
        static_cast<int>(candidates.size()) - 1);
  }

  int confirmedCount = 0;
  int rejectedCount = 0;
  int uncertainCount = 0;
  for (const size_t index : candidates) {
    const auto decision = mechanicsReviewDecisions.find(mechanicsReviewKey(recentUiEvents[index]));
    if (decision == mechanicsReviewDecisions.end()) {
      continue;
    }
    confirmedCount += decision->second == 1 ? 1 : 0;
    rejectedCount += decision->second == 2 ? 1 : 0;
    uncertainCount += decision->second == 3 ? 1 : 0;
  }

  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "REVIEW QUEUE");
  ImGui::Text(
      "%zu candidates | %d confirmed | %d rejected | %d uncertain",
      candidates.size(),
      confirmedCount,
      rejectedCount,
      uncertainCount);
  ImGui::Checkbox("Mechanics", &eventPlaylistMechanicsEnabled);
  ImGui::SameLine();
  ImGui::Checkbox("Team", &eventPlaylistTeamEventsEnabled);
  ImGui::SameLine();
  ImGui::Checkbox("Goal context", &eventPlaylistGoalContextEnabled);

  renderEventFilterCombo("Event filter");
  ImGui::SetNextItemWidth(120.0f);
  ImGui::SliderFloat("Clip lead", &mechanicsReviewClipLeadSeconds, 0.0f, 10.0f, "%.1fs");
  ImGui::SameLine();
  ImGui::SetNextItemWidth(120.0f);
  ImGui::SliderFloat("Clip trail", &mechanicsReviewClipTrailSeconds, 0.0f, 10.0f, "%.1fs");

  ImGui::Separator();
  if (candidates.empty()) {
    ImGui::TextWrapped("No visible events match the current review filters.");
    if (ImGui::Button("Open events")) {
      uiEventsOpen = true;
      eventsPlacement.pending_focus = true;
    }
    ImGui::SameLine();
    if (ImGui::Button("Open playlist")) {
      uiEventPlaylistOpen = true;
      eventPlaylistPlacement.pending_focus = true;
    }
    ImGui::End();
    return;
  }

  UiEventRecord &current = recentUiEvents[candidates[static_cast<size_t>(mechanicsReviewIndex)]];
  const std::string currentKey = mechanicsReviewKey(current);
  const float clipStart = std::max(0.0f, current.time - mechanicsReviewClipLeadSeconds);
  const float clipEnd = current.time + mechanicsReviewClipTrailSeconds;
  ImGui::Text(
      "%d / %zu",
      mechanicsReviewIndex + 1,
      candidates.size());
  ImGui::TextWrapped("%s", current.label.c_str());
  ImGui::Text("Decision: %s", mechanicsReviewDecisionLabel(current));
  ImGui::Text("Mechanic: %s", current.type.c_str());
  ImGui::Text("Player: %s", current.actor.c_str());
  ImGui::Text("Clip: %.2fs to %.2fs", clipStart, clipEnd);
  ImGui::Text("Event: frame %llu", static_cast<unsigned long long>(current.frame_number));
  if (!current.details.empty()) {
    ImGui::TextWrapped("Reason: %s", current.details.c_str());
  }

  if (ImGui::Button("Prev") && mechanicsReviewIndex > 0) {
    mechanicsReviewIndex -= 1;
  }
  ImGui::SameLine();
  if (ImGui::Button("Replay clip")) {
    playbackCurrentTime = clipStart;
    playbackPlaying = true;
    playbackSkipPostGoalTransitions = false;
    playbackSkipKickoffs = false;
    uiPlaybackControlsOpen = true;
    playbackControlsPlacement.pending_focus = true;

    ReplayServerWrapper replayServer = gameWrapper->GetGameEventAsReplay();
    if (!replayServer.IsNull()) {
      replayServer.StartPlaybackAtTime(clipStart);
    } else {
      cvarManager->log(
          "subtr-actor: replay clip selected; open a replay to seek in Rocket League");
    }
  }
  ImGui::SameLine();
  if (ImGui::Button("Next") &&
      mechanicsReviewIndex < static_cast<int>(candidates.size()) - 1) {
    mechanicsReviewIndex += 1;
  }
  ImGui::SameLine();
  if (ImGui::Button("Show playlist")) {
    uiEventPlaylistOpen = true;
    eventPlaylistPlacement.pending_focus = true;
  }

  if (ImGui::Button("Confirm")) {
    mechanicsReviewDecisions[currentKey] = 1;
  }
  ImGui::SameLine();
  if (ImGui::Button("Reject")) {
    mechanicsReviewDecisions[currentKey] = 2;
  }
  ImGui::SameLine();
  if (ImGui::Button("Uncertain")) {
    mechanicsReviewDecisions[currentKey] = 3;
  }
  ImGui::SameLine();
  if (ImGui::Button("Clear decision")) {
    mechanicsReviewDecisions.erase(currentKey);
  }

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "REPLAY");
  ImGui::Text(
      "Replay annotations: %s",
      replayAnnotations ? "loaded" : replayAnnotationLoadFailed ? "failed" : "idle");
  if (!replayAnnotationPath.empty()) {
    ImGui::TextWrapped("%s", replayAnnotationPath.c_str());
  }

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "PLAYLIST");
  ImGui::BeginChild("mechanics-review-list", ImVec2{0.0f, 150.0f}, true);
  for (size_t i = 0; i < candidates.size(); i += 1) {
    const UiEventRecord &event = recentUiEvents[candidates[i]];
    ImGui::PushID(static_cast<int>(i));
    const std::string label = std::format(
        "{} {:.2f}s {} ({})",
        i == static_cast<size_t>(mechanicsReviewIndex) ? ">" : " ",
        event.time,
        event.label,
        mechanicsReviewDecisionLabel(event));
    if (ImGui::Selectable(label.c_str(), i == static_cast<size_t>(mechanicsReviewIndex))) {
      mechanicsReviewIndex = static_cast<int>(i);
    }
    ImGui::PopID();
  }
  ImGui::EndChild();

  ImGui::End();
}

void SubtrActorPlugin::renderReplayLoadingWindow() {
  if (!uiReplayLoadingOpen) {
    return;
  }

  applyWindowPlacement(replayLoadingPlacement, 960.0f, 68.0f, 520.0f, 360.0f);
  if (!ImGui::Begin("Replay loading##subtr-actor", &uiReplayLoadingOpen)) {
    ImGui::End();
    return;
  }
  captureWindowPlacement(replayLoadingPlacement);
  if (renderSingletonWindowHeader("Replay loading", uiReplayLoadingOpen)) {
    ImGui::End();
    return;
  }

  const bool annotationsEnabled = replayAnnotationsEnabled();
  const bool inReplay = gameWrapper->IsInReplay();
  ReplayServerWrapper replayServer = gameWrapper->GetGameEventAsReplay();
  const bool hasReplayServer = !replayServer.IsNull();
  const std::optional<std::string> replayPath =
      hasReplayServer ? currentReplayPath(replayServer) : std::nullopt;
  std::string rawReplayPath;
  if (hasReplayServer) {
    ReplayWrapper replay = replayServer.GetReplay();
    if (!replay.IsNull()) {
      rawReplayPath = replay.GetFilePath().ToString();
    }
  }
  const size_t annotationCount =
      replayAnnotations && replayAnnotationCount ? replayAnnotationCount(replayAnnotations) : 0;
  const char *status = !annotationsEnabled
                           ? "Disabled"
                           : !inReplay       ? "Waiting for replay"
                           : replayAnnotations ? "Loaded"
                           : replayAnnotationLoadFailed ? "Failed"
                                                        : "Scanning";

  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "REPLAY LOADING");
  ImGui::Text("Summary: %s", replayPath ? "1 replay candidate" : "0 replay candidates");
  ImGui::Text("Active: %s", status);
  ImGui::Text("In replay: %s", inReplay ? "yes" : "no");
  if (hasReplayServer) {
    ImGui::Text("Replay time: %.2fs", replayServer.GetReplayTimeElapsed());
  }
  ImGui::Text("Annotations: %zu", annotationCount);

  ImGui::Separator();
  bool annotationsValue = annotationsEnabled;
  if (ImGui::Checkbox("Replay annotations", &annotationsValue)) {
    setCvarBool("subtr_actor_replay_annotations_enabled", annotationsValue);
    if (!annotationsValue) {
      resetReplayAnnotations();
    }
  }
  ImGui::SameLine();
  if (ImGui::Button("Retry load")) {
    resetReplayAnnotations();
    tickReplayAnnotations();
  }
  ImGui::SameLine();
  if (ImGui::Button("Clear load")) {
    resetReplayAnnotations();
  }

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "CURRENT REPLAY");
  if (replayPath) {
    ImGui::TextWrapped("Resolved: %s", replayPath->c_str());
  } else {
    ImGui::TextDisabled("Resolved: --");
  }
  if (!rawReplayPath.empty()) {
    ImGui::TextWrapped("Raw: %s", rawReplayPath.c_str());
  } else {
    ImGui::TextDisabled("Raw: --");
  }
  if (!replayAnnotationPath.empty()) {
    ImGui::TextWrapped("Processed: %s", replayAnnotationPath.c_str());
  } else {
    ImGui::TextDisabled("Processed: --");
  }

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "WINDOWS");
  if (ImGui::Button("Open playback")) {
    uiPlaybackControlsOpen = true;
    playbackControlsPlacement.pending_focus = true;
  }
  ImGui::SameLine();
  if (ImGui::Button("Open review")) {
    uiMechanicsReviewOpen = true;
    mechanicsReviewPlacement.pending_focus = true;
  }
  if (ImGui::Button("Open playlist")) {
    uiEventPlaylistOpen = true;
    eventPlaylistPlacement.pending_focus = true;
  }

  ImGui::End();
}

void SubtrActorPlugin::renderModuleControlsWindow() {
  if (!uiModuleControlsOpen) {
    return;
  }

  applyWindowPlacement(moduleControlsPlacement, 980.0f, 305.0f, 430.0f, 520.0f);
  if (!ImGui::Begin("Module controls##subtr-actor", &uiModuleControlsOpen)) {
    ImGui::End();
    return;
  }
  captureWindowPlacement(moduleControlsPlacement);
  if (renderSingletonWindowHeader("Module controls", uiModuleControlsOpen)) {
    ImGui::End();
    return;
  }

  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "LIVE PIPELINE");
  auto checkboxCvar = [this](const char *label, const char *name, bool defaultValue) {
    bool value = cvarBool(name, defaultValue);
    if (ImGui::Checkbox(label, &value)) {
      setCvarBool(name, value);
    }
  };
  checkboxCvar("Live analysis graph", "subtr_actor_enabled", false);
  checkboxCvar("Canvas HUD overlay", "subtr_actor_overlay_enabled", true);
  checkboxCvar("Canvas status line", "subtr_actor_status_overlay_enabled", true);
  checkboxCvar("Replay annotations", "subtr_actor_replay_annotations_enabled", true);

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "MODULE SUMMARY");
  renderModuleSummaryControls("module-controls-summary");

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "EVENT FILTER");
  renderEventFilterCombo("Event filter");

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "STAT DISPLAY");
  ImGui::TextDisabled("Movement breakdown");
  ImGui::Checkbox("Speed band##movement-breakdown", &movementBreakdownSpeed);
  ImGui::SameLine();
  ImGui::Checkbox("Height band##movement-breakdown", &movementBreakdownHeight);
  if (ImGui::Button("Open movement stats")) {
    createStatsModuleWindow("movement", 0);
  }

  ImGui::TextDisabled("Possession breakdown");
  ImGui::Checkbox("Control##possession-breakdown", &possessionBreakdownState);
  ImGui::SameLine();
  ImGui::Checkbox("Third##possession-breakdown", &possessionBreakdownThird);
  if (ImGui::Button("Open possession stats")) {
    createStatsModuleWindow("possession", 0);
  }

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "GRAPH STATS MODULES");
  const std::vector<std::string> &moduleNames = statsModuleNames();
  if (moduleNames.empty()) {
    ImGui::TextWrapped("Start live analysis to list graph-backed stats modules.");
  } else {
    ImGui::BeginChild("module-controls-module-list", ImVec2{0.0f, 170.0f}, true);
    for (const std::string &moduleName : moduleNames) {
      ImGui::PushID(moduleName.c_str());
      if (ImGui::SmallButton("Frame")) {
        createStatsModuleWindow(moduleName, 0);
      }
      ImGui::SameLine();
      if (ImGui::SmallButton("Module")) {
        createStatsModuleWindow(moduleName, 1);
      }
      ImGui::SameLine();
      if (ImGui::SmallButton("Config")) {
        createStatsModuleWindow(moduleName, 2);
      }
      ImGui::SameLine();
      ImGui::TextWrapped("%s", moduleName.c_str());
      ImGui::PopID();
    }
    ImGui::EndChild();
  }

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "GRAPH INSPECTION");
  if (ImGui::Button("Open graph inspector")) {
    uiGraphInspectorOpen = true;
    graphInspectorPlacement.pending_focus = true;
  }
  ImGui::SameLine();
  if (ImGui::Button("Open camera")) {
    uiCameraOpen = true;
    cameraPlacement.pending_focus = true;
  }
  ImGui::SameLine();
  if (ImGui::Button("Open event playlist")) {
    uiEventPlaylistOpen = true;
    eventPlaylistPlacement.pending_focus = true;
  }
  ImGui::SameLine();
  if (ImGui::Button("Open review")) {
    uiMechanicsReviewOpen = true;
    mechanicsReviewPlacement.pending_focus = true;
  }
  ImGui::SameLine();
  if (ImGui::Button("Open recording")) {
    uiRecordingOpen = true;
    recordingPlacement.pending_focus = true;
  }
  ImGui::SameLine();
  if (ImGui::Button("Open replay loading")) {
    uiReplayLoadingOpen = true;
    replayLoadingPlacement.pending_focus = true;
  }
  ImGui::SameLine();
  if (ImGui::Button("Open touch controls")) {
    uiTouchControlsOpen = true;
    touchControlsPlacement.pending_focus = true;
  }
  ImGui::SameLine();
  if (ImGui::Button("Open boost filters")) {
    uiBoostPickupControlsOpen = true;
    boostPickupControlsPlacement.pending_focus = true;
  }

  ImGui::End();
}

void SubtrActorPlugin::renderBoostPickupControlsWindow() {
  if (!uiBoostPickupControlsOpen) {
    return;
  }

  applyWindowPlacement(boostPickupControlsPlacement, 1000.0f, 560.0f, 430.0f, 420.0f);
  if (!ImGui::Begin("Boost pickup filters##subtr-actor", &uiBoostPickupControlsOpen)) {
    ImGui::End();
    return;
  }
  captureWindowPlacement(boostPickupControlsPlacement);
  if (renderSingletonWindowHeader("Boost pickup filters", uiBoostPickupControlsOpen)) {
    ImGui::End();
    return;
  }

  const int activePadTypes = static_cast<int>(boostPickupPadBig) +
                             static_cast<int>(boostPickupPadSmall) +
                             static_cast<int>(boostPickupPadAmbiguous);
  const int activeActivities = static_cast<int>(boostPickupActivityActive) +
                               static_cast<int>(boostPickupActivityInactive) +
                               static_cast<int>(boostPickupActivityUnknown);
  const int activeFieldHalves = static_cast<int>(boostPickupFieldOwn) +
                                static_cast<int>(boostPickupFieldOpponent) +
                                static_cast<int>(boostPickupFieldUnknown);
  const bool hidden =
      activePadTypes == 0 || activeActivities == 0 || activeFieldHalves == 0;
  const int constrainedGroups = static_cast<int>(activePadTypes < 3) +
                                static_cast<int>(activeActivities < 3) +
                                static_cast<int>(activeFieldHalves < 3);
  const std::string pickupReadout =
      hidden ? "Hidden"
             : constrainedGroups == 0 ? "All labels"
                                      : std::format("{} filters", constrainedGroups);

  ImGui::Text("Pickup labels: %s", pickupReadout.c_str());
  ImGui::Text("Known pads: %zu", boostPadIds.size());
  ImGui::Text("Pending pad events: %zu", pendingBoostPadEvents.size());
  ImGui::Text("Recent boost pickups: %d", recentEventCountForType("boost_pickup"));

  ImGui::Separator();
  ImGui::Checkbox("Boost pickup animation", &boostPickupAnimationEnabled);
  ImGui::Separator();
  if (ImGui::Button("All filters")) {
    boostPickupPadBig = true;
    boostPickupPadSmall = true;
    boostPickupPadAmbiguous = true;
    boostPickupActivityActive = true;
    boostPickupActivityInactive = true;
    boostPickupActivityUnknown = true;
    boostPickupFieldOwn = true;
    boostPickupFieldOpponent = true;
    boostPickupFieldUnknown = true;
  }
  ImGui::SameLine();
  if (ImGui::Button("Hide pickups")) {
    boostPickupPadBig = false;
    boostPickupPadSmall = false;
    boostPickupPadAmbiguous = false;
    boostPickupActivityActive = false;
    boostPickupActivityInactive = false;
    boostPickupActivityUnknown = false;
    boostPickupFieldOwn = false;
    boostPickupFieldOpponent = false;
    boostPickupFieldUnknown = false;
  }

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "PAD TYPE");
  ImGui::Checkbox("Big pads", &boostPickupPadBig);
  ImGui::SameLine();
  ImGui::Checkbox("Small pads", &boostPickupPadSmall);
  ImGui::Checkbox("Ambiguous pads", &boostPickupPadAmbiguous);

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "ACTIVITY");
  ImGui::Checkbox("Active play", &boostPickupActivityActive);
  ImGui::SameLine();
  ImGui::Checkbox("Inactive play", &boostPickupActivityInactive);
  ImGui::Checkbox("Unknown activity", &boostPickupActivityUnknown);

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "FIELD HALF");
  ImGui::Checkbox("Own half", &boostPickupFieldOwn);
  ImGui::SameLine();
  ImGui::Checkbox("Opponent half", &boostPickupFieldOpponent);
  ImGui::Checkbox("Unknown half", &boostPickupFieldUnknown);

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "ACTIONS");
  if (ImGui::Button("Show boost pickups")) {
    setCvarString("subtr_actor_overlay_event_types", "boost_pickup");
    uiEventPlaylistOpen = true;
    eventPlaylistPlacement.pending_focus = true;
  }
  ImGui::SameLine();
  if (ImGui::Button("Open boost stats")) {
    createStatsModuleWindow("boost", 0);
  }
  if (ImGui::Button("Inspect boost nodes")) {
    uiGraphInspectorOpen = true;
    graphInspectorView = 1;
    graphInspectorNodeQuery = "boost";
    graphInspectorPlacement.pending_focus = true;
  }
  ImGui::SameLine();
  if (ImGui::Button("Boost output")) {
    uiGraphInspectorOpen = true;
    graphInspectorView = 0;
    selectedGraphOutput = "events";
    graphInspectorPlacement.pending_focus = true;
  }

  ImGui::End();
}

void SubtrActorPlugin::renderTouchControlsWindow() {
  if (!uiTouchControlsOpen) {
    return;
  }

  applyWindowPlacement(touchControlsPlacement, 980.0f, 160.0f, 410.0f, 380.0f);
  if (!ImGui::Begin("Touch controls##subtr-actor", &uiTouchControlsOpen)) {
    ImGui::End();
    return;
  }
  captureWindowPlacement(touchControlsPlacement);
  if (renderSingletonWindowHeader("Touch controls", uiTouchControlsOpen)) {
    ImGui::End();
    return;
  }

  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "TOUCH MARKERS");
  ImGui::SliderFloat("Marker decay seconds", &touchMarkerDecaySeconds, 1.0f, 10.0f, "%.1fs");
  if (ImGui::RadioButton("Markers##touch-mode", &touchControlsMode, 0)) {
    setCvarString("subtr_actor_overlay_event_types", "touch");
  }
  ImGui::SameLine();
  if (ImGui::RadioButton("Advancement##touch-mode", &touchControlsMode, 1)) {
    setCvarString("subtr_actor_overlay_event_types", "touch_ball_movement");
  }

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "STAT BREAKDOWN");
  ImGui::Checkbox("Kind", &touchBreakdownKind);
  ImGui::SameLine();
  ImGui::Checkbox("Height", &touchBreakdownHeight);
  ImGui::Checkbox("Surface", &touchBreakdownSurface);
  ImGui::SameLine();
  ImGui::Checkbox("Dodge", &touchBreakdownDodge);

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "LIVE TOUCH STATE");
  if (lastTouch) {
    ImGui::Text(
        "Last touch: %s",
        playerLabel(lastTouch->player_index, lastTouch->is_team_0).c_str());
  } else {
    ImGui::Text("Last touch: --");
  }
  ImGui::Text("Pending touches: %zu", pendingTouches.size());
  ImGui::Text("Pending dodge refreshes: %zu", pendingDodgeRefreshes.size());
  ImGui::Text("Recent touch events: %d", recentEventCountForType("touch"));
  ImGui::Text(
      "Recent touch movement: %d",
      recentEventCountForType("touch_ball_movement"));

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "ACTIONS");
  if (ImGui::Button("Show touches")) {
    setCvarString("subtr_actor_overlay_event_types", "touch");
    uiEventPlaylistOpen = true;
    eventPlaylistPlacement.pending_focus = true;
  }
  ImGui::SameLine();
  if (ImGui::Button("Show movement")) {
    setCvarString("subtr_actor_overlay_event_types", "touch_ball_movement");
    uiEventPlaylistOpen = true;
    eventPlaylistPlacement.pending_focus = true;
  }
  if (ImGui::Button("Open touch stats")) {
    createStatsModuleWindow("touch", 0);
  }
  ImGui::SameLine();
  if (ImGui::Button("Inspect touch nodes")) {
    uiGraphInspectorOpen = true;
    graphInspectorView = 1;
    graphInspectorNodeQuery = "touch";
    graphInspectorPlacement.pending_focus = true;
  }

  ImGui::End();
}

void SubtrActorPlugin::renderStatusWindow() {
  if (!uiStatusOpen) {
    return;
  }
  applyWindowPlacement(statusPlacement, 1230.0f, 68.0f, 330.0f, 220.0f);
  if (!ImGui::Begin("Status##subtr-actor", &uiStatusOpen)) {
    ImGui::End();
    return;
  }
  captureWindowPlacement(statusPlacement);
  if (renderSingletonWindowHeader("Status", uiStatusOpen)) {
    ImGui::End();
    return;
  }

  ImGui::Text("Mode: %s", liveProcessingEnabled() ? "live analysis" : "idle");
  ImGui::Text("Replay annotations: %s", replayAnnotations ? "loaded" : "not loaded");
  if (replayAnnotations && replayAnnotationCount) {
    ImGui::Text("Replay events: %zu", replayAnnotationCount(replayAnnotations));
  }
  ImGui::Text("Frame: %llu", static_cast<unsigned long long>(frameNumber));
  ImGui::Text("Sample interval: %.0fms", sampleIntervalSeconds() * 1000.0f);
  ImGui::Text("Players sampled: %zu", sampledPlayers.size());
  ImGui::Text("Recent events: %zu", recentUiEvents.size());
  ImGui::End();
}

void SubtrActorPlugin::renderCameraWindow() {
  if (!uiCameraOpen) {
    return;
  }

  const SaPlayerFrame *selectedPlayer = sampledPlayerByIndex(cameraSelectedPlayerIndex);
  if (cameraViewMode == 1 && selectedPlayer == nullptr && !sampledPlayers.empty()) {
    cameraSelectedPlayerIndex = sampledPlayers.front().player_index;
    selectedPlayer = sampledPlayerByIndex(cameraSelectedPlayerIndex);
  }
  const SaPlayerFrame *targetPlayer = cameraViewMode == 1 ? selectedPlayer : nullptr;

  applyWindowPlacement(cameraPlacement, 720.0f, 68.0f, 360.0f, 500.0f);
  if (!ImGui::Begin("Camera##subtr-actor", &uiCameraOpen)) {
    ImGui::End();
    return;
  }
  captureWindowPlacement(cameraPlacement);
  if (renderSingletonWindowHeader("Camera", uiCameraOpen)) {
    ImGui::End();
    return;
  }

  constexpr std::array<const char *, 4> viewModes{
      "Free",
      "Follow",
      "Overhead",
      "Diagonal",
  };
  cameraViewMode = std::clamp(cameraViewMode, 0, static_cast<int>(viewModes.size()) - 1);

  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "CAMERA PROFILE");
  const std::string selectedLabel =
      targetPlayer == nullptr
          ? "Free camera"
          : playerLabel(targetPlayer->player_index, targetPlayer->is_team_0);
  if (ImGui::BeginCombo("Target", selectedLabel.c_str())) {
    if (ImGui::Selectable("Free camera", cameraViewMode != 1)) {
      cameraViewMode = 0;
    }
    for (const SaPlayerFrame &player : sampledPlayers) {
      const std::string label = std::format(
          "{}##camera-player-{}",
          playerLabel(player.player_index, player.is_team_0),
          player.player_index);
      if (ImGui::Selectable(
              label.c_str(), targetPlayer != nullptr &&
                                 targetPlayer->player_index == player.player_index)) {
        cameraSelectedPlayerIndex = player.player_index;
        cameraViewMode = 1;
      }
    }
    ImGui::EndCombo();
  }

  ImGui::RadioButton("Free##camera-view", &cameraViewMode, 0);
  ImGui::SameLine();
  ImGui::RadioButton("Follow##camera-view", &cameraViewMode, 1);
  ImGui::SameLine();
  ImGui::RadioButton("Overhead##camera-view", &cameraViewMode, 2);
  ImGui::SameLine();
  ImGui::RadioButton("Diagonal##camera-view", &cameraViewMode, 3);

  if (cameraViewMode == 2) {
    cameraFreePreset = 0;
  } else if (cameraViewMode == 3) {
    cameraFreePreset = 1;
  }

  ImGui::SliderFloat("Distance scale", &cameraDistanceScale, 0.75f, 4.0f, "%.2fx");
  ImGui::Checkbox("Ball cam", &cameraBallCamEnabled);
  ImGui::Checkbox("Custom settings", &cameraCustomSettingsEnabled);
  if (cameraCustomSettingsEnabled) {
    ImGui::SliderFloat("FOV", &cameraCustomFov, 60.0f, 130.0f, "%.0f");
    ImGui::SliderFloat("Height", &cameraCustomHeight, 40.0f, 240.0f, "%.0f");
    ImGui::SliderFloat("Pitch", &cameraCustomPitch, -15.0f, 0.0f, "%.1f");
    ImGui::SliderFloat("Distance", &cameraCustomDistance, 120.0f, 500.0f, "%.0f");
    ImGui::SliderFloat("Stiffness", &cameraCustomStiffness, 0.0f, 1.0f, "%.2f");
    ImGui::SliderFloat("Swivel speed", &cameraCustomSwivelSpeed, 0.1f, 10.0f, "%.1f");
    ImGui::SliderFloat(
        "Transition speed", &cameraCustomTransitionSpeed, 0.1f, 10.0f, "%.1f");
  }

  const float fov = cameraCustomSettingsEnabled ? cameraCustomFov : 110.0f;
  const float height = cameraCustomSettingsEnabled ? cameraCustomHeight : 100.0f;
  const float pitch = cameraCustomSettingsEnabled ? cameraCustomPitch : -4.0f;
  const float distance = (cameraCustomSettingsEnabled ? cameraCustomDistance : 270.0f) *
                         cameraDistanceScale;
  const float stiffness = cameraCustomSettingsEnabled ? cameraCustomStiffness : 0.0f;

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "READOUT");
  ImGui::Text("Mode: %s", viewModes[static_cast<size_t>(cameraViewMode)]);
  ImGui::Text("Target: %s", selectedLabel.c_str());
  ImGui::Text("FOV %.0f  Height %.0f  Pitch %.1f", fov, height, pitch);
  ImGui::Text("Distance %.0f  Stiffness %.2f", distance, stiffness);
  ImGui::Text("Ball cam: %s", cameraBallCamEnabled ? "on" : "off");
  if (targetPlayer == nullptr) {
    ImGui::TextDisabled("No player target selected.");
  } else if (targetPlayer->has_rigid_body == 0) {
    ImGui::TextDisabled("Selected player has no rigid body sample.");
  } else {
    const SaVec3 location = targetPlayer->rigid_body.location;
    const SaVec3 velocity = targetPlayer->rigid_body.linear_velocity;
    ImGui::Text(
        "Location: %.0f, %.0f, %.0f", location.x, location.y, location.z);
    if (targetPlayer->rigid_body.has_linear_velocity != 0) {
      ImGui::Text(
          "Velocity: %.0f, %.0f, %.0f", velocity.x, velocity.y, velocity.z);
    }
  }

  ImGui::Separator();
  if (ImGui::Button("Open playback")) {
    uiPlaybackControlsOpen = true;
    playbackControlsPlacement.pending_focus = true;
  }
  ImGui::SameLine();
  if (ImGui::Button("Open recording")) {
    uiRecordingOpen = true;
    recordingPlacement.pending_focus = true;
  }
  if (targetPlayer != nullptr && ImGui::Button("Open player stats")) {
    createStatsWindow(UiStatsWindowKind::Player, true);
    if (!uiStatsWindows.empty()) {
      UiStatsWindow &window = uiStatsWindows.back();
      window.selected_player_index = targetPlayer->player_index;
      window.selected_team_is_team_0 = targetPlayer->is_team_0;
    }
  }

  ImGui::End();
}

void SubtrActorPlugin::renderPlaybackControlsWindow() {
  if (!uiPlaybackControlsOpen) {
    return;
  }

  applyWindowPlacement(playbackControlsPlacement, 880.0f, 68.0f, 360.0f, 430.0f);
  if (!ImGui::Begin("Playback controls##subtr-actor", &uiPlaybackControlsOpen)) {
    ImGui::End();
    return;
  }
  captureWindowPlacement(playbackControlsPlacement);
  if (renderSingletonWindowHeader("Playback controls", uiPlaybackControlsOpen)) {
    ImGui::End();
    return;
  }

  const bool inReplay = gameWrapper->IsInReplay();
  const bool inGame = gameWrapper->IsInGame();
  ReplayServerWrapper replayServer = gameWrapper->GetGameEventAsReplay();
  const bool hasReplayServer = !replayServer.IsNull();
  const char *mode = inReplay ? "replay"
                              : inGame ? "live match/freeplay" : "waiting for game";
  if (hasReplayServer) {
    playbackCurrentTime = replayServer.GetReplayTimeElapsed();
  }

  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "PLAYBACK");
  ImGui::Text("Mode: %s", mode);
  ImGui::Text("Live frame: %llu", static_cast<unsigned long long>(frameNumber));
  ImGui::Text("Live time: %.2fs", lastTime);
  if (hasReplayServer) {
    ImGui::Text("Replay time: %.2fs", replayServer.GetReplayTimeElapsed());
  } else {
    ImGui::TextDisabled("Replay time: --");
  }
  if (lastProcessedGameTime) {
    ImGui::Text("Last sampled: %.2fs", *lastProcessedGameTime);
  } else {
    ImGui::TextDisabled("Last sampled: --");
  }

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "WEB PLAYBACK CONFIG");
  ImGui::SetNextItemWidth(140.0f);
  ImGui::InputFloat("Current time", &playbackCurrentTime, 0.25f, 2.0f, "%.2f");
  playbackCurrentTime = std::max(0.0f, playbackCurrentTime);
  ImGui::SetNextItemWidth(140.0f);
  ImGui::SliderFloat("Rate", &playbackRate, 0.1f, 4.0f, "%.2fx");
  ImGui::Checkbox("Playing", &playbackPlaying);
  ImGui::SameLine();
  ImGui::Checkbox("Skip goal transitions", &playbackSkipPostGoalTransitions);
  ImGui::Checkbox("Skip kickoffs", &playbackSkipKickoffs);
  if (hasReplayServer && ImGui::SmallButton("Capture replay time")) {
    playbackCurrentTime = replayServer.GetReplayTimeElapsed();
  }

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "ANALYSIS");
  auto checkboxCvar = [this](const char *label, const char *name, bool defaultValue) {
    bool value = cvarBool(name, defaultValue);
    if (ImGui::Checkbox(label, &value)) {
      setCvarBool(name, value);
    }
  };
  checkboxCvar("Live analysis graph", "subtr_actor_enabled", false);
  checkboxCvar("Replay annotations", "subtr_actor_replay_annotations_enabled", true);
  checkboxCvar("Profile timing", "subtr_actor_profile_enabled", false);

  auto intervalCvar = cvarManager->getCvar("subtr_actor_sample_interval_ms");
  int intervalMs = static_cast<bool>(intervalCvar) ? intervalCvar.getIntValue() : 8;
  if (ImGui::SliderInt("Sample interval ms", &intervalMs, 1, 1000) &&
      static_cast<bool>(intervalCvar)) {
    intervalCvar.setValue(intervalMs);
  }

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "ANNOTATIONS");
  ImGui::Text(
      "Status: %s",
      replayAnnotations ? "loaded" : replayAnnotationLoadFailed ? "failed" : "idle");
  if (replayAnnotations && replayAnnotationCount) {
    ImGui::Text("Replay events: %zu", replayAnnotationCount(replayAnnotations));
  }
  if (!replayAnnotationPath.empty()) {
    ImGui::TextWrapped("Replay: %s", replayAnnotationPath.c_str());
  }

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "TIMING");
  if (profileSampleCount == 0) {
    ImGui::TextDisabled("No timing samples yet.");
  } else {
    const double divisor = static_cast<double>(profileSampleCount);
    ImGui::Text("Samples: %llu", static_cast<unsigned long long>(profileSampleCount));
    ImGui::Text(
        "Avg total: %.3fms",
        (profileSamplingMs + profileProcessingMs + profileDrainMs) / divisor);
    ImGui::Text(
        "Sample %.3f / process %.3f / drain %.3fms",
        profileSamplingMs / divisor,
        profileProcessingMs / divisor,
        profileDrainMs / divisor);
  }

  ImGui::Separator();
  if (ImGui::Button("Reset live state")) {
    if (engine && engineReset) {
      engineReset(engine);
    }
    resetLiveState();
  }
  ImGui::SameLine();
  if (ImGui::Button("Verify graph")) {
    verifyGraphRuntime({"subtr_actor_verify_graph"});
  }
  if (ImGui::Button("Open status")) {
    uiStatusOpen = true;
    statusPlacement.pending_focus = true;
  }
  ImGui::SameLine();
  if (ImGui::Button("Open playlist")) {
    uiEventPlaylistOpen = true;
    eventPlaylistPlacement.pending_focus = true;
  }
  if (ImGui::Button("Open modules")) {
    uiModuleControlsOpen = true;
    moduleControlsPlacement.pending_focus = true;
  }

  ImGui::End();
}

void SubtrActorPlugin::renderRecordingWindow() {
  if (!uiRecordingOpen) {
    return;
  }

  applyWindowPlacement(recordingPlacement, 990.0f, 250.0f, 400.0f, 380.0f);
  if (!ImGui::Begin("Recording##subtr-actor", &uiRecordingOpen)) {
    ImGui::End();
    return;
  }
  captureWindowPlacement(recordingPlacement);
  if (renderSingletonWindowHeader("Recording", uiRecordingOpen)) {
    ImGui::End();
    return;
  }

  const std::filesystem::path outputDirectory = gameWrapper->GetDataFolder() / "subtr-actor";
  auto graphDumpBytes = [&]() {
    size_t total = 0;
    const std::array<const char *, 7> paths{{
        "graph-events.json",
        "graph-frame.json",
        "graph-timeline.json",
        "graph-stats.json",
        "graph-analysis-nodes.json",
        "graph-event-history.json",
        "graph-info.json",
    }};
    for (const char *path : paths) {
      std::error_code error;
      const uintmax_t size = std::filesystem::file_size(outputDirectory / path, error);
      if (!error) {
        total += static_cast<size_t>(size);
      }
    }
    return total;
  };
  auto dumpSnapshot = [&](bool finish) {
    if (!loaded || !engine) {
      recordingStatus = "Engine not loaded";
      return;
    }
    std::vector<std::string> params{"subtr_actor_dump_graph"};
    if (finish) {
      params.push_back("finish");
    }
    dumpGraphJson(params);
    recordingLastBytes = graphDumpBytes();
    recordingSnapshotCount += 1;
    recordingStatus = finish ? "Finalized graph snapshot written"
                             : "Current graph snapshot written";
  };

  const double elapsedSeconds =
      recordingActive
          ? std::chrono::duration<double>(
                std::chrono::steady_clock::now() - recordingStartedAt)
                .count()
          : 0.0;

  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "RECORDING");
  ImGui::SliderInt("FPS", &recordingFps, 1, 120);
  const std::array<const char *, 4> rates{{"0.5x", "1.0x", "1.5x", "2.0x"}};
  recordingPlaybackRateIndex = std::clamp(recordingPlaybackRateIndex, 0, 3);
  if (ImGui::BeginCombo("Playback rate", rates[static_cast<size_t>(recordingPlaybackRateIndex)])) {
    for (int index = 0; index < static_cast<int>(rates.size()); index += 1) {
      const bool selected = index == recordingPlaybackRateIndex;
      if (ImGui::Selectable(rates[static_cast<size_t>(index)], selected)) {
        recordingPlaybackRateIndex = index;
      }
    }
    ImGui::EndCombo();
  }
  ImGui::Checkbox("Finalize before dump", &recordingFinishBeforeDump);

  ImGui::Separator();
  if (ImGui::Button("Start")) {
    recordingActive = true;
    recordingStartedAt = std::chrono::steady_clock::now();
    recordingStatus = "Recording analysis snapshots";
  }
  ImGui::SameLine();
  if (ImGui::Button("Full replay")) {
    recordingActive = false;
    dumpSnapshot(true);
  }
  ImGui::SameLine();
  if (ImGui::Button("Stop")) {
    recordingActive = false;
    dumpSnapshot(recordingFinishBeforeDump);
  }
  if (ImGui::Button("Snapshot")) {
    dumpSnapshot(false);
  }
  ImGui::SameLine();
  if (ImGui::Button("Log folder")) {
    cvarManager->log(std::format(
        "subtr-actor: recording snapshots are written to {}",
        outputDirectory.string()));
  }
  ImGui::SameLine();
  if (ImGui::Button("Clear")) {
    recordingActive = false;
    recordingSnapshotCount = 0;
    recordingLastBytes = 0;
    recordingStatus = "Idle";
  }

  ImGui::Separator();
  ImGui::Text("Status: %s", recordingStatus.c_str());
  ImGui::Text("Elapsed: %.1fs", elapsedSeconds);
  ImGui::Text("Size: %s", formatByteSize(recordingLastBytes).c_str());
  ImGui::Text("Type: JSON snapshots");
  ImGui::Text("Snapshots: %d", recordingSnapshotCount);
  ImGui::TextWrapped("Folder: %s", outputDirectory.string().c_str());

  ImGui::Separator();
  if (ImGui::Button("Open graph inspector")) {
    uiGraphInspectorOpen = true;
    graphInspectorPlacement.pending_focus = true;
  }
  ImGui::SameLine();
  if (ImGui::Button("Open replay loading")) {
    uiReplayLoadingOpen = true;
    replayLoadingPlacement.pending_focus = true;
  }

  ImGui::End();
}

std::vector<std::string> SubtrActorPlugin::graphOutputNames() {
  std::string graphInfoJson = readJsonBuffer(graphInfoJsonLen, writeGraphInfoJson);
  std::vector<std::string> names =
      parseJsonStringArrayProperty(graphInfoJson, "graph_output_names");
  if (names.empty()) {
    graphInfoJson = readNamedJsonBuffer(graphOutputJsonLen, writeGraphOutputJson, "graph_info");
    names = parseJsonStringArrayProperty(graphInfoJson, "graph_output_names");
  }
  if (names.empty()) {
    names.assign(VERIFY_GRAPH_OUTPUTS.begin(), VERIFY_GRAPH_OUTPUTS.end());
  }
  return names;
}

std::vector<std::string> SubtrActorPlugin::analysisNodeNames() {
  std::vector<std::string> names =
      parseJsonStringArray(readJsonBuffer(analysisNodeNamesJsonLen, writeAnalysisNodeNamesJson));
  if (!names.empty()) {
    return names;
  }

  std::string graphInfoJson = readJsonBuffer(graphInfoJsonLen, writeGraphInfoJson);
  names = parseJsonStringArrayProperty(graphInfoJson, "callable_analysis_node_names");
  if (names.empty()) {
    names = parseJsonStringArrayProperty(graphInfoJson, "node_names");
  }
  if (names.empty()) {
    graphInfoJson = readNamedJsonBuffer(graphOutputJsonLen, writeGraphOutputJson, "graph_info");
    names = parseJsonStringArrayProperty(graphInfoJson, "callable_analysis_node_names");
  }
  if (names.empty()) {
    names = parseJsonStringArrayProperty(graphInfoJson, "node_names");
  }
  return names;
}

void SubtrActorPlugin::renderGraphInspectorWindow() {
  if (!uiGraphInspectorOpen) {
    return;
  }

  applyWindowPlacement(graphInspectorPlacement, 360.0f, 68.0f, 700.0f, 520.0f);
  if (!ImGui::Begin("Graph inspector##subtr-actor", &uiGraphInspectorOpen)) {
    ImGui::End();
    return;
  }
  captureWindowPlacement(graphInspectorPlacement);
  if (renderSingletonWindowHeader("Graph inspector", uiGraphInspectorOpen)) {
    ImGui::End();
    return;
  }

  if (!loaded || !engine) {
    ImGui::TextWrapped("Start live analysis to inspect graph outputs and analysis nodes.");
    ImGui::End();
    return;
  }

  ImGui::RadioButton("Outputs##graph-inspector-view", &graphInspectorView, 0);
  ImGui::SameLine();
  ImGui::RadioButton("Analysis nodes##graph-inspector-view", &graphInspectorView, 1);
  ImGui::SameLine();
  ImGui::RadioButton("Graph info##graph-inspector-view", &graphInspectorView, 2);
  ImGui::Separator();

  if (graphInspectorView == 0) {
    const std::vector<std::string> names = graphOutputNames();
    if (selectedGraphOutput.empty() && !names.empty()) {
      selectedGraphOutput = names.front();
    }
    if (!containsString(names, selectedGraphOutput) && !names.empty()) {
      selectedGraphOutput = names.front();
    }

    const char *selected =
        selectedGraphOutput.empty() ? "Select graph output" : selectedGraphOutput.c_str();
    if (ImGui::BeginCombo("Output", selected)) {
      for (const std::string &name : names) {
        const bool isSelected = name == selectedGraphOutput;
        if (ImGui::Selectable(name.c_str(), isSelected)) {
          selectedGraphOutput = name;
        }
      }
      ImGui::EndCombo();
    }

    if (selectedGraphOutput.empty()) {
      ImGui::TextWrapped("No graph outputs are registered.");
    } else {
      const std::string json =
          readNamedJsonBuffer(graphOutputJsonLen, writeGraphOutputJson, selectedGraphOutput);
      renderJsonInspectorPayload(
          "graph-output",
          std::format("Graph output: {}", selectedGraphOutput),
          json);
    }
  } else if (graphInspectorView == 1) {
    const std::vector<std::string> names = analysisNodeNames();
    if (selectedAnalysisNode.empty() && !names.empty()) {
      selectedAnalysisNode = names.front();
    }
    if (!containsString(names, selectedAnalysisNode) && !names.empty()) {
      selectedAnalysisNode = names.front();
    }

    std::array<char, 160> queryBuffer{};
    const size_t querySize =
        std::min(graphInspectorNodeQuery.size(), queryBuffer.size() - 1);
    std::copy_n(graphInspectorNodeQuery.data(), querySize, queryBuffer.data());
    ImGui::SetNextItemWidth(-1.0f);
    if (ImGui::InputText("Search nodes", queryBuffer.data(), queryBuffer.size())) {
      graphInspectorNodeQuery = queryBuffer.data();
    }
    if (!graphInspectorNodeQuery.empty()) {
      ImGui::SameLine();
      if (ImGui::SmallButton("Clear##graph-node-search")) {
        graphInspectorNodeQuery.clear();
      }
    }

    const std::vector<std::string_view> tokens = statSearchTokens(graphInspectorNodeQuery);
    auto matchesSearch = [&](const std::string &name) {
      if (tokens.empty()) {
        return true;
      }
      const std::string normalized = normalizeStatSearchText(name);
      return std::all_of(tokens.begin(), tokens.end(), [&](std::string_view token) {
        return normalized.find(token) != std::string::npos;
      });
    };

    ImGui::BeginChild("graph-node-list", ImVec2{220.0f, 0.0f}, true);
    size_t visibleCount = 0;
    for (const std::string &name : names) {
      if (!matchesSearch(name)) {
        continue;
      }
      visibleCount += 1;
      const bool isSelected = name == selectedAnalysisNode;
      if (ImGui::Selectable(name.c_str(), isSelected)) {
        selectedAnalysisNode = name;
      }
    }
    if (visibleCount == 0) {
      ImGui::TextWrapped("No matching analysis nodes.");
    }
    ImGui::EndChild();
    ImGui::SameLine();

    ImGui::BeginChild("graph-node-payload", ImVec2{0.0f, 0.0f}, true);
    if (selectedAnalysisNode.empty()) {
      ImGui::TextWrapped("No callable analysis nodes are registered.");
    } else {
      const std::string json =
          readNamedJsonBuffer(analysisNodeJsonLen, writeAnalysisNodeJson, selectedAnalysisNode);
      renderJsonInspectorPayload(
          "analysis-node",
          std::format("Analysis node: {}", selectedAnalysisNode),
          json);
    }
    ImGui::EndChild();
  } else {
    graphInspectorView = 2;
    std::string json = readJsonBuffer(graphInfoJsonLen, writeGraphInfoJson);
    if (json.empty()) {
      json = readNamedJsonBuffer(graphOutputJsonLen, writeGraphOutputJson, "graph_info");
    }
    renderJsonInspectorPayload("graph-info", "Graph info", json);
  }

  ImGui::End();
}

void SubtrActorPlugin::renderSingletonWindowManager() {
  struct SingletonWindowControl {
    const char *label;
    bool *open;
    UiWindowPlacement *placement;
  };

  std::array<SingletonWindowControl, 13> windows{{
      {"Scoreboard", &uiScoreboardOpen, &scoreboardPlacement},
      {"Events", &uiEventsOpen, &eventsPlacement},
      {"Event playlist", &uiEventPlaylistOpen, &eventPlaylistPlacement},
      {"Status", &uiStatusOpen, &statusPlacement},
      {"Camera", &uiCameraOpen, &cameraPlacement},
      {"Playback controls", &uiPlaybackControlsOpen, &playbackControlsPlacement},
      {"Recording", &uiRecordingOpen, &recordingPlacement},
      {"Graph inspector", &uiGraphInspectorOpen, &graphInspectorPlacement},
      {"Mechanics review", &uiMechanicsReviewOpen, &mechanicsReviewPlacement},
      {"Replay loading", &uiReplayLoadingOpen, &replayLoadingPlacement},
      {"Module controls", &uiModuleControlsOpen, &moduleControlsPlacement},
      {"Touch controls", &uiTouchControlsOpen, &touchControlsPlacement},
      {"Boost pickup filters", &uiBoostPickupControlsOpen, &boostPickupControlsPlacement},
  }};

  const size_t visibleCount = static_cast<size_t>(std::count_if(
      windows.begin(),
      windows.end(),
      [](const SingletonWindowControl &window) { return *window.open; }));

  ImGui::Text("%zu visible / %zu singleton windows", visibleCount, windows.size());
  if (!ImGui::TreeNode("Manage windows##singleton-window-manager")) {
    return;
  }

  if (ImGui::SmallButton("Show all##singleton-windows")) {
    for (SingletonWindowControl &window : windows) {
      *window.open = true;
    }
  }
  ImGui::SameLine();
  if (ImGui::SmallButton("Hide all##singleton-windows")) {
    for (SingletonWindowControl &window : windows) {
      *window.open = false;
    }
  }
  ImGui::SameLine();
  if (ImGui::SmallButton("Focus visible##singleton-windows")) {
    for (SingletonWindowControl &window : windows) {
      if (*window.open) {
        window.placement->pending_focus = true;
      }
    }
  }

  ImGui::BeginChild("singleton-window-manager", ImVec2{0.0f, 132.0f}, true);
  for (SingletonWindowControl &window : windows) {
    ImGui::PushID(window.label);
    if (*window.open) {
      if (ImGui::SmallButton("Hide")) {
        *window.open = false;
      }
      ImGui::SameLine();
      if (ImGui::SmallButton("Focus")) {
        window.placement->pending_focus = true;
      }
    } else {
      if (ImGui::SmallButton("Show")) {
        *window.open = true;
        window.placement->pending_focus = true;
      }
      ImGui::SameLine();
      ImGui::TextDisabled("Hidden");
    }
    ImGui::SameLine();
    ImGui::TextWrapped("%s", window.label);
    ImGui::PopID();
  }
  ImGui::EndChild();
  ImGui::TreePop();
}

void SubtrActorPlugin::renderStatsWindowManager() {
  if (uiStatsWindows.empty()) {
    return;
  }

  const size_t hiddenCount = static_cast<size_t>(std::count_if(
      uiStatsWindows.begin(),
      uiStatsWindows.end(),
      [](const UiStatsWindow &window) { return !window.open; }));
  if (ImGui::SmallButton("Show all##stats-windows")) {
    for (UiStatsWindow &window : uiStatsWindows) {
      window.open = true;
      window.pending_focus = true;
    }
  }
  ImGui::SameLine();
  if (ImGui::SmallButton("Hide all##stats-windows")) {
    for (UiStatsWindow &window : uiStatsWindows) {
      window.open = false;
    }
  }
  ImGui::SameLine();
  if (ImGui::SmallButton("Remove hidden##stats-windows")) {
    uiStatsWindows.erase(
        std::remove_if(
            uiStatsWindows.begin(),
            uiStatsWindows.end(),
            [](const UiStatsWindow &window) { return !window.open; }),
        uiStatsWindows.end());
  }
  ImGui::SameLine();
  ImGui::TextDisabled("%zu hidden", hiddenCount);

  ImGui::BeginChild("stats-window-manager", ImVec2{0.0f, 132.0f}, true);
  std::optional<size_t> removeIndex;
  for (size_t index = 0; index < uiStatsWindows.size(); index += 1) {
    UiStatsWindow &window = uiStatsWindows[index];
    ImGui::PushID(static_cast<int>(window.id));
    const std::string label = statsWindowDisplayLabel(window);

    if (window.open) {
      if (ImGui::SmallButton("Hide")) {
        window.open = false;
      }
      ImGui::SameLine();
      if (ImGui::SmallButton("Focus")) {
        window.pending_focus = true;
      }
    } else {
      if (ImGui::SmallButton("Show")) {
        window.open = true;
        window.pending_focus = true;
      }
      ImGui::SameLine();
      ImGui::TextDisabled("Hidden");
    }

    ImGui::SameLine();
    if (ImGui::SmallButton("Remove")) {
      removeIndex = index;
    }
    ImGui::SameLine();
    ImGui::TextWrapped("%s", label.c_str());
    ImGui::PopID();
  }
  if (removeIndex) {
    uiStatsWindows.erase(uiStatsWindows.begin() + static_cast<std::ptrdiff_t>(*removeIndex));
  }
  ImGui::EndChild();
}

void SubtrActorPlugin::focusTopLoadedWindow() {
  int topZIndex = 0;
  UiWindowPlacement *topPlacement = nullptr;
  UiStatsWindow *topStatsWindow = nullptr;

  auto considerPlacement = [&](bool open, UiWindowPlacement &placement) {
    placement.pending_focus = false;
    if (!open || !placement.has_placement || placement.z_index <= topZIndex) {
      return;
    }
    topZIndex = placement.z_index;
    topPlacement = &placement;
    topStatsWindow = nullptr;
  };

  considerPlacement(uiLauncherOpen, launcherPlacement);
  considerPlacement(uiScoreboardOpen, scoreboardPlacement);
  considerPlacement(uiEventsOpen, eventsPlacement);
  considerPlacement(uiStatusOpen, statusPlacement);
  considerPlacement(uiCameraOpen, cameraPlacement);
  considerPlacement(uiPlaybackControlsOpen, playbackControlsPlacement);
  considerPlacement(uiRecordingOpen, recordingPlacement);
  considerPlacement(uiGraphInspectorOpen, graphInspectorPlacement);
  considerPlacement(uiEventPlaylistOpen, eventPlaylistPlacement);
  considerPlacement(uiMechanicsReviewOpen, mechanicsReviewPlacement);
  considerPlacement(uiReplayLoadingOpen, replayLoadingPlacement);
  considerPlacement(uiModuleControlsOpen, moduleControlsPlacement);
  considerPlacement(uiTouchControlsOpen, touchControlsPlacement);
  considerPlacement(uiBoostPickupControlsOpen, boostPickupControlsPlacement);

  for (UiStatsWindow &window : uiStatsWindows) {
    window.pending_focus = false;
    if (!window.open || !window.has_placement || window.z_index <= topZIndex) {
      continue;
    }
    topZIndex = window.z_index;
    topPlacement = nullptr;
    topStatsWindow = &window;
  }

  if (topStatsWindow != nullptr) {
    topStatsWindow->pending_focus = true;
  } else if (topPlacement != nullptr) {
    topPlacement->pending_focus = true;
  }
}

void SubtrActorPlugin::resetWindowPlacements() {
  nextUiWindowZIndex = 1;
  launcherPlacement = UiWindowPlacement{};
  scoreboardPlacement = UiWindowPlacement{};
  eventsPlacement = UiWindowPlacement{};
  eventPlaylistPlacement = UiWindowPlacement{};
  statusPlacement = UiWindowPlacement{};
  cameraPlacement = UiWindowPlacement{};
  playbackControlsPlacement = UiWindowPlacement{};
  recordingPlacement = UiWindowPlacement{};
  graphInspectorPlacement = UiWindowPlacement{};
  mechanicsReviewPlacement = UiWindowPlacement{};
  replayLoadingPlacement = UiWindowPlacement{};
  moduleControlsPlacement = UiWindowPlacement{};
  touchControlsPlacement = UiWindowPlacement{};
  boostPickupControlsPlacement = UiWindowPlacement{};

  launcherPlacement.pending_focus = true;
  for (UiStatsWindow &window : uiStatsWindows) {
    window.has_placement = false;
    window.pending_apply_placement = false;
    window.pending_focus = window.open;
    window.x = 0.0f;
    window.y = 0.0f;
    window.width = 540.0f;
    window.height = 330.0f;
    if (window.kind == UiStatsWindowKind::StatsModule) {
      window.width = 680.0f;
      window.height = 460.0f;
    }
  }
}

void SubtrActorPlugin::resetDefaultStatsWindows() {
  uiStatsWindows.clear();
  nextUiStatsWindowId = 1;
  createStatsWindow(UiStatsWindowKind::Player, true);
  createStatsWindow(UiStatsWindowKind::Team, true);
  createStatsWindow(UiStatsWindowKind::GoalsOverview, true);
}

void SubtrActorPlugin::applyDefaultUiWorkspace() {
  uiLauncherOpen = true;
  uiScoreboardOpen = true;
  uiEventsOpen = true;
  uiEventPlaylistOpen = true;
  uiStatusOpen = true;
  uiCameraOpen = true;
  uiPlaybackControlsOpen = true;
  uiRecordingOpen = false;
  uiGraphInspectorOpen = false;
  uiMechanicsReviewOpen = false;
  uiReplayLoadingOpen = false;
  uiModuleControlsOpen = true;
  uiTouchControlsOpen = false;
  uiBoostPickupControlsOpen = false;
  boostPickupPadBig = true;
  boostPickupPadSmall = true;
  boostPickupPadAmbiguous = true;
  boostPickupActivityActive = true;
  boostPickupActivityInactive = true;
  boostPickupActivityUnknown = true;
  boostPickupFieldOwn = true;
  boostPickupFieldOpponent = true;
  boostPickupFieldUnknown = true;
  eventPlaylistMechanicsEnabled = true;
  eventPlaylistTeamEventsEnabled = true;
  eventPlaylistGoalContextEnabled = true;
  eventPlaylistAutoFollow = true;
  resetWindowPlacements();
  resetDefaultStatsWindows();
}

void SubtrActorPlugin::applyReplayReviewUiWorkspace() {
  uiLauncherOpen = true;
  uiScoreboardOpen = true;
  uiEventsOpen = true;
  uiEventPlaylistOpen = true;
  uiStatusOpen = false;
  uiCameraOpen = true;
  uiPlaybackControlsOpen = true;
  uiRecordingOpen = false;
  uiGraphInspectorOpen = false;
  uiMechanicsReviewOpen = true;
  uiReplayLoadingOpen = true;
  uiModuleControlsOpen = false;
  uiTouchControlsOpen = true;
  uiBoostPickupControlsOpen = true;
  eventPlaylistMechanicsEnabled = true;
  eventPlaylistTeamEventsEnabled = true;
  eventPlaylistGoalContextEnabled = true;
  eventPlaylistAutoFollow = true;
  resetWindowPlacements();
  mechanicsReviewPlacement.pending_focus = true;
  replayLoadingPlacement.pending_focus = true;
}

void SubtrActorPlugin::applyGraphDebugUiWorkspace() {
  uiLauncherOpen = true;
  uiScoreboardOpen = false;
  uiEventsOpen = true;
  uiEventPlaylistOpen = true;
  uiStatusOpen = true;
  uiCameraOpen = false;
  uiPlaybackControlsOpen = true;
  uiRecordingOpen = false;
  uiGraphInspectorOpen = true;
  uiMechanicsReviewOpen = false;
  uiReplayLoadingOpen = false;
  uiModuleControlsOpen = true;
  uiTouchControlsOpen = false;
  uiBoostPickupControlsOpen = false;
  resetWindowPlacements();
  graphInspectorPlacement.pending_focus = true;
  moduleControlsPlacement.pending_focus = true;
}

void SubtrActorPlugin::applyRecordingUiWorkspace() {
  uiLauncherOpen = true;
  uiScoreboardOpen = true;
  uiEventsOpen = false;
  uiEventPlaylistOpen = false;
  uiStatusOpen = true;
  uiCameraOpen = true;
  uiPlaybackControlsOpen = true;
  uiRecordingOpen = true;
  uiGraphInspectorOpen = false;
  uiMechanicsReviewOpen = false;
  uiReplayLoadingOpen = false;
  uiModuleControlsOpen = false;
  uiTouchControlsOpen = false;
  uiBoostPickupControlsOpen = false;
  resetWindowPlacements();
  recordingPlacement.pending_focus = true;
  cameraPlacement.pending_focus = true;
}

void SubtrActorPlugin::createStatsWindow(UiStatsWindowKind kind, bool initializeEntries) {
  UiStatsWindow window{};
  window.id = nextUiStatsWindowId++;
  window.kind = kind;
  window.pending_focus = true;
  initializeStatsWindowPlacement(window);
  if (!sampledPlayers.empty()) {
    window.selected_player_index = sampledPlayers.front().player_index;
    window.selected_team_is_team_0 = sampledPlayers.front().is_team_0;
  }
  if (initializeEntries) {
    initializeStatsWindowEntries(window);
  }
  uiStatsWindows.push_back(window);
}

void SubtrActorPlugin::createStatsModuleWindow(std::string moduleName, int moduleView) {
  UiStatsWindow window{};
  window.id = nextUiStatsWindowId++;
  window.kind = UiStatsWindowKind::StatsModule;
  window.pending_focus = true;
  window.module_name = std::move(moduleName);
  window.module_view = std::clamp(moduleView, 0, 2);
  initializeStatsWindowPlacement(window);
  uiStatsWindows.push_back(std::move(window));
}

void SubtrActorPlugin::initializeStatsWindowPlacement(UiStatsWindow &window) {
  const float offset = static_cast<float>(uiStatsWindows.size() * 18);
  const bool isModuleWindow = window.kind == UiStatsWindowKind::StatsModule;
  window.width = isModuleWindow ? 680.0f : 540.0f;
  window.height = isModuleWindow ? 460.0f : 330.0f;
  const ImVec2 position = mapWindowPositionToViewport(
      96.0f + offset,
      96.0f + offset,
      window.width,
      window.height,
      0.0f,
      0.0f);
  window.x = position.x;
  window.y = position.y;
  const ImVec2 displaySize = ImGui::GetIO().DisplaySize;
  window.viewport_width = displaySize.x;
  window.viewport_height = displaySize.y;
  window.has_placement = true;
  window.pending_apply_placement = true;
  window.z_index = nextUiWindowZIndex++;
}

void SubtrActorPlugin::renderStatsWindows() {
  for (UiStatsWindow &window : uiStatsWindows) {
    if (window.open) {
      renderStatsWindow(window);
    }
  }
}

const char *SubtrActorPlugin::statsWindowKindLabel(UiStatsWindowKind kind) const {
  switch (kind) {
  case UiStatsWindowKind::Player:
    return "Player stats";
  case UiStatsWindowKind::Team:
    return "Team stats";
  case UiStatsWindowKind::AllPlayers:
    return "All players stats";
  case UiStatsWindowKind::AllTeams:
    return "All teams stats";
  case UiStatsWindowKind::GoalsOverview:
    return "Goal labels";
  case UiStatsWindowKind::AdHoc:
    return "Ad hoc stats";
  case UiStatsWindowKind::StatsModule:
    return "Stats module";
  default:
    return "Stats";
  }
}

std::string SubtrActorPlugin::statsWindowTitle(const UiStatsWindow &window) const {
  return std::format("{}##subtr-actor-stats-{}", statsWindowDisplayLabel(window), window.id);
}

std::string SubtrActorPlugin::statsWindowDisplayLabel(const UiStatsWindow &window) const {
  if (window.kind == UiStatsWindowKind::StatsModule && !window.module_name.empty()) {
    return std::format("Stats module: {}", window.module_name);
  }
  return std::format("{} {}", statsWindowKindLabel(window.kind), window.id);
}

const SaPlayerFrame *SubtrActorPlugin::sampledPlayerByIndex(uint32_t playerIndex) const {
  const auto found = std::find_if(
      sampledPlayers.begin(),
      sampledPlayers.end(),
      [playerIndex](const SaPlayerFrame &player) {
        return player.player_index == playerIndex;
      });
  return found == sampledPlayers.end() ? nullptr : &*found;
}

void SubtrActorPlugin::initializeStatsWindowEntries(UiStatsWindow &window) {
  switch (window.kind) {
  case UiStatsWindowKind::Player:
    window.entries = {
        {"score", ""}, {"goals", ""}, {"assists", ""}, {"saves", ""},
        {"shots", ""}, {"boost", ""}, {"recent_events", ""}};
    break;
  case UiStatsWindowKind::Team:
    window.entries = {
        {"players", ""},
        {"score", ""},
        {"goals", ""},
        {"assists", ""},
        {"saves", ""},
        {"shots", ""},
        {"average_boost", ""},
        {"recent_events", ""}};
    break;
  case UiStatsWindowKind::AllPlayers:
    window.entries = {
        {"score", ""}, {"goals", ""}, {"assists", ""}, {"saves", ""},
        {"shots", ""}, {"boost", ""}, {"recent_events", ""}};
    break;
  case UiStatsWindowKind::AllTeams:
    window.entries = {
        {"players", ""},
        {"score", ""},
        {"goals", ""},
        {"shots", ""},
        {"average_boost", ""},
        {"recent_events", ""}};
    break;
  case UiStatsWindowKind::GoalsOverview:
    window.entries.clear();
    break;
  case UiStatsWindowKind::AdHoc:
    window.entries = {
        {"score", defaultAdHocTargetId("score")},
        {"average_boost", defaultAdHocTargetId("average_boost")},
        {"recent_events", defaultAdHocTargetId("recent_events")}};
    break;
  case UiStatsWindowKind::StatsModule:
    window.entries.clear();
    break;
  }
}

bool SubtrActorPlugin::statsWindowSupportsStat(
    const UiStatsWindow &window,
    std::string_view statId) const {
  const std::string localStatId = normalizeUiStatId(statId);
  const UiStatDefinition *definition = localUiStatDefinition(localStatId);
  if (!definition) {
    return false;
  }
  switch (window.kind) {
  case UiStatsWindowKind::Player:
  case UiStatsWindowKind::AllPlayers:
    return definition->player;
  case UiStatsWindowKind::Team:
  case UiStatsWindowKind::AllTeams:
    return definition->team;
  case UiStatsWindowKind::GoalsOverview:
    return definition->event;
  case UiStatsWindowKind::AdHoc:
    return definition->player || definition->team || definition->event;
  case UiStatsWindowKind::StatsModule:
    return false;
  }
  return false;
}

bool SubtrActorPlugin::statsWindowHasStat(
    const UiStatsWindow &window,
    std::string_view statId,
    std::string_view targetId) const {
  const std::string localStatId = normalizeUiStatId(statId);
  return std::find_if(
             window.entries.begin(),
             window.entries.end(),
             [&](const UiStatsWindow::Entry &entry) {
               return normalizeUiStatId(entry.stat_id) == localStatId &&
                      (targetId.empty() || entry.target_id == targetId);
             }) != window.entries.end();
}

int SubtrActorPlugin::recentEventCountForActor(std::string_view actor) const {
  return static_cast<int>(std::count_if(
      recentUiEvents.begin(),
      recentUiEvents.end(),
      [actor](const UiEventRecord &event) { return event.actor == actor; }));
}

int SubtrActorPlugin::recentEventCountForTeam(uint8_t isTeam0) const {
  const std::string label = teamLabel(isTeam0);
  return static_cast<int>(std::count_if(
      recentUiEvents.begin(),
      recentUiEvents.end(),
      [&label](const UiEventRecord &event) { return event.actor == label; }));
}

int SubtrActorPlugin::recentEventCountForType(std::string_view type) const {
  return static_cast<int>(std::count_if(
      recentUiEvents.begin(),
      recentUiEvents.end(),
      [type](const UiEventRecord &event) {
        return type == "all" || type == "recent_events" || event.type == type ||
               event.category == type;
      }));
}

const std::vector<std::string> &SubtrActorPlugin::statsModuleNames() {
  const auto now = std::chrono::steady_clock::now();
  if (now < nextStatsModuleNamesRefresh) {
    return cachedStatsModuleNames;
  }

  nextStatsModuleNamesRefresh = now + std::chrono::seconds(2);
  std::string graphInfoJson = readJsonBuffer(graphInfoJsonLen, writeGraphInfoJson);
  std::vector<std::string> names =
      parseJsonStringArrayProperty(graphInfoJson, "builtin_stats_module_names");
  if (names.empty()) {
    graphInfoJson = readNamedJsonBuffer(graphOutputJsonLen, writeGraphOutputJson, "graph_info");
    names = parseJsonStringArrayProperty(graphInfoJson, "builtin_stats_module_names");
  }
  if (!names.empty()) {
    cachedStatsModuleNames = std::move(names);
  }
  return cachedStatsModuleNames;
}

std::string SubtrActorPlugin::playerStatValue(
    const SaPlayerFrame &player,
    std::string_view statId) const {
  const std::string localStatId = normalizeUiStatId(statId);
  if (localStatId == "score") {
    return std::format("{}", player.has_match_stats != 0 ? player.match_score : 0);
  }
  if (localStatId == "goals") {
    return std::format("{}", player.has_match_stats != 0 ? player.match_goals : 0);
  }
  if (localStatId == "assists") {
    return std::format("{}", player.has_match_stats != 0 ? player.match_assists : 0);
  }
  if (localStatId == "saves") {
    return std::format("{}", player.has_match_stats != 0 ? player.match_saves : 0);
  }
  if (localStatId == "shots") {
    return std::format("{}", player.has_match_stats != 0 ? player.match_shots : 0);
  }
  if (localStatId == "boost") {
    return std::format("{:.0f}", player.boost_amount);
  }
  if (localStatId == "recent_events") {
    return std::format(
        "{}",
        recentEventCountForActor(playerLabel(player.player_index, player.is_team_0)));
  }
  return "--";
}

std::string SubtrActorPlugin::teamStatValue(uint8_t isTeam0, std::string_view statId) const {
  const std::string localStatId = normalizeUiStatId(statId);
  int players = 0;
  int score = 0;
  int goals = 0;
  int assists = 0;
  int saves = 0;
  int shots = 0;
  float boost = 0.0f;
  for (const SaPlayerFrame &player : sampledPlayers) {
    if ((player.is_team_0 != 0) != (isTeam0 != 0)) {
      continue;
    }
    players += 1;
    if (player.has_match_stats != 0) {
      score += player.match_score;
      goals += player.match_goals;
      assists += player.match_assists;
      saves += player.match_saves;
      shots += player.match_shots;
    }
    boost += player.boost_amount;
  }

  if (localStatId == "players") {
    return std::format("{}", players);
  }
  if (localStatId == "score") {
    return std::format("{}", score);
  }
  if (localStatId == "goals") {
    return std::format("{}", goals);
  }
  if (localStatId == "assists") {
    return std::format("{}", assists);
  }
  if (localStatId == "saves") {
    return std::format("{}", saves);
  }
  if (localStatId == "shots") {
    return std::format("{}", shots);
  }
  if (localStatId == "average_boost") {
    return std::format("{:.0f}", players == 0 ? 0.0f : boost / static_cast<float>(players));
  }
  if (localStatId == "recent_events") {
    return std::format("{}", recentEventCountForTeam(isTeam0));
  }
  return "--";
}

std::string SubtrActorPlugin::defaultAdHocTargetId(std::string_view statId) const {
  const std::string localStatId = normalizeUiStatId(statId);
  const UiStatDefinition *definition = localUiStatDefinition(localStatId);
  if (!definition) {
    return "";
  }
  if (definition->player) {
    return sampledPlayers.empty() ? "" : std::to_string(sampledPlayers.front().player_index);
  }
  if (definition->team) {
    return "blue";
  }
  return "";
}

std::string SubtrActorPlugin::adHocStatValue(
    std::string_view statId,
    std::string_view targetId) const {
  const std::string localStatId = normalizeUiStatId(statId);
  const UiStatDefinition *definition = localUiStatDefinition(localStatId);
  if (!definition) {
    return "--";
  }
  if (definition->player) {
    uint32_t playerIndex = sampledPlayers.empty() ? 0 : sampledPlayers.front().player_index;
    if (!targetId.empty()) {
      try {
        playerIndex = static_cast<uint32_t>(std::stoul(std::string{targetId}));
      } catch (...) {
      }
    }
    const SaPlayerFrame *player = sampledPlayerByIndex(playerIndex);
    return player ? playerStatValue(*player, localStatId) : "--";
  }
  if (definition->team) {
    return teamStatValue(targetId == "orange" ? 0 : 1, localStatId);
  }
  return std::format("{}", recentEventCountForType(localStatId));
}

void SubtrActorPlugin::renderAdHocTargetSelector(
    UiStatsWindow &window,
    UiStatsWindow::Entry &entry,
    std::string_view statId,
    size_t index) {
  const UiStatDefinition *definition = uiStatDefinition(statId);
  if (!definition || (!definition->player && !definition->team)) {
    ImGui::TextDisabled("-");
    return;
  }

  if (definition->player) {
    const SaPlayerFrame *selected = nullptr;
    if (!entry.target_id.empty()) {
      try {
        selected = sampledPlayerByIndex(static_cast<uint32_t>(std::stoul(entry.target_id)));
      } catch (...) {
      }
    }
    const std::string selectedLabel =
        selected ? playerLabel(selected->player_index, selected->is_team_0) : "Select player";
    if (ImGui::BeginCombo(std::format("##ad-hoc-target-{}-{}", window.id, index).c_str(),
                          selectedLabel.c_str())) {
      for (const SaPlayerFrame &player : sampledPlayers) {
        const std::string nextTarget = std::to_string(player.player_index);
        const bool isSelected = entry.target_id == nextTarget;
        if (ImGui::Selectable(playerLabel(player.player_index, player.is_team_0).c_str(),
                              isSelected) &&
            !statsWindowHasStat(window, statId, nextTarget)) {
          entry.target_id = nextTarget;
        }
      }
      ImGui::EndCombo();
    }
    return;
  }

  const char *selectedTeam = entry.target_id == "orange" ? "Orange" : "Blue";
  if (ImGui::BeginCombo(
          std::format("##ad-hoc-target-{}-{}", window.id, index).c_str(),
          selectedTeam)) {
    if (ImGui::Selectable("Blue", entry.target_id != "orange") &&
        !statsWindowHasStat(window, statId, "blue")) {
      entry.target_id = "blue";
    }
    if (ImGui::Selectable("Orange", entry.target_id == "orange") &&
        !statsWindowHasStat(window, statId, "orange")) {
      entry.target_id = "orange";
    }
    ImGui::EndCombo();
  }
}

void SubtrActorPlugin::renderStatsWindow(UiStatsWindow &window) {
  applyStatsWindowPlacement(window);
  if (window.pending_focus) {
    ImGui::SetNextWindowFocus();
    window.z_index = nextUiWindowZIndex++;
    window.pending_focus = false;
  }
  const std::string title = statsWindowTitle(window);
  if (!ImGui::Begin(title.c_str(), &window.open)) {
    ImGui::End();
    return;
  }
  captureStatsWindowPlacement(window);

  if (ImGui::SmallButton(std::format("Hide##stats-window-hide-{}", window.id).c_str())) {
    window.open = false;
    ImGui::End();
    return;
  }
  ImGui::SameLine();
  ImGui::TextDisabled("%s", statsWindowKindLabel(window.kind));
  ImGui::Separator();

  renderStatsWindowScopeSelector(window);
  renderStatsWindowAddControl(window);
  renderStatsWindowEntries(window);
  ImGui::End();
}

void SubtrActorPlugin::renderStatsWindowScopeSelector(UiStatsWindow &window) {
  if (window.kind == UiStatsWindowKind::Player) {
    const SaPlayerFrame *selected = sampledPlayerByIndex(window.selected_player_index);
    const std::string selectedLabel =
        selected ? playerLabel(selected->player_index, selected->is_team_0) : "Select player";
    if (ImGui::BeginCombo("Player", selectedLabel.c_str())) {
      for (const SaPlayerFrame &player : sampledPlayers) {
        const std::string label = playerLabel(player.player_index, player.is_team_0);
        const bool isSelected = player.player_index == window.selected_player_index;
        if (ImGui::Selectable(label.c_str(), isSelected)) {
          window.selected_player_index = player.player_index;
        }
      }
      ImGui::EndCombo();
    }
    ImGui::Separator();
    return;
  }

  if (window.kind == UiStatsWindowKind::Team) {
    const char *selectedTeam = window.selected_team_is_team_0 != 0 ? "Blue" : "Orange";
    if (ImGui::BeginCombo("Team", selectedTeam)) {
      if (ImGui::Selectable("Blue", window.selected_team_is_team_0 != 0)) {
        window.selected_team_is_team_0 = 1;
      }
      if (ImGui::Selectable("Orange", window.selected_team_is_team_0 == 0)) {
        window.selected_team_is_team_0 = 0;
      }
      ImGui::EndCombo();
    }
    ImGui::Separator();
    return;
  }

  if (window.kind == UiStatsWindowKind::StatsModule) {
    const std::vector<std::string> &moduleNames = statsModuleNames();
    const char *selectedModule =
        window.module_name.empty() ? "Select module" : window.module_name.c_str();
    if (ImGui::BeginCombo("Module", selectedModule)) {
      for (const std::string &moduleName : moduleNames) {
        const bool selected = moduleName == window.module_name;
        if (ImGui::Selectable(moduleName.c_str(), selected)) {
          window.module_name = moduleName;
        }
      }
      ImGui::EndCombo();
    }
    ImGui::Separator();
  }
}

void SubtrActorPlugin::renderStatsWindowAddControl(UiStatsWindow &window) {
  if (window.kind == UiStatsWindowKind::StatsModule) {
    ImGui::RadioButton(
        std::format("Frame##module-view-{}", window.id).c_str(),
        &window.module_view,
        0);
    ImGui::SameLine();
    ImGui::RadioButton(
        std::format("Module##module-view-{}", window.id).c_str(),
        &window.module_view,
        1);
    ImGui::SameLine();
    ImGui::RadioButton(
        std::format("Config##module-view-{}", window.id).c_str(),
        &window.module_view,
        2);
    ImGui::Separator();
    return;
  }

  if (window.kind == UiStatsWindowKind::GoalsOverview) {
    ImGui::Separator();
    return;
  }

  const std::string addButton = std::format("+ Add stat##{}", window.id);
  if (ImGui::Button(addButton.c_str())) {
    window.picker_open = !window.picker_open;
  }
  ImGui::SameLine();
  const std::string resetButton = std::format("Reset##{}", window.id);
  if (ImGui::Button(resetButton.c_str())) {
    initializeStatsWindowEntries(window);
  }

  if (!window.picker_open) {
    ImGui::Separator();
    return;
  }

  ImGui::BeginChild(
      std::format("stat-picker-{}", window.id).c_str(),
      ImVec2{0.0f, 190.0f},
      true);

  std::array<char, 128> queryBuffer{};
  const size_t querySize = std::min(window.picker_query.size(), queryBuffer.size() - 1);
  std::copy_n(window.picker_query.data(), querySize, queryBuffer.data());
  ImGui::SetNextItemWidth(-1.0f);
  if (ImGui::InputText(
          std::format("Search stats##{}", window.id).c_str(),
          queryBuffer.data(),
          queryBuffer.size())) {
    window.picker_query = queryBuffer.data();
  }
  if (!window.picker_query.empty()) {
    ImGui::SameLine();
    if (ImGui::SmallButton(std::format("Clear##stat-search-{}", window.id).c_str())) {
      window.picker_query.clear();
    }
  }

  std::vector<UiStatDefinitionMatch> matches;
  for (size_t index = 0; index < UI_STAT_DEFINITIONS.size(); index += 1) {
    const UiStatDefinition &definition = UI_STAT_DEFINITIONS[index];
    if (!statsWindowSupportsStat(window, definition.id)) {
      continue;
    }
    const auto score = statDefinitionSearchScore(definition, window.picker_query);
    if (!score) {
      continue;
    }
    matches.push_back(UiStatDefinitionMatch{&definition, *score, index});
  }
  std::sort(matches.begin(), matches.end(), [](const auto &left, const auto &right) {
    return left.score == right.score ? left.index < right.index : left.score < right.score;
  });

  std::vector<std::pair<std::string_view, int>> categoryCounts;
  for (const UiStatDefinitionMatch &match : matches) {
    auto found = std::find_if(
        categoryCounts.begin(),
        categoryCounts.end(),
        [&](const auto &entry) { return entry.first == match.definition->category; });
    if (found == categoryCounts.end()) {
      categoryCounts.emplace_back(match.definition->category, 1);
    } else {
      found->second += 1;
    }
  }

  for (const auto &[category, count] : categoryCounts) {
    if (count < 2) {
      continue;
    }
    const std::string label =
        std::format("Add all {} ({})##{}-{}", category, count, window.id, category);
    if (ImGui::SmallButton(label.c_str())) {
      for (const UiStatDefinitionMatch &match : matches) {
        if (category != match.definition->category) {
          continue;
        }
        const std::string targetId =
            window.kind == UiStatsWindowKind::AdHoc ? defaultAdHocTargetId(match.definition->id)
                                                    : "";
        if (!statsWindowHasStat(window, match.definition->id, targetId)) {
          window.entries.push_back(UiStatsWindow::Entry{match.definition->id, targetId});
        }
      }
    }
  }

  if (matches.empty()) {
    ImGui::Text("No matching stats.");
    ImGui::EndChild();
    ImGui::Separator();
    return;
  }

  const char *currentCategory = nullptr;
  for (const UiStatDefinitionMatch &match : matches) {
    const UiStatDefinition &definition = *match.definition;
    if (currentCategory == nullptr || std::string_view(currentCategory) != definition.category) {
      currentCategory = definition.category;
      ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "%s", currentCategory);
    }
    const bool alreadySelected = statsWindowHasStat(window, definition.id);
    const std::string itemLabel =
        std::format("{}  [{}]##{}-{}", definition.label, definition.id, window.id, definition.id);
    if (alreadySelected && window.kind != UiStatsWindowKind::AdHoc) {
      ImGui::TextDisabled("%s  [selected]", definition.label);
      continue;
    }
    if (ImGui::Selectable(itemLabel.c_str(), alreadySelected)) {
      if (window.kind == UiStatsWindowKind::AdHoc) {
        const std::string targetId = defaultAdHocTargetId(definition.id);
        if (!statsWindowHasStat(window, definition.id, targetId)) {
          window.entries.push_back(UiStatsWindow::Entry{definition.id, targetId});
        }
      } else {
        window.entries.push_back(UiStatsWindow::Entry{definition.id, ""});
      }
    }
  }
  ImGui::EndChild();
  ImGui::Separator();
}

void SubtrActorPlugin::renderStatsWindowEntries(UiStatsWindow &window) {
  if (window.kind == UiStatsWindowKind::StatsModule) {
    renderStatsModuleWindow(window);
    return;
  }

  if (window.kind == UiStatsWindowKind::GoalsOverview) {
    renderGoalsOverviewStats(window);
    return;
  }

  if (window.entries.empty()) {
    ImGui::Text("No stats added.");
    return;
  }

  switch (window.kind) {
  case UiStatsWindowKind::Player:
    if (const SaPlayerFrame *player = sampledPlayerByIndex(window.selected_player_index)) {
      renderPlayerStatsTable(window, *player);
    } else {
      ImGui::Text("Waiting for selected player.");
    }
    break;
  case UiStatsWindowKind::Team:
    renderTeamStatsTable(window, window.selected_team_is_team_0);
    break;
  case UiStatsWindowKind::AllPlayers:
    renderAllPlayersStatsTable(window);
    break;
  case UiStatsWindowKind::AllTeams:
    renderAllTeamsStatsTable(window);
    break;
  case UiStatsWindowKind::GoalsOverview:
    break;
  case UiStatsWindowKind::AdHoc:
    renderAdHocStatsWindow(window);
    break;
  case UiStatsWindowKind::StatsModule:
    break;
  }
}

void SubtrActorPlugin::renderPlayerStatsTable(
    UiStatsWindow &window,
    const SaPlayerFrame &player) {
  const std::string label = playerLabel(player.player_index, player.is_team_0);
  const LinearColor color =
      player.is_team_0 != 0 ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255};
  ImGui::TextColored(toImVec4(color), "%s", label.c_str());
  ImGui::Columns(3, "player-stat-rows", false);
  for (size_t i = 0; i < window.entries.size();) {
    const std::string &statId = window.entries[i].stat_id;
    if (!statsWindowSupportsStat(window, statId)) {
      ++i;
      continue;
    }
    ImGui::Text("%s", uiStatLabel(statId));
    ImGui::NextColumn();
    ImGui::Text("%s", playerStatValue(player, statId).c_str());
    ImGui::NextColumn();
    if (ImGui::Button(std::format("Remove##{}-{}", window.id, i).c_str())) {
      window.entries.erase(window.entries.begin() + static_cast<std::ptrdiff_t>(i));
      ImGui::Columns(1);
      return;
    }
    ImGui::NextColumn();
    ++i;
  }
  ImGui::Columns(1);
}

void SubtrActorPlugin::renderTeamStatsTable(UiStatsWindow &window, uint8_t isTeam0) {
  const LinearColor color =
      isTeam0 != 0 ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255};
  ImGui::TextColored(toImVec4(color), "%s", teamLabel(isTeam0).c_str());
  ImGui::Columns(3, "team-stat-rows", false);
  for (size_t i = 0; i < window.entries.size();) {
    const std::string &statId = window.entries[i].stat_id;
    if (!statsWindowSupportsStat(window, statId)) {
      ++i;
      continue;
    }
    ImGui::Text("%s", uiStatLabel(statId));
    ImGui::NextColumn();
    ImGui::Text("%s", teamStatValue(isTeam0, statId).c_str());
    ImGui::NextColumn();
    if (ImGui::Button(std::format("Remove##{}-{}", window.id, i).c_str())) {
      window.entries.erase(window.entries.begin() + static_cast<std::ptrdiff_t>(i));
      ImGui::Columns(1);
      return;
    }
    ImGui::NextColumn();
    ++i;
  }
  ImGui::Columns(1);
}

void SubtrActorPlugin::renderAllPlayersStatsTable(UiStatsWindow &window) {
  if (sampledPlayers.empty()) {
    ImGui::Text("Waiting for sampled players.");
    return;
  }

  auto renderTeamGroup = [&](uint8_t isTeam0) {
    const LinearColor color =
        isTeam0 != 0 ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255};
    const size_t playerCount = static_cast<size_t>(std::count_if(
        sampledPlayers.begin(),
        sampledPlayers.end(),
        [isTeam0](const SaPlayerFrame &player) { return player.is_team_0 == isTeam0; }));
    if (playerCount == 0) {
      return false;
    }

    ImGui::TextColored(toImVec4(color), "%s team", teamLabel(isTeam0).c_str());
    ImGui::SameLine();
    ImGui::TextDisabled("%zu player%s", playerCount, playerCount == 1 ? "" : "s");
    for (const SaPlayerFrame &player : sampledPlayers) {
      if (player.is_team_0 != isTeam0) {
        continue;
      }

      ImGui::PushID(static_cast<int>(player.player_index));
      const std::string playerName = playerLabel(player.player_index, player.is_team_0);
      if (ImGui::TreeNodeEx(playerName.c_str(), ImGuiTreeNodeFlags_DefaultOpen)) {
        ImGui::Columns(
            3,
            std::format("all-player-stat-rows-{}", player.player_index).c_str(),
            false);
        for (size_t i = 0; i < window.entries.size();) {
          const std::string &statId = window.entries[i].stat_id;
          if (!statsWindowSupportsStat(window, statId)) {
            ++i;
            continue;
          }
          ImGui::Text("%s", uiStatLabel(statId));
          ImGui::NextColumn();
          ImGui::Text("%s", playerStatValue(player, statId).c_str());
          ImGui::NextColumn();
          if (ImGui::SmallButton(std::format("Remove##{}-{}", window.id, i).c_str())) {
            window.entries.erase(window.entries.begin() + static_cast<std::ptrdiff_t>(i));
            ImGui::Columns(1);
            ImGui::TreePop();
            ImGui::PopID();
            return true;
          }
          ImGui::NextColumn();
          ++i;
        }
        ImGui::Columns(1);
        ImGui::TreePop();
      }
      ImGui::PopID();
    }
    return false;
  };

  if (renderTeamGroup(1)) {
    return;
  }
  if (renderTeamGroup(0)) {
    return;
  }
}

void SubtrActorPlugin::renderAllTeamsStatsTable(UiStatsWindow &window) {
  for (const uint8_t isTeam0 : {static_cast<uint8_t>(1), static_cast<uint8_t>(0)}) {
    const LinearColor color =
        isTeam0 != 0 ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255};
    ImGui::TextColored(toImVec4(color), "%s", teamLabel(isTeam0).c_str());
    ImGui::Columns(3, std::format("all-team-stat-rows-{}", isTeam0).c_str(), false);
    for (size_t i = 0; i < window.entries.size();) {
      const std::string &statId = window.entries[i].stat_id;
      if (!statsWindowSupportsStat(window, statId)) {
        ++i;
        continue;
      }
      ImGui::Text("%s", uiStatLabel(statId));
      ImGui::NextColumn();
      ImGui::Text("%s", teamStatValue(isTeam0, statId).c_str());
      ImGui::NextColumn();
      if (ImGui::SmallButton(std::format("Remove##{}-{}-{}", window.id, isTeam0, i).c_str())) {
        window.entries.erase(window.entries.begin() + static_cast<std::ptrdiff_t>(i));
        ImGui::Columns(1);
        return;
      }
      ImGui::NextColumn();
      ++i;
    }
    ImGui::Columns(1);
    ImGui::Separator();
  }
}

void SubtrActorPlugin::renderGoalsOverviewStats(UiStatsWindow &window) {
  if (lastTeamScores) {
    ImGui::Text("Score: Blue %d - Orange %d", lastTeamScores->first, lastTeamScores->second);
  } else {
    ImGui::Text("Waiting for score data.");
  }
  ImGui::Columns(2, "goal-overview-stat-rows", false);
  for (size_t i = 0; i < window.entries.size();) {
    const std::string &statId = window.entries[i].stat_id;
    if (!statsWindowSupportsStat(window, statId)) {
      ++i;
      continue;
    }
    ImGui::Text("%s", uiStatLabel(statId));
    ImGui::NextColumn();
    ImGui::Text("%d", recentEventCountForType(statId));
    ImGui::SameLine();
    if (ImGui::Button(std::format("Remove##{}-{}", window.id, i).c_str())) {
      window.entries.erase(window.entries.begin() + static_cast<std::ptrdiff_t>(i));
      ImGui::Columns(1);
      return;
    }
    ImGui::NextColumn();
    ++i;
  }
  ImGui::Columns(1);
  ImGui::Separator();
  ImGui::BeginChild("goal-labels", ImVec2{0.0f, 0.0f}, true);
  for (const UiEventRecord &event : recentUiEvents) {
    if (event.category != "goal_context" && event.type != "goal") {
      continue;
    }
    ImGui::TextColored(toImVec4(event.color), "%.2fs %s", event.time, event.actor.c_str());
    ImGui::SameLine();
    ImGui::TextWrapped("%s", event.label.c_str());
  }
  ImGui::EndChild();
}

void SubtrActorPlugin::renderAdHocStatsWindow(UiStatsWindow &window) {
  ImGui::Columns(4, "ad-hoc-stat-rows", false);
  for (size_t i = 0; i < window.entries.size();) {
    UiStatsWindow::Entry &entry = window.entries[i];
    const std::string &statId = entry.stat_id;
    if (!statsWindowSupportsStat(window, statId)) {
      ++i;
      continue;
    }
    ImGui::Text("%s", uiStatLabel(statId));
    ImGui::NextColumn();
    renderAdHocTargetSelector(window, entry, statId, i);
    ImGui::NextColumn();
    ImGui::Text("%s", adHocStatValue(statId, entry.target_id).c_str());
    ImGui::NextColumn();
    if (ImGui::Button(std::format("Remove##{}-{}", window.id, i).c_str())) {
      window.entries.erase(window.entries.begin() + static_cast<std::ptrdiff_t>(i));
      ImGui::Columns(1);
      return;
    }
    ImGui::NextColumn();
    ++i;
  }
  ImGui::Columns(1);
  ImGui::Separator();
  ImGui::BeginChild("ad-hoc-events", ImVec2{0.0f, 0.0f}, true);
  for (const UiEventRecord &event : recentUiEvents) {
    const bool selected = std::any_of(
        window.entries.begin(),
        window.entries.end(),
        [&](const UiStatsWindow::Entry &entry) {
          if (entry.stat_id == "recent_events") {
            return true;
          }
          return eventFilterAllows(entry.stat_id, event.category, event.type);
        });
    if (!selected) {
      continue;
    }
    ImGui::TextColored(toImVec4(event.color), "%.2fs %s", event.time, event.actor.c_str());
    ImGui::SameLine();
    ImGui::TextWrapped("%s", event.label.c_str());
  }
  ImGui::EndChild();
}

void SubtrActorPlugin::renderJsonSummary(const std::string &json) {
  auto renderFields = [](const char *tableId, const std::vector<JsonFieldSummary> &fields) {
    ImGui::Columns(2, tableId, false);
    for (const JsonFieldSummary &field : fields) {
      ImGui::TextWrapped("%s", field.label.c_str());
      ImGui::NextColumn();
      ImGui::TextWrapped("%s", field.value.c_str());
      ImGui::NextColumn();
    }
    ImGui::Columns(1);
  };

  bool renderedAny = false;
  auto renderObjectSection = [&](const char *label, const char *propertyName, size_t maxFields) {
    const auto object = parseJsonObjectProperty(json, propertyName);
    if (!object) {
      return;
    }
    renderedAny = true;
    std::vector<JsonFieldSummary> fields;
    collectJsonFieldSummaries(*object, "", fields, maxFields, 2);
    if (ImGui::TreeNode(label)) {
      if (fields.empty()) {
        ImGui::Text("No scalar fields.");
      } else {
        renderFields(std::format("{}-fields", propertyName).c_str(), fields);
      }
      ImGui::TreePop();
    }
  };

  const std::array<const char *, 4> arrayProperties{
      "events",
      "timeline",
      "ledger_events",
      "state_events",
  };
  std::vector<JsonFieldSummary> counts;
  for (const char *propertyName : arrayProperties) {
    const auto count = parseJsonArrayPropertyElementCount(json, propertyName);
    if (count) {
      counts.push_back(JsonFieldSummary{
          propertyName,
          std::format("{} item{}", *count, *count == 1 ? "" : "s"),
      });
    }
  }
  if (!counts.empty()) {
    renderedAny = true;
    if (ImGui::TreeNode("Event collections")) {
      renderFields("module-event-counts", counts);
      ImGui::TreePop();
    }
  }

  renderObjectSection("Team zero", "team_zero", 16);
  renderObjectSection("Team one", "team_one", 16);
  renderObjectSection("Stats", "stats", 24);

  const std::vector<std::string> playerStats = parseJsonObjectArrayProperty(json, "player_stats");
  if (!playerStats.empty()) {
    renderedAny = true;
    if (ImGui::TreeNode(std::format("Player stats ({})", playerStats.size()).c_str())) {
      for (size_t index = 0; index < playerStats.size(); index += 1) {
        ImGui::PushID(static_cast<int>(index));
        const auto playerId = parseJsonObjectProperty(playerStats[index], "player_id");
        const std::string playerLabel =
            playerId ? clippedDisplayText(*playerId, 96) : std::format("Player {}", index + 1);
        if (ImGui::TreeNode(playerLabel.c_str())) {
          const auto stats = parseJsonObjectProperty(playerStats[index], "stats");
          if (stats) {
            std::vector<JsonFieldSummary> fields;
            collectJsonFieldSummaries(*stats, "", fields, 18, 2);
            renderFields("player-stats-fields", fields);
          } else {
            ImGui::Text("No stats object.");
          }
          ImGui::TreePop();
        }
        ImGui::PopID();
      }
      ImGui::TreePop();
    }
  }

  if (!renderedAny) {
    std::vector<JsonFieldSummary> fields;
    collectJsonFieldSummaries(json, "", fields, 32, 2);
    if (fields.empty()) {
      ImGui::TextWrapped("No structured summary is available for this JSON shape.");
    } else {
      renderFields("module-top-level-fields", fields);
    }
  }
}

void SubtrActorPlugin::renderJsonInspectorPayload(
    const char *id,
    const std::string &label,
    const std::string &json) {
  if (json.empty()) {
    ImGui::TextWrapped("%s is not available from the live graph yet.", label.c_str());
    return;
  }

  ImGui::Text("%s (%zu bytes)", label.c_str(), json.size());
  ImGui::SameLine();
  if (ImGui::SmallButton(std::format("Copy##{}-json", id).c_str())) {
    ImGui::SetClipboardText(json.c_str());
  }

  renderJsonSummary(json);
  ImGui::Separator();

  const std::string display = clippedDisplayText(json);
  if (ImGui::TreeNode(std::format("Raw JSON##{}-raw", id).c_str())) {
    ImGui::BeginChild(
        std::format("{}-json", id).c_str(),
        ImVec2{0.0f, 220.0f},
        true,
        ImGuiWindowFlags_HorizontalScrollbar);
    ImGui::TextUnformatted(display.c_str(), display.c_str() + display.size());
    ImGui::EndChild();
    ImGui::TreePop();
  }
}

void SubtrActorPlugin::renderStatsModuleWindow(UiStatsWindow &window) {
  if (!loaded || !engine) {
    ImGui::TextWrapped("Start live analysis to inspect graph-backed stats modules.");
    return;
  }

  const std::vector<std::string> &moduleNames = statsModuleNames();
  if (window.module_name.empty() && !moduleNames.empty()) {
    window.module_name = moduleNames.front();
  }
  if (window.module_name.empty()) {
    ImGui::TextWrapped("No builtin stats modules are available yet.");
    return;
  }

  const char *viewLabel = "frame";
  std::string json;
  if (window.module_view == 1) {
    viewLabel = "module";
    json = readNamedJsonBuffer(statsModuleJsonLen, writeStatsModuleJson, window.module_name);
  } else if (window.module_view == 2) {
    viewLabel = "config";
    json = readNamedJsonBuffer(
        statsModuleConfigJsonLen,
        writeStatsModuleConfigJson,
        window.module_name);
  } else {
    window.module_view = 0;
    json = readNamedJsonBuffer(
        statsModuleFrameJsonLen,
        writeStatsModuleFrameJson,
        window.module_name);
  }

  if (json.empty()) {
    ImGui::TextWrapped(
        "The '%s' %s JSON is not available from the live graph yet.",
        window.module_name.c_str(),
        viewLabel);
    return;
  }

  ImGui::Text(
      "%s %s JSON (%zu bytes)",
      window.module_name.c_str(),
      viewLabel,
      json.size());
  ImGui::SameLine();
  if (ImGui::SmallButton(std::format("Copy##module-json-{}", window.id).c_str())) {
    ImGui::SetClipboardText(json.c_str());
  }

  renderJsonSummary(json);
  ImGui::Separator();

  const std::string display = clippedDisplayText(std::move(json));
  if (ImGui::TreeNode(std::format("Raw JSON##module-raw-{}", window.id).c_str())) {
    ImGui::BeginChild(
        std::format("module-json-{}", window.id).c_str(),
        ImVec2{0.0f, 220.0f},
        true,
        ImGuiWindowFlags_HorizontalScrollbar);
    ImGui::TextUnformatted(display.c_str(), display.c_str() + display.size());
    ImGui::EndChild();
    ImGui::TreePop();
  }
}

void SubtrActorPlugin::render(CanvasWrapper canvas) {
  auto overlayEnabledCvar = cvarManager->getCvar("subtr_actor_overlay_enabled");
  const bool overlayEnabled =
      !static_cast<bool>(overlayEnabledCvar) || overlayEnabledCvar.getBoolValue();
  auto statusOverlayEnabledCvar = cvarManager->getCvar("subtr_actor_status_overlay_enabled");
  const bool statusOverlayEnabled = !static_cast<bool>(statusOverlayEnabledCvar) ||
                                    statusOverlayEnabledCvar.getBoolValue();
  const float scale = overlayScale();
  const int lineHeight = static_cast<int>(std::round(24.0f * scale));
  const int messageLineHeight =
      static_cast<int>(std::round(static_cast<float>(lineHeight) * 1.25f));
  const Vector2 panelPosition{overlayX(), overlayY()};

  if (overlayEnabled) {
    const auto now = std::chrono::steady_clock::now();
    while (!messages.empty() && messages.front().expires_at <= now) {
      messages.pop_front();
    }
  }

  std::optional<std::pair<std::string, LinearColor>> statusLine;

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
    statusLine = std::pair{
        status,
        (processingEnabled || replayAnnotationActive) ? LinearColor{80, 255, 150, 255}
                                                      : LinearColor{180, 180, 180, 255}};
  }

  if (!statusLine && (!overlayEnabled || messages.empty())) {
    return;
  }

  float panelWidth = 0.0f;
  int panelHeight = 0;
  if (statusLine) {
    panelWidth =
        std::max(panelWidth, canvas.GetStringSize(statusLine->first, scale, scale).X);
    panelHeight += lineHeight;
  }
  if (overlayEnabled) {
    for (const OverlayMessage &message : messages) {
      panelWidth = std::max(
          panelWidth,
          canvas.GetStringSize(message.text, scale * 1.25f, scale * 1.25f).X);
      panelHeight += messageLineHeight;
    }
  }

  constexpr float panelPaddingX = 12.0f;
  constexpr float panelPaddingY = 10.0f;
  canvas.SetPosition(Vector2F{
      static_cast<float>(panelPosition.X) - panelPaddingX,
      static_cast<float>(panelPosition.Y) - panelPaddingY});
  canvas.SetColor(LinearColor{8, 12, 16, 180});
  canvas.FillBox(Vector2F{
      panelWidth + panelPaddingX * 2.0f,
      static_cast<float>(panelHeight) + panelPaddingY * 2.0f});

  Vector2 position = panelPosition;
  if (statusLine) {
    canvas.SetPosition(position);
    canvas.SetColor(statusLine->second);
    canvas.DrawString(statusLine->first, scale, scale, true);
    position.Y += lineHeight;
  }

  if (!overlayEnabled) {
    return;
  }

  for (const OverlayMessage &message : messages) {
    canvas.SetPosition(position);
    canvas.SetColor(message.color);
    canvas.DrawString(message.text, scale * 1.25f, scale * 1.25f, true);
    position.Y += messageLineHeight;
  }
}
