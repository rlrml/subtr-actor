// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
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
  const std::vector<std::string> moduleNames = availableStatsModuleNames();
  if (moduleNames.empty()) {
    ImGui::TextWrapped("Start live analysis or load replay annotations to list graph-backed stats modules.");
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
  if (ImGui::Button("Open event playlist")) {
    showSingletonWindow(uiEventPlaylistOpen, eventPlaylistPlacement);
  }
  ImGui::SameLine();
  if (ImGui::Button("Open recording")) {
    showSingletonWindow(uiRecordingOpen, recordingPlacement);
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

  auto selectedCount = [](std::initializer_list<bool> values) {
    return static_cast<int>(std::count(values.begin(), values.end(), true));
  };
  const int padSelected =
      selectedCount({boostPickupPadBig, boostPickupPadSmall, boostPickupPadAmbiguous});
  const int activitySelected = selectedCount(
      {boostPickupActivityActive, boostPickupActivityInactive, boostPickupActivityUnknown});
  const int fieldSelected =
      selectedCount({boostPickupFieldOwn, boostPickupFieldOpponent, boostPickupFieldUnknown});
  const int playerSelected = boostPickupPlayerFilterEnabled
                                 ? static_cast<int>(boostPickupPlayerIds.size())
                                 : static_cast<int>(sampledPlayers.size());
  const bool pickupsHidden = padSelected == 0 || activitySelected == 0 || fieldSelected == 0 ||
                             (boostPickupPlayerFilterEnabled && playerSelected == 0);
  const int constrainedGroups =
      (padSelected < 3 ? 1 : 0) + (activitySelected < 3 ? 1 : 0) +
      (fieldSelected < 3 ? 1 : 0) +
      (boostPickupPlayerFilterEnabled &&
               playerSelected < static_cast<int>(sampledPlayers.size())
           ? 1
           : 0);
  const std::string pickupReadout =
      pickupsHidden ? "Hidden"
                    : constrainedGroups == 0 ? "All labels"
                                             : std::format("{} filters", constrainedGroups);
  const float pickupReadoutWidth = ImGui::CalcTextSize(pickupReadout.c_str()).x;
  ImGui::SetCursorPosX(std::max(
      ImGui::GetCursorPosX(),
      ImGui::GetWindowContentRegionMax().x - pickupReadoutWidth));
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "%s", pickupReadout.c_str());

  auto renderBoostFilterGroupTitle = [](const char *title) {
    ImGui::TextColored(ImVec4{0.84f, 0.90f, 0.94f, 1.0f}, "%s", title);
  };
  auto renderBoostFilterCheckbox = [&](const char *label, bool &value, bool sameLine) {
    if (sameLine) {
      ImGui::SameLine();
    }
    if (ImGui::Checkbox(label, &value)) {
      scheduleUiConfigAutosave();
    }
  };

  ImGui::Columns(2, "boost-pickup-filter-grid", false);
  renderBoostFilterGroupTitle("Pad type");
  renderBoostFilterCheckbox("Big pads", boostPickupPadBig, false);
  renderBoostFilterCheckbox("Small pads", boostPickupPadSmall, true);
  renderBoostFilterCheckbox("Ambiguous pads", boostPickupPadAmbiguous, false);
  ImGui::NextColumn();
  renderBoostFilterGroupTitle("Activity");
  renderBoostFilterCheckbox("Active play", boostPickupActivityActive, false);
  renderBoostFilterCheckbox("Inactive play", boostPickupActivityInactive, true);
  renderBoostFilterCheckbox("Unknown activity", boostPickupActivityUnknown, false);
  ImGui::NextColumn();
  renderBoostFilterGroupTitle("Field half");
  renderBoostFilterCheckbox("Own half", boostPickupFieldOwn, false);
  renderBoostFilterCheckbox("Opponent half", boostPickupFieldOpponent, true);
  renderBoostFilterCheckbox("Unknown half", boostPickupFieldUnknown, false);
  ImGui::Columns(1);

  if (!sampledPlayers.empty()) {
    ImGui::Spacing();
    renderBoostFilterGroupTitle("Player");
    for (const SaPlayerFrame &player : sampledPlayers) {
      const std::string playerId = webPlayerIdForIndex(player.player_index);
      bool selected =
          !boostPickupPlayerFilterEnabled || containsString(boostPickupPlayerIds, playerId);
      const auto playerName = playerNamesByIndex.find(player.player_index);
      const std::string displayName =
          playerName != playerNamesByIndex.end() && !playerName->second.empty()
              ? playerName->second
              : std::format("Player {}", player.player_index + 1);
      const std::string label = std::format(
          "{} ({})##boost-pickup-player-{}",
          displayName,
          teamLabel(player.is_team_0),
          player.player_index);
      if (ImGui::Checkbox(label.c_str(), &selected)) {
        if (!boostPickupPlayerFilterEnabled) {
          boostPickupPlayerIds.clear();
          for (const SaPlayerFrame &candidate : sampledPlayers) {
            boostPickupPlayerIds.push_back(webPlayerIdForIndex(candidate.player_index));
          }
          boostPickupPlayerFilterEnabled = true;
        }
        if (selected) {
          if (!containsString(boostPickupPlayerIds, playerId)) {
            boostPickupPlayerIds.push_back(playerId);
          }
        } else {
          boostPickupPlayerIds.erase(
              std::remove(boostPickupPlayerIds.begin(), boostPickupPlayerIds.end(), playerId),
              boostPickupPlayerIds.end());
        }
        scheduleUiConfigAutosave();
      }
    }
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
  auto renderTouchSettingsHeader = [](const char *eyebrow,
                                      const char *title,
                                      const std::string &readout) {
    ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "%s", eyebrow);
    const float readoutWidth = ImGui::CalcTextSize(readout.c_str()).x;
    const float readoutX =
        std::max(ImGui::GetCursorPosX(), ImGui::GetWindowContentRegionMax().x - readoutWidth);
    ImGui::Text("%s", title);
    ImGui::SameLine(readoutX);
    ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "%s", readout.c_str());
  };

  ImGui::PushStyleVar(ImGuiStyleVar_ChildRounding, 8.0f);
  ImGui::PushStyleVar(ImGuiStyleVar_WindowPadding, ImVec2{13.0f, 12.0f});
  ImGui::PushStyleColor(ImGuiCol_ChildBg, ImVec4{1.0f, 1.0f, 1.0f, 0.035f});
  ImGui::PushStyleColor(ImGuiCol_Border, ImVec4{1.0f, 1.0f, 1.0f, 0.08f});
  ImGui::BeginChild("touch-settings-card", ImVec2{0.0f, 0.0f}, true);

  renderTouchSettingsHeader(
      "Touch markers",
      "Touch decay",
      std::format("{:.1f}s", touchMarkerDecaySeconds));
  ImGui::TextDisabled("Keep each marker visible after the touch");
  if (ImGui::SliderFloat(
          "##touch-marker-decay-seconds", &touchMarkerDecaySeconds, 1.0f, 10.0f, "%.1fs")) {
    scheduleUiConfigAutosave();
  }

  ImGui::Separator();
  renderTouchSettingsHeader(
      "Overlay",
      "Touch mode",
      touchControlsMode == 1 ? "Advancement" : "Markers");
  if (ImGui::RadioButton("Markers##touch-mode", &touchControlsMode, 0)) {
    setCvarString("subtr_actor_overlay_event_types", "touch");
    scheduleUiConfigAutosave();
  }
  ImGui::SameLine();
  if (ImGui::RadioButton("Advancement##touch-mode", &touchControlsMode, 1)) {
    setCvarString("subtr_actor_overlay_event_types", "touch");
    scheduleUiConfigAutosave();
  }

  ImGui::Separator();
  renderTouchSettingsHeader("Stat display", "Touch breakdown", touchBreakdownReadout());
  if (ImGui::Checkbox("Kind", &touchBreakdownKind)) {
    scheduleUiConfigAutosave();
  }
  ImGui::SameLine();
  if (ImGui::Checkbox("Height", &touchBreakdownHeight)) {
    scheduleUiConfigAutosave();
  }
  if (ImGui::Checkbox("Surface", &touchBreakdownSurface)) {
    scheduleUiConfigAutosave();
  }
  ImGui::SameLine();
  if (ImGui::Checkbox("Dodge", &touchBreakdownDodge)) {
    scheduleUiConfigAutosave();
  }

  ImGui::EndChild();
  ImGui::PopStyleColor(2);
  ImGui::PopStyleVar(2);
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

void SubtrActorPlugin::applyPlaybackConfigToReplay(std::string_view sourceLabel) {
  (void)sourceLabel;
  if (!gameWrapper || !gameWrapper->IsInReplay()) {
    return;
  }

  ReplayServerWrapper replayServer = gameWrapper->GetGameEventAsReplay();
  if (replayServer.IsNull()) {
    return;
  }

  mechanicsReviewClipActive = false;
  playbackCurrentTime = std::max(0.0f, playbackCurrentTime);
  if (playbackPlaying) {
    replayServer.StartPlaybackAtTime(playbackCurrentTime);
    return;
  }

  replayServer.SkipToTime(playbackCurrentTime);
  ReplayWrapper replay = replayServer.GetReplay();
  if (!replay.IsNull()) {
    replay.StopPlayback();
  }
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

  ReplayServerWrapper replayServer = gameWrapper->GetGameEventAsReplay();
  const bool hasReplayServer = !replayServer.IsNull();
  if (hasReplayServer) {
    playbackCurrentTime = replayServer.GetReplayTimeElapsed();
  }

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
  auto playbackButton = [&](const char *label, bool disabled, float width) {
    pushPlaybackDisabledStyle(disabled);
    const bool clicked = ImGui::Button(label, ImVec2{width, 0.0f});
    popPlaybackDisabledStyle(disabled);
    return clicked && !disabled;
  };
  auto applyPlaybackState = [&](bool shouldPlay) {
    mechanicsReviewClipActive = false;
    playbackCurrentTime = std::max(0.0f, playbackCurrentTime);
    playbackPlaying = shouldPlay;
    if (!hasReplayServer) {
      scheduleUiConfigAutosave();
      return;
    }

    if (shouldPlay) {
      replayServer.StartPlaybackAtTime(playbackCurrentTime);
      scheduleUiConfigAutosave();
      return;
    }

    replayServer.SkipToTime(playbackCurrentTime);
    ReplayWrapper replay = replayServer.GetReplay();
    if (!replay.IsNull()) {
      replay.StopPlayback();
    }
    scheduleUiConfigAutosave();
  };

  playbackCurrentTime = std::max(0.0f, playbackCurrentTime);

  const float playbackTransportWidth = ImGui::GetContentRegionAvail().x;
  const float playbackTransportGap = ImGui::GetStyle().ItemSpacing.x;
  const float playbackTransportItemWidth =
      std::max(72.0f, (playbackTransportWidth - playbackTransportGap) * 0.5f);
  if (playbackButton(
          playbackPlaying ? "Pause" : "Play",
          !transportEnabled,
          playbackTransportItemWidth)) {
    applyPlaybackState(!playbackPlaying);
  }
  ImGui::SameLine(0.0f, playbackTransportGap);
  constexpr std::array<const char *, 5> playbackRateLabels{{"0.25x", "0.5x", "1.0x", "1.5x", "2.0x"}};
  constexpr std::array<float, 5> playbackRateValues{{0.25f, 0.5f, 1.0f, 1.5f, 2.0f}};
  size_t playbackRateIndex = 2;
  float playbackRateDistance = std::numeric_limits<float>::infinity();
  for (size_t index = 0; index < playbackRateValues.size(); index += 1) {
    const float distance = std::abs(playbackRate - playbackRateValues[index]);
    if (distance < playbackRateDistance) {
      playbackRateDistance = distance;
      playbackRateIndex = index;
    }
  }
  const bool playbackRateDisabled = !transportEnabled;
  pushPlaybackDisabledStyle(playbackRateDisabled);
  ImGui::SetNextItemWidth(playbackTransportItemWidth);
  const bool playbackRateOpen =
      ImGui::BeginCombo("##playback-rate", playbackRateLabels[playbackRateIndex]);
  popPlaybackDisabledStyle(playbackRateDisabled);
  if (playbackRateOpen) {
    if (playbackRateDisabled) {
      ImGui::CloseCurrentPopup();
      ImGui::EndCombo();
    } else {
      for (size_t index = 0; index < playbackRateValues.size(); index += 1) {
        const bool selected = index == playbackRateIndex;
        if (ImGui::Selectable(playbackRateLabels[index], selected)) {
          playbackRate = playbackRateValues[index];
          playbackRateIndex = index;
          scheduleUiConfigAutosave();
        }
        if (selected) {
          ImGui::SetItemDefaultFocus();
        }
      }
      ImGui::EndCombo();
    }
  }

  bool nextSkipPostGoalTransitions = playbackSkipPostGoalTransitions;
  pushPlaybackDisabledStyle(!transportEnabled);
  const bool skipGoalChanged =
      ImGui::Checkbox("Skip post-goal resets", &nextSkipPostGoalTransitions);
  popPlaybackDisabledStyle(!transportEnabled);
  if (transportEnabled && skipGoalChanged) {
    playbackSkipPostGoalTransitions = nextSkipPostGoalTransitions;
    scheduleUiConfigAutosave();
  }
  bool nextSkipKickoffs = playbackSkipKickoffs;
  pushPlaybackDisabledStyle(!transportEnabled);
  const bool skipKickoffsChanged = ImGui::Checkbox("Skip kickoff countdowns", &nextSkipKickoffs);
  popPlaybackDisabledStyle(!transportEnabled);
  if (transportEnabled && skipKickoffsChanged) {
    playbackSkipKickoffs = nextSkipKickoffs;
    scheduleUiConfigAutosave();
  }

  ImGui::Separator();
  const float durationSeconds = std::max(lastTime, playbackCurrentTime);
  ImGui::Columns(2, "playback-detail-grid", false);
  renderWebDetailGridCell("Time", std::format("{:.2f}s", playbackCurrentTime));
  ImGui::NextColumn();
  renderWebDetailGridCell(
      "Frame",
      std::format("{}", static_cast<unsigned long long>(frameNumber)));
  ImGui::NextColumn();
  renderWebDetailGridCell("Duration", std::format("{:.2f}s", durationSeconds));
  ImGui::NextColumn();
  renderWebDetailGridCell(
      "Status",
      playbackPlaying ? "Playing" : transportEnabled ? "Paused" : "Stopped");
  ImGui::Columns(1);

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
  auto recordingButton = [](const char *label, bool disabled, float width) {
    if (disabled) {
      ImGui::PushStyleVar(ImGuiStyleVar_Alpha, ImGui::GetStyle().Alpha * 0.45f);
    }
    const bool clicked = ImGui::Button(label, ImVec2{width, 0.0f});
    if (disabled) {
      ImGui::PopStyleVar();
    }
    return clicked && !disabled;
  };

  const float recordingControlGap = ImGui::GetStyle().ItemSpacing.x;
  const float recordingControlWidth =
      std::max(96.0f, (ImGui::GetContentRegionAvail().x - recordingControlGap) * 0.5f);
  int nextRecordingFps = recordingFps;
  ImGui::BeginGroup();
  ImGui::TextDisabled("FPS");
  pushRecordingDisabledStyle(recordingSettingsLocked);
  ImGui::SetNextItemWidth(recordingControlWidth);
  const bool fpsChanged =
      ImGui::InputInt(
          "##recording-fps",
          &nextRecordingFps,
          1,
          10,
          recordingSettingsLocked ? ImGuiInputTextFlags_ReadOnly : 0);
  popRecordingDisabledStyle(recordingSettingsLocked);
  ImGui::EndGroup();
  if (!recordingSettingsLocked && fpsChanged) {
    recordingFps = std::clamp(nextRecordingFps, 1, 120);
    scheduleUiConfigAutosave();
  }
  ImGui::SameLine(0.0f, recordingControlGap);
  ImGui::BeginGroup();
  ImGui::TextDisabled("Playback rate");
  const std::array<const char *, 4> rates{{"0.5x", "1.0x", "1.5x", "2.0x"}};
  recordingPlaybackRateIndex = std::clamp(recordingPlaybackRateIndex, 0, 3);
  int nextRecordingPlaybackRateIndex = recordingPlaybackRateIndex;
  const bool recordingPlaybackRateDisabled = recordingSettingsLocked;
  pushRecordingDisabledStyle(recordingPlaybackRateDisabled);
  ImGui::SetNextItemWidth(recordingControlWidth);
  const bool recordingPlaybackRateOpen = ImGui::BeginCombo(
      "##recording-playback-rate",
      rates[static_cast<size_t>(recordingPlaybackRateIndex)]);
  popRecordingDisabledStyle(recordingPlaybackRateDisabled);
  if (recordingPlaybackRateOpen) {
    if (recordingPlaybackRateDisabled) {
      ImGui::CloseCurrentPopup();
      ImGui::EndCombo();
    } else {
      for (int index = 0; index < static_cast<int>(rates.size()); index += 1) {
        const bool selected = index == recordingPlaybackRateIndex;
        if (ImGui::Selectable(rates[static_cast<size_t>(index)], selected)) {
          nextRecordingPlaybackRateIndex = index;
        }
        if (selected) {
          ImGui::SetItemDefaultFocus();
        }
      }
      ImGui::EndCombo();
    }
  }
  if (!recordingPlaybackRateDisabled &&
      nextRecordingPlaybackRateIndex != recordingPlaybackRateIndex) {
    recordingPlaybackRateIndex = nextRecordingPlaybackRateIndex;
    scheduleUiConfigAutosave();
  }
  ImGui::EndGroup();
  ImGui::Spacing();

  const float recordingPrimaryRowWidth = ImGui::GetContentRegionAvail().x;
  const float recordingPrimaryButtonWidth =
      std::max(68.0f, (recordingPrimaryRowWidth - recordingControlGap * 2.0f) / 3.0f);
  if (recordingButton(
          "Start",
          recordingActive || !loaded || !engine,
          recordingPrimaryButtonWidth)) {
    recordingActive = true;
    recordingStartedAt = std::chrono::steady_clock::now();
    recordingStatus = "Recording analysis snapshots";
  }
  ImGui::SameLine(0.0f, recordingControlGap);
  if (recordingButton(
          "Full replay",
          recordingActive || !loaded || !engine,
          recordingPrimaryButtonWidth)) {
    recordingActive = false;
    dumpSnapshot(true);
  }
  ImGui::SameLine(0.0f, recordingControlGap);
  if (recordingButton("Stop", !recordingActive, recordingPrimaryButtonWidth)) {
    recordingActive = false;
    dumpSnapshot(false);
  }
  const float recordingSecondaryRowWidth = recordingPrimaryRowWidth;
  const float recordingSecondaryButtonWidth =
      std::max(88.0f, (recordingSecondaryRowWidth - recordingControlGap) * 0.5f);
  if (recordingButton(
          "Download",
          recordingActive || !hasGraphSnapshot,
          recordingSecondaryButtonWidth)) {
    cvarManager->log(std::format(
        "subtr-actor: recording snapshots are written to {}",
        outputDirectory.string()));
    recordingStatus = "Snapshot location logged";
  }
  ImGui::SameLine(0.0f, recordingControlGap);
  if (recordingButton(
          "Clear",
          recordingActive || !hasGraphSnapshot,
          recordingSecondaryButtonWidth)) {
    recordingActive = false;
    recordingSnapshotCount = 0;
    recordingLastBytes = 0;
    recordingStatus = "Idle";
  }

  ImGui::Separator();
  const std::string recordingStatusReadout =
      recordingActive   ? "Recording"
      : hasGraphSnapshot ? "Ready"
      : !loaded || !engine ? "No replay"
                           : recordingStatus;
  ImGui::Columns(2, "recording-detail-grid", false);
  renderWebDetailGridCell("Status", recordingStatusReadout);
  ImGui::NextColumn();
  renderWebDetailGridCell("Elapsed", std::format("{:.1f}s", elapsedSeconds));
  ImGui::NextColumn();
  renderWebDetailGridCell("Size", formatByteSize(recordingLastBytes));
  ImGui::NextColumn();
  renderWebDetailGridCell("Type", "JSON snapshots");
  ImGui::Columns(1);

  ImGui::End();
}
