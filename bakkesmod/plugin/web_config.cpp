// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
std::optional<std::string> SubtrActorPlugin::statsPlayerCfgJsonFromClipboard(
    std::string_view clipboardText) {
  const std::optional<std::string> value = statsPlayerCfgValueFromClipboard(clipboardText);
  if (!value) {
    return std::nullopt;
  }

  const size_t firstByte = value->find_first_not_of(" \t\r\n");
  if (firstByte == std::string::npos) {
    return std::nullopt;
  }
  if ((*value)[firstByte] == '{') {
    return value->substr(firstByte);
  }

  if (!decodedStatsPlayerConfigJsonLen || !writeDecodedStatsPlayerConfigJson) {
    return std::nullopt;
  }

  const std::string encoded = value->substr(firstByte);
  if (encoded.find('\0') != std::string::npos) {
    return std::nullopt;
  }
  const size_t byteCount = decodedStatsPlayerConfigJsonLen(encoded.c_str());
  if (byteCount == 0) {
    return std::nullopt;
  }

  std::string json(byteCount, '\0');
  const size_t written = writeDecodedStatsPlayerConfigJson(
      encoded.c_str(),
      reinterpret_cast<uint8_t *>(json.data()),
      json.size());
  if (written == 0 || written > json.size()) {
    return std::nullopt;
  }
  json.resize(written);
  const size_t jsonFirstByte = json.find_first_not_of(" \t\r\n");
  if (jsonFirstByte == std::string::npos || json[jsonFirstByte] != '{') {
    return std::nullopt;
  }
  return json.substr(jsonFirstByte);
}

std::optional<std::string> SubtrActorPlugin::statsPlayerCfgFromJson(const std::string &json) {
  if (!encodedStatsPlayerConfigLen || !writeEncodedStatsPlayerConfig ||
      json.find('\0') != std::string::npos) {
    return std::nullopt;
  }

  const size_t byteCount = encodedStatsPlayerConfigLen(json.c_str());
  if (byteCount == 0) {
    return std::nullopt;
  }

  std::string encoded(byteCount, '\0');
  const size_t written = writeEncodedStatsPlayerConfig(
      json.c_str(),
      reinterpret_cast<uint8_t *>(encoded.data()),
      encoded.size());
  if (written == 0 || written > encoded.size()) {
    return std::nullopt;
  }
  encoded.resize(written);
  return std::format("#cfg={}", encoded);
}

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

std::optional<std::string> SubtrActorPlugin::webPlayerIdForIndexIfKnown(
    uint32_t playerIndex) const {
  const auto uniqueId = playerUniqueIdsByIndex.find(playerIndex);
  if (uniqueId != playerUniqueIdsByIndex.end() && !uniqueId->second.empty()) {
    return uniqueId->second;
  }
  const auto sampledPlayer = std::find_if(
      sampledPlayers.begin(),
      sampledPlayers.end(),
      [playerIndex](const SaPlayerFrame &player) {
        return player.player_index == playerIndex;
      });
  if (sampledPlayer != sampledPlayers.end()) {
    return std::to_string(playerIndex);
  }
  return std::nullopt;
}

std::string SubtrActorPlugin::webPlayerIdForWindow(const UiStatsWindow &window) const {
  if (!window.selected_player_id.empty() &&
      !parseUnsignedIntegerString(window.selected_player_id)) {
    return window.selected_player_id;
  }
  return webPlayerIdForIndex(window.selected_player_index);
}

std::optional<std::string> SubtrActorPlugin::webPlayerIdForWindowConfig(
    const UiStatsWindow &window) const {
  if (!window.selected_player_id.empty()) {
    return window.selected_player_id;
  }
  return webPlayerIdForIndexIfKnown(window.selected_player_index);
}

void SubtrActorPlugin::resolveStatsWindowPlayerSelection(UiStatsWindow &window) {
  if (window.selected_player_id.empty()) {
    if (const auto playerId = webPlayerIdForIndexIfKnown(window.selected_player_index)) {
      window.selected_player_id = *playerId;
    }
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

std::optional<std::string> SubtrActorPlugin::webCameraPlayerIdConfig() const {
  if (!cameraSelectedPlayerId.empty()) {
    return cameraSelectedPlayerId;
  }
  return webPlayerIdForIndexIfKnown(cameraSelectedPlayerIndex);
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
      "subtr_actor_dump_products",
      [this](std::vector<std::string> params) { dumpProductsJson(params); },
      "Writes Rocket League product metadata JSON from BakkesMod's item database. "
      "Defaults to body products only; pass 'all' to include every slot.",
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
        showLauncherWindow();
      },
      "Opens the subtr-actor in-game launcher window.",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "subtr_actor_toggle_ui",
      [this](std::vector<std::string>) {
        uiWindowOpen = true;
        if (uiLauncherOpen) {
          hideLauncherWindow();
        } else {
          showLauncherWindow();
        }
      },
      "Toggles the subtr-actor in-game launcher window.",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "subtr_actor_apply_ui_config",
      [this](std::vector<std::string> params) { applyUiConfigParams(std::move(params)); },
      "Applies a subtr-actor UI config. Usage: subtr_actor_apply_ui_config <json|cfg|url>",
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
  showLauncherWindow();
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
  decodedStatsPlayerConfigJsonLen = reinterpret_cast<DecodedStatsPlayerConfigJsonLen>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_decoded_stats_player_config_json_len"));
  writeDecodedStatsPlayerConfigJson = reinterpret_cast<WriteDecodedStatsPlayerConfigJson>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_write_decoded_stats_player_config_json"));
  encodedStatsPlayerConfigLen = reinterpret_cast<EncodedStatsPlayerConfigLen>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_encoded_stats_player_config_len"));
  writeEncodedStatsPlayerConfig = reinterpret_cast<WriteEncodedStatsPlayerConfig>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_write_encoded_stats_player_config"));
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
  writeReplayAnnotationFramePlayers = reinterpret_cast<WriteReplayAnnotationFramePlayers>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_write_replay_annotation_frame_players"));
  replayAnnotationFrameJsonLen = reinterpret_cast<ReplayAnnotationFrameJsonLen>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_replay_annotation_frame_json_len"));
  writeReplayAnnotationFrameJson = reinterpret_cast<WriteReplayAnnotationFrameJson>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_write_replay_annotation_frame_json"));
  replayAnnotationScoreAtTime = reinterpret_cast<ReplayAnnotationScoreAtTime>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_replay_annotation_score_at_time"));
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
      !decodedStatsPlayerConfigJsonLen || !writeDecodedStatsPlayerConfigJson ||
      !encodedStatsPlayerConfigLen || !writeEncodedStatsPlayerConfig ||
      !drainEvents || !drainTeamEvents || !drainGoalContextEvents ||
      !replayAnnotationsCreate || !replayAnnotationsDestroy || !replayAnnotationCount ||
      !replayAnnotationPlayerCount || !writeReplayAnnotationPlayers ||
      !writeReplayAnnotationFramePlayers || !replayAnnotationFrameJsonLen ||
      !writeReplayAnnotationFrameJson || !replayAnnotationScoreAtTime || !pollReplayAnnotations) {
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
  decodedStatsPlayerConfigJsonLen = nullptr;
  writeDecodedStatsPlayerConfigJson = nullptr;
  encodedStatsPlayerConfigLen = nullptr;
  writeEncodedStatsPlayerConfig = nullptr;
  drainEvents = nullptr;
  drainTeamEvents = nullptr;
  drainGoalContextEvents = nullptr;
  replayAnnotationsCreate = nullptr;
  replayAnnotationsDestroy = nullptr;
  replayAnnotationCount = nullptr;
  replayAnnotationPlayerCount = nullptr;
  writeReplayAnnotationPlayers = nullptr;
  writeReplayAnnotationFramePlayers = nullptr;
  replayAnnotationFrameJsonLen = nullptr;
  writeReplayAnnotationFrameJson = nullptr;
  replayAnnotationScoreAtTime = nullptr;
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
    const bool changed = cvar.getBoolValue() != value;
    cvar.setValue(value ? 1 : 0);
    if (changed) {
      scheduleUiConfigAutosave();
    }
  }
}

std::string SubtrActorPlugin::cvarString(const char *name, std::string_view defaultValue) const {
  auto cvar = cvarManager->getCvar(name);
  return static_cast<bool>(cvar) ? cvar.getStringValue() : std::string(defaultValue);
}

void SubtrActorPlugin::setCvarString(const char *name, std::string_view value) {
  auto cvar = cvarManager->getCvar(name);
  if (static_cast<bool>(cvar)) {
    const bool changed = cvar.getStringValue() != value;
    cvar.setValue(std::string(value));
    if (changed) {
      scheduleUiConfigAutosave();
    }
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

void SubtrActorPlugin::applyUiConfigParams(std::vector<std::string> params) {
  if (params.size() < 2) {
    cvarManager->log(
        "subtr-actor: usage: subtr_actor_apply_ui_config <json|cfg|stats-player-url>");
    return;
  }

  std::vector<std::string> configParts(params.begin() + 1, params.end());
  const std::string configText = joinStrings(configParts, " ");
  if (const std::optional<std::string> configJson =
          statsPlayerCfgJsonFromClipboard(configText)) {
    applyUiConfigJson(*configJson, "console");
    saveUiConfig();
    cvarManager->log("subtr-actor: applied UI config from console");
    return;
  }

  cvarManager->log(
      "subtr-actor: console argument does not contain UI config JSON or a stats-player cfg value");
}
