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
#include <tuple>
#include <type_traits>
#include <unordered_set>

#include "imgui/imgui.h"

BAKKESMOD_PLUGIN(
    SubtrActorPlugin,
    "subtr-actor mechanic overlay",
    "0.1.0",
    PLUGINTYPE_FREEPLAY | PLUGINTYPE_CUSTOM_TRAINING | PLUGINTYPE_REPLAY)

namespace {

constexpr float PI = 3.14159265358979323846f;
constexpr float UNREAL_ROTATOR_TO_RADIANS = (2.0f * PI) / 65536.0f;
constexpr float GOAL_WATCH_LEAD_SECONDS = 4.0f;
constexpr ImGuiWindowFlags UI_FLOATING_WINDOW_FLAGS =
    ImGuiWindowFlags_NoTitleBar | ImGuiWindowFlags_NoCollapse;
constexpr ImGuiWindowFlags UI_CHROME_WINDOW_FLAGS =
    ImGuiWindowFlags_NoTitleBar | ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoMove |
    ImGuiWindowFlags_NoScrollbar | ImGuiWindowFlags_NoCollapse |
    ImGuiWindowFlags_AlwaysAutoResize | ImGuiWindowFlags_NoSavedSettings;
constexpr ImGuiWindowFlags UI_LAUNCHER_MENU_WINDOW_FLAGS =
    ImGuiWindowFlags_NoTitleBar | ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoMove |
    ImGuiWindowFlags_NoCollapse | ImGuiWindowFlags_NoSavedSettings;
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

float rightAnchoredUiX(float width, float margin = 16.0f) {
  const ImVec2 displaySize = ImGui::GetIO().DisplaySize;
  const float viewportWidth = displaySize.x > 0.0f ? displaySize.x : 1440.0f;
  return std::max(16.0f, viewportWidth - width - margin);
}

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
  const char *group;
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

struct UiStatDefinitionCandidate {
  std::string id;
  std::string label;
  std::string category;
  bool player = false;
  bool team = false;
  bool event = false;
};

struct UiStatDefinitionMatch {
  UiStatDefinitionCandidate definition;
  double score = 0.0;
  size_t index = 0;
};

struct UiStatIdAlias {
  const char *external_id;
  const char *local_id;
};

std::string normalizeUiStatId(std::string_view statId);

constexpr std::array<EventFilterOption, 33> EVENT_FILTER_OPTIONS{{
    {"all", "All events", "All"},
    {"goal", "Goal", "Replay"},
    {"mechanics", "All mechanics", "Sources"},
    {"team", "Team events", "Sources"},
    {"goal_context", "Goal context", "Sources"},
    {"touch", "Touch", "Stats"},
    {"touch_ball_movement", "Touch movement", "Stats"},
    {"dodge_reset", "Dodge refresh", "Stats"},
    {"boost_pickup", "Boost pickup", "Stats"},
    {"boost_ledger", "Boost ledger", "Stats"},
    {"boost_state", "Boost state", "Stats"},
    {"backboard", "Backboard", "Mechanics"},
    {"speed_flip", "Speed flip", "Mechanics"},
    {"half_flip", "Half flip", "Mechanics"},
    {"powerslide", "Powerslide", "Mechanics"},
    {"wavedash", "Wavedash", "Mechanics"},
    {"ball_carry", "Ball carry", "Mechanics"},
    {"air_dribble", "Air dribble", "Mechanics"},
    {"ceiling_shot", "Ceiling shot", "Mechanics"},
    {"wall_aerial", "Wall aerial", "Mechanics"},
    {"wall_aerial_shot", "Wall aerial shot", "Mechanics"},
    {"center", "Center", "Mechanics"},
    {"flip_reset", "Flip reset", "Mechanics"},
    {"double_tap", "Double tap", "Mechanics"},
    {"fifty_fifty", "50/50", "Mechanics"},
    {"flick", "Flick", "Mechanics"},
    {"musty_flick", "Musty flick", "Mechanics"},
    {"one_timer", "One timer", "Mechanics"},
    {"pass", "Pass", "Mechanics"},
    {"half_volley", "Half volley", "Mechanics"},
    {"whiff", "Whiff", "Mechanics"},
    {"bump", "Bump", "Mechanics"},
    {"demo", "Demo", "Mechanics"},
}};

constexpr std::array<const char *, 24> MECHANIC_FILTER_TOKENS{{
    "speed_flip",
    "half_flip",
    "powerslide",
    "wavedash",
    "ball_carry",
    "air_dribble",
    "backboard",
    "ceiling_shot",
    "wall_aerial",
    "wall_aerial_shot",
    "center",
    "flip_reset",
    "double_tap",
    "fifty_fifty",
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

constexpr auto UI_STAT_DEFINITIONS = std::to_array<UiStatDefinition>({
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
    {"player:touch.touch_count", "Touches", "Touch", true, false, false},
    {"player:touch.aerial_touch_count", "Aerial touches", "Touch", true, false, false},
    {"player:touch.wall_touch_count", "Wall touches", "Touch", true, false, false},
    {"player:boost.amount_used", "Boost used", "Boost", true, false, false},
    {"player:boost.amount_collected", "Boost collected", "Boost", true, false, false},
    {"player:boost.big_pads_collected", "Big pads", "Boost", true, false, false},
    {"player:boost.small_pads_collected", "Small pads", "Boost", true, false, false},
    {"player:movement.total_distance", "Distance", "Movement", true, false, false},
    {"player:movement.time_supersonic_speed", "Supersonic time", "Movement", true, false, false},
    {"player:speed_flip.count", "Speed flips", "Mechanics", true, false, false},
    {"player:speed_flip.high_confidence_count", "Clean speed flips", "Mechanics", true, false, false},
    {"player:half_flip.count", "Half flips", "Mechanics", true, false, false},
    {"player:half_flip.high_confidence_count", "Clean half flips", "Mechanics", true, false, false},
    {"player:wavedash.count", "Wavedashes", "Mechanics", true, false, false},
    {"player:wavedash.high_confidence_count", "Clean wavedashes", "Mechanics", true, false, false},
    {"player:demo.demos_inflicted", "Demos inflicted", "Contact", true, false, false},
    {"player:demo.demos_taken", "Demos taken", "Contact", true, false, false},
    {"player:bump.bumps_inflicted", "Bumps inflicted", "Contact", true, false, false},
    {"player:bump.bumps_taken", "Bumps taken", "Contact", true, false, false},
    {"player:double_tap.count", "Double taps", "Mechanics", true, false, false},
    {"player:air_dribble.count", "Air dribbles", "Mechanics", true, false, false},
    {"player:ball_carry.carry_count", "Ball carries", "Mechanics", true, false, false},
    {"player:pass.completed_pass_count", "Completed passes", "Team play", true, false, false},
    {"player:pass.received_pass_count", "Received passes", "Team play", true, false, false},
    {"player:flick.count", "Flicks", "Mechanics", true, false, false},
    {"player:musty_flick.count", "Musty flicks", "Mechanics", true, false, false},
    {"player:wall_aerial.count", "Wall aerials", "Mechanics", true, false, false},
    {"player:wall_aerial_shot.count", "Wall aerial shots", "Mechanics", true, false, false},
    {"player:ceiling_shot.count", "Ceiling shots", "Mechanics", true, false, false},
    {"player:whiff.whiff_count", "Whiffs", "Mechanics", true, false, false},
    {"player:powerslide.press_count", "Powerslides", "Movement", true, false, false},
    {"player:powerslide.total_duration", "Powerslide time", "Movement", true, false, false},
    {"team:possession.possession_time", "Possession time", "Possession", false, true, false},
    {"team:possession.opponent_possession_time", "Opponent possession", "Possession", false, true, false},
    {"team:pressure.offensive_half_time", "Offensive pressure", "Pressure", false, true, false},
    {"team:pressure.defensive_half_time", "Defensive pressure", "Pressure", false, true, false},
    {"team:boost.amount_used", "Boost used", "Boost", false, true, false},
    {"team:boost.amount_collected", "Boost collected", "Boost", false, true, false},
    {"team:movement.total_distance", "Distance", "Movement", false, true, false},
    {"team:movement.time_supersonic_speed", "Supersonic time", "Movement", false, true, false},
    {"team:demo.demos_inflicted", "Demos inflicted", "Contact", false, true, false},
    {"team:bump.bumps_inflicted", "Bumps inflicted", "Contact", false, true, false},
    {"team:double_tap.count", "Double taps", "Mechanics", false, true, false},
    {"team:air_dribble.count", "Air dribbles", "Mechanics", false, true, false},
    {"team:ball_carry.carry_count", "Ball carries", "Mechanics", false, true, false},
    {"team:pass.completed_pass_count", "Completed passes", "Team play", false, true, false},
    {"team:rush.count", "Rushes", "Team play", false, true, false},
    {"team:fifty_fifty.wins", "50/50 wins", "Team play", false, true, false},
    {"team:fifty_fifty.losses", "50/50 losses", "Team play", false, true, false},
});

constexpr auto UI_STAT_ID_ALIASES = std::to_array<UiStatIdAlias>({
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
});

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

void collectJsonLeafStatPaths(
    const std::string &json,
    std::string_view prefix,
    std::vector<std::string> &out,
    int maxDepth) {
  if (maxDepth < 0) {
    return;
  }

  size_t offset = 0;
  skipJsonWhitespace(json, offset);
  if (offset >= json.size() || json[offset] != '{') {
    return;
  }
  ++offset;
  skipJsonWhitespace(json, offset);

  while (offset < json.size()) {
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
    if (offset < json.size() && json[offset] == '{') {
      size_t end = offset;
      if (!skipJsonValue(json, end)) {
        return;
      }
      collectJsonLeafStatPaths(json.substr(valueStart, end - valueStart), label, out, maxDepth - 1);
      offset = end;
    } else {
      out.push_back(label);
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

std::string urlEncode(std::string_view value) {
  constexpr char HEX[] = "0123456789ABCDEF";
  std::string encoded;
  encoded.reserve(value.size());
  for (const unsigned char byte : value) {
    const bool unreserved =
        (byte >= 'A' && byte <= 'Z') || (byte >= 'a' && byte <= 'z') ||
        (byte >= '0' && byte <= '9') || byte == '-' || byte == '_' ||
        byte == '.' || byte == '~';
    if (unreserved) {
      encoded.push_back(static_cast<char>(byte));
      continue;
    }
    encoded.push_back('%');
    encoded.push_back(HEX[byte >> 4]);
    encoded.push_back(HEX[byte & 0x0F]);
  }
  return encoded;
}

int urlHexValue(char ch) {
  if (ch >= '0' && ch <= '9') {
    return ch - '0';
  }
  if (ch >= 'A' && ch <= 'F') {
    return ch - 'A' + 10;
  }
  if (ch >= 'a' && ch <= 'f') {
    return ch - 'a' + 10;
  }
  return -1;
}

std::optional<std::string> urlDecode(std::string_view value) {
  std::string decoded;
  decoded.reserve(value.size());
  for (size_t index = 0; index < value.size(); index += 1) {
    const char ch = value[index];
    if (ch == '+') {
      decoded.push_back(' ');
      continue;
    }
    if (ch != '%') {
      decoded.push_back(ch);
      continue;
    }
    if (index + 2 >= value.size()) {
      return std::nullopt;
    }
    const int high = urlHexValue(value[index + 1]);
    const int low = urlHexValue(value[index + 2]);
    if (high < 0 || low < 0) {
      return std::nullopt;
    }
    decoded.push_back(static_cast<char>((high << 4) | low));
    index += 2;
  }
  return decoded;
}

std::optional<std::string> statsPlayerCfgJsonFromClipboard(std::string_view clipboardText) {
  const size_t firstByte = clipboardText.find_first_not_of(" \t\r\n");
  if (firstByte == std::string_view::npos) {
    return std::nullopt;
  }
  if (clipboardText[firstByte] == '{') {
    return std::string{clipboardText.substr(firstByte)};
  }

  const size_t cfgOffset = clipboardText.find("cfg=");
  if (cfgOffset == std::string_view::npos) {
    return std::nullopt;
  }
  const size_t valueStart = cfgOffset + 4;
  size_t valueEnd = clipboardText.find_first_of("&# \t\r\n", valueStart);
  if (valueEnd == std::string_view::npos) {
    valueEnd = clipboardText.size();
  }
  std::optional<std::string> decoded =
      urlDecode(clipboardText.substr(valueStart, valueEnd - valueStart));
  if (!decoded) {
    return std::nullopt;
  }
  const size_t decodedFirstByte = decoded->find_first_not_of(" \t\r\n");
  if (decodedFirstByte == std::string::npos || (*decoded)[decodedFirstByte] != '{') {
    return std::nullopt;
  }
  return decoded->substr(decodedFirstByte);
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

std::string formatGraphStatNumber(double value) {
  if (!std::isfinite(value)) {
    return "--";
  }
  if (std::trunc(value) == value) {
    return std::format("{:.0f}", value);
  }

  double rounded = std::round(value * 1000.0) / 1000.0;
  if (rounded == 0.0) {
    rounded = 0.0;
  }

  std::string formatted = std::format("{:.3f}", rounded);
  while (!formatted.empty() && formatted.back() == '0') {
    formatted.pop_back();
  }
  if (!formatted.empty() && formatted.back() == '.') {
    formatted.pop_back();
  }
  return formatted.empty() ? "--" : formatted;
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

std::optional<std::string> parseJsonPropertyValue(
    const std::string &json,
    std::string_view propertyName) {
  size_t offset = 0;
  if (!findJsonPropertyValueOffset(json, std::string{propertyName}, offset)) {
    return std::nullopt;
  }
  const size_t start = offset;
  size_t end = offset;
  if (!skipJsonValue(json, end)) {
    return std::nullopt;
  }
  return json.substr(start, end - start);
}

std::vector<std::string_view> dotPathSegments(std::string_view path) {
  std::vector<std::string_view> segments;
  size_t offset = 0;
  while (offset < path.size()) {
    const size_t end = path.find('.', offset);
    segments.emplace_back(
        path.data() + offset,
        (end == std::string_view::npos ? path.size() : end) - offset);
    if (end == std::string_view::npos) {
      break;
    }
    offset = end + 1;
  }
  return segments;
}

std::string formatGraphStatJsonValueAt(const std::string &json, size_t offset) {
  skipJsonWhitespace(json, offset);
  if (offset >= json.size()) {
    return "--";
  }

  if (json.compare(offset, 4, "null") == 0) {
    return "--";
  }
  if (json.compare(offset, 4, "true") == 0) {
    return "true";
  }
  if (json.compare(offset, 5, "false") == 0) {
    return "false";
  }

  if (json[offset] == '"') {
    auto value = parseJsonString(json, offset);
    return value ? clippedDisplayText(*value, 240) : "--";
  }

  if (json[offset] == '[') {
    const auto count = parseJsonArrayElementCountAt(json, offset);
    if (count && *count == 0) {
      return "[]";
    }
    const size_t start = offset;
    size_t end = offset;
    if (!skipJsonValue(json, end)) {
      return "--";
    }
    return clippedDisplayText(json.substr(start, end - start), 240);
  }

  if (json[offset] == '{') {
    return "object";
  }

  const size_t start = offset;
  size_t end = offset;
  if (!skipJsonValue(json, end)) {
    return "--";
  }
  try {
    return formatGraphStatNumber(std::stod(json.substr(start, end - start)));
  } catch (...) {
    return "--";
  }
}

std::optional<std::string> jsonDisplayValueAtPath(
    const std::string &json,
    std::string_view path) {
  std::string current = json;
  const std::vector<std::string_view> segments = dotPathSegments(path);
  if (segments.empty()) {
    return std::nullopt;
  }
  for (size_t index = 0; index < segments.size(); index += 1) {
    const auto value = parseJsonPropertyValue(current, segments[index]);
    if (!value) {
      return std::nullopt;
    }
    if (index + 1 == segments.size()) {
      return formatGraphStatJsonValueAt(*value, 0);
    }
    size_t offset = 0;
    skipJsonWhitespace(*value, offset);
    if (offset >= value->size() || (*value)[offset] != '{') {
      return std::nullopt;
    }
    current = *value;
  }
  return std::nullopt;
}

std::optional<double> parseJsonNumberValue(const std::string &json) {
  size_t offset = 0;
  skipJsonWhitespace(json, offset);
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

struct GraphStatId {
  std::string_view scope;
  std::string_view module;
  std::string_view path;
};

std::optional<GraphStatId> parseGraphStatId(std::string_view statId) {
  const size_t scopeEnd = statId.find(':');
  if (scopeEnd == std::string_view::npos) {
    return std::nullopt;
  }
  const size_t moduleEnd = statId.find('.', scopeEnd + 1);
  if (moduleEnd == std::string_view::npos || moduleEnd + 1 >= statId.size()) {
    return std::nullopt;
  }
  return GraphStatId{
      statId.substr(0, scopeEnd),
      statId.substr(scopeEnd + 1, moduleEnd - scopeEnd - 1),
      statId.substr(moduleEnd + 1),
  };
}

bool jsonPlayerIdMatchesIndex(const std::string &playerIdJson, uint32_t playerIndex) {
  const auto splitScreenValue = parseJsonPropertyValue(playerIdJson, "SplitScreen");
  if (!splitScreenValue) {
    return false;
  }
  const auto parsedIndex = parseJsonNumberValue(*splitScreenValue);
  return parsedIndex && static_cast<uint32_t>(*parsedIndex) == playerIndex;
}

std::string graphStatLabel(const GraphStatId &stat) {
  return std::format("{}.{}", stat.module, stat.path);
}

std::vector<UiStatDefinitionCandidate> graphStatDefinitionsFromStatsJson(
    const std::string &statsJson) {
  std::vector<UiStatDefinitionCandidate> definitions;
  const auto frame = parseJsonObjectProperty(statsJson, "frame");
  if (!frame) {
    return definitions;
  }
  const auto modules = parseJsonObjectProperty(*frame, "modules");
  if (!modules) {
    return definitions;
  }

  std::unordered_set<std::string> seenIds;
  for (const std::string &moduleName : parseJsonObjectKeys(*modules)) {
    const auto module = parseJsonObjectProperty(*modules, moduleName);
    if (!module) {
      continue;
    }

    auto appendStatsObject = [&](const std::string &statsObject, const char *scope) {
      std::vector<std::string> paths;
      collectJsonLeafStatPaths(statsObject, "", paths, 8);
      for (const std::string &path : paths) {
        const std::string id = std::format("{}:{}.{}", scope, moduleName, path);
        if (!seenIds.insert(id).second || normalizeUiStatId(id) != id) {
          continue;
        }
        definitions.push_back(UiStatDefinitionCandidate{
            id,
            std::format("{}.{}", moduleName, path),
            moduleName,
            std::string_view{scope} == "player",
            std::string_view{scope} == "team",
            false,
        });
      }
    };

    const std::vector<std::string> playerStats =
        parseJsonObjectArrayProperty(*module, "player_stats");
    if (!playerStats.empty()) {
      if (const auto stats = parseJsonObjectProperty(playerStats.front(), "stats")) {
        appendStatsObject(*stats, "player");
      }
    }
    if (const auto team = parseJsonObjectProperty(*module, "team_zero")) {
      appendStatsObject(*team, "team");
    } else if (const auto teamOne = parseJsonObjectProperty(*module, "team_one")) {
      appendStatsObject(*teamOne, "team");
    }
  }

  return definitions;
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

static_assert(std::is_standard_layout_v<SaReplayPlayerInfo>);
static_assert(sizeof(SaReplayPlayerInfo) == 16);
static_assert(alignof(SaReplayPlayerInfo) == 8);
static_assert(offsetof(SaReplayPlayerInfo, player_index) == 0);
static_assert(offsetof(SaReplayPlayerInfo, is_team_0) == 4);
static_assert(offsetof(SaReplayPlayerInfo, name) == 8);

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

std::string uiStatLabel(std::string_view statId) {
  if (const UiStatDefinition *definition = uiStatDefinition(statId)) {
    return definition->label;
  }
  const auto parsed = parseGraphStatId(statId);
  return parsed ? graphStatLabel(*parsed) : "Stat";
}

const char *uiStatScopeLabel(bool player, bool team, bool event) {
  if (player && team) {
    return "player/team";
  }
  if (player) {
    return "player";
  }
  if (team) {
    return "team";
  }
  if (event) {
    return "event";
  }
  return "stat";
}

const char *uiStatScopeLabel(const UiStatDefinition &definition) {
  return uiStatScopeLabel(definition.player, definition.team, definition.event);
}

const char *uiStatScopeLabel(const UiStatDefinitionCandidate &definition) {
  return uiStatScopeLabel(definition.player, definition.team, definition.event);
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
    std::string_view category,
    std::string_view label,
    std::string_view id,
    std::string_view query) {
  const std::vector<std::string_view> tokens = statSearchTokens(query);
  if (tokens.empty()) {
    return 0.0;
  }

  const std::string searchText = normalizeStatSearchText(std::format(
      "{} {} {}",
      category,
      label,
      id));
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

std::optional<double> statDefinitionSearchScore(
    const UiStatDefinitionCandidate &definition,
    std::string_view query) {
  return statDefinitionSearchScore(
      definition.category,
      definition.label,
      definition.id,
      query);
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

std::optional<uint32_t> SubtrActorPlugin::playerIndexForTargetId(
    std::string_view targetId) const {
  if (const auto parsedPlayerIndex = parseUnsignedIntegerString(targetId)) {
    return *parsedPlayerIndex;
  }
  if (const auto uniquePlayerIndex = uniqueIdPlayerIndices.find(std::string{targetId});
      uniquePlayerIndex != uniqueIdPlayerIndices.end()) {
    return uniquePlayerIndex->second;
  }
  return std::nullopt;
}

std::string SubtrActorPlugin::webPlayerIdForIndex(uint32_t playerIndex) const {
  const auto uniqueId = playerUniqueIdsByIndex.find(playerIndex);
  if (uniqueId != playerUniqueIdsByIndex.end() && !uniqueId->second.empty()) {
    return uniqueId->second;
  }
  return std::to_string(playerIndex);
}

std::string SubtrActorPlugin::webPlayerIdForWindow(const UiStatsWindow &window) const {
  if (!window.selected_player_id.empty() &&
      !parseUnsignedIntegerString(window.selected_player_id)) {
    return window.selected_player_id;
  }
  return webPlayerIdForIndex(window.selected_player_index);
}

void SubtrActorPlugin::resolveStatsWindowPlayerSelection(UiStatsWindow &window) {
  if (window.selected_player_id.empty()) {
    window.selected_player_id = webPlayerIdForIndex(window.selected_player_index);
    return;
  }
  if (const auto parsedPlayerIndex = parseUnsignedIntegerString(window.selected_player_id)) {
    window.selected_player_index = *parsedPlayerIndex;
    return;
  }
  if (const auto uniquePlayerIndex = uniqueIdPlayerIndices.find(window.selected_player_id);
      uniquePlayerIndex != uniqueIdPlayerIndices.end()) {
    window.selected_player_index = uniquePlayerIndex->second;
  }
}

std::string SubtrActorPlugin::webCameraPlayerId() const {
  if (!cameraSelectedPlayerId.empty() &&
      !parseUnsignedIntegerString(cameraSelectedPlayerId)) {
    return cameraSelectedPlayerId;
  }
  return webPlayerIdForIndex(cameraSelectedPlayerIndex);
}

void SubtrActorPlugin::resolveCameraPlayerSelection() {
  if (cameraSelectedPlayerId.empty()) {
    return;
  }
  if (const auto parsedPlayerIndex = parseUnsignedIntegerString(cameraSelectedPlayerId)) {
    cameraSelectedPlayerIndex = *parsedPlayerIndex;
    return;
  }
  if (const auto uniquePlayerIndex = uniqueIdPlayerIndices.find(cameraSelectedPlayerId);
      uniquePlayerIndex != uniqueIdPlayerIndices.end()) {
    cameraSelectedPlayerIndex = uniquePlayerIndex->second;
  }
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
        showSingletonWindow(uiLauncherOpen, launcherPlacement);
      },
      "Opens the subtr-actor in-game launcher window.",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "subtr_actor_toggle_ui",
      [this](std::vector<std::string>) {
        uiWindowOpen = true;
        uiLauncherOpen = !uiLauncherOpen;
        if (uiLauncherOpen) {
          focusSingletonWindow(launcherPlacement);
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
  showSingletonWindow(uiLauncherOpen, launcherPlacement);
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
  replayAnnotationPlayerCount = reinterpret_cast<ReplayAnnotationPlayerCount>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_replay_annotation_player_count"));
  writeReplayAnnotationPlayers = reinterpret_cast<WriteReplayAnnotationPlayers>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_write_replay_annotation_players"));
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
      !replayAnnotationPlayerCount || !writeReplayAnnotationPlayers ||
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
  replayAnnotationPlayerCount = nullptr;
  writeReplayAnnotationPlayers = nullptr;
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
  auto applyDefaultPlacementSize =
      [](const std::string &object, UiWindowPlacement &out, float width, float height) {
        if (!parseJsonNumberProperty(object, "width") || out.width <= 0.0f) {
          out.width = width;
        }
        if (!parseJsonNumberProperty(object, "height") || out.height <= 0.0f) {
          out.height = height;
        }
      };
  for (const SingletonWindowControl &window : singletonWindowControls()) {
    *window.open =
        parseJsonBoolProperty(json, window.legacy_open_key).value_or(*window.open);
  }
  eventPlaylistMechanicsEnabled = parseJsonBoolProperty(json, "event_playlist_mechanics_enabled")
                                      .value_or(eventPlaylistMechanicsEnabled);
  eventPlaylistTeamEventsEnabled = parseJsonBoolProperty(json, "event_playlist_team_enabled")
                                       .value_or(eventPlaylistTeamEventsEnabled);
  eventPlaylistGoalContextEnabled =
      parseJsonBoolProperty(json, "event_playlist_goal_context_enabled")
          .value_or(eventPlaylistGoalContextEnabled);
  eventPlaylistAutoFollow =
      parseJsonBoolProperty(json, "event_playlist_auto_follow").value_or(eventPlaylistAutoFollow);
  eventPlaylistStatus =
      parseJsonStringProperty(json, "event_playlist_status").value_or(eventPlaylistStatus);
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
  cameraSelectedPlayerId =
      parseJsonStringProperty(json, "camera_selected_player_id").value_or("");
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
    if (jsonPropertyExists(*camera, "attachedPlayerId")) {
      cameraSelectedPlayerId.clear();
    }
    if (const auto attachedPlayerId = parseJsonStringProperty(*camera, "attachedPlayerId")) {
      cameraSelectedPlayerId = *attachedPlayerId;
      if (const auto parsedPlayerIndex = parseUnsignedIntegerString(*attachedPlayerId)) {
        cameraSelectedPlayerIndex = *parsedPlayerIndex;
      } else if (const auto uniquePlayerIndex = uniqueIdPlayerIndices.find(*attachedPlayerId);
                 uniquePlayerIndex != uniqueIdPlayerIndices.end()) {
        cameraSelectedPlayerIndex = uniquePlayerIndex->second;
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
    playbackStatus = parseJsonStringProperty(*playback, "status").value_or(playbackStatus);
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
  mechanicsReviewStatus =
      parseJsonStringProperty(json, "mechanics_review_status").value_or(mechanicsReviewStatus);
  selectedGraphOutput = parseJsonStringProperty(json, "selected_graph_output").value_or("");
  selectedAnalysisNode = parseJsonStringProperty(json, "selected_analysis_node").value_or("");
  graphInspectorNodeQuery =
      parseJsonStringProperty(json, "graph_inspector_node_query").value_or("");

  if (const auto placements = parseJsonObjectProperty(json, "placements")) {
    for (const SingletonWindowControl &window : singletonWindowControls()) {
      const auto object = parseJsonObjectProperty(*placements, window.legacy_placement_key);
      if (!object) {
        continue;
      }
      loadPlacementObject(*object, *window.placement, window.open);
      applyDefaultPlacementSize(*object, *window.placement, window.width, window.height);
    }
  }
  auto loadWindowArray = [&](const char *propertyName, bool webConfig) {
    for (const std::string &object : parseJsonObjectArrayProperty(json, propertyName)) {
      const std::string id = parseJsonStringProperty(object, "id").value_or("");
      const auto placement = parseJsonObjectProperty(object, "placement");
      if (!placement) {
        continue;
      }
      for (const SingletonWindowControl &window : singletonWindowControls()) {
        if (window.web_config != webConfig || id != window.config_id) {
          continue;
        }
        loadPlacementObject(*placement, *window.placement, window.open);
        applyDefaultPlacementSize(*placement, *window.placement, window.width, window.height);
        break;
      }
    }
  };
  loadWindowArray("singletonWindows", true);
  loadWindowArray("pluginWindows", false);
  loadWindowArray("plugin_windows", false);

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

    const std::optional<UiStatsWindowKind> parsedKind = parseStatsWindowKind(*kind);
    if (!parsedKind) {
      continue;
    }

    UiStatsWindow window{};
    window.kind = *parsedKind;

    if (const auto idString = parseJsonStringProperty(object, "id")) {
      window.config_id = *idString;
      const size_t digitOffset = idString->find_first_of("0123456789");
      if (digitOffset != std::string::npos) {
        try {
          window.id = static_cast<uint32_t>(std::stoul(idString->substr(digitOffset)));
        } catch (const std::exception &) {
          window.id = 0;
        }
      }
    }
    if (const auto configId = parseJsonStringProperty(object, "config_id")) {
      window.config_id = *configId;
    }
    window.id = static_cast<uint32_t>(parseJsonNumberProperty(object, "id").value_or(window.id));
    if (window.id == 0) {
      window.id = nextUiStatsWindowId;
    }
    if (window.config_id.empty()) {
      window.config_id = std::format("stats-{}", window.id);
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
    window.selected_player_id =
        parseJsonStringProperty(object, "selected_player_id").value_or("");
    if (const auto playerId = parseJsonStringProperty(object, "playerId")) {
      window.selected_player_id = *playerId;
      if (const auto parsedPlayerIndex = parseUnsignedIntegerString(*playerId)) {
        window.selected_player_index = *parsedPlayerIndex;
      } else if (const auto uniquePlayerIndex = uniqueIdPlayerIndices.find(*playerId);
                 uniquePlayerIndex != uniqueIdPlayerIndices.end()) {
        window.selected_player_index = uniquePlayerIndex->second;
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
    const auto [defaultWidth, defaultHeight] = defaultStatsWindowSize(window.kind);
    if (window.width <= 0.0f) {
      window.width = defaultWidth;
    }
    if (window.height <= 0.0f) {
      window.height = defaultHeight;
    }
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

std::string SubtrActorPlugin::uiConfigJson() {
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
  for (const SingletonWindowControl &window : singletonWindowControls()) {
    const bool visible = window.open != nullptr && *window.open;
    file << "  \"" << window.legacy_open_key << "\": "
         << (visible ? "true" : "false") << ",\n";
  }
  file << "  \"event_playlist_mechanics_enabled\": "
       << (eventPlaylistMechanicsEnabled ? "true" : "false") << ",\n";
  file << "  \"event_playlist_team_enabled\": "
       << (eventPlaylistTeamEventsEnabled ? "true" : "false") << ",\n";
  file << "  \"event_playlist_goal_context_enabled\": "
       << (eventPlaylistGoalContextEnabled ? "true" : "false") << ",\n";
  file << "  \"event_playlist_auto_follow\": "
       << (eventPlaylistAutoFollow ? "true" : "false") << ",\n";
  file << "  \"event_playlist_status\": \"" << escapeJsonString(eventPlaylistStatus) << "\",\n";
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
       << ",\"status\":\"" << escapeJsonString(playbackStatus) << "\""
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
    file << "\"" << escapeJsonString(webCameraPlayerId()) << "\"";
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
  file << "  \"camera_selected_player_id\": \"" << escapeJsonString(webCameraPlayerId())
       << "\",\n";
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
  file << "  \"mechanics_review_status\": \"" << escapeJsonString(mechanicsReviewStatus)
       << "\",\n";
  file << "  \"selected_graph_output\": \"" << escapeJsonString(selectedGraphOutput)
       << "\",\n";
  file << "  \"selected_analysis_node\": \"" << escapeJsonString(selectedAnalysisNode)
       << "\",\n";
  file << "  \"graph_inspector_node_query\": \""
       << escapeJsonString(graphInspectorNodeQuery) << "\",\n";
  file << "  \"placements\": {\n";
  const std::array<SingletonWindowControl, 13> singletonWindows = singletonWindowControls();
  for (size_t index = 0; index < singletonWindows.size(); index += 1) {
    const SingletonWindowControl &window = singletonWindows[index];
    const bool visible = window.open != nullptr && *window.open;
    file << "    \"" << window.legacy_placement_key << "\": ";
    writePlacement(file, *window.placement, visible);
    if (index + 1 != singletonWindows.size()) {
      file << ",";
    }
    file << "\n";
  }
  file << "  },\n";
  file << "  \"singletonWindows\": [\n";
  auto writeWindowConfig = [&](const SingletonWindowControl &window, bool last) {
    const bool visible = window.open != nullptr && *window.open;
    file << "    {\"id\":\"" << window.config_id << "\",\"placement\":";
    writePlacement(file, *window.placement, visible);
    file << "}";
    if (!last) {
      file << ",";
    }
    file << "\n";
  };
  std::vector<SingletonWindowControl> webWindows;
  std::vector<SingletonWindowControl> pluginWindows;
  for (const SingletonWindowControl &window : singletonWindowControls()) {
    if (!window.web_config) {
      pluginWindows.push_back(window);
    }
  }
  webWindows = webSingletonWindowControls();
  for (size_t index = 0; index < webWindows.size(); index += 1) {
    writeWindowConfig(webWindows[index], index + 1 == webWindows.size());
  }
  file << "  ],\n";
  file << "  \"pluginWindows\": [\n";
  for (size_t index = 0; index < pluginWindows.size(); index += 1) {
    writeWindowConfig(pluginWindows[index], index + 1 == pluginWindows.size());
  }
  file << "  ],\n";
  file << "  \"stats_windows\": [\n";
  for (size_t i = 0; i < uiStatsWindows.size(); i += 1) {
    const UiStatsWindow &window = uiStatsWindows[i];
    file << "    {\"id\":" << window.id << ",\"kind\":\""
         << statsWindowKindConfigId(window.kind)
         << "\",\"open\":" << (window.open ? "true" : "false")
         << ",\"visible\":" << (window.open ? "true" : "false")
         << ",\"config_id\":\""
         << escapeJsonString(window.config_id.empty() ? std::format("stats-{}", window.id)
                                                      : window.config_id)
         << "\""
         << ",\"placement\":{\"x\":" << window.x << ",\"y\":" << window.y
         << ",\"viewport\":{\"width\":" << window.viewport_width
         << ",\"height\":" << window.viewport_height << "}"
         << ",\"zIndex\":" << window.z_index
         << ",\"visible\":" << (window.open ? "true" : "false") << "}"
         << ",\"selected_player_index\":" << window.selected_player_index
         << ",\"selected_player_id\":\"" << escapeJsonString(webPlayerIdForWindow(window)) << "\""
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
  const std::array<StatsWindowKindControl, 7> statsWindowKinds = statsWindowKindControls();
  for (size_t i = 0; i < uiStatsWindows.size(); i += 1) {
    const UiStatsWindow &window = uiStatsWindows[i];
    const auto kind = std::find_if(
        statsWindowKinds.begin(),
        statsWindowKinds.end(),
        [&](const StatsWindowKindControl &control) { return control.kind == window.kind; });
    if (kind == statsWindowKinds.end() || !kind->web_config) {
      continue;
    }
    if (wroteWebStatsWindow) {
      file << ",\n";
    }
    const std::string configId =
        window.config_id.empty() ? std::format("stats-{}", window.id) : window.config_id;
    file << "    {\"id\":\"" << escapeJsonString(configId) << "\",\"kind\":\""
         << statsWindowKindConfigId(window.kind)
         << "\",\"placement\":{\"x\":" << window.x << ",\"y\":" << window.y
         << ",\"viewport\":{\"width\":" << window.viewport_width
         << ",\"height\":" << window.viewport_height << "}"
         << ",\"zIndex\":" << window.z_index
         << ",\"visible\":" << (window.open ? "true" : "false") << "}";
    file << ",\"playerId\":";
    if (window.kind == UiStatsWindowKind::Player) {
      file << "\"" << escapeJsonString(webPlayerIdForWindow(window)) << "\"";
    } else {
      file << "null";
    }
    file << ",\"team\":";
    if (window.kind == UiStatsWindowKind::Team) {
      file << "\"" << (window.selected_team_is_team_0 != 0 ? "blue" : "orange") << "\"";
    } else {
      file << "null";
    }
    file << ",\"entries\":[";
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

void SubtrActorPlugin::importReplayAnnotationPlayers() {
  if (!replayAnnotations || !replayAnnotationPlayerCount || !writeReplayAnnotationPlayers) {
    return;
  }

  const size_t playerCount = replayAnnotationPlayerCount(replayAnnotations);
  if (playerCount == 0) {
    return;
  }

  std::vector<SaReplayPlayerInfo> players(playerCount);
  const size_t copied =
      writeReplayAnnotationPlayers(replayAnnotations, players.data(), players.size());
  for (size_t i = 0; i < copied; i += 1) {
    const SaReplayPlayerInfo &player = players[i];
    if (player.name != nullptr && player.name[0] != '\0') {
      playerNamesByIndex[player.player_index] = player.name;
    }
    playerTeamsByIndex[player.player_index] = player.is_team_0;
  }
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
    importReplayAnnotationPlayers();
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

void SubtrActorPlugin::tickMechanicsReviewClipBoundary() {
  if (!mechanicsReviewClipActive) {
    return;
  }
  if (!gameWrapper->IsInReplay()) {
    mechanicsReviewClipActive = false;
    playbackPlaying = false;
    mechanicsReviewStatus = "Clip stopped because Rocket League left replay mode";
    return;
  }

  ReplayServerWrapper replayServer = gameWrapper->GetGameEventAsReplay();
  if (replayServer.IsNull()) {
    mechanicsReviewClipActive = false;
    playbackPlaying = false;
    mechanicsReviewStatus = "Clip stopped because replay playback is unavailable";
    return;
  }

  const float currentTime = replayServer.GetReplayTimeElapsed();
  playbackCurrentTime = currentTime;
  if (currentTime < mechanicsReviewClipStartSeconds - 0.25f) {
    replayServer.StartPlaybackAtTime(mechanicsReviewClipStartSeconds);
    playbackCurrentTime = mechanicsReviewClipStartSeconds;
    playbackPlaying = true;
    mechanicsReviewStatus = std::format(
        "Returned to clip start at {:.2f}s",
        mechanicsReviewClipStartSeconds);
    return;
  }
  if (currentTime < mechanicsReviewClipEndSeconds - 0.025f) {
    return;
  }

  ReplayWrapper replay = replayServer.GetReplay();
  if (!replay.IsNull()) {
    replay.StopPlayback();
  }
  playbackCurrentTime = mechanicsReviewClipEndSeconds;
  playbackPlaying = false;
  mechanicsReviewClipActive = false;
  mechanicsReviewStatus =
      std::format("Finished clip at {:.2f}s", mechanicsReviewClipEndSeconds);
}

void SubtrActorPlugin::tick(std::string) {
  if (!loaded || !engine) {
    return;
  }

  tickReplayAnnotations();
  tickMechanicsReviewClipBoundary();

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
  cachedStatsJson.clear();
  cachedStatsJsonFrameNumber = std::numeric_limits<uint64_t>::max();

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
  playerUniqueIdsByIndex.clear();
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
  cachedStatsJson.clear();
  cachedStatsJsonFrameNumber = std::numeric_limits<uint64_t>::max();
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
      playerUniqueIdsByIndex[existing->second] = uniqueId;
      return existing->second;
    }

    const uint32_t playerIndex = nextPlayerIndex++;
    uniqueIdPlayerIndices[uniqueId] = playerIndex;
    playerUniqueIdsByIndex[playerIndex] = uniqueId;
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

std::string SubtrActorPlugin::readJsonBuffer(JsonLen len, WriteJson write) const {
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
    const std::string &name) const {
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

  renderLauncherToggleChrome();
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

  std::string_view currentGroup;
  for (const EventFilterOption &option : EVENT_FILTER_OPTIONS) {
    if (std::string_view{option.value} == "all") {
      continue;
    }
    const std::string_view optionGroup{option.group};
    if (currentGroup != optionGroup) {
      if (!currentGroup.empty()) {
        ImGui::Separator();
      }
      ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "%s", option.group);
      currentGroup = optionGroup;
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
  }
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

void SubtrActorPlugin::applySingletonWindowPlacement(UiWindowPlacement &placement) {
  for (const SingletonWindowControl &window : singletonWindowControls()) {
    if (window.placement != &placement) {
      continue;
    }
    if (window.placement == &scoreboardPlacement) {
      applyScoreboardWindowPlacement();
      return;
    }
    applyWindowPlacement(placement, window.x, window.y, window.width, window.height);
    return;
  }
  applyWindowPlacement(placement, 16.0f, 68.0f, 340.0f, 430.0f);
}

void SubtrActorPlugin::resetSingletonWindowPlacement(
    UiWindowPlacement &placement,
    float x,
    float y,
    float width,
    float height,
    bool focus) {
  const ImVec2 position =
      mapWindowPositionToViewport(x, y, width, height, 0.0f, 0.0f);
  const ImVec2 displaySize = ImGui::GetIO().DisplaySize;
  placement.has_placement = true;
  placement.pending_apply_placement = true;
  placement.pending_focus = focus;
  placement.x = position.x;
  placement.y = position.y;
  placement.width = width;
  placement.height = height;
  placement.viewport_width = displaySize.x;
  placement.viewport_height = displaySize.y;
  placement.z_index = nextUiWindowZIndex++;
}

void SubtrActorPlugin::resetScoreboardWindowPlacement(bool focus) {
  constexpr float width = 88.0f;
  constexpr float height = 34.0f;
  const ImVec2 displaySize = ImGui::GetIO().DisplaySize;
  const float x = displaySize.x > 0.0f ? (displaySize.x - width) * 0.5f : 760.0f;
  resetSingletonWindowPlacement(scoreboardPlacement, x, 11.0f, width, height, focus);
}

void SubtrActorPlugin::focusSingletonWindow(UiWindowPlacement &placement) {
  placement.pending_focus = true;
  placement.z_index = nextUiWindowZIndex++;
}

void SubtrActorPlugin::showSingletonWindow(bool &open, UiWindowPlacement &placement) {
  open = true;
  focusSingletonWindow(placement);
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
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "%s", label);
  ImGui::SameLine();
  const std::string hideLabel = std::format("Hide##singleton-window-hide-{}", label);
  const float hideWidth =
      ImGui::CalcTextSize("Hide").x + ImGui::GetStyle().FramePadding.x * 2.0f;
  const float rightAlignedX = ImGui::GetWindowContentRegionMax().x - hideWidth;
  if (rightAlignedX > ImGui::GetCursorPosX()) {
    ImGui::SetCursorPosX(rightAlignedX);
  }
  if (ImGui::SmallButton(hideLabel.c_str())) {
    open = false;
    return true;
  }
  ImGui::Separator();
  return false;
}

void SubtrActorPlugin::applyScoreboardWindowPlacement() {
  auto applyFocus = [&]() {
    if (scoreboardPlacement.pending_focus) {
      ImGui::SetNextWindowFocus();
      scoreboardPlacement.z_index = nextUiWindowZIndex++;
      scoreboardPlacement.pending_focus = false;
    }
  };

  if (scoreboardPlacement.has_placement) {
    const ImGuiCond condition =
        scoreboardPlacement.pending_apply_placement ? ImGuiCond_Always : ImGuiCond_FirstUseEver;
    const float width = std::max(70.0f, scoreboardPlacement.width);
    const float height = std::max(28.0f, scoreboardPlacement.height);
    const ImVec2 position = mapWindowPositionToViewport(
        scoreboardPlacement.x,
        scoreboardPlacement.y,
        width,
        height,
        scoreboardPlacement.viewport_width,
        scoreboardPlacement.viewport_height);
    ImGui::SetNextWindowPos(position, condition);
    scoreboardPlacement.x = position.x;
    scoreboardPlacement.y = position.y;
    scoreboardPlacement.width = width;
    scoreboardPlacement.height = height;
    scoreboardPlacement.pending_apply_placement = false;
    applyFocus();
    return;
  }

  constexpr float width = 88.0f;
  constexpr float height = 34.0f;
  const ImVec2 displaySize = ImGui::GetIO().DisplaySize;
  const float x = displaySize.x > 0.0f ? (displaySize.x - width) * 0.5f : 760.0f;
  const ImVec2 position = mapWindowPositionToViewport(x, 11.0f, width, height, 0.0f, 0.0f);
  ImGui::SetNextWindowPos(position, ImGuiCond_FirstUseEver);
  applyFocus();
}

void SubtrActorPlugin::applyStatsWindowPlacement(UiStatsWindow &window) {
  if (window.has_placement) {
    const ImGuiCond condition =
        window.pending_apply_placement ? ImGuiCond_Always : ImGuiCond_FirstUseEver;
    const auto [defaultWidth, defaultHeight] = defaultStatsWindowSize(window.kind);
    const ImVec2 size{
        std::max(180.0f, window.width > 0.0f ? window.width : defaultWidth),
        std::max(120.0f, window.height > 0.0f ? window.height : defaultHeight),
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
  const auto [width, height] = defaultStatsWindowSize(window.kind);
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

void SubtrActorPlugin::focusStatsWindow(UiStatsWindow &window) {
  window.pending_focus = true;
  window.z_index = nextUiWindowZIndex++;
}

void SubtrActorPlugin::showStatsWindow(UiStatsWindow &window) {
  window.open = true;
  focusStatsWindow(window);
}

bool SubtrActorPlugin::renderModuleSummaryToggle(
    const char *label,
    bool active,
    const char *idSuffix) {
  const std::string buttonLabel =
      std::format("{}   {}##{}-{}", label, active ? "On" : "Off", idSuffix, label);
  if (active) {
    ImGui::PushStyleColor(ImGuiCol_Button, ImVec4{0.16f, 0.35f, 0.28f, 1.0f});
    ImGui::PushStyleColor(ImGuiCol_ButtonHovered, ImVec4{0.20f, 0.45f, 0.36f, 1.0f});
  }
  const bool clicked = ImGui::Button(buttonLabel.c_str(), ImVec2{230.0f, 0.0f});
  if (active) {
    ImGui::PopStyleColor(2);
  }
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

void SubtrActorPlugin::renderEventFilterModuleSummaryToggle(
    const char *label,
    const char *token,
    const char *idSuffix) {
  std::vector<std::string> selected =
      selectedEventSourceTokens(cvarString("subtr_actor_overlay_event_types", "all"));
  const bool active =
      eventPlaylistMechanicsEnabled &&
      (containsString(selected, "mechanics") || containsString(selected, token));
  if (!renderModuleSummaryToggle(label, active, idSuffix)) {
    return;
  }

  selected.erase(
      std::remove(selected.begin(), selected.end(), std::string{"mechanics"}),
      selected.end());
  if (active) {
    selected.erase(
        std::remove(selected.begin(), selected.end(), std::string{token}),
        selected.end());
  } else {
    appendUniqueFilterToken(selected, token);
  }

  const bool hasMechanicsSelection = std::any_of(
      selected.begin(),
      selected.end(),
      [](const std::string &selectedToken) {
        return selectedToken == "mechanics" || selectedToken == "touch" ||
               isMechanicFilterToken(selectedToken);
      });
  eventPlaylistMechanicsEnabled = hasMechanicsSelection;
  setCvarString("subtr_actor_overlay_event_types", eventFilterFromSelectedSources(selected));
}

void SubtrActorPlugin::renderModuleSummaryControls(const char *idSuffix) {
  if (ImGui::TreeNodeEx(
          std::format("Timeline visualizations##{}-timeline", idSuffix).c_str(),
          ImGuiTreeNodeFlags_DefaultOpen)) {
    renderEventFilterModuleSummaryToggle("Touch", "touch", idSuffix);
    renderEventFilterModuleSummaryToggle("Dodge refresh", "dodge_reset", idSuffix);
    renderEventFilterModuleSummaryToggle("Backboard", "backboard", idSuffix);
    renderEventFilterModuleSummaryToggle("Speed flip", "speed_flip", idSuffix);
    renderEventFilterModuleSummaryToggle("Half flip", "half_flip", idSuffix);
    renderEventFilterModuleSummaryToggle("Powerslide", "powerslide", idSuffix);
    renderEventFilterModuleSummaryToggle("Wavedash", "wavedash", idSuffix);
    renderEventFilterModuleSummaryToggle("Ball carry", "ball_carry", idSuffix);
    renderEventFilterModuleSummaryToggle("Ceiling shot", "ceiling_shot", idSuffix);
    renderEventFilterModuleSummaryToggle("Flip reset", "flip_reset", idSuffix);
    renderEventFilterModuleSummaryToggle("Double tap", "double_tap", idSuffix);
    renderEventFilterModuleSummaryToggle("50/50", "fifty_fifty", idSuffix);
    renderEventFilterModuleSummaryToggle("Musty flick", "musty_flick", idSuffix);
    renderEventFilterModuleSummaryToggle("Whiff", "whiff", idSuffix);
    renderEventFilterModuleSummaryToggle("Bump", "bump", idSuffix);
    renderEventFilterModuleSummaryToggle("Demo", "demo", idSuffix);
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

  if (ImGui::TreeNodeEx(
          std::format("In-game visualizations##{}-ingame", idSuffix).c_str(),
          ImGuiTreeNodeFlags_DefaultOpen)) {
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

void SubtrActorPlugin::renderModuleSettingsControls(
    const char *idSuffix,
    bool includeOpenButtons) {
  ImGui::PushID(idSuffix);

  auto settingReadout = [](std::initializer_list<std::pair<bool, const char *>> parts,
                           std::string_view separator) {
    std::string readout;
    for (const auto &[enabled, label] : parts) {
      if (!enabled) {
        continue;
      }
      if (!readout.empty()) {
        readout += separator;
      }
      readout += label;
    }
    return readout.empty() ? std::string{"Total only"} : readout;
  };

  ImGui::TextDisabled("Movement breakdown");
  ImGui::SameLine();
  ImGui::TextColored(
      ImVec4{0.53f, 0.69f, 0.83f, 1.0f},
      "%s",
      settingReadout(
          {{movementBreakdownSpeed, "Speed band"}, {movementBreakdownHeight, "Height band"}},
          " + ")
          .c_str());
  ImGui::Checkbox("Speed band##movement-breakdown", &movementBreakdownSpeed);
  ImGui::SameLine();
  ImGui::Checkbox("Height band##movement-breakdown", &movementBreakdownHeight);
  if (includeOpenButtons && ImGui::Button("Open movement stats")) {
    createStatsModuleWindow("movement", 0);
  }

  ImGui::TextDisabled("Possession breakdown");
  ImGui::SameLine();
  ImGui::TextColored(
      ImVec4{0.53f, 0.69f, 0.83f, 1.0f},
      "%s",
      settingReadout(
          {{possessionBreakdownState, "Control"}, {possessionBreakdownThird, "Third"}},
          " x ")
          .c_str());
  ImGui::Checkbox("Control##possession-breakdown", &possessionBreakdownState);
  ImGui::SameLine();
  ImGui::Checkbox("Third##possession-breakdown", &possessionBreakdownThird);
  if (includeOpenButtons && ImGui::Button("Open possession stats")) {
    createStatsModuleWindow("possession", 0);
  }

  ImGui::PopID();
}

void SubtrActorPlugin::renderLauncherToggleChrome() {
  if (!uiEnabled() && !uiLauncherOpen) {
    return;
  }

  ImGui::SetNextWindowPos(ImVec2{12.0f, 12.0f}, ImGuiCond_Always);
  ImGui::SetNextWindowBgAlpha(0.62f);
  ImGui::PushStyleVar(ImGuiStyleVar_WindowPadding, ImVec2{5.0f, 5.0f});
  ImGui::PushStyleVar(ImGuiStyleVar_WindowRounding, 8.0f);
  if (!ImGui::Begin("subtr-actor launcher toggle##subtr-actor", nullptr, UI_CHROME_WINDOW_FLAGS)) {
    ImGui::End();
    ImGui::PopStyleVar(2);
    return;
  }

  if (uiLauncherOpen) {
    ImGui::PushStyleColor(ImGuiCol_Button, ImVec4{0.16f, 0.35f, 0.46f, 0.92f});
    ImGui::PushStyleColor(ImGuiCol_ButtonHovered, ImVec4{0.22f, 0.46f, 0.60f, 0.98f});
    ImGui::PushStyleColor(ImGuiCol_ButtonActive, ImVec4{0.28f, 0.57f, 0.72f, 1.0f});
  }

  if (ImGui::Button("Menu##subtr-actor-launcher-toggle", ImVec2{44.0f, 28.0f})) {
    uiWindowOpen = true;
    if (uiLauncherOpen) {
      uiLauncherOpen = false;
    } else {
      showSingletonWindow(uiLauncherOpen, launcherPlacement);
    }
  }

  if (uiLauncherOpen) {
    ImGui::PopStyleColor(3);
  }
  ImGui::End();
  ImGui::PopStyleVar(2);
}

void SubtrActorPlugin::applyLauncherMenuPlacement() {
  const ImVec2 displaySize = ImGui::GetIO().DisplaySize;
  const float width = 340.0f;
  const float height = std::max(320.0f, displaySize.y > 0.0f ? displaySize.y - 68.0f : 650.0f);
  ImGui::SetNextWindowPos(ImVec2{12.0f, 50.0f}, ImGuiCond_Always);
  ImGui::SetNextWindowSize(ImVec2{width, height}, ImGuiCond_Always);
  ImGui::SetNextWindowBgAlpha(0.92f);
  if (launcherPlacement.pending_focus) {
    ImGui::SetNextWindowFocus();
    launcherPlacement.z_index = nextUiWindowZIndex++;
    launcherPlacement.pending_focus = false;
  }
}

void SubtrActorPlugin::renderLauncherWorkspaceControls() {
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "WORKSPACES");
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
    for (const SingletonWindowControl &window : singletonWindowControls()) {
      if (std::string_view{window.config_id} == "scoreboard") {
        continue;
      }
      *window.open = false;
    }
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
  if (ImGui::Button("Copy layout cfg")) {
    const std::string json = uiConfigJson();
    const std::string cfg = std::format("#cfg={}", urlEncode(json));
    ImGui::SetClipboardText(cfg.c_str());
    cvarManager->log(std::format("subtr-actor: copied {} UI config hash bytes", cfg.size()));
  }
  ImGui::SameLine();
  if (ImGui::Button("Paste layout")) {
    const char *clipboardText = ImGui::GetClipboardText();
    if (clipboardText == nullptr || clipboardText[0] == '\0') {
      cvarManager->log("subtr-actor: clipboard does not contain UI config JSON or cfg");
    } else if (const std::optional<std::string> configJson =
                   statsPlayerCfgJsonFromClipboard(clipboardText)) {
      applyUiConfigJson(*configJson, "clipboard");
    } else {
      cvarManager->log(
          "subtr-actor: clipboard does not contain raw UI config JSON or a raw JSON cfg value");
    }
  }
}

void SubtrActorPlugin::renderLauncherWindow() {
  if (!uiLauncherOpen) {
    return;
  }
  applyLauncherMenuPlacement();
  if (!ImGui::Begin(
          "subtr-actor menu##subtr-actor",
          &uiLauncherOpen,
          UI_LAUNCHER_MENU_WINDOW_FLAGS)) {
    ImGui::End();
    return;
  }
  captureWindowPlacement(launcherPlacement);

  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "ACTIONS");
  if (ImGui::Button("Load Replay...")) {
    showSingletonWindow(uiReplayLoadingOpen, replayLoadingPlacement);
    resetReplayAnnotations();
    tickReplayAnnotations();
    uiLauncherOpen = false;
  }
  const bool liveAnalysis = liveProcessingEnabled();
  if (renderModuleSummaryToggle("Live analysis graph", liveAnalysis, "launcher-actions")) {
    setCvarBool("subtr_actor_enabled", !liveAnalysis);
  }

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "WINDOWS");
  auto renderLauncherWindowToggle = [&](const SingletonWindowControl &window) {
    ImGui::PushID(window.label);
    const bool isOpen = window.open != nullptr && *window.open;
    if (isOpen) {
      ImGui::PushStyleColor(ImGuiCol_Button, ImVec4{0.16f, 0.35f, 0.28f, 1.0f});
      ImGui::PushStyleColor(ImGuiCol_ButtonHovered, ImVec4{0.20f, 0.45f, 0.36f, 1.0f});
      ImGui::PushStyleColor(ImGuiCol_ButtonActive, ImVec4{0.25f, 0.55f, 0.43f, 1.0f});
    }
    const std::string buttonLabel =
        std::format("{}   {}", window.label, isOpen ? "Hide" : "Show");
    if (ImGui::Button(buttonLabel.c_str(), ImVec2{210.0f, 0.0f})) {
      if (*window.open) {
        *window.open = false;
      } else {
        showSingletonWindow(*window.open, *window.placement);
      }
      uiLauncherOpen = false;
    }
    if (isOpen) {
      ImGui::PopStyleColor(3);
    }
    ImGui::PopID();
  };
  auto renderStatsWindowCreateButton = [&](const char *label, UiStatsWindowKind kind) {
    if (ImGui::Button(label, ImVec2{170.0f, 0.0f})) {
      createStatsWindow(kind);
      uiLauncherOpen = false;
    }
  };

  for (const SingletonWindowControl &window : webSingletonWindowControls()) {
    renderLauncherWindowToggle(window);
  }
  for (const StatsWindowKindControl &kind : statsWindowKindControls()) {
    if (kind.web_config) {
      renderStatsWindowCreateButton(kind.create_label, kind.kind);
    }
  }
  if (!uiStatsWindows.empty()) {
    const size_t visibleStatsWindows = static_cast<size_t>(std::count_if(
        uiStatsWindows.begin(),
        uiStatsWindows.end(),
        [](const UiStatsWindow &window) { return window.open; }));
    ImGui::Separator();
    ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "STATS WINDOWS");
    ImGui::Text(
        "%zu visible / %zu stats windows",
        visibleStatsWindows,
        uiStatsWindows.size());
    renderStatsWindowManager();
  }

  ImGui::Separator();
  renderLauncherWorkspaceControls();

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "VISUALIZATIONS");
  renderModuleSummaryControls("launcher-module-summary");

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "MODULE SETTINGS");
  renderModuleSettingsControls("launcher-module-settings", true);

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "GRAPH STATS MODULES");
  const std::vector<std::string> &moduleNames = statsModuleNames();
  if (moduleNames.empty()) {
    ImGui::TextWrapped("Start live analysis to list graph-backed stats modules.");
  } else {
    ImGui::BeginChild("launcher-graph-stats-modules", ImVec2{0.0f, 130.0f}, true);
    for (const std::string &moduleName : moduleNames) {
      ImGui::PushID(moduleName.c_str());
      if (ImGui::SmallButton("Frame")) {
        createStatsModuleWindow(moduleName, 0);
        uiLauncherOpen = false;
      }
      ImGui::SameLine();
      if (ImGui::SmallButton("Module")) {
        createStatsModuleWindow(moduleName, 1);
        uiLauncherOpen = false;
      }
      ImGui::SameLine();
      if (ImGui::SmallButton("Config")) {
        createStatsModuleWindow(moduleName, 2);
        uiLauncherOpen = false;
      }
      ImGui::SameLine();
      ImGui::TextWrapped("%s", moduleName.c_str());
      ImGui::PopID();
    }
    ImGui::EndChild();
  }

  if (ImGui::TreeNode("Plugin tools##launcher-plugin-tools")) {
    if (ImGui::Button("Verify graph", ImVec2{170.0f, 0.0f})) {
      showSingletonWindow(uiGraphInspectorOpen, graphInspectorPlacement);
      verifyGraphRuntime({"subtr_actor_verify_graph"});
      uiLauncherOpen = false;
    }
    if (ImGui::Button("Open modules", ImVec2{170.0f, 0.0f})) {
      showSingletonWindow(uiModuleControlsOpen, moduleControlsPlacement);
      uiLauncherOpen = false;
    }
    if (ImGui::Button("Close launcher", ImVec2{170.0f, 0.0f})) {
      uiLauncherOpen = false;
    }

    ImGui::Separator();
    for (const SingletonWindowControl &window : singletonWindowControls()) {
      if (!window.web_config) {
        renderLauncherWindowToggle(window);
      }
    }
    renderSingletonWindowManager();

    ImGui::Separator();
    renderSharedSettingsControls();
    ImGui::TreePop();
  }

  ImGui::End();
}

void SubtrActorPlugin::renderScoreboardWindow() {
  if (!uiScoreboardOpen) {
    return;
  }
  applyScoreboardWindowPlacement();
  constexpr ImGuiWindowFlags scoreboardFlags =
      ImGuiWindowFlags_NoTitleBar | ImGuiWindowFlags_AlwaysAutoResize |
      ImGuiWindowFlags_NoScrollbar | ImGuiWindowFlags_NoCollapse;
  ImGui::PushStyleVar(ImGuiStyleVar_WindowPadding, ImVec2{10.0f, 6.0f});
  ImGui::PushStyleVar(ImGuiStyleVar_WindowRounding, 999.0f);
  if (!ImGui::Begin("Scoreboard##subtr-actor", &uiScoreboardOpen, scoreboardFlags)) {
    ImGui::End();
    ImGui::PopStyleVar(2);
    return;
  }
  captureWindowPlacement(scoreboardPlacement);

  if (lastTeamScores) {
    ImGui::PushStyleVar(ImGuiStyleVar_ItemSpacing, ImVec2{8.0f, 0.0f});
    ImGui::SetWindowFontScale(1.2f);
    ImGui::TextColored(ImVec4{0.31f, 0.75f, 1.0f, 1.0f}, "%d", lastTeamScores->first);
    ImGui::SameLine();
    ImGui::TextDisabled("-");
    ImGui::SameLine();
    ImGui::TextColored(ImVec4{1.0f, 0.69f, 0.31f, 1.0f}, "%d", lastTeamScores->second);
    ImGui::SetWindowFontScale(1.0f);
    ImGui::PopStyleVar();
  } else {
    ImGui::TextDisabled("Load a replay to show the scoreboard.");
  }
  ImGui::End();
  ImGui::PopStyleVar(2);
}

void SubtrActorPlugin::renderEventsWindow() {
  if (!uiEventsOpen) {
    return;
  }
  applySingletonWindowPlacement(eventsPlacement);
  if (!ImGui::Begin("Events##subtr-actor", &uiEventsOpen, UI_FLOATING_WINDOW_FLAGS)) {
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

  const std::string currentFilter = cvarString("subtr_actor_overlay_event_types", "all");
  const bool allSelected = allEventSourcesSelected(currentFilter);
  std::vector<std::string> selected =
      selectedEventSourceTokens(currentFilter);
  auto applySelection = [&]() {
    setCvarString("subtr_actor_overlay_event_types", eventFilterFromSelectedSources(selected));
  };

  struct DisplaySource {
    const EventFilterOption *option = nullptr;
    size_t count = 0;
    bool enabled = false;
  };

  std::vector<DisplaySource> displaySources;
  displaySources.reserve(EVENT_FILTER_OPTIONS.size());
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

    const bool enabled = containsString(selected, option.value);
    if (count == 0 && (allSelected || !enabled)) {
      continue;
    }
    displaySources.push_back(DisplaySource{&option, count, enabled});
  }

  if (ImGui::SmallButton("All events##event-sources")) {
    selected = selectedEventSourceTokens("all");
    applySelection();
  }
  ImGui::SameLine();
  if (ImGui::SmallButton("No events##event-sources")) {
    selected.clear();
    applySelection();
  }
  ImGui::SameLine();
  const std::string preview =
      eventFilterPreview(cvarString("subtr_actor_overlay_event_types", "all"));
  ImGui::TextDisabled("%s", preview.c_str());
  ImGui::TextDisabled("%zu loaded sources", displaySources.size());

  ImGui::BeginChild("event-source-list", ImVec2{0.0f, 185.0f}, true);
  if (displaySources.empty()) {
    ImGui::TextDisabled("No events loaded.");
    ImGui::EndChild();
    ImGui::TreePop();
    return;
  }

  std::string_view currentGroup;
  for (const DisplaySource &source : displaySources) {
    const EventFilterOption &option = *source.option;
    const std::string_view optionGroup{option.group};
    if (currentGroup != optionGroup) {
      if (!currentGroup.empty()) {
        ImGui::Separator();
      }
      ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "%s", option.group);
      currentGroup = optionGroup;
    }

    ImGui::PushID(option.value);
    bool enabled = source.enabled;
    const std::string label = std::format("{} ({})", option.label, source.count);
    if (renderModuleSummaryToggle(label.c_str(), enabled, "event-sources")) {
      if (enabled) {
        selected.erase(
            std::remove(selected.begin(), selected.end(), std::string{option.value}),
            selected.end());
      } else {
        selected.emplace_back(option.value);
      }
      applySelection();
    }
    ImGui::PopID();
  }
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

  applySingletonWindowPlacement(eventPlaylistPlacement);
  if (!ImGui::Begin(
          "Event playlist##subtr-actor",
          &uiEventPlaylistOpen,
          UI_FLOATING_WINDOW_FLAGS)) {
    ImGui::End();
    return;
  }
  captureWindowPlacement(eventPlaylistPlacement);
  if (renderSingletonWindowHeader("Event playlist", uiEventPlaylistOpen)) {
    ImGui::End();
    return;
  }

  const std::string currentFilter = cvarString("subtr_actor_overlay_event_types", "all");
  const bool allEventSourcesEnabled = allEventSourcesSelected(currentFilter);
  std::vector<std::string> selectedSources = selectedEventSourceTokens(currentFilter);

  auto sourceHasEnabledPlaylistGroup = [&](std::string_view source) {
    for (const UiEventRecord &event : recentUiEvents) {
      if (eventFilterAllows(source, event.category, event.type) &&
          eventPlaylistSourceEnabled(event)) {
        return true;
      }
    }
    return false;
  };
  auto enableMatchingPlaylistGroups = [&](std::string_view source) {
    for (const UiEventRecord &event : recentUiEvents) {
      if (!eventFilterAllows(source, event.category, event.type)) {
        continue;
      }
      if (event.category == "mechanics") {
        eventPlaylistMechanicsEnabled = true;
      } else if (event.category == "team") {
        eventPlaylistTeamEventsEnabled = true;
      } else if (event.category == "goal_context" || event.type == "goal") {
        eventPlaylistGoalContextEnabled = true;
      }
    }
  };

  struct PlaylistSource {
    const EventFilterOption *option = nullptr;
    size_t count = 0;
    bool enabled = false;
  };

  std::vector<PlaylistSource> playlistSources;
  playlistSources.reserve(EVENT_FILTER_OPTIONS.size());
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
    const bool selected = containsString(selectedSources, option.value);
    if (count == 0 && (allEventSourcesEnabled || !selected)) {
      continue;
    }
    playlistSources.push_back(
        PlaylistSource{&option, count, selected && sourceHasEnabledPlaylistGroup(option.value)});
  }

  const size_t selectedSourceCount = static_cast<size_t>(std::count_if(
      playlistSources.begin(),
      playlistSources.end(),
      [](const PlaylistSource &source) { return source.enabled; }));

  std::vector<size_t> playlistEventIndexes;
  playlistEventIndexes.reserve(recentUiEvents.size());
  for (size_t index = 0; index < recentUiEvents.size(); index += 1) {
    const UiEventRecord &event = recentUiEvents[index];
    if (eventPlaylistSourceEnabled(event) && uiEventVisible(event)) {
      playlistEventIndexes.push_back(index);
    }
  }
  std::sort(
      playlistEventIndexes.begin(),
      playlistEventIndexes.end(),
      [&](size_t left, size_t right) {
        const UiEventRecord &leftEvent = recentUiEvents[left];
        const UiEventRecord &rightEvent = recentUiEvents[right];
        if (leftEvent.time != rightEvent.time) {
          return leftEvent.time < rightEvent.time;
        }
        if (leftEvent.label != rightEvent.label) {
          return leftEvent.label < rightEvent.label;
        }
        return left < right;
      });

  const bool allSourcesEnabled = eventPlaylistMechanicsEnabled && eventPlaylistTeamEventsEnabled &&
                                 eventPlaylistGoalContextEnabled && allEventSourcesEnabled;
  const bool noSourcesEnabled =
      (!eventPlaylistMechanicsEnabled && !eventPlaylistTeamEventsEnabled &&
       !eventPlaylistGoalContextEnabled) ||
      selectedSources.empty();
  ImGui::Text("Filters %zu / %zu", selectedSourceCount, playlistSources.size());
  if (renderModuleSummaryToggle(
          std::format("All ({})", recentUiEvents.size()).c_str(),
          allSourcesEnabled,
          "event-playlist-sources")) {
    eventPlaylistMechanicsEnabled = true;
    eventPlaylistTeamEventsEnabled = true;
    eventPlaylistGoalContextEnabled = true;
    setCvarString("subtr_actor_overlay_event_types", "all");
  }
  if (renderModuleSummaryToggle("None", noSourcesEnabled, "event-playlist-sources")) {
    eventPlaylistMechanicsEnabled = false;
    eventPlaylistTeamEventsEnabled = false;
    eventPlaylistGoalContextEnabled = false;
    setCvarString("subtr_actor_overlay_event_types", "none");
  }
  ImGui::Checkbox("Auto-follow", &eventPlaylistAutoFollow);

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "FILTERS");
  if (playlistSources.empty()) {
    ImGui::TextDisabled("No events loaded.");
  } else {
    ImGui::BeginChild("event-playlist-source-list", ImVec2{0.0f, 170.0f}, true);
    std::string_view currentGroup;
    for (const PlaylistSource &source : playlistSources) {
      const EventFilterOption &option = *source.option;
      const std::string_view optionGroup{option.group};
      if (currentGroup != optionGroup) {
        if (!currentGroup.empty()) {
          ImGui::Separator();
        }
        ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "%s", option.group);
        currentGroup = optionGroup;
      }

      ImGui::PushID(option.value);
      const std::string label = std::format("{} ({})", option.label, source.count);
      if (renderModuleSummaryToggle(label.c_str(), source.enabled, "event-playlist-sources")) {
        if (source.enabled) {
          selectedSources.erase(
              std::remove(selectedSources.begin(), selectedSources.end(), std::string{option.value}),
              selectedSources.end());
        } else {
          appendUniqueFilterToken(selectedSources, option.value);
          enableMatchingPlaylistGroups(option.value);
        }
        setCvarString(
            "subtr_actor_overlay_event_types",
            eventFilterFromSelectedSources(selectedSources));
      }
      ImGui::PopID();
    }
    ImGui::EndChild();
  }

  ImGui::Text("%zu selected / %zu recent", playlistEventIndexes.size(), recentUiEvents.size());
  if (!eventPlaylistStatus.empty()) {
    ImGui::TextWrapped("Status: %s", eventPlaylistStatus.c_str());
  }
  ImGui::Separator();

  ReplayServerWrapper replayServer = gameWrapper->GetGameEventAsReplay();
  const bool hasReplayServer = !replayServer.IsNull();
  const float currentPlaybackTime =
      hasReplayServer ? replayServer.GetReplayTimeElapsed() : playbackCurrentTime;
  auto eventSeekTime = [](const UiEventRecord &event) {
    const float leadSeconds = event.category == "goal_context" || event.type == "goal"
                                  ? GOAL_WATCH_LEAD_SECONDS
                                  : 2.0f;
    return std::max(0.0f, event.time - leadSeconds);
  };

  std::optional<size_t> activeEventIndex;
  float activeEventDistance = std::numeric_limits<float>::infinity();
  for (const size_t index : playlistEventIndexes) {
    const UiEventRecord &event = recentUiEvents[index];
    const float distance = std::abs(event.time - currentPlaybackTime);
    if (distance < activeEventDistance) {
      activeEventDistance = distance;
      activeEventIndex = index;
    }
  }

  ImGui::BeginChild("event-playlist-list", ImVec2{0.0f, 0.0f}, true);
  for (const size_t index : playlistEventIndexes) {
    const UiEventRecord &event = recentUiEvents[index];

    ImGui::PushID(static_cast<int>(index));
    const ImVec4 color = toImVec4(event.color);
    const bool active = activeEventIndex && *activeEventIndex == index;
    const float seekTime = eventSeekTime(event);
    const std::string buttonLabel =
        std::format("{} {:.2f}s##event-playlist-cue", active ? ">" : "Cue", event.time);
    if (ImGui::SmallButton(buttonLabel.c_str())) {
      mechanicsReviewClipActive = false;
      playbackCurrentTime = seekTime;
      playbackSkipPostGoalTransitions = false;
      playbackSkipKickoffs = false;
      showSingletonWindow(uiPlaybackControlsOpen, playbackControlsPlacement);
      if (hasReplayServer) {
        replayServer.SkipToTime(seekTime);
        eventPlaylistStatus =
            std::format("Cued {} at {:.2f}s", event.label, seekTime);
      } else {
        eventPlaylistStatus =
            std::format("Selected {} at {:.2f}s; open a replay to seek", event.label, seekTime);
      }
    }
    ImGui::SameLine();
    ImGui::TextColored(color, "%.2fs", event.time);
    ImGui::SameLine();
    ImGui::TextColored(color, "%s", event.actor.c_str());
    ImGui::SameLine();
    ImGui::TextWrapped("%s", event.label.c_str());
    if (!event.details.empty()) {
      ImGui::TextDisabled("%s", event.details.c_str());
    }
    ImGui::TextDisabled("%s / %s", event.category.c_str(), event.type.c_str());
    if (active && eventPlaylistAutoFollow) {
      ImGui::SetScrollHereY(0.5f);
    }
    ImGui::Separator();
    ImGui::PopID();
  }
  if (playlistEventIndexes.empty()) {
    ImGui::TextWrapped(
        noSourcesEnabled ? "No event types selected."
                         : "No events match the selected playlist filters.");
  }
  ImGui::EndChild();
  ImGui::End();
}

void SubtrActorPlugin::renderMechanicsReviewWindow() {
  if (!uiMechanicsReviewOpen) {
    return;
  }

  applySingletonWindowPlacement(mechanicsReviewPlacement);
  if (!ImGui::Begin(
          "Mechanics review##subtr-actor",
          &uiMechanicsReviewOpen,
          UI_FLOATING_WINDOW_FLAGS)) {
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
      showSingletonWindow(uiEventsOpen, eventsPlacement);
    }
    ImGui::SameLine();
    if (ImGui::Button("Open playlist")) {
      showSingletonWindow(uiEventPlaylistOpen, eventPlaylistPlacement);
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
  if (mechanicsReviewClipActive) {
    ImGui::TextColored(
        ImVec4{0.53f, 0.69f, 0.83f, 1.0f},
        "Active clip: %.2fs to %.2fs",
        mechanicsReviewClipStartSeconds,
        mechanicsReviewClipEndSeconds);
  }
  if (!mechanicsReviewStatus.empty()) {
    ImGui::TextWrapped("Status: %s", mechanicsReviewStatus.c_str());
  }
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
    mechanicsReviewClipActive = true;
    mechanicsReviewClipStartSeconds = clipStart;
    mechanicsReviewClipEndSeconds = clipEnd;
    mechanicsReviewStatus = std::format("Playing clip {:.2f}s to {:.2f}s", clipStart, clipEnd);
    showSingletonWindow(uiPlaybackControlsOpen, playbackControlsPlacement);

    ReplayServerWrapper replayServer = gameWrapper->GetGameEventAsReplay();
    if (!replayServer.IsNull()) {
      replayServer.StartPlaybackAtTime(clipStart);
    } else {
      mechanicsReviewClipActive = false;
      playbackPlaying = false;
      mechanicsReviewStatus = "Open a Rocket League replay to seek this clip";
      cvarManager->log(
          "subtr-actor: replay clip selected; open a replay to seek in Rocket League");
    }
  }
  ImGui::SameLine();
  if (mechanicsReviewClipActive && ImGui::Button("Stop clip")) {
    mechanicsReviewClipActive = false;
    playbackPlaying = false;
    mechanicsReviewStatus = "Clip stopped";
    ReplayServerWrapper replayServer = gameWrapper->GetGameEventAsReplay();
    if (!replayServer.IsNull()) {
      ReplayWrapper replay = replayServer.GetReplay();
      if (!replay.IsNull()) {
        replay.StopPlayback();
      }
    }
  }
  if (mechanicsReviewClipActive) {
    ImGui::SameLine();
  }
  if (ImGui::Button("Next") &&
      mechanicsReviewIndex < static_cast<int>(candidates.size()) - 1) {
    mechanicsReviewIndex += 1;
  }
  ImGui::SameLine();
  if (ImGui::Button("Show playlist")) {
    showSingletonWindow(uiEventPlaylistOpen, eventPlaylistPlacement);
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

  applySingletonWindowPlacement(replayLoadingPlacement);
  if (!ImGui::Begin(
          "Replay loading##subtr-actor",
          &uiReplayLoadingOpen,
          UI_FLOATING_WINDOW_FLAGS)) {
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
  std::vector<SaReplayPlayerInfo> annotationPlayers;
  if (replayAnnotations && replayAnnotationPlayerCount && writeReplayAnnotationPlayers) {
    annotationPlayers.resize(replayAnnotationPlayerCount(replayAnnotations));
    const size_t copied = writeReplayAnnotationPlayers(
        replayAnnotations,
        annotationPlayers.data(),
        annotationPlayers.size());
    annotationPlayers.resize(copied);
  }
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
  ImGui::Text("Players: %zu", annotationPlayers.size());

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "REPLAY SOURCES");
  ImGui::BeginChild("replay-loading-list", ImVec2{0.0f, 112.0f}, true);
  if (!replayPath && rawReplayPath.empty() && replayAnnotationPath.empty()) {
    ImGui::TextDisabled("No replay sources.");
  } else {
    const std::string title = replayPath ? *replayPath
                              : !rawReplayPath.empty() ? rawReplayPath
                                                       : replayAnnotationPath;
    ImGui::TextWrapped("%s", title.c_str());
    std::vector<std::string> replayMeta;
    if (!rawReplayPath.empty()) {
      replayMeta.push_back(std::format("raw: {}", rawReplayPath));
    }
    if (!replayAnnotationPath.empty() && replayAnnotationPath != title) {
      replayMeta.push_back(std::format("processed: {}", replayAnnotationPath));
    }
    if (annotationCount > 0) {
      replayMeta.push_back(std::format("{} events", annotationCount));
    }
    if (!annotationPlayers.empty()) {
      replayMeta.push_back(std::format("{} players", annotationPlayers.size()));
    }
    if (!replayMeta.empty()) {
      std::string metaText;
      for (const std::string &part : replayMeta) {
        if (!metaText.empty()) {
          metaText += " | ";
        }
        metaText += part;
      }
      ImGui::TextDisabled("%s", metaText.c_str());
    }
    ImGui::TextColored(
        replayAnnotations ? ImVec4{0.50f, 0.86f, 0.62f, 1.0f}
                          : replayAnnotationLoadFailed
                              ? ImVec4{0.95f, 0.45f, 0.45f, 1.0f}
                              : ImVec4{0.72f, 0.78f, 0.86f, 1.0f},
        "%s",
        status);
  }
  ImGui::EndChild();

  if (!annotationPlayers.empty()) {
    ImGui::Separator();
    ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "PLAYERS");
    ImGui::BeginChild("replay-loading-players", ImVec2{0.0f, 96.0f}, true);
    for (const SaReplayPlayerInfo &player : annotationPlayers) {
      const char *name = player.name == nullptr || player.name[0] == '\0' ? "--" : player.name;
      ImGui::TextColored(
          player.is_team_0 != 0 ? ImVec4{0.31f, 0.75f, 1.0f, 1.0f}
                                : ImVec4{1.0f, 0.69f, 0.31f, 1.0f},
          "%s",
          player.is_team_0 != 0 ? "Blue" : "Orange");
      ImGui::SameLine();
      ImGui::Text("#%u %s", player.player_index + 1, name);
    }
    ImGui::EndChild();
  }

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
    showSingletonWindow(uiPlaybackControlsOpen, playbackControlsPlacement);
  }
  ImGui::SameLine();
  if (ImGui::Button("Open review")) {
    showSingletonWindow(uiMechanicsReviewOpen, mechanicsReviewPlacement);
  }
  if (ImGui::Button("Open playlist")) {
    showSingletonWindow(uiEventPlaylistOpen, eventPlaylistPlacement);
  }

  ImGui::End();
}

void SubtrActorPlugin::renderModuleControlsWindow() {
  if (!uiModuleControlsOpen) {
    return;
  }

  applySingletonWindowPlacement(moduleControlsPlacement);
  if (!ImGui::Begin(
          "Module controls##subtr-actor",
          &uiModuleControlsOpen,
          UI_FLOATING_WINDOW_FLAGS)) {
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
  renderModuleSettingsControls("module-controls-settings", true);

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
    showSingletonWindow(uiGraphInspectorOpen, graphInspectorPlacement);
  }
  ImGui::SameLine();
  if (ImGui::Button("Open camera")) {
    showSingletonWindow(uiCameraOpen, cameraPlacement);
  }
  ImGui::SameLine();
  if (ImGui::Button("Open event playlist")) {
    showSingletonWindow(uiEventPlaylistOpen, eventPlaylistPlacement);
  }
  ImGui::SameLine();
  if (ImGui::Button("Open review")) {
    showSingletonWindow(uiMechanicsReviewOpen, mechanicsReviewPlacement);
  }
  ImGui::SameLine();
  if (ImGui::Button("Open recording")) {
    showSingletonWindow(uiRecordingOpen, recordingPlacement);
  }
  ImGui::SameLine();
  if (ImGui::Button("Open replay loading")) {
    showSingletonWindow(uiReplayLoadingOpen, replayLoadingPlacement);
  }
  ImGui::SameLine();
  if (ImGui::Button("Open touch controls")) {
    showSingletonWindow(uiTouchControlsOpen, touchControlsPlacement);
  }
  ImGui::SameLine();
  if (ImGui::Button("Open boost filters")) {
    showSingletonWindow(uiBoostPickupControlsOpen, boostPickupControlsPlacement);
  }

  ImGui::End();
}

void SubtrActorPlugin::renderBoostPickupControlsWindow() {
  if (!uiBoostPickupControlsOpen) {
    return;
  }

  applySingletonWindowPlacement(boostPickupControlsPlacement);
  if (!ImGui::Begin(
          "Boost pickup filters##subtr-actor",
          &uiBoostPickupControlsOpen,
          UI_FLOATING_WINDOW_FLAGS)) {
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
    showSingletonWindow(uiEventPlaylistOpen, eventPlaylistPlacement);
  }
  ImGui::SameLine();
  if (ImGui::Button("Open boost stats")) {
    createStatsModuleWindow("boost", 0);
  }
  if (ImGui::Button("Inspect boost nodes")) {
    graphInspectorView = 1;
    graphInspectorNodeQuery = "boost";
    showSingletonWindow(uiGraphInspectorOpen, graphInspectorPlacement);
  }
  ImGui::SameLine();
  if (ImGui::Button("Boost output")) {
    graphInspectorView = 0;
    selectedGraphOutput = "events";
    showSingletonWindow(uiGraphInspectorOpen, graphInspectorPlacement);
  }

  ImGui::End();
}

void SubtrActorPlugin::renderTouchControlsWindow() {
  if (!uiTouchControlsOpen) {
    return;
  }

  applySingletonWindowPlacement(touchControlsPlacement);
  if (!ImGui::Begin(
          "Touch controls##subtr-actor",
          &uiTouchControlsOpen,
          UI_FLOATING_WINDOW_FLAGS)) {
    ImGui::End();
    return;
  }
  captureWindowPlacement(touchControlsPlacement);
  if (renderSingletonWindowHeader("Touch controls", uiTouchControlsOpen)) {
    ImGui::End();
    return;
  }

  auto touchBreakdownReadout = [&]() {
    std::string readout;
    for (const auto &[enabled, label] : {
             std::pair<bool, const char *>{touchBreakdownKind, "Kind"},
             std::pair<bool, const char *>{touchBreakdownHeight, "Height"},
             std::pair<bool, const char *>{touchBreakdownSurface, "Surface"},
             std::pair<bool, const char *>{touchBreakdownDodge, "Dodge"},
         }) {
      if (!enabled) {
        continue;
      }
      if (!readout.empty()) {
        readout += " + ";
      }
      readout += label;
    }
    return readout.empty() ? std::string{"Total only"} : readout;
  };

  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "TOUCH MARKERS");
  ImGui::SameLine();
  ImGui::TextColored(
      ImVec4{0.53f, 0.69f, 0.83f, 1.0f},
      "%.1fs",
      touchMarkerDecaySeconds);
  ImGui::SliderFloat("Marker decay seconds", &touchMarkerDecaySeconds, 1.0f, 10.0f, "%.1fs");

  ImGui::TextDisabled("Touch mode");
  ImGui::SameLine();
  ImGui::TextColored(
      ImVec4{0.53f, 0.69f, 0.83f, 1.0f},
      "%s",
      touchControlsMode == 1 ? "Advancement" : "Markers");
  if (ImGui::RadioButton("Markers##touch-mode", &touchControlsMode, 0)) {
    setCvarString("subtr_actor_overlay_event_types", "touch");
  }
  ImGui::SameLine();
  if (ImGui::RadioButton("Advancement##touch-mode", &touchControlsMode, 1)) {
    setCvarString("subtr_actor_overlay_event_types", "touch_ball_movement");
  }

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "STAT BREAKDOWN");
  ImGui::SameLine();
  ImGui::TextColored(
      ImVec4{0.53f, 0.69f, 0.83f, 1.0f},
      "%s",
      touchBreakdownReadout().c_str());
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
    showSingletonWindow(uiEventPlaylistOpen, eventPlaylistPlacement);
  }
  ImGui::SameLine();
  if (ImGui::Button("Show movement")) {
    setCvarString("subtr_actor_overlay_event_types", "touch_ball_movement");
    showSingletonWindow(uiEventPlaylistOpen, eventPlaylistPlacement);
  }
  if (ImGui::Button("Open touch stats")) {
    createStatsModuleWindow("touch", 0);
  }
  ImGui::SameLine();
  if (ImGui::Button("Inspect touch nodes")) {
    graphInspectorView = 1;
    graphInspectorNodeQuery = "touch";
    showSingletonWindow(uiGraphInspectorOpen, graphInspectorPlacement);
  }

  ImGui::End();
}

void SubtrActorPlugin::renderStatusWindow() {
  if (!uiStatusOpen) {
    return;
  }
  applySingletonWindowPlacement(statusPlacement);
  if (!ImGui::Begin("Status##subtr-actor", &uiStatusOpen, UI_FLOATING_WINDOW_FLAGS)) {
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

  if (cameraViewMode == 1) {
    resolveCameraPlayerSelection();
  }
  const SaPlayerFrame *selectedPlayer = sampledPlayerByIndex(cameraSelectedPlayerIndex);
  if (cameraViewMode == 1 && selectedPlayer == nullptr && cameraSelectedPlayerId.empty() &&
      !sampledPlayers.empty()) {
    cameraSelectedPlayerIndex = sampledPlayers.front().player_index;
    cameraSelectedPlayerId = webPlayerIdForIndex(cameraSelectedPlayerIndex);
    selectedPlayer = sampledPlayerByIndex(cameraSelectedPlayerIndex);
  }
  const SaPlayerFrame *targetPlayer = cameraViewMode == 1 ? selectedPlayer : nullptr;

  applySingletonWindowPlacement(cameraPlacement);
  if (!ImGui::Begin("Camera##subtr-actor", &uiCameraOpen, UI_FLOATING_WINDOW_FLAGS)) {
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
        cameraSelectedPlayerId = webPlayerIdForIndex(cameraSelectedPlayerIndex);
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

  const bool hasAttachedCamera = targetPlayer != nullptr;
  auto pushCameraDisabledStyle = [](bool disabled) {
    if (disabled) {
      ImGui::PushStyleVar(ImGuiStyleVar_Alpha, ImGui::GetStyle().Alpha * 0.45f);
    }
  };
  auto popCameraDisabledStyle = [](bool disabled) {
    if (disabled) {
      ImGui::PopStyleVar();
    }
  };

  float nextDistanceScale = cameraDistanceScale;
  pushCameraDisabledStyle(!hasAttachedCamera);
  const bool distanceScaleChanged =
      ImGui::SliderFloat("Distance scale", &nextDistanceScale, 0.75f, 4.0f, "%.2fx");
  popCameraDisabledStyle(!hasAttachedCamera);
  if (hasAttachedCamera && distanceScaleChanged) {
    cameraDistanceScale = nextDistanceScale;
  }

  bool nextBallCamEnabled = cameraBallCamEnabled;
  pushCameraDisabledStyle(!hasAttachedCamera);
  const bool ballCamChanged = ImGui::Checkbox("Ball cam", &nextBallCamEnabled);
  popCameraDisabledStyle(!hasAttachedCamera);
  if (hasAttachedCamera && ballCamChanged) {
    cameraBallCamEnabled = nextBallCamEnabled;
  }

  bool nextCustomSettingsEnabled = cameraCustomSettingsEnabled;
  pushCameraDisabledStyle(!hasAttachedCamera);
  const bool customSettingsChanged =
      ImGui::Checkbox("Custom settings", &nextCustomSettingsEnabled);
  popCameraDisabledStyle(!hasAttachedCamera);
  if (hasAttachedCamera && customSettingsChanged) {
    cameraCustomSettingsEnabled = nextCustomSettingsEnabled;
  }
  if (cameraCustomSettingsEnabled) {
    const bool customControlsDisabled = !hasAttachedCamera;
    auto renderCustomSlider = [&](const char *label, float &value, float min, float max,
                                  const char *format) {
      float next = value;
      pushCameraDisabledStyle(customControlsDisabled);
      const bool changed = ImGui::SliderFloat(label, &next, min, max, format);
      popCameraDisabledStyle(customControlsDisabled);
      if (!customControlsDisabled && changed) {
        value = next;
      }
    };
    renderCustomSlider("FOV", cameraCustomFov, 60.0f, 130.0f, "%.0f");
    renderCustomSlider("Height", cameraCustomHeight, 40.0f, 250.0f, "%.0f");
    renderCustomSlider("Pitch", cameraCustomPitch, -30.0f, 30.0f, "%.1f");
    renderCustomSlider("Distance", cameraCustomDistance, 100.0f, 500.0f, "%.0f");
    renderCustomSlider("Stiffness", cameraCustomStiffness, 0.0f, 1.0f, "%.2f");
    renderCustomSlider("Swivel speed", cameraCustomSwivelSpeed, 1.0f, 10.0f, "%.1f");
    renderCustomSlider(
        "Transition speed",
        cameraCustomTransitionSpeed,
        0.5f,
        2.0f,
        "%.2f");
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
    showSingletonWindow(uiPlaybackControlsOpen, playbackControlsPlacement);
  }
  ImGui::SameLine();
  if (ImGui::Button("Open recording")) {
    showSingletonWindow(uiRecordingOpen, recordingPlacement);
  }
  if (targetPlayer != nullptr && ImGui::Button("Open player stats")) {
    createStatsWindow(UiStatsWindowKind::Player, true);
    if (!uiStatsWindows.empty()) {
      UiStatsWindow &window = uiStatsWindows.back();
      window.selected_player_index = targetPlayer->player_index;
      window.selected_player_id = webPlayerIdForIndex(window.selected_player_index);
      window.selected_team_is_team_0 = targetPlayer->is_team_0;
    }
  }

  ImGui::End();
}

void SubtrActorPlugin::renderPlaybackControlsWindow() {
  if (!uiPlaybackControlsOpen) {
    return;
  }

  applySingletonWindowPlacement(playbackControlsPlacement);
  if (!ImGui::Begin(
          "Playback##subtr-actor",
          &uiPlaybackControlsOpen,
          UI_FLOATING_WINDOW_FLAGS)) {
    ImGui::End();
    return;
  }
  captureWindowPlacement(playbackControlsPlacement);
  if (renderSingletonWindowHeader("Playback", uiPlaybackControlsOpen)) {
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
  ImGui::Text("Status: %s", playbackStatus.c_str());

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "WEB PLAYBACK CONFIG");
  const bool transportEnabled = hasReplayServer;
  auto pushPlaybackDisabledStyle = [](bool disabled) {
    if (disabled) {
      ImGui::PushStyleVar(ImGuiStyleVar_Alpha, ImGui::GetStyle().Alpha * 0.45f);
    }
  };
  auto popPlaybackDisabledStyle = [](bool disabled) {
    if (disabled) {
      ImGui::PopStyleVar();
    }
  };
  auto playbackButton = [&](const char *label, bool disabled) {
    pushPlaybackDisabledStyle(disabled);
    const bool clicked = ImGui::Button(label);
    popPlaybackDisabledStyle(disabled);
    return clicked && !disabled;
  };
  auto applyPlaybackState = [&](bool shouldPlay) {
    mechanicsReviewClipActive = false;
    playbackCurrentTime = std::max(0.0f, playbackCurrentTime);
    playbackPlaying = shouldPlay;
    if (!hasReplayServer) {
      playbackStatus = "Open a Rocket League replay to control playback";
      return;
    }

    if (shouldPlay) {
      replayServer.StartPlaybackAtTime(playbackCurrentTime);
      playbackStatus = std::format("Playing from {:.2f}s", playbackCurrentTime);
      return;
    }

    replayServer.SkipToTime(playbackCurrentTime);
    ReplayWrapper replay = replayServer.GetReplay();
    if (!replay.IsNull()) {
      replay.StopPlayback();
    }
    playbackStatus = std::format("Paused at {:.2f}s", playbackCurrentTime);
  };

  ImGui::SetNextItemWidth(140.0f);
  float nextPlaybackCurrentTime = playbackCurrentTime;
  pushPlaybackDisabledStyle(!transportEnabled);
  const bool currentTimeChanged =
      ImGui::InputFloat("Current time", &nextPlaybackCurrentTime, 0.25f, 2.0f, "%.2f");
  popPlaybackDisabledStyle(!transportEnabled);
  if (transportEnabled && currentTimeChanged) {
    playbackCurrentTime = nextPlaybackCurrentTime;
  }
  playbackCurrentTime = std::max(0.0f, playbackCurrentTime);
  ImGui::SetNextItemWidth(140.0f);
  float nextPlaybackRate = playbackRate;
  pushPlaybackDisabledStyle(!transportEnabled);
  const bool rateChanged =
      ImGui::SliderFloat("Rate", &nextPlaybackRate, 0.25f, 2.0f, "%.2fx");
  popPlaybackDisabledStyle(!transportEnabled);
  if (transportEnabled && rateChanged) {
    playbackRate = nextPlaybackRate;
  }
  if (playbackButton(playbackPlaying ? "Pause" : "Play", !transportEnabled)) {
    applyPlaybackState(!playbackPlaying);
  }
  ImGui::SameLine();
  if (playbackButton("Seek", !transportEnabled)) {
    applyPlaybackState(false);
  }
  ImGui::SameLine();
  if (playbackButton("Live replay time", !hasReplayServer)) {
    playbackCurrentTime = replayServer.GetReplayTimeElapsed();
    playbackStatus = std::format("Captured {:.2f}s", playbackCurrentTime);
  }
  bool nextPlaying = playbackPlaying;
  pushPlaybackDisabledStyle(!transportEnabled);
  const bool playingChanged = ImGui::Checkbox("Playing", &nextPlaying);
  popPlaybackDisabledStyle(!transportEnabled);
  if (transportEnabled && playingChanged) {
    applyPlaybackState(nextPlaying);
  }
  ImGui::SameLine();
  bool nextSkipPostGoalTransitions = playbackSkipPostGoalTransitions;
  pushPlaybackDisabledStyle(!transportEnabled);
  const bool skipGoalChanged =
      ImGui::Checkbox("Skip goal transitions", &nextSkipPostGoalTransitions);
  popPlaybackDisabledStyle(!transportEnabled);
  if (transportEnabled && skipGoalChanged) {
    playbackSkipPostGoalTransitions = nextSkipPostGoalTransitions;
  }
  bool nextSkipKickoffs = playbackSkipKickoffs;
  pushPlaybackDisabledStyle(!transportEnabled);
  const bool skipKickoffsChanged = ImGui::Checkbox("Skip kickoffs", &nextSkipKickoffs);
  popPlaybackDisabledStyle(!transportEnabled);
  if (transportEnabled && skipKickoffsChanged) {
    playbackSkipKickoffs = nextSkipKickoffs;
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
    showSingletonWindow(uiStatusOpen, statusPlacement);
  }
  ImGui::SameLine();
  if (ImGui::Button("Open playlist")) {
    showSingletonWindow(uiEventPlaylistOpen, eventPlaylistPlacement);
  }
  if (ImGui::Button("Open modules")) {
    showSingletonWindow(uiModuleControlsOpen, moduleControlsPlacement);
  }

  ImGui::End();
}

void SubtrActorPlugin::renderRecordingWindow() {
  if (!uiRecordingOpen) {
    return;
  }

  applySingletonWindowPlacement(recordingPlacement);
  if (!ImGui::Begin("Recording##subtr-actor", &uiRecordingOpen, UI_FLOATING_WINDOW_FLAGS)) {
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
  const bool hasGraphSnapshot = recordingSnapshotCount > 0 || recordingLastBytes > 0;
  const bool recordingSettingsLocked = recordingActive;
  auto pushRecordingDisabledStyle = [](bool disabled) {
    if (disabled) {
      ImGui::PushStyleVar(ImGuiStyleVar_Alpha, ImGui::GetStyle().Alpha * 0.45f);
    }
  };
  auto popRecordingDisabledStyle = [](bool disabled) {
    if (disabled) {
      ImGui::PopStyleVar();
    }
  };
  auto recordingButton = [](const char *label, bool disabled) {
    if (disabled) {
      ImGui::PushStyleVar(ImGuiStyleVar_Alpha, ImGui::GetStyle().Alpha * 0.45f);
    }
    const bool clicked = ImGui::Button(label);
    if (disabled) {
      ImGui::PopStyleVar();
    }
    return clicked && !disabled;
  };

  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "RECORDING");
  int nextRecordingFps = recordingFps;
  pushRecordingDisabledStyle(recordingSettingsLocked);
  const bool fpsChanged = ImGui::SliderInt("FPS", &nextRecordingFps, 1, 120);
  popRecordingDisabledStyle(recordingSettingsLocked);
  if (!recordingSettingsLocked && fpsChanged) {
    recordingFps = nextRecordingFps;
  }
  const std::array<const char *, 4> rates{{"0.5x", "1.0x", "1.5x", "2.0x"}};
  recordingPlaybackRateIndex = std::clamp(recordingPlaybackRateIndex, 0, 3);
  int nextRecordingPlaybackRateIndex = recordingPlaybackRateIndex;
  pushRecordingDisabledStyle(recordingSettingsLocked);
  if (ImGui::BeginCombo("Playback rate", rates[static_cast<size_t>(recordingPlaybackRateIndex)])) {
    for (int index = 0; index < static_cast<int>(rates.size()); index += 1) {
      const bool selected = index == recordingPlaybackRateIndex;
      if (ImGui::Selectable(rates[static_cast<size_t>(index)], selected)) {
        nextRecordingPlaybackRateIndex = index;
      }
    }
    ImGui::EndCombo();
  }
  popRecordingDisabledStyle(recordingSettingsLocked);
  if (!recordingSettingsLocked) {
    recordingPlaybackRateIndex = nextRecordingPlaybackRateIndex;
  }
  bool nextFinishBeforeDump = recordingFinishBeforeDump;
  pushRecordingDisabledStyle(recordingSettingsLocked);
  const bool finishBeforeDumpChanged =
      ImGui::Checkbox("Finalize before dump", &nextFinishBeforeDump);
  popRecordingDisabledStyle(recordingSettingsLocked);
  if (!recordingSettingsLocked && finishBeforeDumpChanged) {
    recordingFinishBeforeDump = nextFinishBeforeDump;
  }

  ImGui::Separator();
  if (recordingButton("Start", recordingActive)) {
    recordingActive = true;
    recordingStartedAt = std::chrono::steady_clock::now();
    recordingStatus = "Recording analysis snapshots";
  }
  ImGui::SameLine();
  if (recordingButton("Full replay", recordingActive || !loaded || !engine)) {
    recordingActive = false;
    dumpSnapshot(true);
  }
  ImGui::SameLine();
  if (recordingButton("Stop", !recordingActive)) {
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
  if (recordingButton("Clear", recordingActive || !hasGraphSnapshot)) {
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
    showSingletonWindow(uiGraphInspectorOpen, graphInspectorPlacement);
  }
  ImGui::SameLine();
  if (ImGui::Button("Open replay loading")) {
    showSingletonWindow(uiReplayLoadingOpen, replayLoadingPlacement);
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

  applySingletonWindowPlacement(graphInspectorPlacement);
  if (!ImGui::Begin(
          "Graph inspector##subtr-actor",
          &uiGraphInspectorOpen,
          UI_FLOATING_WINDOW_FLAGS)) {
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

std::array<SubtrActorPlugin::SingletonWindowControl, 13>
SubtrActorPlugin::singletonWindowControls() {
  const float eventPlaylistX = rightAnchoredUiX(430.0f);
  const float statusX = rightAnchoredUiX(330.0f);
  const float playbackX = rightAnchoredUiX(336.0f, 32.0f);
  const float recordingX = rightAnchoredUiX(416.0f, 32.0f);
  const float mechanicsReviewX = rightAnchoredUiX(500.0f);
  const float replayLoadingX = rightAnchoredUiX(520.0f);
  const float moduleControlsX = rightAnchoredUiX(430.0f);
  const float touchControlsX = rightAnchoredUiX(410.0f);

  return {{
      {"Scoreboard",
       "scoreboard",
       "scoreboard_open",
       "scoreboard",
       true,
       1,
       &uiScoreboardOpen,
       &scoreboardPlacement,
       0.0f,
       11.0f,
       88.0f,
       34.0f},
      {"Events",
       "mechanics",
       "events_open",
       "events",
       true,
       4,
       &uiEventsOpen,
       &eventsPlacement,
       16.0f,
       256.0f,
       520.0f,
       360.0f},
      {"Event playlist",
       "event-playlist",
       "event_playlist_open",
       "event_playlist",
       true,
       5,
       &uiEventPlaylistOpen,
       &eventPlaylistPlacement,
       eventPlaylistX,
       256.0f,
       430.0f,
       430.0f},
      {"Status",
       "status",
       "status_open",
       "status",
       false,
       100,
       &uiStatusOpen,
       &statusPlacement,
       statusX,
       68.0f,
       330.0f,
       220.0f},
      {"Camera",
       "camera",
       "camera_open",
       "camera",
       true,
       0,
       &uiCameraOpen,
       &cameraPlacement,
       16.0f,
       68.0f,
       416.0f,
       500.0f},
      {"Playback",
       "playback",
       "playback_controls_open",
       "playback_controls",
       true,
       2,
       &uiPlaybackControlsOpen,
       &playbackControlsPlacement,
       playbackX,
       68.0f,
       336.0f,
       430.0f},
      {"Recording",
       "recording",
       "recording_open",
       "recording",
       true,
       3,
       &uiRecordingOpen,
       &recordingPlacement,
       recordingX,
       384.0f,
       416.0f,
       380.0f},
      {"Graph inspector",
       "graph-inspector",
       "graph_inspector_open",
       "graph_inspector",
       false,
       101,
       &uiGraphInspectorOpen,
       &graphInspectorPlacement,
       360.0f,
       68.0f,
       700.0f,
       520.0f},
      {"Mechanics review",
       "mechanics-review",
       "mechanics_review_open",
       "mechanics_review",
       true,
       6,
       &uiMechanicsReviewOpen,
       &mechanicsReviewPlacement,
       mechanicsReviewX,
       256.0f,
       500.0f,
       560.0f},
      {"Replay loading",
       "replay-loading",
       "replay_loading_open",
       "replay_loading",
       true,
       7,
       &uiReplayLoadingOpen,
       &replayLoadingPlacement,
       replayLoadingX,
       68.0f,
       520.0f,
       360.0f},
      {"Module controls",
       "module-controls",
       "module_controls_open",
       "module_controls",
       false,
       102,
       &uiModuleControlsOpen,
       &moduleControlsPlacement,
       moduleControlsX,
       305.0f,
       430.0f,
       520.0f},
      {"Touch controls",
       "touch-controls",
       "touch_controls_open",
       "touch_controls",
       true,
       9,
       &uiTouchControlsOpen,
       &touchControlsPlacement,
       touchControlsX,
       256.0f,
       410.0f,
       380.0f},
      {"Boost pickup filters",
       "boost-pickups",
       "boost_pickup_controls_open",
       "boost_pickup_controls",
       true,
       8,
       &uiBoostPickupControlsOpen,
       &boostPickupControlsPlacement,
       16.0f,
       448.0f,
       544.0f,
       420.0f},
  }};
}

std::vector<SubtrActorPlugin::SingletonWindowControl>
SubtrActorPlugin::webSingletonWindowControls() {
  std::vector<SingletonWindowControl> windows;
  for (const SingletonWindowControl &window : singletonWindowControls()) {
    if (window.web_config) {
      windows.push_back(window);
    }
  }
  std::sort(
      windows.begin(),
      windows.end(),
      [](const SingletonWindowControl &left, const SingletonWindowControl &right) {
        return left.launcher_order == right.launcher_order
                   ? std::string_view{left.config_id} < std::string_view{right.config_id}
                   : left.launcher_order < right.launcher_order;
      });
  return windows;
}

void SubtrActorPlugin::renderSingletonWindowManager() {
  std::array<SingletonWindowControl, 13> windows = singletonWindowControls();

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
      showSingletonWindow(*window.open, *window.placement);
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
        focusSingletonWindow(*window.placement);
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
        focusSingletonWindow(*window.placement);
      }
    } else {
      if (ImGui::SmallButton("Show")) {
        showSingletonWindow(*window.open, *window.placement);
      }
      ImGui::SameLine();
      ImGui::TextDisabled("Hidden");
    }
    ImGui::SameLine();
    if (ImGui::SmallButton("Reset")) {
      *window.open = true;
      if (window.placement == &scoreboardPlacement) {
        resetScoreboardWindowPlacement(true);
      } else {
        resetSingletonWindowPlacement(
            *window.placement,
            window.x,
            window.y,
            window.width,
            window.height,
            true);
      }
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
      showStatsWindow(window);
    }
  }
  ImGui::SameLine();
  if (ImGui::SmallButton("Hide all##stats-windows")) {
    for (UiStatsWindow &window : uiStatsWindows) {
      window.open = false;
    }
  }
  ImGui::SameLine();
  if (ImGui::SmallButton("Focus visible##stats-windows")) {
    for (UiStatsWindow &window : uiStatsWindows) {
      if (window.open) {
        focusStatsWindow(window);
      }
    }
  }
  ImGui::SameLine();
  if (ImGui::SmallButton("Reset positions##stats-windows")) {
    for (size_t index = 0; index < uiStatsWindows.size(); index += 1) {
      resetStatsWindowPlacement(uiStatsWindows[index], index);
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
  std::optional<UiStatsWindow> duplicateWindow;
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
        focusStatsWindow(window);
      }
    } else {
      if (ImGui::SmallButton("Show")) {
        showStatsWindow(window);
      }
      ImGui::SameLine();
      ImGui::TextDisabled("Hidden");
    }

    ImGui::SameLine();
    if (ImGui::SmallButton("Remove")) {
      removeIndex = index;
    }
    ImGui::SameLine();
    if (ImGui::SmallButton("Duplicate")) {
      UiStatsWindow copy = window;
      copy.id = nextUiStatsWindowId++;
      copy.config_id = std::format("stats-{}", copy.id);
      copy.open = true;
      copy.pending_focus = true;
      copy.picker_open = false;
      copy.pending_apply_placement = true;
      copy.x += 24.0f;
      copy.y += 24.0f;
      copy.z_index = nextUiWindowZIndex++;
      duplicateWindow = std::move(copy);
    }
    ImGui::SameLine();
    if (ImGui::SmallButton("Reset")) {
      resetStatsWindowPlacement(window, index);
    }
    ImGui::SameLine();
    ImGui::TextWrapped("%s", label.c_str());
    ImGui::PopID();
  }
  if (removeIndex) {
    uiStatsWindows.erase(uiStatsWindows.begin() + static_cast<std::ptrdiff_t>(*removeIndex));
  }
  if (duplicateWindow) {
    uiStatsWindows.push_back(std::move(*duplicateWindow));
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

  for (const SingletonWindowControl &window : singletonWindowControls()) {
    considerPlacement(*window.open, *window.placement);
  }

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
  resetSingletonWindowPlacement(launcherPlacement, 16.0f, 68.0f, 340.0f, 430.0f, true);
  for (const SingletonWindowControl &window : singletonWindowControls()) {
    if (window.placement == &scoreboardPlacement) {
      resetScoreboardWindowPlacement();
      continue;
    }
    resetSingletonWindowPlacement(
        *window.placement,
        window.x,
        window.y,
        window.width,
        window.height);
  }
  for (size_t index = 0; index < uiStatsWindows.size(); index += 1) {
    resetStatsWindowPlacement(uiStatsWindows[index], index);
  }
}

void SubtrActorPlugin::resetDefaultStatsWindows() {
  uiStatsWindows.clear();
  nextUiStatsWindowId = 1;
  for (const StatsWindowKindControl &window : statsWindowKindControls()) {
    if (window.default_window) {
      createStatsWindow(window.kind, true);
    }
  }
}

void SubtrActorPlugin::applyWorkspaceWindowVisibility(
    bool launcherOpen,
    std::initializer_list<std::string_view> openWindowIds) {
  uiLauncherOpen = launcherOpen;
  for (const SingletonWindowControl &window : singletonWindowControls()) {
    *window.open = std::find(openWindowIds.begin(), openWindowIds.end(), window.config_id) !=
                   openWindowIds.end();
  }
}

void SubtrActorPlugin::applyDefaultUiWorkspace() {
  applyWorkspaceWindowVisibility(false, {"scoreboard", "camera"});
  resetWindowPlacements();
  cameraPlacement.pending_focus = true;
}

void SubtrActorPlugin::applyReplayReviewUiWorkspace() {
  applyWorkspaceWindowVisibility(
      true,
      {"scoreboard",
       "mechanics",
       "event-playlist",
       "camera",
       "playback",
       "mechanics-review",
       "replay-loading",
       "touch-controls",
       "boost-pickups"});
  eventPlaylistMechanicsEnabled = true;
  eventPlaylistTeamEventsEnabled = true;
  eventPlaylistGoalContextEnabled = true;
  eventPlaylistAutoFollow = true;
  resetWindowPlacements();
  mechanicsReviewPlacement.pending_focus = true;
  replayLoadingPlacement.pending_focus = true;
}

void SubtrActorPlugin::applyGraphDebugUiWorkspace() {
  applyWorkspaceWindowVisibility(
      true,
      {"mechanics",
       "event-playlist",
       "status",
       "playback",
       "graph-inspector",
       "module-controls"});
  resetWindowPlacements();
  graphInspectorPlacement.pending_focus = true;
  moduleControlsPlacement.pending_focus = true;
}

void SubtrActorPlugin::applyRecordingUiWorkspace() {
  applyWorkspaceWindowVisibility(
      true,
      {"scoreboard", "status", "camera", "playback", "recording"});
  resetWindowPlacements();
  recordingPlacement.pending_focus = true;
  cameraPlacement.pending_focus = true;
}

void SubtrActorPlugin::createStatsWindow(UiStatsWindowKind kind, bool initializeEntries) {
  UiStatsWindow window{};
  window.id = nextUiStatsWindowId++;
  window.config_id = std::format("stats-{}", window.id);
  window.kind = kind;
  initializeStatsWindowPlacement(window);
  focusStatsWindow(window);
  if (!sampledPlayers.empty()) {
    window.selected_player_index = sampledPlayers.front().player_index;
    window.selected_player_id = webPlayerIdForIndex(window.selected_player_index);
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
  window.config_id = std::format("stats-{}", window.id);
  window.kind = UiStatsWindowKind::StatsModule;
  window.module_name = std::move(moduleName);
  window.module_view = std::clamp(moduleView, 0, 2);
  initializeStatsWindowPlacement(window);
  focusStatsWindow(window);
  uiStatsWindows.push_back(std::move(window));
}

std::pair<float, float> SubtrActorPlugin::defaultStatsWindowSize(UiStatsWindowKind kind) const {
  if (kind == UiStatsWindowKind::StatsModule) {
    return {680.0f, 460.0f};
  }
  return {540.0f, 330.0f};
}

void SubtrActorPlugin::initializeStatsWindowPlacement(UiStatsWindow &window) {
  resetStatsWindowPlacement(window, uiStatsWindows.size());
}

void SubtrActorPlugin::resetStatsWindowPlacement(UiStatsWindow &window, size_t stackIndex) {
  const float offset = static_cast<float>(stackIndex * 18);
  const auto [width, height] = defaultStatsWindowSize(window.kind);
  window.width = width;
  window.height = height;
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
  window.pending_focus = window.open;
  window.z_index = nextUiWindowZIndex++;
}

void SubtrActorPlugin::renderStatsWindows() {
  for (UiStatsWindow &window : uiStatsWindows) {
    if (window.open) {
      renderStatsWindow(window);
    }
  }
}

std::array<SubtrActorPlugin::StatsWindowKindControl, 7>
SubtrActorPlugin::statsWindowKindControls() const {
  return {{
      {UiStatsWindowKind::Player,
       "player",
       "Player stats",
       "New player stats",
       UI_STAT_SCOPE_PLAYER,
       true,
       true,
       true,
       true},
      {UiStatsWindowKind::Team,
       "team",
       "Team stats",
       "New team stats",
       UI_STAT_SCOPE_TEAM,
       true,
       true,
       true,
       true},
      {UiStatsWindowKind::AllPlayers,
       "all-players",
       "All players stats",
       "New all players stats",
       UI_STAT_SCOPE_PLAYER,
       false,
       true,
       true,
       false},
      {UiStatsWindowKind::AllTeams,
       "all-teams",
       "All teams stats",
       "New all teams stats",
       UI_STAT_SCOPE_TEAM,
       false,
       true,
       true,
       false},
      {UiStatsWindowKind::GoalsOverview,
       "goals-overview",
       "Goal labels",
       "New goal labels",
       UI_STAT_SCOPE_EVENT,
       false,
       false,
       true,
       true},
      {UiStatsWindowKind::AdHoc,
       "ad-hoc",
       "Ad hoc stats",
       "New ad hoc stats",
       static_cast<uint8_t>(UI_STAT_SCOPE_PLAYER | UI_STAT_SCOPE_TEAM | UI_STAT_SCOPE_EVENT),
       false,
       true,
       true,
       false},
      {UiStatsWindowKind::StatsModule,
       "stats-module",
       "Stats module",
       "New stats module",
       0,
       false,
       false,
       false,
       false},
  }};
}

std::optional<SubtrActorPlugin::UiStatsWindowKind>
SubtrActorPlugin::parseStatsWindowKind(std::string_view value) const {
  for (const StatsWindowKindControl &control : statsWindowKindControls()) {
    if (value == control.config_id) {
      return control.kind;
    }
  }
  return std::nullopt;
}

const char *SubtrActorPlugin::statsWindowKindConfigId(UiStatsWindowKind kind) const {
  for (const StatsWindowKindControl &control : statsWindowKindControls()) {
    if (control.kind == kind) {
      return control.config_id;
    }
  }
  return "player";
}

const char *SubtrActorPlugin::statsWindowKindLabel(UiStatsWindowKind kind) const {
  for (const StatsWindowKindControl &control : statsWindowKindControls()) {
    if (control.kind == kind) {
      return control.label;
    }
  }
  return "Stats";
}

uint8_t SubtrActorPlugin::statsWindowKindStatScopes(UiStatsWindowKind kind) const {
  for (const StatsWindowKindControl &control : statsWindowKindControls()) {
    if (control.kind == kind) {
      return control.stat_scopes;
    }
  }
  return 0;
}

bool SubtrActorPlugin::statsWindowKindHasScopeSelector(UiStatsWindowKind kind) const {
  for (const StatsWindowKindControl &control : statsWindowKindControls()) {
    if (control.kind == kind) {
      return control.scope_selector;
    }
  }
  return false;
}

bool SubtrActorPlugin::statsWindowKindHasStatPicker(UiStatsWindowKind kind) const {
  for (const StatsWindowKindControl &control : statsWindowKindControls()) {
    if (control.kind == kind) {
      return control.stat_picker;
    }
  }
  return false;
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
        {"shots", ""}, {"boost", ""}, {"player:touch.touch_count", ""},
        {"player:boost.amount_used", ""}, {"player:speed_flip.count", ""},
        {"player:half_flip.count", ""}, {"player:wavedash.count", ""},
        {"recent_events", ""}};
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
        {"team:possession.possession_time", ""},
        {"team:pressure.offensive_half_time", ""},
        {"team:boost.amount_used", ""},
        {"recent_events", ""}};
    break;
  case UiStatsWindowKind::AllPlayers:
    window.entries = {
        {"score", ""}, {"goals", ""}, {"assists", ""}, {"saves", ""},
        {"shots", ""}, {"boost", ""}, {"player:touch.touch_count", ""},
        {"player:boost.amount_used", ""}, {"recent_events", ""}};
    break;
  case UiStatsWindowKind::AllTeams:
    window.entries = {
        {"players", ""},
        {"score", ""},
        {"goals", ""},
        {"shots", ""},
        {"average_boost", ""},
        {"team:possession.possession_time", ""},
        {"team:boost.amount_used", ""},
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
  uint8_t definitionScopes = 0;
  const auto graphStat = parseGraphStatId(localStatId);
  const bool playerScoped =
      definition ? definition->player : (graphStat && graphStat->scope == "player");
  const bool teamScoped =
      definition ? definition->team : (graphStat && graphStat->scope == "team");
  const bool eventScoped = definition ? definition->event : false;
  if (!definition && !graphStat) {
    return false;
  }
  if (playerScoped) {
    definitionScopes |= UI_STAT_SCOPE_PLAYER;
  }
  if (teamScoped) {
    definitionScopes |= UI_STAT_SCOPE_TEAM;
  }
  if (eventScoped) {
    definitionScopes |= UI_STAT_SCOPE_EVENT;
  }
  return (statsWindowKindStatScopes(window.kind) & definitionScopes) != 0;
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
                      (targetId.empty() ||
                       statsWindowTargetsEqual(localStatId, entry.target_id, targetId));
             }) != window.entries.end();
}

bool SubtrActorPlugin::statsWindowTargetsEqual(
    std::string_view statId,
    std::string_view lhsTargetId,
    std::string_view rhsTargetId) const {
  if (lhsTargetId == rhsTargetId) {
    return true;
  }
  const std::string localStatId = normalizeUiStatId(statId);
  const UiStatDefinition *definition = localUiStatDefinition(localStatId);
  const auto graphStat = parseGraphStatId(localStatId);
  const bool playerScoped =
      definition ? definition->player : (graphStat && graphStat->scope == "player");
  if (!playerScoped) {
    return false;
  }
  const std::optional<uint32_t> lhsPlayerIndex = playerIndexForTargetId(lhsTargetId);
  const std::optional<uint32_t> rhsPlayerIndex = playerIndexForTargetId(rhsTargetId);
  return lhsPlayerIndex && rhsPlayerIndex && *lhsPlayerIndex == *rhsPlayerIndex;
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

const std::string &SubtrActorPlugin::currentStatsJson() const {
  if (!loaded || !engine || !statsJsonLen || !writeStatsJson) {
    cachedStatsJson.clear();
    cachedStatsJsonFrameNumber = std::numeric_limits<uint64_t>::max();
    return cachedStatsJson;
  }

  if (cachedStatsJsonFrameNumber != frameNumber) {
    cachedStatsJson = readJsonBuffer(statsJsonLen, writeStatsJson);
    cachedStatsJsonFrameNumber = frameNumber;
  }
  return cachedStatsJson;
}

std::optional<std::string> SubtrActorPlugin::graphPlayerStatValue(
    const SaPlayerFrame &player,
    std::string_view statId) const {
  const auto parsed = parseGraphStatId(statId);
  if (!parsed || parsed->scope != "player") {
    return std::nullopt;
  }

  const std::string &statsJson = currentStatsJson();
  if (statsJson.empty()) {
    return std::nullopt;
  }
  const auto frame = parseJsonObjectProperty(statsJson, "frame");
  if (!frame) {
    return std::nullopt;
  }
  const auto modules = parseJsonObjectProperty(*frame, "modules");
  if (!modules) {
    return std::nullopt;
  }
  const auto module = parseJsonObjectProperty(*modules, std::string{parsed->module});
  if (!module) {
    return std::nullopt;
  }

  const std::vector<std::string> playerStats =
      parseJsonObjectArrayProperty(*module, "player_stats");
  for (const std::string &entry : playerStats) {
    const auto playerId = parseJsonObjectProperty(entry, "player_id");
    if (!playerId || !jsonPlayerIdMatchesIndex(*playerId, player.player_index)) {
      continue;
    }
    const auto stats = parseJsonObjectProperty(entry, "stats");
    if (!stats) {
      return std::nullopt;
    }
    return jsonDisplayValueAtPath(*stats, parsed->path);
  }
  return std::nullopt;
}

std::optional<std::string> SubtrActorPlugin::graphTeamStatValue(
    uint8_t isTeam0,
    std::string_view statId) const {
  const auto parsed = parseGraphStatId(statId);
  if (!parsed || parsed->scope != "team") {
    return std::nullopt;
  }

  const std::string &statsJson = currentStatsJson();
  if (statsJson.empty()) {
    return std::nullopt;
  }
  const auto frame = parseJsonObjectProperty(statsJson, "frame");
  if (!frame) {
    return std::nullopt;
  }
  const auto modules = parseJsonObjectProperty(*frame, "modules");
  if (!modules) {
    return std::nullopt;
  }
  const auto module = parseJsonObjectProperty(*modules, std::string{parsed->module});
  if (!module) {
    return std::nullopt;
  }
  const auto team =
      parseJsonObjectProperty(*module, isTeam0 != 0 ? "team_zero" : "team_one");
  if (!team) {
    return std::nullopt;
  }
  return jsonDisplayValueAtPath(*team, parsed->path);
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
  if (const auto graphValue = graphPlayerStatValue(player, localStatId)) {
    return *graphValue;
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
  if (const auto graphValue = graphTeamStatValue(isTeam0, localStatId)) {
    return *graphValue;
  }
  return "--";
}

std::string SubtrActorPlugin::defaultAdHocTargetId(std::string_view statId) const {
  const std::string localStatId = normalizeUiStatId(statId);
  const UiStatDefinition *definition = localUiStatDefinition(localStatId);
  const auto graphStat = parseGraphStatId(localStatId);
  const bool playerScoped =
      definition ? definition->player : (graphStat && graphStat->scope == "player");
  const bool teamScoped =
      definition ? definition->team : (graphStat && graphStat->scope == "team");
  if (playerScoped) {
    return sampledPlayers.empty() ? "" : webPlayerIdForIndex(sampledPlayers.front().player_index);
  }
  if (teamScoped) {
    return "blue";
  }
  return "";
}

std::string SubtrActorPlugin::adHocStatValue(
    std::string_view statId,
    std::string_view targetId) const {
  const std::string localStatId = normalizeUiStatId(statId);
  const UiStatDefinition *definition = localUiStatDefinition(localStatId);
  const auto graphStat = parseGraphStatId(localStatId);
  const bool playerScoped =
      definition ? definition->player : (graphStat && graphStat->scope == "player");
  const bool teamScoped =
      definition ? definition->team : (graphStat && graphStat->scope == "team");
  if (!definition && !graphStat) {
    return "--";
  }
  if (playerScoped) {
    uint32_t playerIndex = sampledPlayers.empty() ? 0 : sampledPlayers.front().player_index;
    if (const std::optional<uint32_t> resolvedPlayerIndex =
            playerIndexForTargetId(targetId)) {
      playerIndex = *resolvedPlayerIndex;
    }
    const SaPlayerFrame *player = sampledPlayerByIndex(playerIndex);
    return player ? playerStatValue(*player, localStatId) : "--";
  }
  if (teamScoped) {
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
  const auto graphStat = parseGraphStatId(normalizeUiStatId(statId));
  const bool playerScoped =
      definition ? definition->player : (graphStat && graphStat->scope == "player");
  const bool teamScoped =
      definition ? definition->team : (graphStat && graphStat->scope == "team");
  if (!playerScoped && !teamScoped) {
    ImGui::TextDisabled("-");
    return;
  }

  if (playerScoped) {
    const SaPlayerFrame *selected = nullptr;
    if (const std::optional<uint32_t> selectedPlayerIndex =
            playerIndexForTargetId(entry.target_id)) {
      selected = sampledPlayerByIndex(*selectedPlayerIndex);
    }
    const std::string selectedLabel =
        selected ? playerLabel(selected->player_index, selected->is_team_0) : "Select player";
    if (ImGui::BeginCombo(std::format("##ad-hoc-target-{}-{}", window.id, index).c_str(),
                          selectedLabel.c_str())) {
      for (uint8_t isTeam0 : {uint8_t{1}, uint8_t{0}}) {
        const bool hasTeamPlayers = std::any_of(
            sampledPlayers.begin(),
            sampledPlayers.end(),
            [isTeam0](const SaPlayerFrame &player) { return player.is_team_0 == isTeam0; });
        if (!hasTeamPlayers) {
          continue;
        }

        const LinearColor color =
            isTeam0 != 0 ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255};
        ImGui::TextColored(toImVec4(color), "%s team", teamLabel(isTeam0).c_str());
        for (const SaPlayerFrame &player : sampledPlayers) {
          if (player.is_team_0 != isTeam0) {
            continue;
          }
          const std::string nextTarget = webPlayerIdForIndex(player.player_index);
          const bool isSelected = statsWindowTargetsEqual(statId, entry.target_id, nextTarget);
          if (ImGui::Selectable(playerLabel(player.player_index, player.is_team_0).c_str(),
                                isSelected) &&
              !statsWindowHasStat(window, statId, nextTarget)) {
            entry.target_id = nextTarget;
          }
        }
        ImGui::Separator();
      }
      if (sampledPlayers.empty()) {
        ImGui::TextDisabled("Waiting for sampled players.");
      }
      ImGui::EndCombo();
    }
    return;
  }

  const char *selectedTeam = entry.target_id == "orange" ? "Orange" : "Blue";
  if (ImGui::BeginCombo(
          std::format("##ad-hoc-target-{}-{}", window.id, index).c_str(),
          selectedTeam)) {
    for (const auto &[label, targetId, isTeam0] : {
             std::tuple<const char *, const char *, uint8_t>{"Blue", "blue", uint8_t{1}},
             std::tuple<const char *, const char *, uint8_t>{"Orange", "orange", uint8_t{0}},
         }) {
      const LinearColor color =
          isTeam0 != 0 ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255};
      ImGui::PushStyleColor(ImGuiCol_Text, toImVec4(color));
      if (ImGui::Selectable(label, entry.target_id == targetId) &&
          !statsWindowHasStat(window, statId, targetId)) {
        entry.target_id = targetId;
      }
      ImGui::PopStyleColor();
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
  if (!ImGui::Begin(title.c_str(), &window.open, UI_FLOATING_WINDOW_FLAGS)) {
    ImGui::End();
    return;
  }
  captureStatsWindowPlacement(window);

  const bool scopeHeaderOnly = statsWindowKindHasScopeSelector(window.kind);
  if (!scopeHeaderOnly) {
    ImGui::TextColored(
        ImVec4{0.53f, 0.69f, 0.83f, 1.0f},
        "%s",
        statsWindowKindLabel(window.kind));
    ImGui::SameLine();
  }
  const std::string hideLabel = std::format("Hide##stats-window-hide-{}", window.id);
  const float hideWidth =
      ImGui::CalcTextSize("Hide").x + ImGui::GetStyle().FramePadding.x * 2.0f;
  const float rightAlignedX = ImGui::GetWindowContentRegionMax().x - hideWidth;
  if (rightAlignedX > ImGui::GetCursorPosX()) {
    ImGui::SetCursorPosX(rightAlignedX);
  }
  if (ImGui::SmallButton(hideLabel.c_str())) {
    window.open = false;
    ImGui::End();
    return;
  }
  ImGui::Separator();

  renderStatsWindowScopeSelector(window);
  renderStatsWindowAddControl(window);
  renderStatsWindowEntries(window);
  ImGui::End();
}

void SubtrActorPlugin::renderStatsWindowScopeSelector(UiStatsWindow &window) {
  if (window.kind == UiStatsWindowKind::Player) {
    resolveStatsWindowPlayerSelection(window);
    const SaPlayerFrame *selected = sampledPlayerByIndex(window.selected_player_index);
    const std::string selectedLabel =
        selected ? playerLabel(selected->player_index, selected->is_team_0) : "Select player";
    if (ImGui::BeginCombo("Player", selectedLabel.c_str())) {
      for (uint8_t isTeam0 : {uint8_t{1}, uint8_t{0}}) {
        const bool hasTeamPlayers = std::any_of(
            sampledPlayers.begin(),
            sampledPlayers.end(),
            [isTeam0](const SaPlayerFrame &player) { return player.is_team_0 == isTeam0; });
        if (!hasTeamPlayers) {
          continue;
        }

        const LinearColor color =
            isTeam0 != 0 ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255};
        ImGui::TextColored(toImVec4(color), "%s team", teamLabel(isTeam0).c_str());
        for (const SaPlayerFrame &player : sampledPlayers) {
          if (player.is_team_0 != isTeam0) {
            continue;
          }
          const std::string label = playerLabel(player.player_index, player.is_team_0);
          const bool isSelected = player.player_index == window.selected_player_index;
          if (ImGui::Selectable(label.c_str(), isSelected)) {
            window.selected_player_index = player.player_index;
            window.selected_player_id = webPlayerIdForIndex(window.selected_player_index);
          }
        }
        ImGui::Separator();
      }
      if (sampledPlayers.empty()) {
        ImGui::TextDisabled("Waiting for sampled players.");
      }
      ImGui::EndCombo();
    }
    return;
  }

  if (window.kind == UiStatsWindowKind::Team) {
    const char *selectedTeam = window.selected_team_is_team_0 != 0 ? "Blue" : "Orange";
    if (ImGui::BeginCombo("Team", selectedTeam)) {
      for (uint8_t isTeam0 : {uint8_t{1}, uint8_t{0}}) {
        const LinearColor color =
            isTeam0 != 0 ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255};
        const std::string label = teamLabel(isTeam0);
        const bool selected = (window.selected_team_is_team_0 != 0) == (isTeam0 != 0);
        ImGui::PushStyleColor(ImGuiCol_Text, toImVec4(color));
        if (ImGui::Selectable(label.c_str(), selected)) {
          window.selected_team_is_team_0 = isTeam0;
        }
        ImGui::PopStyleColor();
      }
      ImGui::EndCombo();
    }
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

  if (!statsWindowKindHasStatPicker(window.kind)) {
    ImGui::Separator();
    return;
  }

  const std::string addButton = std::format("+##add-stat-{}", window.id);
  if (statsWindowKindHasScopeSelector(window.kind)) {
    ImGui::SameLine();
  }
  if (ImGui::Button(addButton.c_str())) {
    window.picker_open = !window.picker_open;
  }
  if (ImGui::IsItemHovered()) {
    ImGui::SetTooltip("Add stat");
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

  std::vector<UiStatDefinitionCandidate> definitions;
  std::unordered_set<std::string> definitionIds;
  for (size_t index = 0; index < UI_STAT_DEFINITIONS.size(); index += 1) {
    const UiStatDefinition &definition = UI_STAT_DEFINITIONS[index];
    definitions.push_back(UiStatDefinitionCandidate{
        definition.id,
        definition.label,
        definition.category,
        definition.player,
        definition.team,
        definition.event,
    });
    definitionIds.insert(definition.id);
  }
  for (UiStatDefinitionCandidate &definition :
       graphStatDefinitionsFromStatsJson(currentStatsJson())) {
    if (definitionIds.insert(definition.id).second) {
      definitions.push_back(std::move(definition));
    }
  }

  std::vector<UiStatDefinitionMatch> matches;
  for (size_t index = 0; index < definitions.size(); index += 1) {
    const UiStatDefinitionCandidate &definition = definitions[index];
    if (!statsWindowSupportsStat(window, definition.id)) {
      continue;
    }
    const auto score = statDefinitionSearchScore(definition, window.picker_query);
    if (!score) {
      continue;
    }
    matches.push_back(UiStatDefinitionMatch{definition, *score, index});
  }
  std::sort(matches.begin(), matches.end(), [](const auto &left, const auto &right) {
    return left.score == right.score ? left.index < right.index : left.score < right.score;
  });

  std::vector<std::pair<std::string, int>> categoryCounts;
  for (const UiStatDefinitionMatch &match : matches) {
    auto found = std::find_if(
        categoryCounts.begin(),
        categoryCounts.end(),
        [&](const auto &entry) { return entry.first == match.definition.category; });
    if (found == categoryCounts.end()) {
      categoryCounts.emplace_back(match.definition.category, 1);
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
        if (category != match.definition.category) {
          continue;
        }
        const std::string targetId =
            window.kind == UiStatsWindowKind::AdHoc ? defaultAdHocTargetId(match.definition.id)
                                                    : "";
        if (!statsWindowHasStat(window, match.definition.id, targetId)) {
          window.entries.push_back(UiStatsWindow::Entry{match.definition.id, targetId});
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

  for (const UiStatDefinitionMatch &match : matches) {
    const UiStatDefinitionCandidate &definition = match.definition;
    const bool alreadySelected = statsWindowHasStat(window, definition.id);
    const std::string itemLabel = std::format(
        "{}  [{}]##{}-{}",
        definition.label,
        uiStatScopeLabel(definition),
        window.id,
        definition.id);
    if (alreadySelected && window.kind != UiStatsWindowKind::AdHoc) {
      ImGui::TextDisabled(
          "%s  [%s selected]",
          definition.label.c_str(),
          uiStatScopeLabel(definition));
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

  const bool hasSupportedEntries = std::any_of(
      window.entries.begin(),
      window.entries.end(),
      [&](const UiStatsWindow::Entry &entry) {
        return statsWindowSupportsStat(window, entry.stat_id);
      });
  if (!hasSupportedEntries) {
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
    if (sampledPlayers.empty() && currentStatsJson().empty()) {
      ImGui::TextWrapped("Start live analysis or load replay stats to show team stats.");
      break;
    }
    renderTeamStatsTable(window, window.selected_team_is_team_0);
    break;
  case UiStatsWindowKind::AllPlayers:
    renderAllPlayersStatsTable(window);
    break;
  case UiStatsWindowKind::AllTeams:
    if (sampledPlayers.empty() && currentStatsJson().empty()) {
      ImGui::TextWrapped("Start live analysis or load replay stats to show team stats.");
      break;
    }
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
    const std::string statLabel = uiStatLabel(statId);
    ImGui::Text("%s", statLabel.c_str());
    ImGui::NextColumn();
    ImGui::Text("%s", playerStatValue(player, statId).c_str());
    ImGui::NextColumn();
    if (ImGui::SmallButton(std::format("x##remove-stat-{}-{}", window.id, i).c_str())) {
      window.entries.erase(window.entries.begin() + static_cast<std::ptrdiff_t>(i));
      ImGui::Columns(1);
      return;
    }
    if (ImGui::IsItemHovered()) {
      ImGui::SetTooltip("Remove stat");
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
    const std::string statLabel = uiStatLabel(statId);
    ImGui::Text("%s", statLabel.c_str());
    ImGui::NextColumn();
    ImGui::Text("%s", teamStatValue(isTeam0, statId).c_str());
    ImGui::NextColumn();
    if (ImGui::SmallButton(std::format("x##remove-stat-{}-{}", window.id, i).c_str())) {
      window.entries.erase(window.entries.begin() + static_cast<std::ptrdiff_t>(i));
      ImGui::Columns(1);
      return;
    }
    if (ImGui::IsItemHovered()) {
      ImGui::SetTooltip("Remove stat");
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
          const std::string statLabel = uiStatLabel(statId);
          ImGui::Text("%s", statLabel.c_str());
          ImGui::NextColumn();
          ImGui::Text("%s", playerStatValue(player, statId).c_str());
          ImGui::NextColumn();
          if (ImGui::SmallButton(std::format("x##remove-stat-{}-{}", window.id, i).c_str())) {
            window.entries.erase(window.entries.begin() + static_cast<std::ptrdiff_t>(i));
            ImGui::Columns(1);
            ImGui::TreePop();
            ImGui::PopID();
            return true;
          }
          if (ImGui::IsItemHovered()) {
            ImGui::SetTooltip("Remove stat");
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
      const std::string statLabel = uiStatLabel(statId);
      ImGui::Text("%s", statLabel.c_str());
      ImGui::NextColumn();
      ImGui::Text("%s", teamStatValue(isTeam0, statId).c_str());
      ImGui::NextColumn();
      if (ImGui::SmallButton(
              std::format("x##remove-stat-{}-{}-{}", window.id, isTeam0, i).c_str())) {
        window.entries.erase(window.entries.begin() + static_cast<std::ptrdiff_t>(i));
        ImGui::Columns(1);
        return;
      }
      if (ImGui::IsItemHovered()) {
        ImGui::SetTooltip("Remove stat");
      }
      ImGui::NextColumn();
      ++i;
    }
    ImGui::Columns(1);
    ImGui::Separator();
  }
}

void SubtrActorPlugin::renderGoalsOverviewStats(UiStatsWindow &window) {
  (void)window;
  std::vector<size_t> goalEventIndexes;
  goalEventIndexes.reserve(recentUiEvents.size());
  for (size_t index = 0; index < recentUiEvents.size(); index += 1) {
    const UiEventRecord &event = recentUiEvents[index];
    if (event.category == "goal_context" || event.type == "goal") {
      goalEventIndexes.push_back(index);
    }
  }
  std::sort(goalEventIndexes.begin(), goalEventIndexes.end(), [&](size_t left, size_t right) {
    const UiEventRecord &leftEvent = recentUiEvents[left];
    const UiEventRecord &rightEvent = recentUiEvents[right];
    if (leftEvent.time == rightEvent.time) {
      return leftEvent.frame_number < rightEvent.frame_number;
    }
    return leftEvent.time < rightEvent.time;
  });

  ImGui::BeginChild("goal-labels", ImVec2{0.0f, 0.0f}, true);
  ReplayServerWrapper replayServer = gameWrapper->GetGameEventAsReplay();
  const bool hasReplayServer = !replayServer.IsNull();
  for (size_t ordinal = 0; ordinal < goalEventIndexes.size(); ordinal += 1) {
    const size_t index = goalEventIndexes[ordinal];
    const UiEventRecord &event = recentUiEvents[index];
    const float seekTime = std::max(0.0f, event.time - GOAL_WATCH_LEAD_SECONDS);
    ImGui::PushID(static_cast<int>(index));
    ImGui::TextColored(toImVec4(event.color), "Goal %zu", ordinal + 1);
    ImGui::SameLine();
    ImGui::TextDisabled("%.2fs - %s", event.time, event.actor.c_str());
    ImGui::TextWrapped("%s", event.label.c_str());
    if (!event.details.empty()) {
      ImGui::TextDisabled("%s", event.details.c_str());
    } else {
      ImGui::TextDisabled("Unlabeled");
    }
    if (ImGui::SmallButton("Watch")) {
      mechanicsReviewClipActive = false;
      playbackCurrentTime = seekTime;
      playbackPlaying = true;
      playbackSkipPostGoalTransitions = false;
      playbackSkipKickoffs = false;
      showSingletonWindow(uiPlaybackControlsOpen, playbackControlsPlacement);
      if (hasReplayServer) {
        replayServer.StartPlaybackAtTime(seekTime);
        playbackStatus = std::format("Watching goal from {:.2f}s", seekTime);
      } else {
        playbackStatus =
            std::format("Selected goal at {:.2f}s; open a replay to seek", seekTime);
      }
    }
    ImGui::SameLine();
    if (ImGui::SmallButton("Cue")) {
      mechanicsReviewClipActive = false;
      playbackCurrentTime = seekTime;
      playbackPlaying = false;
      playbackSkipPostGoalTransitions = false;
      playbackSkipKickoffs = false;
      showSingletonWindow(uiPlaybackControlsOpen, playbackControlsPlacement);
      if (hasReplayServer) {
        replayServer.SkipToTime(seekTime);
        ReplayWrapper replay = replayServer.GetReplay();
        if (!replay.IsNull()) {
          replay.StopPlayback();
        }
        playbackStatus = std::format("Cued goal at {:.2f}s", seekTime);
      } else {
        playbackStatus =
            std::format("Selected goal at {:.2f}s; open a replay to seek", seekTime);
      }
    }
    ImGui::Separator();
    ImGui::PopID();
  }
  if (goalEventIndexes.empty()) {
    ImGui::TextWrapped("No goals loaded.");
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
    const std::string statLabel = uiStatLabel(statId);
    ImGui::Text("%s", statLabel.c_str());
    ImGui::NextColumn();
    renderAdHocTargetSelector(window, entry, statId, i);
    ImGui::NextColumn();
    ImGui::Text("%s", adHocStatValue(statId, entry.target_id).c_str());
    ImGui::NextColumn();
    if (ImGui::SmallButton(std::format("x##remove-stat-{}-{}", window.id, i).c_str())) {
      window.entries.erase(window.entries.begin() + static_cast<std::ptrdiff_t>(i));
      ImGui::Columns(1);
      return;
    }
    if (ImGui::IsItemHovered()) {
      ImGui::SetTooltip("Remove stat");
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
