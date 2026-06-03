// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
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

  if (ImGui::RadioButton("Outputs##graph-inspector-view", &graphInspectorView, 0)) {
    scheduleUiConfigAutosave();
  }
  ImGui::SameLine();
  if (ImGui::RadioButton("Analysis nodes##graph-inspector-view", &graphInspectorView, 1)) {
    scheduleUiConfigAutosave();
  }
  ImGui::SameLine();
  if (ImGui::RadioButton("Graph info##graph-inspector-view", &graphInspectorView, 2)) {
    scheduleUiConfigAutosave();
  }
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
          scheduleUiConfigAutosave();
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
      scheduleUiConfigAutosave();
    }
    if (!graphInspectorNodeQuery.empty()) {
      ImGui::SameLine();
      if (ImGui::SmallButton("Clear##graph-node-search")) {
        graphInspectorNodeQuery.clear();
        scheduleUiConfigAutosave();
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
        scheduleUiConfigAutosave();
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
  const float eventPlaylistX = rightAnchoredUiX(432.0f);
  const float statusX = rightAnchoredUiX(330.0f);
  const float playbackX = rightAnchoredUiX(336.0f, 32.0f);
  const float recordingX = rightAnchoredUiX(416.0f, 32.0f);
  const float mechanicsReviewX = rightAnchoredUiX(480.0f);
  const float replayLoadingX = rightAnchoredUiX(512.0f);
  const float moduleControlsX = rightAnchoredUiX(430.0f);
  const float touchControlsX = rightAnchoredUiX(384.0f);

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
       432.0f,
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
       false,
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
       false,
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
       false,
       6,
       &uiMechanicsReviewOpen,
       &mechanicsReviewPlacement,
       mechanicsReviewX,
       256.0f,
       480.0f,
       560.0f},
      {"Replay loading",
       "replay-loading",
       "replay_loading_open",
       "replay_loading",
       false,
       7,
       &uiReplayLoadingOpen,
       &replayLoadingPlacement,
       replayLoadingX,
       68.0f,
       512.0f,
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
       384.0f,
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
      hideSingletonWindow(*window.open);
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
        hideSingletonWindow(*window.open);
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
      hideStatsWindow(window);
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
    scheduleUiConfigAutosave();
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
        hideStatsWindow(window);
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
    scheduleUiConfigAutosave();
  }
  if (duplicateWindow) {
    uiStatsWindows.push_back(std::move(*duplicateWindow));
    scheduleUiConfigAutosave();
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
  launcherPlacement = {};
  launcherPlacement.pending_focus = uiLauncherOpen;
  auto resetSingletonPlacement = [&](const SingletonWindowControl &window) {
    if (window.placement == &scoreboardPlacement) {
      resetScoreboardWindowPlacement();
      return;
    }
    resetSingletonWindowPlacement(
        *window.placement,
        window.x,
        window.y,
        window.width,
        window.height);
  };
  for (const SingletonWindowControl &window : webSingletonWindowControls()) {
    resetSingletonPlacement(window);
  }
  for (const SingletonWindowControl &window : singletonWindowControls()) {
    if (window.web_config) {
      continue;
    }
    resetSingletonPlacement(window);
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
  scheduleUiConfigAutosave();
}

void SubtrActorPlugin::applyWorkspaceWindowVisibility(
    bool launcherOpen,
    std::initializer_list<std::string_view> openWindowIds) {
  uiLauncherOpen = launcherOpen;
  for (const SingletonWindowControl &window : singletonWindowControls()) {
    *window.open = std::find(openWindowIds.begin(), openWindowIds.end(), window.config_id) !=
                   openWindowIds.end();
  }
  scheduleUiConfigAutosave();
}

void SubtrActorPlugin::applyDefaultUiWorkspace() {
  applyWorkspaceWindowVisibility(false, {"scoreboard"});
  resetWindowPlacements();
}

void SubtrActorPlugin::applyReplayReviewUiWorkspace() {
  applyWorkspaceWindowVisibility(
      true,
      {"scoreboard",
       "mechanics",
       "event-playlist",
       "touch-controls",
       "boost-pickups"});
  eventPlaylistMechanicsEnabled = true;
  eventPlaylistTeamEventsEnabled = true;
  eventPlaylistGoalContextEnabled = true;
  eventPlaylistAutoFollow = true;
  eventPlaylistSourceFilter = "default";
  eventPlaylistLastActiveKey.clear();
  resetWindowPlacements();
  eventPlaylistPlacement.pending_focus = true;
}

void SubtrActorPlugin::applyGraphDebugUiWorkspace() {
  applyWorkspaceWindowVisibility(
      true,
      {"mechanics",
       "event-playlist",
       "status",
       "graph-inspector",
       "module-controls"});
  resetWindowPlacements();
  graphInspectorPlacement.pending_focus = true;
  moduleControlsPlacement.pending_focus = true;
}

void SubtrActorPlugin::applyRecordingUiWorkspace() {
  applyWorkspaceWindowVisibility(true, {"scoreboard", "status", "recording"});
  resetWindowPlacements();
  recordingPlacement.pending_focus = true;
}
