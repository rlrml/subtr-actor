// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
void SubtrActorPlugin::Render() {
  if (imguiContext != 0) {
    ImGui::SetCurrentContext(reinterpret_cast<ImGuiContext *>(imguiContext));
  }

  if (!uiEnabled()) {
    renderLauncherToggleChrome();
    renderLauncherWindow();
    maybeAutosaveUiConfig();
    return;
  }

  renderEmptyStateWindow();
  renderFloatingWindowLayer();
  renderLauncherToggleChrome();
  renderLauncherWindow();
  maybeAutosaveUiConfig();
}

void SubtrActorPlugin::renderFloatingWindowLayer() {
  enum class RenderEntryKind {
    Singleton,
    Stats,
  };
  struct RenderEntry {
    RenderEntryKind kind;
    std::string_view singleton_id;
    size_t stats_index = 0;
    int z_index = 0;
    int fallback_order = 0;
  };

  auto renderWithFloatingWindowStyle = [](auto &&renderWindow) {
    pushWebFloatingWindowStyle();
    renderWindow();
    popWebFloatingWindowStyle();
  };

  std::vector<RenderEntry> renderOrder;
  for (const SingletonWindowControl &window : singletonWindowControls()) {
    if (*window.open) {
      renderOrder.push_back(RenderEntry{
          RenderEntryKind::Singleton,
          std::string_view{window.config_id},
          0,
          window.placement->z_index,
          window.launcher_order});
    }
  }
  for (size_t index = 0; index < uiStatsWindows.size(); index += 1) {
    const UiStatsWindow &window = uiStatsWindows[index];
    if (window.open) {
      renderOrder.push_back(RenderEntry{
          RenderEntryKind::Stats,
          {},
          index,
          window.z_index,
          static_cast<int>(1000 + index)});
    }
  }
  std::stable_sort(
      renderOrder.begin(),
      renderOrder.end(),
      [](const RenderEntry &left, const RenderEntry &right) {
        if (left.z_index != right.z_index) {
          return left.z_index < right.z_index;
        }
        return left.fallback_order < right.fallback_order;
      });

  for (const RenderEntry &entry : renderOrder) {
    if (entry.kind == RenderEntryKind::Stats) {
      renderWithFloatingWindowStyle([&]() {
        renderStatsWindow(uiStatsWindows[entry.stats_index], entry.stats_index);
      });
      continue;
    }

    const std::string_view id = entry.singleton_id;
    if (id == "scoreboard") {
      renderScoreboardWindow();
    } else if (id == "mechanics") {
      renderWithFloatingWindowStyle([&]() { renderEventsWindow(); });
    } else if (id == "event-playlist") {
      renderWithFloatingWindowStyle([&]() { renderEventPlaylistWindow(); });
    } else if (id == "status") {
      renderWithFloatingWindowStyle([&]() { renderStatusWindow(); });
    } else if (id == "playback") {
      renderWithFloatingWindowStyle([&]() { renderPlaybackControlsWindow(); });
    } else if (id == "recording") {
      renderWithFloatingWindowStyle([&]() { renderRecordingWindow(); });
    } else if (id == "graph-inspector") {
      renderWithFloatingWindowStyle([&]() { renderGraphInspectorWindow(); });
    } else if (id == "mechanics-review") {
      renderWithFloatingWindowStyle([&]() { renderMechanicsReviewWindow(); });
    } else if (id == "replay-loading") {
      renderWithFloatingWindowStyle([&]() { renderReplayLoadingWindow(); });
    } else if (id == "module-controls") {
      renderWithFloatingWindowStyle([&]() { renderModuleControlsWindow(); });
    } else if (id == "touch-controls") {
      renderWithFloatingWindowStyle([&]() { renderTouchControlsWindow(); });
    } else if (id == "boost-pickups") {
      renderWithFloatingWindowStyle([&]() { renderBoostPickupControlsWindow(); });
    }
  }
}

void SubtrActorPlugin::RenderSettings() {
  if (imguiContext != 0) {
    ImGui::SetCurrentContext(reinterpret_cast<ImGuiContext *>(imguiContext));
  }
  renderSharedSettingsControls();
  renderSettingsWindowControls();
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

  ImGui::Separator();
  ImGui::Text("Layout");
  renderLayoutConfigControls("settings-layout");
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
  scheduleUiConfigAutosave();
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
  scheduleUiConfigAutosave();
}

void SubtrActorPlugin::showSingletonWindow(bool &open, UiWindowPlacement &placement) {
  open = true;
  focusSingletonWindow(placement);
}

void SubtrActorPlugin::hideSingletonWindow(bool &open) {
  if (!open) {
    return;
  }
  open = false;
  scheduleUiConfigAutosave();
}

void SubtrActorPlugin::hideStatsWindow(UiStatsWindow &window) {
  if (!window.open) {
    return;
  }
  window.open = false;
  scheduleUiConfigAutosave();
}

void SubtrActorPlugin::showLauncherWindow() {
  uiWindowOpen = true;
  uiLauncherOpen = true;
  launcherPlacement.pending_focus = true;
}

void SubtrActorPlugin::hideLauncherWindow() {
  if (!uiLauncherOpen) {
    return;
  }
  uiLauncherOpen = false;
}

void SubtrActorPlugin::captureWindowPlacement(UiWindowPlacement &placement) {
  const bool hadPlacement = placement.has_placement;
  const float previousX = placement.x;
  const float previousY = placement.y;
  const float previousWidth = placement.width;
  const float previousHeight = placement.height;
  const float previousViewportWidth = placement.viewport_width;
  const float previousViewportHeight = placement.viewport_height;
  const int previousZIndex = placement.z_index;
  const ImVec2 position = ImGui::GetWindowPos();
  const ImVec2 size = ImGui::GetWindowSize();
  const ImVec2 displaySize = ImGui::GetIO().DisplaySize;
  const ImVec2 clampedPosition =
      mapWindowPositionToViewport(position.x, position.y, size.x, size.y, 0.0f, 0.0f);
  if (clampedPosition.x != position.x || clampedPosition.y != position.y) {
    ImGui::SetWindowPos(clampedPosition, ImGuiCond_Always);
  }
  placement.has_placement = true;
  placement.x = clampedPosition.x;
  placement.y = clampedPosition.y;
  placement.width = size.x;
  placement.height = size.y;
  placement.viewport_width = displaySize.x;
  placement.viewport_height = displaySize.y;
  if (ImGui::IsWindowHovered(ImGuiHoveredFlags_RootAndChildWindows) &&
      ImGui::IsMouseClicked(ImGuiMouseButton_Left) && !ImGui::IsAnyItemActive()) {
    placement.z_index = nextUiWindowZIndex++;
  }
  if (!hadPlacement || previousX != placement.x || previousY != placement.y ||
      previousWidth != placement.width || previousHeight != placement.height ||
      previousViewportWidth != placement.viewport_width ||
      previousViewportHeight != placement.viewport_height ||
      previousZIndex != placement.z_index) {
    scheduleUiConfigAutosave();
  }
}

bool SubtrActorPlugin::renderSingletonWindowHeader(const char *label, bool &open) {
  const std::string headerLabel = uppercaseHeaderLabel(label);
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "%s", headerLabel.c_str());
  ImGui::SameLine();
  const std::string hideLabel = std::format("Hide##singleton-window-hide-{}", label);
  const float hideWidth =
      ImGui::CalcTextSize("Hide").x + ImGui::GetStyle().FramePadding.x * 2.0f;
  const float rightAlignedX = ImGui::GetWindowContentRegionMax().x - hideWidth;
  if (rightAlignedX > ImGui::GetCursorPosX()) {
    ImGui::SetCursorPosX(rightAlignedX);
  }
  ImGui::PushStyleVar(ImGuiStyleVar_FrameRounding, 6.0f);
  const bool hideClicked = ImGui::Button(hideLabel.c_str());
  ImGui::PopStyleVar();
  if (hideClicked) {
    hideSingletonWindow(open);
    return true;
  }
  ImGui::Separator();
  return false;
}

void SubtrActorPlugin::applyScoreboardWindowPlacement() {
  auto applyFocus = [&]() {
    if (scoreboardPlacement.pending_focus) {
      ImGui::SetNextWindowFocus();
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
  const auto [width, height] = defaultStatsWindowSize(window.kind);
  const auto [x, y] = defaultStatsWindowPosition(window.id - 1);
  ImGui::SetNextWindowPos(ImVec2{x, y}, ImGuiCond_FirstUseEver);
  ImGui::SetNextWindowSize(ImVec2{width, height}, ImGuiCond_FirstUseEver);
}

void SubtrActorPlugin::captureStatsWindowPlacement(UiStatsWindow &window) {
  const bool hadPlacement = window.has_placement;
  const float previousX = window.x;
  const float previousY = window.y;
  const float previousWidth = window.width;
  const float previousHeight = window.height;
  const float previousViewportWidth = window.viewport_width;
  const float previousViewportHeight = window.viewport_height;
  const int previousZIndex = window.z_index;
  const ImVec2 position = ImGui::GetWindowPos();
  const ImVec2 size = ImGui::GetWindowSize();
  const ImVec2 displaySize = ImGui::GetIO().DisplaySize;
  const ImVec2 clampedPosition =
      mapWindowPositionToViewport(position.x, position.y, size.x, size.y, 0.0f, 0.0f);
  if (clampedPosition.x != position.x || clampedPosition.y != position.y) {
    ImGui::SetWindowPos(clampedPosition, ImGuiCond_Always);
  }
  window.has_placement = true;
  window.x = clampedPosition.x;
  window.y = clampedPosition.y;
  window.width = size.x;
  window.height = size.y;
  window.viewport_width = displaySize.x;
  window.viewport_height = displaySize.y;
  if (ImGui::IsWindowHovered(ImGuiHoveredFlags_RootAndChildWindows) &&
      ImGui::IsMouseClicked(ImGuiMouseButton_Left) && !ImGui::IsAnyItemActive()) {
    window.z_index = nextUiWindowZIndex++;
  }
  if (!hadPlacement || previousX != window.x || previousY != window.y ||
      previousWidth != window.width || previousHeight != window.height ||
      previousViewportWidth != window.viewport_width ||
      previousViewportHeight != window.viewport_height ||
      previousZIndex != window.z_index) {
    scheduleUiConfigAutosave();
  }
}

void SubtrActorPlugin::focusStatsWindow(UiStatsWindow &window) {
  window.pending_focus = true;
  window.z_index = nextUiWindowZIndex++;
  scheduleUiConfigAutosave();
}

void SubtrActorPlugin::showStatsWindow(UiStatsWindow &window) {
  window.open = true;
  focusStatsWindow(window);
}

bool SubtrActorPlugin::renderModuleSummaryToggle(
    const char *label,
    bool active,
    const char *idSuffix,
    float width) {
  const std::string buttonLabel =
      std::format("{}   {}##{}-{}", label, active ? "On" : "Off", idSuffix, label);
  pushWebModuleSummaryButtonStyle(active);
  const bool clicked = ImGui::Button(buttonLabel.c_str(), ImVec2{width, 0.0f});
  popWebModuleSummaryButtonStyle();
  if (width <= 0.0f) {
    const float itemRight = ImGui::GetItemRectMax().x;
    const float contentRight = ImGui::GetWindowPos().x + ImGui::GetWindowContentRegionMax().x;
    if (contentRight - itemRight > 112.0f) {
      ImGui::SameLine();
    }
  }
  return clicked;
}

void SubtrActorPlugin::renderCvarModuleSummaryToggle(
    const char *label,
    const char *name,
    bool defaultValue,
    const char *idSuffix,
    float width) {
  const bool active = cvarBool(name, defaultValue);
  if (renderModuleSummaryToggle(label, active, idSuffix, width)) {
    setCvarBool(name, !active);
  }
}

void SubtrActorPlugin::renderBoolModuleSummaryToggle(
    const char *label,
    bool &active,
    const char *idSuffix,
    float width) {
  if (renderModuleSummaryToggle(label, active, idSuffix, width)) {
    active = !active;
    scheduleUiConfigAutosave();
  }
}

void SubtrActorPlugin::renderEventFilterModuleSummaryToggle(
    const char *label,
    const char *token,
    const char *idSuffix,
    float width) {
  std::vector<std::string> selected =
      selectedEventSourceTokens(cvarString("subtr_actor_overlay_event_types", "all"));
  const bool active =
      eventPlaylistMechanicsEnabled &&
      (containsString(selected, "mechanics") || containsString(selected, token));
  if (!renderModuleSummaryToggle(label, active, idSuffix, width)) {
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

void SubtrActorPlugin::renderModuleSummaryControls(
    const char *idSuffix,
    bool collapsibleGroups,
    float toggleWidth,
    bool includePluginControls) {
  auto renderTimelineControls = [&]() {
    renderEventFilterModuleSummaryToggle("Backboard", "backboard", idSuffix, toggleWidth);
    renderBoolModuleSummaryToggle(
        "Possession",
        timelineRangePossessionEnabled,
        idSuffix,
        toggleWidth);
    renderEventFilterModuleSummaryToggle("50/50", "fifty_fifty", idSuffix, toggleWidth);
    renderBoolModuleSummaryToggle(
        "Half control",
        timelineRangePressureEnabled,
        idSuffix,
        toggleWidth);
    renderBoolModuleSummaryToggle("Rush", timelineRangeRushEnabled, idSuffix, toggleWidth);
    renderBoolModuleSummaryToggle(
        "Position zones",
        timelineRangeAbsolutePositioningEnabled,
        idSuffix,
        toggleWidth);
    renderEventFilterModuleSummaryToggle("Wavedash", "wavedash", idSuffix, toggleWidth);
    renderEventFilterModuleSummaryToggle("Touch", "touch", idSuffix, toggleWidth);
    renderEventFilterModuleSummaryToggle("Whiff", "whiff", idSuffix, toggleWidth);
    renderBoolModuleSummaryToggle(
        "Boost pickup timeline",
        timelineRangeBoostEnabled,
        idSuffix,
        toggleWidth);
    renderEventFilterModuleSummaryToggle("Powerslide", "powerslide", idSuffix, toggleWidth);
    renderEventFilterModuleSummaryToggle("Bump", "bump", idSuffix, toggleWidth);
    if (includePluginControls) {
      renderEventFilterModuleSummaryToggle("Dodge refresh", "dodge_reset", idSuffix, toggleWidth);
      renderEventFilterModuleSummaryToggle("Speed flip", "speed_flip", idSuffix, toggleWidth);
      renderEventFilterModuleSummaryToggle("Half flip", "half_flip", idSuffix, toggleWidth);
      renderEventFilterModuleSummaryToggle("Ball carry", "ball_carry", idSuffix, toggleWidth);
      renderEventFilterModuleSummaryToggle("Ceiling shot", "ceiling_shot", idSuffix, toggleWidth);
      renderEventFilterModuleSummaryToggle("Flip reset", "flip_reset", idSuffix, toggleWidth);
      renderEventFilterModuleSummaryToggle("Double tap", "double_tap", idSuffix, toggleWidth);
      renderEventFilterModuleSummaryToggle("Demos", "demo", idSuffix, toggleWidth);
      renderBoolModuleSummaryToggle(
          "Team event playlist",
          eventPlaylistTeamEventsEnabled,
          idSuffix,
          toggleWidth);
      renderBoolModuleSummaryToggle(
          "Goal context playlist",
          eventPlaylistGoalContextEnabled,
          idSuffix,
          toggleWidth);
      renderBoolModuleSummaryToggle(
          "Playlist follow",
          eventPlaylistAutoFollow,
          idSuffix,
          toggleWidth);
    }
  };

  auto renderInGameControls = [&]() {
    if (includePluginControls) {
      renderCvarModuleSummaryToggle(
          "Canvas status line",
          "subtr_actor_status_overlay_enabled",
          true,
          idSuffix,
          toggleWidth);
      renderCvarModuleSummaryToggle(
          "HUD mechanics",
          "subtr_actor_overlay_mechanics_enabled",
          true,
          idSuffix,
          toggleWidth);
      renderCvarModuleSummaryToggle(
          "HUD team events",
          "subtr_actor_overlay_team_events_enabled",
          true,
          idSuffix,
          toggleWidth);
      renderCvarModuleSummaryToggle(
          "HUD goal context",
          "subtr_actor_overlay_goal_context_enabled",
          true,
          idSuffix,
          toggleWidth);
    }
    renderBoolModuleSummaryToggle(
        "Ceiling shot labels",
        renderEffectCeilingShotEnabled,
        idSuffix,
        toggleWidth);
    renderBoolModuleSummaryToggle(
        "50/50 labels",
        renderEffectFiftyFiftyEnabled,
        idSuffix,
        toggleWidth);
    renderBoolModuleSummaryToggle("Half control", renderEffectPressureEnabled, idSuffix, toggleWidth);
    renderBoolModuleSummaryToggle(
        "Player roles",
        renderEffectRelativePositioningEnabled,
        idSuffix,
        toggleWidth);
    renderBoolModuleSummaryToggle(
        "Position zones",
        renderEffectAbsolutePositioningEnabled,
        idSuffix,
        toggleWidth);
    renderBoolModuleSummaryToggle(
        "Speed flip labels",
        renderEffectSpeedFlipEnabled,
        idSuffix,
        toggleWidth);
    renderBoolModuleSummaryToggle("Touch labels", renderEffectTouchEnabled, idSuffix, toggleWidth);
    renderBoolModuleSummaryToggle(
        "Boost pickup animation",
        boostPickupAnimationEnabled,
        idSuffix,
        toggleWidth);
    const bool boostPadsEnabled =
        boostPickupPadBig || boostPickupPadSmall || boostPickupPadAmbiguous;
    if (renderModuleSummaryToggle("Boost pad locations", boostPadsEnabled, idSuffix, toggleWidth)) {
      const bool next = !boostPadsEnabled;
      boostPickupPadBig = next;
      boostPickupPadSmall = next;
      boostPickupPadAmbiguous = next;
      scheduleUiConfigAutosave();
    }
  };

  if (collapsibleGroups) {
    if (ImGui::TreeNodeEx(
            std::format("Timeline visualizations##{}-timeline", idSuffix).c_str(),
            ImGuiTreeNodeFlags_DefaultOpen)) {
      renderTimelineControls();
      ImGui::TreePop();
    }

    if (ImGui::TreeNodeEx(
            std::format("In-game visualizations##{}-ingame", idSuffix).c_str(),
            ImGuiTreeNodeFlags_DefaultOpen)) {
      renderInGameControls();
      ImGui::TreePop();
    }
    return;
  }

  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "TIMELINE VISUALIZATIONS");
  renderTimelineControls();
  if (toggleWidth <= 0.0f) {
    ImGui::NewLine();
  }
  ImGui::Spacing();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "IN-GAME VISUALIZATIONS");
  renderInGameControls();
  if (toggleWidth <= 0.0f) {
    ImGui::NewLine();
  }
}

void SubtrActorPlugin::renderModuleSettingsControls(
    const char *idSuffix,
    bool includeOpenButtons,
    bool webCardHeaders,
    bool onlyWebActivePanels) {
  ImGui::PushID(idSuffix);
  bool renderedPanel = false;

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

  auto renderSettingsHeader = [&](const char *title, const std::string &readout) {
    if (webCardHeaders) {
      ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "STAT DISPLAY");
      ImGui::Text("%s", title);
      ImGui::SameLine(std::max(
          ImGui::GetCursorPosX(),
          ImGui::GetWindowContentRegionMax().x - ImGui::CalcTextSize(readout.c_str()).x));
      ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "%s", readout.c_str());
      return;
    } else {
      ImGui::TextDisabled("%s", title);
    }
    ImGui::SameLine();
    ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "%s", readout.c_str());
  };

  auto beginModuleSettingsCard = [&](const char *cardId, float height) {
    if (!webCardHeaders) {
      return false;
    }
    ImGui::PushStyleVar(ImGuiStyleVar_ChildRounding, 8.0f);
    ImGui::PushStyleVar(ImGuiStyleVar_WindowPadding, ImVec2{12.0f, 10.0f});
    ImGui::PushStyleColor(ImGuiCol_ChildBg, ImVec4{1.0f, 1.0f, 1.0f, 0.035f});
    ImGui::PushStyleColor(ImGuiCol_Border, ImVec4{1.0f, 1.0f, 1.0f, 0.08f});
    ImGui::BeginChild(cardId, ImVec2{0.0f, height}, true);
    return true;
  };

  auto endModuleSettingsCard = [&]() {
    if (!webCardHeaders) {
      return;
    }
    ImGui::EndChild();
    ImGui::PopStyleColor(2);
    ImGui::PopStyleVar(2);
  };

  if (!onlyWebActivePanels) {
    const bool movementCard = beginModuleSettingsCard(
        "module-settings-card-movement",
        includeOpenButtons ? 116.0f : 84.0f);
    renderSettingsHeader(
        "Movement breakdown",
        settingReadout(
            {{movementBreakdownSpeed, "Speed band"}, {movementBreakdownHeight, "Height band"}},
            " + "));
    if (ImGui::Checkbox("Speed band##movement-breakdown", &movementBreakdownSpeed)) {
      scheduleUiConfigAutosave();
    }
    ImGui::SameLine();
    if (ImGui::Checkbox("Height band##movement-breakdown", &movementBreakdownHeight)) {
      scheduleUiConfigAutosave();
    }
    if (includeOpenButtons && ImGui::Button("Open movement stats")) {
      createStatsModuleWindow("movement", 0);
    }
    if (movementCard) {
      endModuleSettingsCard();
    }
    renderedPanel = true;
  }

  if (webCardHeaders && renderedPanel && (!onlyWebActivePanels || timelineRangePossessionEnabled)) {
    ImGui::Spacing();
  }
  if (!onlyWebActivePanels || timelineRangePossessionEnabled) {
    const bool possessionCard = beginModuleSettingsCard(
        "module-settings-card-possession",
        includeOpenButtons ? 116.0f : 84.0f);
    renderSettingsHeader(
        "Possession breakdown",
        settingReadout(
            {{possessionBreakdownState, "Control"}, {possessionBreakdownThird, "Third"}},
            " x "));
    if (ImGui::Checkbox("Control##possession-breakdown", &possessionBreakdownState)) {
      scheduleUiConfigAutosave();
    }
    ImGui::SameLine();
    if (ImGui::Checkbox("Third##possession-breakdown", &possessionBreakdownThird)) {
      scheduleUiConfigAutosave();
    }
    if (includeOpenButtons && ImGui::Button("Open possession stats")) {
      createStatsModuleWindow("possession", 0);
    }
    if (possessionCard) {
      endModuleSettingsCard();
    }
    renderedPanel = true;
  }

  ImGui::PopID();
}
