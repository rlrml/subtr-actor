// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
namespace {

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
  case SaMechanicKindSustainedPressureGoal:
    return "Sustained pressure goal";
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
  case SaMechanicKindBumpGoal:
    return "Bump goal";
  case SaMechanicKindDemoGoal:
    return "Demo goal";
  default:
    return "Mechanic";
  }
}

bool corePlayerStatMechanicKind(SaMechanicKind kind) {
  return kind == SaMechanicKindShot || kind == SaMechanicKindSave ||
         kind == SaMechanicKindAssist;
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

std::string webTimelineEventSourceIdForFilterToken(std::string_view token) {
  if (token == "goal" || token == "mechanics" || token == "team" || token == "goal_context") {
    return {};
  }
  if (token != "core" && token != "touch" && !isMechanicFilterToken(token)) {
    return {};
  }

  std::string id{token};
  std::replace(id.begin(), id.end(), '_', '-');
  return id;
}

std::string_view playerStatEventType(SaPlayerStatEventKind kind) {
  switch (kind) {
  case SaPlayerStatEventKindShot:
    return "shot";
  case SaPlayerStatEventKindSave:
    return "save";
  case SaPlayerStatEventKindAssist:
    return "assist";
  default:
    return "core";
  }
}

std::string_view playerStatEventLabel(SaPlayerStatEventKind kind) {
  switch (kind) {
  case SaPlayerStatEventKindShot:
    return "Shot";
  case SaPlayerStatEventKindSave:
    return "Save";
  case SaPlayerStatEventKindAssist:
    return "Assist";
  default:
    return "Core event";
  }
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

bool eventPlaylistUsesDefaultSources(std::string_view rawFilter) {
  const std::vector<std::string> tokens = eventFilterTokens(rawFilter);
  return tokens.size() == 1 && tokens.front() == "default";
}

bool defaultEventPlaylistSourceOption(std::string_view optionValue) {
  return optionValue != "all" && optionValue != "mechanics" && optionValue != "touch" &&
         optionValue != "powerslide";
}

std::vector<std::string> defaultEventPlaylistSourceTokens() {
  std::vector<std::string> tokens;
  for (const EventFilterOption &option : EVENT_FILTER_OPTIONS) {
    if (defaultEventPlaylistSourceOption(option.value)) {
      tokens.emplace_back(option.value);
    }
  }
  return tokens;
}

bool defaultEventPlaylistSourceAllows(std::string_view category, std::string_view type) {
  for (const EventFilterOption &option : EVENT_FILTER_OPTIONS) {
    if (defaultEventPlaylistSourceOption(option.value) &&
        eventFilterAllows(option.value, category, type)) {
      return true;
    }
  }
  return false;
}

bool allEventSourcesSelected(std::string_view rawFilter) {
  const std::vector<std::string> tokens = eventFilterTokens(rawFilter);
  return tokens.empty() || containsString(tokens, "all");
}

std::vector<std::string> selectedEventSourceTokens(std::string_view rawFilter) {
  std::vector<std::string> tokens;
  if (eventPlaylistUsesDefaultSources(rawFilter)) {
    return defaultEventPlaylistSourceTokens();
  }
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

std::string eventTypeDisplayLabel(std::string_view value) {
  const std::string normalized = normalizeEventFilterToken(value);
  if (normalized == "shot") {
    return "Shot";
  }
  if (normalized == "save") {
    return "Save";
  }
  if (normalized == "assist") {
    return "Assist";
  }
  if (normalized == "core") {
    return "Core event";
  }
  if (normalized == "goal") {
    return "Goal";
  }
  for (const EventFilterOption &option : EVENT_FILTER_OPTIONS) {
    if (normalized == option.value) {
      return option.label;
    }
  }

  std::string label;
  label.reserve(normalized.size());
  bool capitalizeNext = true;
  for (const char ch : normalized) {
    if (ch == '_') {
      label.push_back(' ');
      capitalizeNext = false;
      continue;
    }
    label.push_back(
        capitalizeNext ? static_cast<char>(std::toupper(static_cast<unsigned char>(ch))) : ch);
    capitalizeNext = false;
  }
  return label.empty() ? "Event" : label;
}

std::string eventFilterPreview(std::string_view rawFilter) {
  if (eventPlaylistUsesDefaultSources(rawFilter)) {
    return "Default events";
  }
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

LinearColor eventPlaylistPlayerColor(uint32_t playerIndex) {
  const std::array<LinearColor, 8> colors{{
      {0x3b, 0x82, 0xf6, 0xff},
      {0x06, 0xb6, 0xd4, 0xff},
      {0x22, 0xc5, 0x5e, 0xff},
      {0xa8, 0x55, 0xf7, 0xff},
      {0xf9, 0x73, 0x16, 0xff},
      {0xef, 0x44, 0x44, 0xff},
      {0xf5, 0x9e, 0x0b, 0xff},
      {0xec, 0x48, 0x99, 0xff},
  }};
  return colors[playerIndex % colors.size()];
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
