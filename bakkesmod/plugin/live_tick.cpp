// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
void SubtrActorPlugin::renderLayoutConfigControls(const char *idSuffix, bool fullWidth) {
  ImGui::PushID(idSuffix);
  auto buttonSize = [fullWidth]() {
    return ImVec2{fullWidth ? ImGui::GetContentRegionAvail().x : 0.0f, 0.0f};
  };
  if (ImGui::Button("Save layout", buttonSize())) {
    saveUiConfig();
  }
  if (!fullWidth) {
    ImGui::SameLine();
  }
  if (ImGui::Button("Reload layout", buttonSize())) {
    loadUiConfig();
  }
  if (ImGui::Button("Copy layout JSON", buttonSize())) {
    const std::string json = uiConfigJson();
    ImGui::SetClipboardText(json.c_str());
    cvarManager->log(std::format("subtr-actor: copied {} UI config bytes", json.size()));
  }
  if (!fullWidth) {
    ImGui::SameLine();
  }
  if (ImGui::Button("Copy layout cfg", buttonSize())) {
    const std::string json = uiConfigJson();
    const std::optional<std::string> encodedCfg = statsPlayerCfgFromJson(json);
    const std::string cfg = encodedCfg.value_or(std::format("#cfg={}", urlEncode(json)));
    ImGui::SetClipboardText(cfg.c_str());
    cvarManager->log(std::format("subtr-actor: copied {} UI config hash bytes", cfg.size()));
  }
  if (!fullWidth) {
    ImGui::SameLine();
  }
  if (ImGui::Button("Paste layout", buttonSize())) {
    const char *clipboardText = ImGui::GetClipboardText();
    if (clipboardText == nullptr || clipboardText[0] == '\0') {
      cvarManager->log("subtr-actor: clipboard does not contain UI config JSON or cfg");
    } else if (const std::optional<std::string> configJson =
                   statsPlayerCfgJsonFromClipboard(clipboardText)) {
      applyUiConfigJson(*configJson, "clipboard");
    } else {
      cvarManager->log(
          "subtr-actor: clipboard does not contain UI config JSON or a stats-player cfg value");
    }
  }
  ImGui::PopID();
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
  cachedReplayFrameJson.clear();
  cachedReplayFrameJsonTime = -1.0f;
  if (gameWrapper && gameWrapper->IsInReplay()) {
    sampledPlayers.clear();
    sampledPlayerNames.clear();
  }
}

void SubtrActorPlugin::importReplayAnnotationPlayers(float replayTime) {
  if (!replayAnnotations || !replayAnnotationPlayerCount || !writeReplayAnnotationPlayers) {
    return;
  }

  const size_t playerCount = replayAnnotationPlayerCount(replayAnnotations);
  if (playerCount == 0) {
    return;
  }

  if (writeReplayAnnotationFramePlayers) {
    std::vector<SaPlayerFrame> framePlayers(playerCount);
    const size_t copiedFramePlayers = writeReplayAnnotationFramePlayers(
        replayAnnotations,
        replayTime,
        framePlayers.data(),
        framePlayers.size());
    if (copiedFramePlayers > 0) {
      sampledPlayers.assign(framePlayers.begin(), framePlayers.begin() + copiedFramePlayers);
      sampledPlayerNames.clear();
      for (const SaPlayerFrame &player : sampledPlayers) {
        if (player.player_name != nullptr && player.player_name[0] != '\0') {
          playerNamesByIndex[player.player_index] = player.player_name;
        }
        playerTeamsByIndex[player.player_index] = player.is_team_0;
      }
      return;
    }
  }

  std::vector<SaReplayPlayerInfo> players(playerCount);
  const size_t copied =
      writeReplayAnnotationPlayers(replayAnnotations, players.data(), players.size());
  sampledPlayers.clear();
  sampledPlayerNames.clear();
  for (size_t i = 0; i < copied; i += 1) {
    const SaReplayPlayerInfo &player = players[i];
    if (player.name != nullptr && player.name[0] != '\0') {
      playerNamesByIndex[player.player_index] = player.name;
    }
    playerTeamsByIndex[player.player_index] = player.is_team_0;
    SaPlayerFrame frame{};
    frame.player_index = player.player_index;
    frame.player_name = player.name;
    frame.is_team_0 = player.is_team_0;
    sampledPlayers.push_back(frame);
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
    importReplayAnnotationPlayers(replayServer.GetReplayTimeElapsed());
    cvarManager->log(std::format(
        "subtr-actor: loaded {} replay annotations from normal replay processor for {}",
        annotationCount,
        *replayPath));
  }

  importReplayAnnotationPlayers(replayServer.GetReplayTimeElapsed());
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
