// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
namespace {

constexpr float PI = 3.14159265358979323846f;
constexpr float UNREAL_ROTATOR_TO_RADIANS = (2.0f * PI) / 65536.0f;
constexpr float GOAL_WATCH_LEAD_SECONDS = 4.0f;
constexpr ImGuiWindowFlags UI_FLOATING_WINDOW_FLAGS =
    ImGuiWindowFlags_NoTitleBar | ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoCollapse |
    ImGuiWindowFlags_NoSavedSettings;
constexpr ImGuiWindowFlags UI_CHROME_WINDOW_FLAGS =
    ImGuiWindowFlags_NoTitleBar | ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoMove |
    ImGuiWindowFlags_NoScrollbar | ImGuiWindowFlags_NoCollapse |
    ImGuiWindowFlags_AlwaysAutoResize | ImGuiWindowFlags_NoSavedSettings;
constexpr ImGuiWindowFlags UI_LAUNCHER_MENU_WINDOW_FLAGS =
    ImGuiWindowFlags_NoTitleBar | ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoMove |
    ImGuiWindowFlags_NoCollapse | ImGuiWindowFlags_AlwaysAutoResize |
    ImGuiWindowFlags_NoSavedSettings;
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

std::string formatEventPlaylistTime(float seconds) {
  if (!std::isfinite(seconds)) {
    return "--";
  }
  const float clampedSeconds = std::max(0.0f, seconds);
  const int minutes = static_cast<int>(std::floor(clampedSeconds / 60.0f));
  const float remainingSeconds = clampedSeconds - static_cast<float>(minutes * 60);
  return std::format("{}:{:04.1f}", minutes, remainingSeconds);
}

std::string replaySourceDisplayLabel(std::string_view path) {
  constexpr std::string_view prefix = "path:";
  if (path.starts_with(prefix)) {
    path.remove_prefix(prefix.size());
  }
  const size_t lastSeparator = path.find_last_of("/\\");
  if (lastSeparator != std::string_view::npos) {
    path.remove_prefix(lastSeparator + 1);
  }
  return path.empty() ? "review replay" : std::string{path};
}

std::string joinStrings(const std::vector<std::string> &parts, std::string_view separator) {
  std::string joined;
  for (const std::string &part : parts) {
    if (!joined.empty()) {
      joined.append(separator);
    }
    joined.append(part);
  }
  return joined;
}

std::string uppercaseHeaderLabel(std::string_view value) {
  std::string label;
  label.reserve(value.size());
  for (char ch : value) {
    label.push_back(static_cast<char>(std::toupper(static_cast<unsigned char>(ch))));
  }
  return label;
}

void pushWebFloatingWindowStyle() {
  ImGui::PushStyleVar(ImGuiStyleVar_WindowPadding, ImVec2{14.0f, 12.0f});
  ImGui::PushStyleVar(ImGuiStyleVar_WindowRounding, 8.0f);
  ImGui::PushStyleVar(ImGuiStyleVar_FrameRounding, 6.0f);
  ImGui::PushStyleVar(ImGuiStyleVar_ItemSpacing, ImVec2{8.0f, 8.0f});
  ImGui::PushStyleColor(ImGuiCol_WindowBg, ImVec4{0.03f, 0.07f, 0.10f, 0.88f});
  ImGui::PushStyleColor(ImGuiCol_Border, ImVec4{1.0f, 1.0f, 1.0f, 0.12f});
}

void popWebFloatingWindowStyle() {
  ImGui::PopStyleColor(2);
  ImGui::PopStyleVar(4);
}

void pushWebModuleSummaryButtonStyle(bool active) {
  ImGui::PushStyleVar(ImGuiStyleVar_FramePadding, ImVec2{10.0f, 5.0f});
  ImGui::PushStyleVar(ImGuiStyleVar_FrameRounding, 999.0f);
  ImGui::PushStyleVar(ImGuiStyleVar_FrameBorderSize, 1.0f);
  ImGui::PushStyleColor(
      ImGuiCol_Button,
      active ? ImVec4{0.29f, 0.58f, 1.0f, 0.08f} : ImVec4{1.0f, 1.0f, 1.0f, 0.03f});
  ImGui::PushStyleColor(
      ImGuiCol_ButtonHovered,
      active ? ImVec4{0.29f, 0.58f, 1.0f, 0.14f} : ImVec4{1.0f, 1.0f, 1.0f, 0.07f});
  ImGui::PushStyleColor(
      ImGuiCol_ButtonActive,
      active ? ImVec4{0.29f, 0.58f, 1.0f, 0.20f} : ImVec4{1.0f, 1.0f, 1.0f, 0.11f});
  ImGui::PushStyleColor(
      ImGuiCol_Border,
      active ? ImVec4{0.29f, 0.58f, 1.0f, 0.22f} : ImVec4{1.0f, 1.0f, 1.0f, 0.10f});
  ImGui::PushStyleColor(
      ImGuiCol_Text,
      active ? ImVec4{0.86f, 0.92f, 0.98f, 1.0f} : ImVec4{0.63f, 0.70f, 0.76f, 1.0f});
}

void popWebModuleSummaryButtonStyle() {
  ImGui::PopStyleColor(5);
  ImGui::PopStyleVar(3);
}

void renderWebDetailGridCell(std::string_view label, std::string_view value) {
  const std::string labelString{label};
  const std::string valueString{value};
  ImGui::TextColored(ImVec4{0.54f, 0.64f, 0.73f, 1.0f}, "%s", labelString.c_str());
  ImGui::TextColored(ImVec4{0.93f, 0.96f, 0.98f, 1.0f}, "%s", valueString.c_str());
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
constexpr std::array<const char *, 50> GRAPH_EVENT_FIELDS{
    "timeline",
    "mechanics",
    "goal_context",
    "core_player",
    "core_player_goal_context",
    "possession",
    "pressure",
    "territorial_pressure",
    "movement",
    "positioning_activity",
    "positioning_possession",
    "positioning_field_zone",
    "positioning_ball_depth",
    "positioning_teammate_role",
    "positioning_ball_proximity",
    "positioning_goal_context",
    "rotation_player",
    "rotation_team",
    "rotation_role_span",
    "rotation_depth_span",
    "rotation_first_man_stint",
    "backboard",
    "ball_carry",
    "controlled_play",
    "ceiling_shot",
    "wall_aerial",
    "wall_aerial_shot",
    "center",
    "double_tap",
    "fifty_fifty",
    "kickoff",
    "flick",
    "musty_flick",
    "one_timer",
    "pass",
    "rush",
    "flip_impulse",
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
    {"goal", "Goals", "Replay"},
    {"core", "Shots, saves, assists", "Replay"},
    {"demo", "Demos", "Replay"},
    {"mechanics", "All mechanics", "Sources"},
    {"team", "Team events", "Sources"},
    {"goal_context", "Goal context", "Sources"},
    {"touch", "Touch", "Stats"},
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
    float /*width*/,
    float /*height*/,
    float sourceViewportWidth,
    float sourceViewportHeight) {
  const ImVec2 displaySize = ImGui::GetIO().DisplaySize;
  if (displaySize.x <= 0.0f || displaySize.y <= 0.0f) {
    return ImVec2{x, y};
  }

  constexpr float margin = 8.0f;
  constexpr float minimumVisibleWidth = 120.0f;
  constexpr float minimumVisibleHeight = 100.0f;
  const float scaleX = sourceViewportWidth > 0.0f ? displaySize.x / sourceViewportWidth : 1.0f;
  const float scaleY = sourceViewportHeight > 0.0f ? displaySize.y / sourceViewportHeight : 1.0f;
  const float maxX = std::max(margin, displaySize.x - minimumVisibleWidth);
  const float maxY = std::max(margin, displaySize.y - minimumVisibleHeight);
  return ImVec2{
      std::clamp(x * scaleX, margin, maxX),
      std::clamp(y * scaleY, margin, maxY),
  };
}


} // namespace
