// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
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
  auto writeStatsPlayerPlacement = [](
                                       std::ostream &out,
                                       const UiWindowPlacement &placement,
                                       bool visible) {
    const ImVec2 displaySize = ImGui::GetIO().DisplaySize;
    const float viewportWidth =
        placement.viewport_width > 0.0f ? placement.viewport_width : std::max(1.0f, displaySize.x);
    const float viewportHeight = placement.viewport_height > 0.0f ? placement.viewport_height
                                                                  : std::max(1.0f, displaySize.y);
    out << "{\"x\":" << placement.x << ",\"y\":" << placement.y
        << ",\"viewport\":{\"width\":" << viewportWidth
        << ",\"height\":" << viewportHeight << "}"
        << ",\"zIndex\":" << placement.z_index
        << ",\"visible\":" << (visible ? "true" : "false") << "}";
  };
  auto writeStatsPlayerStatsWindowPlacement = [](
                                                  std::ostream &out,
                                                  const UiStatsWindow &window) {
    const ImVec2 displaySize = ImGui::GetIO().DisplaySize;
    const float viewportWidth =
        window.viewport_width > 0.0f ? window.viewport_width : std::max(1.0f, displaySize.x);
    const float viewportHeight =
        window.viewport_height > 0.0f ? window.viewport_height : std::max(1.0f, displaySize.y);
    out << "{\"x\":" << window.x << ",\"y\":" << window.y
        << ",\"viewport\":{\"width\":" << viewportWidth
        << ",\"height\":" << viewportHeight << "}"
        << ",\"zIndex\":" << window.z_index
        << ",\"visible\":" << (window.open ? "true" : "false") << "}";
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
  auto writeStringArray = [](std::ostream &out, const std::vector<std::string> &values) {
    out << "[";
    for (size_t index = 0; index < values.size(); ++index) {
      if (index > 0) {
        out << ",";
      }
      out << "\"" << escapeJsonString(values[index]) << "\"";
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
  file << "  \"event_playlist_source_filter\": \""
       << escapeJsonString(eventPlaylistSourceFilter) << "\",\n";
  file << "  \"overlays\": {\n";
  const std::string currentEventFilter = cvarString("subtr_actor_overlay_event_types", "all");
  const std::vector<std::string> currentEventFilterTokens =
      selectedEventSourceTokens(currentEventFilter);
  file << "    \"timelineEvents\": [";
  bool wroteOverlayValue = false;
  std::unordered_set<std::string> wroteOverlayIds;
  auto resetOverlayWriter = [&]() {
    wroteOverlayValue = false;
    wroteOverlayIds.clear();
  };
  auto writeOverlayId = [&](std::string_view id, bool enabled) {
    if (!enabled) {
      return;
    }
    std::string idString{id};
    if (!wroteOverlayIds.insert(idString).second) {
      return;
    }
    if (wroteOverlayValue) {
      file << ",";
    }
    file << "\"" << escapeJsonString(id) << "\"";
    wroteOverlayValue = true;
  };
  for (const std::string &token : currentEventFilterTokens) {
    if (const std::string sourceId = webTimelineEventSourceIdForFilterToken(token);
        !sourceId.empty()) {
      writeOverlayId(sourceId, true);
    }
  }
  file << "],\n";
  file << "    \"timelineRanges\": [";
  resetOverlayWriter();
  writeOverlayId("boost", timelineRangeBoostEnabled);
  writeOverlayId("possession", timelineRangePossessionEnabled);
  writeOverlayId("pressure", timelineRangePressureEnabled);
  writeOverlayId("rush", timelineRangeRushEnabled);
  writeOverlayId("absolute-positioning", timelineRangeAbsolutePositioningEnabled);
  file << "],\n";
  file << "    \"mechanics\": [";
  const bool allMechanicsSelected =
      eventPlaylistMechanicsEnabled &&
      (allEventSourcesSelected(currentEventFilter) ||
       containsString(currentEventFilterTokens, "mechanics"));
  bool wroteMechanicFilter = false;
  auto writeMechanicFilterId = [&](std::string_view token) {
    if (wroteMechanicFilter) {
      file << ",";
    }
    file << "\"" << escapeJsonString(token) << "\"";
    wroteMechanicFilter = true;
  };
  if (allMechanicsSelected) {
    for (const char *token : MECHANIC_FILTER_TOKENS) {
      writeMechanicFilterId(token);
    }
  } else if (eventPlaylistMechanicsEnabled) {
    for (const std::string &token : currentEventFilterTokens) {
      if (!isMechanicFilterToken(token)) {
        continue;
      }
      writeMechanicFilterId(token);
    }
  }
  file << "],\n";
  file << "    \"renderEffects\": [";
  resetOverlayWriter();
  const bool hudOverlayEnabled = cvarBool("subtr_actor_overlay_enabled", true);
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
  file << "    \"pluginRenderEffects\": [";
  resetOverlayWriter();
  writeOverlayId(
      "mechanics",
      cvarBool("subtr_actor_overlay_mechanics_enabled", true));
  writeOverlayId("team", cvarBool("subtr_actor_overlay_team_events_enabled", true));
  writeOverlayId("goal_context", cvarBool("subtr_actor_overlay_goal_context_enabled", true));
  file << "],\n";
  file << "    \"pluginHudOverlay\": " << (hudOverlayEnabled ? "true" : "false") << ",\n";
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
    if (const auto attachedPlayerId = webCameraPlayerIdConfig()) {
      file << "\"" << escapeJsonString(*attachedPlayerId) << "\"";
    } else {
      file << "null";
    }
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
  file << ",\"playerIds\":";
  if (boostPickupPlayerFilterEnabled) {
    writeStringArray(file, boostPickupPlayerIds);
  } else {
    file << "null";
  }
  file << "},\n";
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
  auto writeWindowConfig = [&](const SingletonWindowControl &window, bool last, bool webConfig) {
    const bool visible = window.open != nullptr && *window.open;
    file << "    {\"id\":\"" << window.config_id << "\",\"placement\":";
    if (webConfig) {
      writeStatsPlayerPlacement(file, *window.placement, visible);
    } else {
      writePlacement(file, *window.placement, visible);
    }
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
    writeWindowConfig(webWindows[index], index + 1 == webWindows.size(), true);
  }
  file << "  ],\n";
  file << "  \"pluginWindows\": [\n";
  for (size_t index = 0; index < pluginWindows.size(); index += 1) {
    writeWindowConfig(pluginWindows[index], index + 1 == pluginWindows.size(), false);
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
         << ",\"placement\":";
    writeStatsPlayerStatsWindowPlacement(file, window);
    file << ",\"selected_player_index\":" << window.selected_player_index
         << ",\"selected_player_id\":\"" << escapeJsonString(webPlayerIdForWindow(window)) << "\""
         << ",\"selected_team_is_team_0\":"
         << (window.selected_team_is_team_0 != 0 ? "true" : "false")
         << ",\"module_name\":\"" << escapeJsonString(window.module_name) << "\""
         << ",\"module_view\":" << window.module_view
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
         << "\",\"placement\":";
    writeStatsPlayerStatsWindowPlacement(file, window);
    file << ",\"playerId\":";
    if (const auto playerId = webPlayerIdForWindowConfig(window)) {
      file << "\"" << escapeJsonString(*playerId) << "\"";
    } else {
      file << "null";
    }
    file << ",\"team\":\"" << (window.selected_team_is_team_0 != 0 ? "blue" : "orange") << "\"";
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

void SubtrActorPlugin::scheduleUiConfigAutosave(std::chrono::milliseconds delay) {
  const auto nextSave = std::chrono::steady_clock::now() + delay;
  if (nextSave < nextUiConfigAutosave) {
    nextUiConfigAutosave = nextSave;
  }
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
