// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
void SubtrActorPlugin::createStatsWindow(UiStatsWindowKind kind, bool initializeEntries) {
  UiStatsWindow window{};
  window.id = nextUiStatsWindowId++;
  window.config_id = std::format("stats-{}", window.id);
  window.kind = kind;
  initializeStatsWindowPlacement(window);
  if (kind == UiStatsWindowKind::StatsModule) {
    const std::vector<std::string> moduleNames = availableStatsModuleNames();
    if (!moduleNames.empty()) {
      window.module_name = moduleNames.front();
    }
  }
  if (!sampledPlayers.empty()) {
    window.selected_player_index = sampledPlayers.front().player_index;
    window.selected_player_id = webPlayerIdForIndex(window.selected_player_index);
    window.selected_team_is_team_0 = sampledPlayers.front().is_team_0;
  }
  if (initializeEntries) {
    initializeStatsWindowEntries(window);
  }
  uiStatsWindows.push_back(window);
  scheduleUiConfigAutosave();
}

void SubtrActorPlugin::createStatsModuleWindow(std::string moduleName, int moduleView) {
  UiStatsWindow window{};
  window.id = nextUiStatsWindowId++;
  window.config_id = std::format("stats-{}", window.id);
  window.kind = UiStatsWindowKind::StatsModule;
  window.module_name = std::move(moduleName);
  window.module_view = std::clamp(moduleView, 0, 2);
  initializeStatsWindowPlacement(window);
  uiStatsWindows.push_back(std::move(window));
  scheduleUiConfigAutosave();
}

std::pair<float, float> SubtrActorPlugin::defaultStatsWindowSize(UiStatsWindowKind kind) const {
  if (kind == UiStatsWindowKind::StatsModule) {
    return {680.0f, 460.0f};
  }
  return {416.0f, 330.0f};
}

std::pair<float, float> SubtrActorPlugin::defaultStatsWindowPosition(size_t stackIndex) const {
  const float offset = static_cast<float>(stackIndex * 18);
  const ImVec2 displaySize = ImGui::GetIO().DisplaySize;
  const float viewportWidth = displaySize.x > 0.0f ? displaySize.x : 1440.0f;
  const float viewportHeight = displaySize.y > 0.0f ? displaySize.y : 900.0f;
  return {
      std::max(12.0f, std::min(viewportWidth - 360.0f, 96.0f + offset)),
      std::max(64.0f, std::min(viewportHeight - 240.0f, 96.0f + offset)),
  };
}

void SubtrActorPlugin::initializeStatsWindowPlacement(UiStatsWindow &window) {
  resetStatsWindowPlacement(window, uiStatsWindows.size());
}

void SubtrActorPlugin::resetStatsWindowPlacement(UiStatsWindow &window, size_t stackIndex) {
  const auto [width, height] = defaultStatsWindowSize(window.kind);
  window.width = width;
  window.height = height;
  const auto [x, y] = defaultStatsWindowPosition(stackIndex);
  window.x = x;
  window.y = y;
  const ImVec2 displaySize = ImGui::GetIO().DisplaySize;
  window.viewport_width = displaySize.x;
  window.viewport_height = displaySize.y;
  window.has_placement = true;
  window.pending_apply_placement = true;
  window.pending_focus = window.open;
  window.z_index = nextUiWindowZIndex++;
  scheduleUiConfigAutosave();
}

std::array<SubtrActorPlugin::StatsWindowKindControl, 8>
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
      {UiStatsWindowKind::KickoffOverview,
       "kickoff-overview",
       "Kickoff details",
       "New kickoff details",
       UI_STAT_SCOPE_EVENT,
       false,
       false,
       true,
       true},
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
  case UiStatsWindowKind::KickoffOverview:
    window.entries.clear();
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

std::vector<std::string> SubtrActorPlugin::availableStatsModuleNames() {
  std::vector<std::string> names = statsModuleNames();
  std::unordered_set<std::string> seen(names.begin(), names.end());
  auto appendName = [&](const std::string &moduleName) {
    if (seen.insert(moduleName).second) {
      names.push_back(moduleName);
    }
  };
  for (const std::string &moduleName :
       replayStatsModuleNamesFromFrameJson(currentReplayFrameJson())) {
    appendName(moduleName);
  }
  return names;
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

const std::string &SubtrActorPlugin::currentReplayFrameJson() const {
  if (!replayAnnotations || !replayAnnotationFrameJsonLen || !writeReplayAnnotationFrameJson ||
      !gameWrapper || !gameWrapper->IsInReplay()) {
    cachedReplayFrameJson.clear();
    cachedReplayFrameJsonTime = -1.0f;
    return cachedReplayFrameJson;
  }

  ReplayServerWrapper replayServer = gameWrapper->GetGameEventAsReplay();
  if (replayServer.IsNull()) {
    cachedReplayFrameJson.clear();
    cachedReplayFrameJsonTime = -1.0f;
    return cachedReplayFrameJson;
  }

  const float replayTime = replayServer.GetReplayTimeElapsed();
  if (cachedReplayFrameJsonTime == replayTime) {
    return cachedReplayFrameJson;
  }

  const size_t byteCount = replayAnnotationFrameJsonLen(replayAnnotations, replayTime);
  if (byteCount == 0) {
    cachedReplayFrameJson.clear();
    cachedReplayFrameJsonTime = replayTime;
    return cachedReplayFrameJson;
  }

  std::string buffer(byteCount, '\0');
  const size_t written = writeReplayAnnotationFrameJson(
      replayAnnotations,
      replayTime,
      reinterpret_cast<uint8_t *>(buffer.data()),
      buffer.size());
  buffer.resize(written);
  cachedReplayFrameJson = std::move(buffer);
  cachedReplayFrameJsonTime = replayTime;
  return cachedReplayFrameJson;
}

std::optional<std::string> SubtrActorPlugin::graphPlayerStatValue(
    const SaPlayerFrame &player,
    std::string_view statId) const {
  const auto parsed = parseGraphStatId(statId);
  if (!parsed || parsed->scope != "player") {
    return std::nullopt;
  }

  if (const std::string &statsJson = currentStatsJson(); !statsJson.empty()) {
    const auto frame = parseJsonObjectProperty(statsJson, "frame");
    const auto modules = frame ? parseJsonObjectProperty(*frame, "modules") : std::nullopt;
    const auto module =
        modules ? parseJsonObjectProperty(*modules, std::string{parsed->module})
                : std::nullopt;
    if (module) {
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
    }
  }

  const std::string &replayFrameJson = currentReplayFrameJson();
  if (replayFrameJson.empty()) {
    return std::nullopt;
  }
  const std::vector<std::string> replayPlayers =
      parseJsonObjectArrayProperty(replayFrameJson, "players");
  for (size_t index = 0; index < replayPlayers.size(); index += 1) {
    const std::string &entry = replayPlayers[index];
    const auto playerId = parseJsonObjectProperty(entry, "player_id");
    const bool matchesById = playerId && jsonPlayerIdMatchesIndex(*playerId, player.player_index);
    const bool matchesByReplayOrder = index == player.player_index;
    if (!matchesById && !matchesByReplayOrder) {
      continue;
    }
    const auto module = parseJsonObjectProperty(entry, std::string{parsed->module});
    return module ? jsonDisplayValueAtPath(*module, parsed->path) : std::nullopt;
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

  if (const std::string &statsJson = currentStatsJson(); !statsJson.empty()) {
    const auto frame = parseJsonObjectProperty(statsJson, "frame");
    const auto modules = frame ? parseJsonObjectProperty(*frame, "modules") : std::nullopt;
    const auto module =
        modules ? parseJsonObjectProperty(*modules, std::string{parsed->module})
                : std::nullopt;
    const auto team =
        module ? parseJsonObjectProperty(*module, isTeam0 != 0 ? "team_zero" : "team_one")
               : std::nullopt;
    if (team) {
      return jsonDisplayValueAtPath(*team, parsed->path);
    }
  }

  const std::string &replayFrameJson = currentReplayFrameJson();
  if (replayFrameJson.empty()) {
    return std::nullopt;
  }
  const auto teamSnapshot =
      parseJsonObjectProperty(replayFrameJson, isTeam0 != 0 ? "team_zero" : "team_one");
  const auto module =
      teamSnapshot ? parseJsonObjectProperty(*teamSnapshot, std::string{parsed->module})
                   : std::nullopt;
  return module ? jsonDisplayValueAtPath(*module, parsed->path) : std::nullopt;
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
