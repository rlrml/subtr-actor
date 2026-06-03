// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
void SubtrActorPlugin::renderLauncherToggleChrome() {
  uiLauncherToggleHovered = false;
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

  const bool launcherToggleClicked =
      ImGui::Button("##subtr-actor-launcher-toggle", ImVec2{42.0f, 42.0f});
  const ImVec2 toggleMin = ImGui::GetItemRectMin();
  const ImVec2 toggleMax = ImGui::GetItemRectMax();
  const float toggleCenterX = (toggleMin.x + toggleMax.x) * 0.5f;
  const float toggleCenterY = (toggleMin.y + toggleMax.y) * 0.5f;
  ImDrawList *drawList = ImGui::GetWindowDrawList();
  const ImU32 barColor = ImGui::GetColorU32(ImVec4{0.93f, 0.96f, 0.98f, 1.0f});
  for (const float yOffset : {-6.0f, 0.0f, 6.0f}) {
    drawList->AddLine(
        ImVec2{toggleCenterX - 9.5f, toggleCenterY + yOffset},
        ImVec2{toggleCenterX + 9.5f, toggleCenterY + yOffset},
        barColor,
        2.0f);
  }

  if (launcherToggleClicked) {
    uiWindowOpen = true;
    if (uiLauncherOpen) {
      hideLauncherWindow();
    } else {
      showLauncherWindow();
    }
  }

  if (uiLauncherOpen) {
    ImGui::PopStyleColor(3);
  }
  uiLauncherToggleHovered = ImGui::IsWindowHovered(ImGuiHoveredFlags_RootAndChildWindows);
  ImGui::End();
  ImGui::PopStyleVar(2);
}

void SubtrActorPlugin::applyLauncherMenuPlacement() {
  const ImVec2 displaySize = ImGui::GetIO().DisplaySize;
  const float width = 352.0f;
  const float maxHeight = std::max(320.0f, displaySize.y > 0.0f ? displaySize.y - 120.0f : 650.0f);
  ImGui::SetNextWindowPos(ImVec2{12.0f, 64.0f}, ImGuiCond_Always);
  ImGui::SetNextWindowSizeConstraints(ImVec2{width, 0.0f}, ImVec2{width, maxHeight});
  ImGui::SetNextWindowSize(ImVec2{width, 0.0f}, ImGuiCond_Always);
  ImGui::SetNextWindowBgAlpha(0.92f);
  if (launcherPlacement.pending_focus) {
    ImGui::SetNextWindowFocus();
    launcherPlacement.z_index = nextUiWindowZIndex++;
    launcherPlacement.pending_focus = false;
  }
}

void SubtrActorPlugin::renderLauncherWorkspaceControls() {
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "WORKSPACES");
  const float workspaceButtonWidth = ImGui::GetContentRegionAvail().x;
  if (ImGui::Button("Default workspace", ImVec2{workspaceButtonWidth, 0.0f})) {
    applyDefaultUiWorkspace();
  }
  if (ImGui::Button("Review workspace", ImVec2{workspaceButtonWidth, 0.0f})) {
    applyReplayReviewUiWorkspace();
  }
  if (ImGui::Button("Debug workspace", ImVec2{workspaceButtonWidth, 0.0f})) {
    applyGraphDebugUiWorkspace();
  }
  if (ImGui::Button("Recording workspace", ImVec2{workspaceButtonWidth, 0.0f})) {
    applyRecordingUiWorkspace();
  }
  if (ImGui::Button("Reset positions", ImVec2{workspaceButtonWidth, 0.0f})) {
    resetWindowPlacements();
  }
  if (ImGui::Button("Default stats windows", ImVec2{workspaceButtonWidth, 0.0f})) {
    resetDefaultStatsWindows();
  }
  if (ImGui::Button("Hide side windows", ImVec2{workspaceButtonWidth, 0.0f})) {
    for (const SingletonWindowControl &window : singletonWindowControls()) {
      if (std::string_view{window.config_id} == "scoreboard") {
        continue;
      }
      hideSingletonWindow(*window.open);
    }
  }
  renderLayoutConfigControls("launcher-workspace-layout", true);
}

void SubtrActorPlugin::renderWebWindowToggleControls(
    const char *idSuffix,
    bool closeLauncherOnToggle,
    bool includeState,
    bool fullWidth) {
  ImGui::PushID(idSuffix);
  for (const SingletonWindowControl &window : webSingletonWindowControls()) {
    ImGui::PushID(window.label);
    const bool isOpen = window.open != nullptr && *window.open;
    if (includeState && isOpen) {
      pushWebModuleSummaryButtonStyle(true);
    }
    const std::string buttonLabel{window.label};
    const float buttonWidth = fullWidth ? ImGui::GetContentRegionAvail().x : 210.0f;
    if (ImGui::Button(buttonLabel.c_str(), ImVec2{buttonWidth, 0.0f})) {
      if (*window.open) {
        hideSingletonWindow(*window.open);
      } else {
        showSingletonWindow(*window.open, *window.placement);
      }
      if (closeLauncherOnToggle) {
        hideLauncherWindow();
      }
    }
    if (includeState && isOpen) {
      popWebModuleSummaryButtonStyle();
    }
    ImGui::PopID();
  }
  ImGui::PopID();
}

void SubtrActorPlugin::renderStatsWindowCreationControls(
    const char *idSuffix,
    bool closeLauncherOnCreate,
    bool includeHeading,
    bool includeManager,
    bool fullWidth) {
  ImGui::PushID(idSuffix);
  if (includeHeading) {
    ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "STATS WINDOWS");
  }
  for (const StatsWindowKindControl &kind : statsWindowKindControls()) {
    if (!kind.web_config) {
      continue;
    }
    const float buttonWidth = fullWidth ? ImGui::GetContentRegionAvail().x : 170.0f;
    if (ImGui::Button(kind.create_label, ImVec2{buttonWidth, 0.0f})) {
      createStatsWindow(kind.kind);
      if (closeLauncherOnCreate) {
        hideLauncherWindow();
      }
    }
  }
  if (!includeManager || uiStatsWindows.empty()) {
    ImGui::PopID();
    return;
  }

  const size_t visibleStatsWindows = static_cast<size_t>(std::count_if(
      uiStatsWindows.begin(),
      uiStatsWindows.end(),
      [](const UiStatsWindow &window) { return window.open; }));
  ImGui::Text(
      "%zu visible / %zu stats windows",
      visibleStatsWindows,
      uiStatsWindows.size());
  renderStatsWindowManager();
  ImGui::PopID();
}

void SubtrActorPlugin::renderSettingsWindowControls() {
  ImGui::Separator();
  ImGui::Text("Windows");
  renderWebWindowToggleControls("settings-web-windows", false);
  renderSingletonWindowManager();

  ImGui::Separator();
  renderStatsWindowCreationControls("settings-stats-windows", false);
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
  const bool launcherHovered = ImGui::IsWindowHovered(ImGuiHoveredFlags_RootAndChildWindows);

  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "WINDOWS");
  renderWebWindowToggleControls("launcher-web-windows", true, false, true);
  renderStatsWindowCreationControls("launcher-stats-windows", true, false, false, true);

  ImGui::Separator();
  renderModuleSummaryControls("launcher-module-summary", false, 0.0f, false);

  if (timelineRangePossessionEnabled) {
    ImGui::Separator();
    renderModuleSettingsControls("launcher-module-settings", false, true, true);
  }

  if (uiLauncherOpen && ImGui::IsMouseClicked(ImGuiMouseButton_Left) && !launcherHovered &&
      !uiLauncherToggleHovered) {
    hideLauncherWindow();
  }

  ImGui::End();
}

void SubtrActorPlugin::renderEmptyStateWindow() {
  if (uiLauncherOpen || uiReplayLoadingOpen || liveProcessingEnabled() || replayAnnotations ||
      !sampledPlayers.empty()) {
    return;
  }

  const ImVec2 displaySize = ImGui::GetIO().DisplaySize;
  ImGui::SetNextWindowPos(
      ImVec2{displaySize.x * 0.5f, displaySize.y * 0.5f},
      ImGuiCond_Always,
      ImVec2{0.5f, 0.5f});
  ImGui::SetNextWindowBgAlpha(0.88f);
  constexpr ImGuiWindowFlags emptyStateFlags =
      ImGuiWindowFlags_NoTitleBar | ImGuiWindowFlags_AlwaysAutoResize |
      ImGuiWindowFlags_NoCollapse | ImGuiWindowFlags_NoSavedSettings;
  ImGui::PushStyleVar(ImGuiStyleVar_WindowPadding, ImVec2{18.0f, 14.0f});
  ImGui::PushStyleVar(ImGuiStyleVar_WindowRounding, 8.0f);
  if (!ImGui::Begin("subtr-actor empty state##subtr-actor", nullptr, emptyStateFlags)) {
    ImGui::End();
    ImGui::PopStyleVar(2);
    return;
  }

  ImGui::TextUnformatted("Open a replay in Rocket League to start.");
  if (gameWrapper->IsInReplay() && ImGui::Button("Refresh current replay", ImVec2{190.0f, 0.0f})) {
    resetReplayAnnotations();
    tickReplayAnnotations();
  }

  ImGui::End();
  ImGui::PopStyleVar(2);
}

std::optional<std::pair<int32_t, int32_t>> SubtrActorPlugin::currentScoreboardScore() const {
  if (replayAnnotations && replayAnnotationScoreAtTime && gameWrapper->IsInReplay()) {
    ReplayServerWrapper replayServer = gameWrapper->GetGameEventAsReplay();
    const float replayTime =
        replayServer.IsNull() ? playbackCurrentTime : replayServer.GetReplayTimeElapsed();
    SaReplayScore score{};
    if (replayAnnotationScoreAtTime(replayAnnotations, replayTime, &score) == 0 &&
        score.has_team_zero_score != 0 && score.has_team_one_score != 0) {
      return std::make_pair(score.team_zero_score, score.team_one_score);
    }
  }
  return lastTeamScores;
}

void SubtrActorPlugin::renderScoreboardWindow() {
  if (!uiScoreboardOpen) {
    return;
  }
  applyScoreboardWindowPlacement();
  constexpr ImGuiWindowFlags scoreboardFlags =
      ImGuiWindowFlags_NoTitleBar | ImGuiWindowFlags_AlwaysAutoResize |
      ImGuiWindowFlags_NoScrollbar | ImGuiWindowFlags_NoCollapse |
      ImGuiWindowFlags_NoSavedSettings;
  ImGui::PushStyleVar(ImGuiStyleVar_WindowPadding, ImVec2{10.0f, 6.0f});
  ImGui::PushStyleVar(ImGuiStyleVar_WindowRounding, 999.0f);
  ImGui::PushStyleVar(ImGuiStyleVar_WindowBorderSize, 1.0f);
  ImGui::PushStyleColor(ImGuiCol_WindowBg, ImVec4{0.03f, 0.07f, 0.10f, 0.88f});
  ImGui::PushStyleColor(ImGuiCol_Border, ImVec4{1.0f, 1.0f, 1.0f, 0.12f});
  if (!ImGui::Begin("Scoreboard##subtr-actor", &uiScoreboardOpen, scoreboardFlags)) {
    ImGui::End();
    ImGui::PopStyleColor(2);
    ImGui::PopStyleVar(3);
    return;
  }
  captureWindowPlacement(scoreboardPlacement);

  const auto score = currentScoreboardScore();
  if (score) {
    ImGui::PushStyleVar(ImGuiStyleVar_ItemSpacing, ImVec2{8.0f, 0.0f});
    ImGui::SetWindowFontScale(1.2f);
    ImGui::TextColored(ImVec4{0.31f, 0.75f, 1.0f, 1.0f}, "%d", score->first);
    ImGui::SameLine();
    ImGui::TextDisabled("-");
    ImGui::SameLine();
    ImGui::TextColored(ImVec4{1.0f, 0.69f, 0.31f, 1.0f}, "%d", score->second);
    ImGui::SetWindowFontScale(1.0f);
    ImGui::PopStyleVar();
  } else {
    ImGui::TextDisabled("Load a replay to show the scoreboard.");
  }
  ImGui::End();
  ImGui::PopStyleColor(2);
  ImGui::PopStyleVar(3);
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

  renderEventSourceControls();
  ImGui::End();
}

void SubtrActorPlugin::renderEventSourceControls() {
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
  std::sort(
      displaySources.begin(),
      displaySources.end(),
      [](const DisplaySource &left, const DisplaySource &right) {
        return std::string_view{left.option->label} < std::string_view{right.option->label};
      });

  if (displaySources.empty()) {
    ImGui::TextDisabled("No events loaded.");
    return;
  }

  auto renderEventSourceAction = [](const std::string &label) {
    pushWebModuleSummaryButtonStyle(false);
    const bool clicked = ImGui::Button(label.c_str());
    popWebModuleSummaryButtonStyle();
    const float itemRight = ImGui::GetItemRectMax().x;
    const float contentRight = ImGui::GetWindowPos().x + ImGui::GetWindowContentRegionMax().x;
    if (contentRight - itemRight > 112.0f) {
      ImGui::SameLine();
    }
    return clicked;
  };
  auto renderEventSourceRow = [](const EventFilterOption &option, bool enabled, size_t count) {
    const std::string label = std::format(
        "{}   {} {}##event-sources-{}",
        option.label,
        enabled ? "On" : "Off",
        count,
        option.value);
    pushWebModuleSummaryButtonStyle(enabled);
    const bool clicked = ImGui::Button(label.c_str());
    popWebModuleSummaryButtonStyle();
    return clicked;
  };

  if (renderEventSourceAction(
          std::format("All events   {}##event-sources-actions-all", displaySources.size()))) {
    selected.clear();
    selected.reserve(displaySources.size());
    for (const DisplaySource &source : displaySources) {
      selected.emplace_back(source.option->value);
    }
    applySelection();
  }
  if (renderEventSourceAction("No events   Off##event-sources-actions-none")) {
    selected.clear();
    applySelection();
  }

  ImGui::Separator();
  ImGui::BeginChild("event-source-list", ImVec2{0.0f, 0.0f}, true);

  for (const DisplaySource &source : displaySources) {
    const EventFilterOption &option = *source.option;
    ImGui::PushID(option.value);
    if (renderEventSourceRow(option, source.enabled, source.count)) {
      if (source.enabled) {
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
}
