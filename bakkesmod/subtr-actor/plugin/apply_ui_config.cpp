// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
void SubtrActorPlugin::applyUiConfigJson(
    const std::string &json,
    std::string_view sourceLabel) {
  const size_t firstJsonByte = json.find_first_not_of(" \t\r\n");
  if (firstJsonByte == std::string::npos || json[firstJsonByte] != '{' ||
      json.find("\"version\"") == std::string::npos) {
    cvarManager->log(std::format("subtr-actor: ignored invalid UI config from {}", sourceLabel));
    return;
  }

  auto loadPlacementObject = [this](
                                 const std::string &object,
                                 UiWindowPlacement &out,
                                 bool *visible = nullptr) {
    if (visible != nullptr) {
      *visible = parseJsonBoolProperty(object, "visible").value_or(*visible);
    }
    out.has_placement = parseJsonBoolProperty(object, "has_placement").value_or(true);
    out.pending_apply_placement = out.has_placement;
    out.x = static_cast<float>(parseJsonNumberProperty(object, "x").value_or(out.x));
    out.y = static_cast<float>(parseJsonNumberProperty(object, "y").value_or(out.y));
    out.width = static_cast<float>(parseJsonNumberProperty(object, "width").value_or(out.width));
    out.height =
        static_cast<float>(parseJsonNumberProperty(object, "height").value_or(out.height));
    out.viewport_width = static_cast<float>(
        parseJsonNumberProperty(object, "viewport_width").value_or(out.viewport_width));
    out.viewport_height = static_cast<float>(
        parseJsonNumberProperty(object, "viewport_height").value_or(out.viewport_height));
    if (const auto viewport = parseJsonObjectProperty(object, "viewport")) {
      out.viewport_width = static_cast<float>(
          parseJsonNumberProperty(*viewport, "width").value_or(out.viewport_width));
      out.viewport_height = static_cast<float>(
          parseJsonNumberProperty(*viewport, "height").value_or(out.viewport_height));
    }
    out.z_index = static_cast<int>(parseJsonNumberProperty(object, "zIndex").value_or(out.z_index));
    nextUiWindowZIndex = std::max(nextUiWindowZIndex, out.z_index + 1);
  };
  auto applyDefaultPlacementSize =
      [](const std::string &object, UiWindowPlacement &out, float width, float height) {
        if (!parseJsonNumberProperty(object, "width") || out.width <= 0.0f) {
          out.width = width;
        }
        if (!parseJsonNumberProperty(object, "height") || out.height <= 0.0f) {
          out.height = height;
        }
      };
  for (const SingletonWindowControl &window : singletonWindowControls()) {
    *window.open =
        parseJsonBoolProperty(json, window.legacy_open_key).value_or(*window.open);
  }
  eventPlaylistMechanicsEnabled = parseJsonBoolProperty(json, "event_playlist_mechanics_enabled")
                                      .value_or(eventPlaylistMechanicsEnabled);
  eventPlaylistTeamEventsEnabled = parseJsonBoolProperty(json, "event_playlist_team_enabled")
                                       .value_or(eventPlaylistTeamEventsEnabled);
  eventPlaylistGoalContextEnabled =
      parseJsonBoolProperty(json, "event_playlist_goal_context_enabled")
          .value_or(eventPlaylistGoalContextEnabled);
  eventPlaylistAutoFollow =
      parseJsonBoolProperty(json, "event_playlist_auto_follow").value_or(eventPlaylistAutoFollow);
  eventPlaylistSourceFilter =
      parseJsonStringProperty(json, "event_playlist_source_filter")
          .value_or(eventPlaylistSourceFilter);
  if (const auto overlays = parseJsonObjectProperty(json, "overlays")) {
    const std::vector<std::string> timelineEvents =
        parseJsonStringArrayProperty(*overlays, "timelineEvents");
    const bool hasTimelineEvents = jsonPropertyExists(*overlays, "timelineEvents");
    if (hasTimelineEvents) {
      eventPlaylistMechanicsEnabled = containsString(timelineEvents, "mechanics");
      eventPlaylistTeamEventsEnabled = containsString(timelineEvents, "team");
      eventPlaylistGoalContextEnabled = containsString(timelineEvents, "goal_context");
    }

    const std::vector<std::string> mechanicFilters =
        parseJsonStringArrayProperty(*overlays, "mechanics");
    const bool hasMechanicFilters = jsonPropertyExists(*overlays, "mechanics");
    if (hasTimelineEvents || hasMechanicFilters) {
      std::vector<std::string> selectedFilters;
      for (const std::string &id : timelineEvents) {
        const std::string token = normalizeEventFilterToken(id);
        if (token == "mechanics" && hasMechanicFilters && !mechanicFilters.empty()) {
          continue;
        }
        if (isKnownEventFilterToken(token)) {
          appendUniqueFilterToken(selectedFilters, token);
        }
      }
      for (const std::string &id : mechanicFilters) {
        const std::string token = normalizeEventFilterToken(id);
        if (!token.empty()) {
          appendUniqueFilterToken(selectedFilters, token);
        }
      }
      const bool hasSelectedMechanicFilter = std::any_of(
          selectedFilters.begin(),
          selectedFilters.end(),
          [](const std::string &token) {
            return token == "mechanics" || isMechanicFilterToken(token);
          });
      if ((hasMechanicFilters && !mechanicFilters.empty()) || hasSelectedMechanicFilter) {
        eventPlaylistMechanicsEnabled = true;
      }
      setCvarString(
          "subtr_actor_overlay_event_types",
          eventFilterFromSelectedSources(selectedFilters));
    }

    const std::vector<std::string> timelineRanges =
        parseJsonStringArrayProperty(*overlays, "timelineRanges");
    if (jsonPropertyExists(*overlays, "timelineRanges")) {
      timelineRangeBoostEnabled = containsString(timelineRanges, "boost");
      timelineRangePossessionEnabled = containsString(timelineRanges, "possession");
      timelineRangePressureEnabled = containsString(timelineRanges, "pressure");
      timelineRangeRushEnabled = containsString(timelineRanges, "rush");
      timelineRangeAbsolutePositioningEnabled =
          containsString(timelineRanges, "absolute-positioning");
    }

    const std::vector<std::string> renderEffects =
        parseJsonStringArrayProperty(*overlays, "renderEffects");
    const bool hasRenderEffects = jsonPropertyExists(*overlays, "renderEffects");
    bool anyRenderEffect = false;
    if (hasRenderEffects) {
      anyRenderEffect = !renderEffects.empty();
      renderEffectCeilingShotEnabled = containsString(renderEffects, "ceiling-shot");
      renderEffectFiftyFiftyEnabled = containsString(renderEffects, "fifty-fifty");
      renderEffectPressureEnabled = containsString(renderEffects, "pressure");
      renderEffectRelativePositioningEnabled =
          containsString(renderEffects, "relative-positioning");
      renderEffectAbsolutePositioningEnabled =
          containsString(renderEffects, "absolute-positioning");
      renderEffectSpeedFlipEnabled = containsString(renderEffects, "speed-flip");
      renderEffectTouchEnabled = containsString(renderEffects, "touch");
    }
    const bool hasPluginRenderEffects = jsonPropertyExists(*overlays, "pluginRenderEffects");
    std::vector<std::string> pluginRenderEffects =
        parseJsonStringArrayProperty(*overlays, "pluginRenderEffects");
    if (!hasPluginRenderEffects && hasRenderEffects) {
      for (const char *id : {"mechanics", "team", "goal_context"}) {
        if (containsString(renderEffects, id)) {
          pluginRenderEffects.emplace_back(id);
        }
      }
    }
    if (const auto pluginHudOverlay = parseJsonBoolProperty(*overlays, "pluginHudOverlay")) {
      setCvarBool("subtr_actor_overlay_enabled", *pluginHudOverlay);
    } else if (hasPluginRenderEffects) {
      setCvarBool("subtr_actor_overlay_enabled", anyRenderEffect || !pluginRenderEffects.empty());
    } else if (hasRenderEffects) {
      setCvarBool("subtr_actor_overlay_enabled", anyRenderEffect);
    }
    if (hasPluginRenderEffects || hasRenderEffects) {
      setCvarBool(
          "subtr_actor_overlay_mechanics_enabled",
          containsString(pluginRenderEffects, "mechanics") ||
              renderEffectCeilingShotEnabled ||
              renderEffectFiftyFiftyEnabled ||
              renderEffectRelativePositioningEnabled ||
              renderEffectAbsolutePositioningEnabled ||
              renderEffectSpeedFlipEnabled ||
              renderEffectTouchEnabled);
      setCvarBool(
          "subtr_actor_overlay_team_events_enabled",
          containsString(pluginRenderEffects, "team") || renderEffectPressureEnabled);
      setCvarBool(
          "subtr_actor_overlay_goal_context_enabled",
          containsString(pluginRenderEffects, "goal_context"));
    }

    const std::optional<bool> boostPads = parseJsonBoolProperty(*overlays, "boostPads");
    if (boostPads) {
      boostPickupPadBig = *boostPads;
      boostPickupPadSmall = *boostPads;
      boostPickupPadAmbiguous = *boostPads;
    }
    boostPickupAnimationEnabled = parseJsonBoolProperty(*overlays, "boostPickupAnimation")
                                      .value_or(boostPickupAnimationEnabled);
  }
  recordingFps = static_cast<int>(
      std::clamp(parseJsonNumberProperty(json, "recording_fps").value_or(60.0), 1.0, 120.0));
  recordingPlaybackRateIndex = static_cast<int>(std::clamp(
      parseJsonNumberProperty(json, "recording_playback_rate_index").value_or(1.0),
      0.0,
      3.0));

  if (const auto recording = parseJsonObjectProperty(json, "recording")) {
    recordingFps = static_cast<int>(
        std::clamp(parseJsonNumberProperty(*recording, "fps").value_or(recordingFps), 1.0, 120.0));
    if (const auto playbackRate = parseJsonNumberProperty(*recording, "playbackRate")) {
      recordingPlaybackRateIndex = recordingPlaybackRateIndexForValue(*playbackRate);
    }
  }
  bool applyPlaybackConfig = false;
  if (const auto playback = parseJsonObjectProperty(json, "playback")) {
    playbackCurrentTime = static_cast<float>(std::max(
        0.0,
        parseJsonNumberProperty(*playback, "currentTime").value_or(playbackCurrentTime)));
    playbackPlaying = parseJsonBoolProperty(*playback, "playing").value_or(playbackPlaying);
    playbackRate = static_cast<float>(
        std::clamp(parseJsonNumberProperty(*playback, "rate").value_or(playbackRate), 0.1, 4.0));
    playbackSkipPostGoalTransitions =
        parseJsonBoolProperty(*playback, "skipPostGoalTransitions")
            .value_or(playbackSkipPostGoalTransitions);
    playbackSkipKickoffs =
        parseJsonBoolProperty(*playback, "skipKickoffs").value_or(playbackSkipKickoffs);
    applyPlaybackConfig = true;
  }
  touchControlsMode = static_cast<int>(
      std::clamp(
          parseJsonNumberProperty(json, "touch_controls_mode").value_or(touchControlsMode),
          0.0,
          1.0));
  touchMarkerDecaySeconds = static_cast<float>(std::clamp(
      parseJsonNumberProperty(json, "touch_marker_decay_seconds").value_or(touchMarkerDecaySeconds),
      1.0,
      10.0));
  touchBreakdownKind =
      parseJsonBoolProperty(json, "touch_breakdown_kind").value_or(touchBreakdownKind);
  touchBreakdownHeight =
      parseJsonBoolProperty(json, "touch_breakdown_height").value_or(touchBreakdownHeight);
  touchBreakdownSurface =
      parseJsonBoolProperty(json, "touch_breakdown_surface").value_or(touchBreakdownSurface);
  touchBreakdownDodge =
      parseJsonBoolProperty(json, "touch_breakdown_dodge").value_or(touchBreakdownDodge);
  movementBreakdownSpeed =
      parseJsonBoolProperty(json, "movement_breakdown_speed").value_or(movementBreakdownSpeed);
  movementBreakdownHeight =
      parseJsonBoolProperty(json, "movement_breakdown_height").value_or(movementBreakdownHeight);
  possessionBreakdownState = parseJsonBoolProperty(json, "possession_breakdown_state")
                                 .value_or(possessionBreakdownState);
  possessionBreakdownThird = parseJsonBoolProperty(json, "possession_breakdown_third")
                                 .value_or(possessionBreakdownThird);
  boostPickupPadBig =
      parseJsonBoolProperty(json, "boost_pickup_pad_big").value_or(boostPickupPadBig);
  boostPickupPadSmall =
      parseJsonBoolProperty(json, "boost_pickup_pad_small").value_or(boostPickupPadSmall);
  boostPickupPadAmbiguous =
      parseJsonBoolProperty(json, "boost_pickup_pad_ambiguous").value_or(boostPickupPadAmbiguous);
  boostPickupAnimationEnabled = parseJsonBoolProperty(json, "boost_pickup_animation_enabled")
                                    .value_or(boostPickupAnimationEnabled);
  boostPickupActivityActive = parseJsonBoolProperty(json, "boost_pickup_activity_active")
                                  .value_or(boostPickupActivityActive);
  boostPickupActivityInactive = parseJsonBoolProperty(json, "boost_pickup_activity_inactive")
                                    .value_or(boostPickupActivityInactive);
  boostPickupActivityUnknown = parseJsonBoolProperty(json, "boost_pickup_activity_unknown")
                                   .value_or(boostPickupActivityUnknown);
  boostPickupFieldOwn =
      parseJsonBoolProperty(json, "boost_pickup_field_own").value_or(boostPickupFieldOwn);
  boostPickupFieldOpponent = parseJsonBoolProperty(json, "boost_pickup_field_opponent")
                                 .value_or(boostPickupFieldOpponent);
  boostPickupFieldUnknown = parseJsonBoolProperty(json, "boost_pickup_field_unknown")
                                .value_or(boostPickupFieldUnknown);
  if (const auto moduleConfigs = parseJsonObjectProperty(json, "moduleConfigs")) {
    std::optional<std::string> boostConfig = parseJsonObjectProperty(*moduleConfigs, "boost");
    if (!boostConfig) {
      boostConfig = parseJsonObjectProperty(*moduleConfigs, "boost-pickup-animation");
    }
    if (boostConfig) {
      const std::vector<std::string> padTypes =
          parseJsonStringArrayProperty(*boostConfig, "padTypes");
      if (jsonPropertyExists(*boostConfig, "padTypes")) {
        boostPickupPadBig = containsString(padTypes, "big");
        boostPickupPadSmall = containsString(padTypes, "small");
        boostPickupPadAmbiguous = containsString(padTypes, "ambiguous");
      }
      const std::vector<std::string> activities =
          parseJsonStringArrayProperty(*boostConfig, "activities");
      if (jsonPropertyExists(*boostConfig, "activities")) {
        boostPickupActivityActive = containsString(activities, "active");
        boostPickupActivityInactive = containsString(activities, "inactive");
        boostPickupActivityUnknown = containsString(activities, "unknown");
      }
      const std::vector<std::string> fieldHalves =
          parseJsonStringArrayProperty(*boostConfig, "fieldHalves");
      if (jsonPropertyExists(*boostConfig, "fieldHalves")) {
        boostPickupFieldOwn = containsString(fieldHalves, "own");
        boostPickupFieldOpponent = containsString(fieldHalves, "opponent");
        boostPickupFieldUnknown = containsString(fieldHalves, "unknown");
      }
      if (jsonPropertyExists(*boostConfig, "playerIds")) {
        boostPickupPlayerFilterEnabled = !jsonPropertyIsNull(*boostConfig, "playerIds");
        boostPickupPlayerIds = boostPickupPlayerFilterEnabled
                                   ? parseJsonStringArrayProperty(*boostConfig, "playerIds")
                                   : std::vector<std::string>{};
      }
    }
    if (const auto touchConfig = parseJsonObjectProperty(*moduleConfigs, "touch")) {
      touchMarkerDecaySeconds = static_cast<float>(std::clamp(
          parseJsonNumberProperty(*touchConfig, "decaySeconds").value_or(touchMarkerDecaySeconds),
          1.0,
          10.0));
      if (const auto overlayMode = parseJsonStringProperty(*touchConfig, "overlayMode")) {
        if (*overlayMode == "markers") {
          touchControlsMode = 0;
        } else if (*overlayMode == "advancement") {
          touchControlsMode = 1;
        }
      }
      const std::vector<std::string> breakdownClasses =
          parseJsonStringArrayProperty(*touchConfig, "breakdownClasses");
      if (jsonPropertyExists(*touchConfig, "breakdownClasses")) {
        touchBreakdownKind = containsString(breakdownClasses, "kind");
        touchBreakdownHeight = containsString(breakdownClasses, "height_band");
        touchBreakdownSurface = containsString(breakdownClasses, "surface");
        touchBreakdownDodge = containsString(breakdownClasses, "dodge_state");
      }
    }
    if (const auto movementConfig = parseJsonObjectProperty(*moduleConfigs, "movement")) {
      const std::vector<std::string> breakdownClasses =
          parseJsonStringArrayProperty(*movementConfig, "breakdownClasses");
      if (jsonPropertyExists(*movementConfig, "breakdownClasses")) {
        movementBreakdownSpeed = containsString(breakdownClasses, "speed_band");
        movementBreakdownHeight = containsString(breakdownClasses, "height_band");
      }
    }
    if (const auto possessionConfig = parseJsonObjectProperty(*moduleConfigs, "possession")) {
      const std::vector<std::string> breakdownClasses =
          parseJsonStringArrayProperty(*possessionConfig, "breakdownClasses");
      if (jsonPropertyExists(*possessionConfig, "breakdownClasses")) {
        possessionBreakdownState = containsString(breakdownClasses, "possession_state");
        possessionBreakdownThird = containsString(breakdownClasses, "field_third");
      }
    }
  }
  graphInspectorView = static_cast<int>(
      std::max(0.0, parseJsonNumberProperty(json, "graph_inspector_view").value_or(0.0)));
  mechanicsReviewIndex = static_cast<int>(
      std::max(0.0, parseJsonNumberProperty(json, "mechanics_review_index").value_or(0.0)));
  mechanicsReviewClipLeadSeconds = static_cast<float>(std::clamp(
      parseJsonNumberProperty(json, "mechanics_review_clip_lead_seconds")
          .value_or(mechanicsReviewClipLeadSeconds),
      0.0,
      10.0));
  mechanicsReviewClipTrailSeconds = static_cast<float>(std::clamp(
      parseJsonNumberProperty(json, "mechanics_review_clip_trail_seconds")
          .value_or(mechanicsReviewClipTrailSeconds),
      0.0,
      10.0));
  selectedGraphOutput = parseJsonStringProperty(json, "selected_graph_output").value_or("");
  selectedAnalysisNode = parseJsonStringProperty(json, "selected_analysis_node").value_or("");
  graphInspectorNodeQuery =
      parseJsonStringProperty(json, "graph_inspector_node_query").value_or("");

  if (const auto placements = parseJsonObjectProperty(json, "placements")) {
    for (const SingletonWindowControl &window : singletonWindowControls()) {
      const auto object = parseJsonObjectProperty(*placements, window.legacy_placement_key);
      if (!object) {
        continue;
      }
      loadPlacementObject(*object, *window.placement, window.open);
      applyDefaultPlacementSize(*object, *window.placement, window.width, window.height);
    }
  }
  auto loadWindowArray = [&](const char *propertyName, bool webConfig) {
    for (const std::string &object : parseJsonObjectArrayProperty(json, propertyName)) {
      const std::string id = parseJsonStringProperty(object, "id").value_or("");
      std::optional<std::string> placement = parseJsonObjectProperty(object, "placement");
      if (!placement) {
        if (!webConfig) {
          continue;
        }
        placement = R"({"x":8,"y":8,"viewport":{"width":1,"height":1},"visible":true})";
      }
      for (const SingletonWindowControl &window : singletonWindowControls()) {
        if (window.web_config != webConfig || id != window.config_id) {
          continue;
        }
        loadPlacementObject(*placement, *window.placement, window.open);
        applyDefaultPlacementSize(*placement, *window.placement, window.width, window.height);
        break;
      }
    }
  };
  loadWindowArray("singletonWindows", true);
  loadWindowArray("pluginWindows", false);
  loadWindowArray("plugin_windows", false);

  uiStatsWindows.clear();
  nextUiStatsWindowId = 1;
  const bool hasLegacyStatsWindows = jsonPropertyExists(json, "stats_windows");
  const bool hasWebStatsWindows = jsonPropertyExists(json, "statsWindows");
  const bool statsWindowObjectsFromWeb = !hasLegacyStatsWindows && hasWebStatsWindows;
  const std::vector<std::string> statsWindowObjects =
      hasLegacyStatsWindows
          ? parseJsonObjectArrayProperty(json, "stats_windows")
          : hasWebStatsWindows ? parseJsonObjectArrayProperty(json, "statsWindows")
                               : std::vector<std::string>{};
  for (const std::string &object : statsWindowObjects) {
    const auto idString = parseJsonStringProperty(object, "id");
    if (statsWindowObjectsFromWeb && !idString) {
      continue;
    }
    const auto kind = parseJsonStringProperty(object, "kind");
    if (!kind) {
      continue;
    }

    const std::optional<UiStatsWindowKind> parsedKind = parseStatsWindowKind(*kind);
    if (!parsedKind) {
      continue;
    }

    UiStatsWindow window{};
    window.kind = *parsedKind;

    if (idString) {
      window.config_id = *idString;
      const size_t digitOffset = idString->find_first_of("0123456789");
      if (digitOffset != std::string::npos) {
        try {
          window.id = static_cast<uint32_t>(std::stoul(idString->substr(digitOffset)));
        } catch (const std::exception &) {
          window.id = 0;
        }
      }
    }
    if (const auto configId = parseJsonStringProperty(object, "config_id")) {
      window.config_id = *configId;
    }
    window.id = static_cast<uint32_t>(parseJsonNumberProperty(object, "id").value_or(window.id));
    if (window.id == 0) {
      window.id = nextUiStatsWindowId;
    }
    if (window.config_id.empty()) {
      window.config_id = std::format("stats-{}", window.id);
    }
    nextUiStatsWindowId = std::max(nextUiStatsWindowId, window.id + 1);
    window.open = parseJsonBoolProperty(object, "open").value_or(true);
    window.open = parseJsonBoolProperty(object, "visible").value_or(window.open);
    std::optional<std::string> placement = parseJsonObjectProperty(object, "placement");
    if (!placement && statsWindowObjectsFromWeb) {
      placement = R"({"x":8,"y":8,"viewport":{"width":1,"height":1},"visible":true})";
    }
    if (placement) {
      window.open = parseJsonBoolProperty(*placement, "visible").value_or(window.open);
      window.has_placement = true;
      window.pending_apply_placement = true;
      window.x = static_cast<float>(parseJsonNumberProperty(*placement, "x").value_or(window.x));
      window.y = static_cast<float>(parseJsonNumberProperty(*placement, "y").value_or(window.y));
      window.width =
          static_cast<float>(parseJsonNumberProperty(*placement, "width").value_or(window.width));
      window.height =
          static_cast<float>(parseJsonNumberProperty(*placement, "height").value_or(window.height));
      window.viewport_width = static_cast<float>(
          parseJsonNumberProperty(*placement, "viewport_width").value_or(window.viewport_width));
      window.viewport_height = static_cast<float>(
          parseJsonNumberProperty(*placement, "viewport_height").value_or(window.viewport_height));
      if (const auto viewport = parseJsonObjectProperty(*placement, "viewport")) {
        window.viewport_width = static_cast<float>(
            parseJsonNumberProperty(*viewport, "width").value_or(window.viewport_width));
        window.viewport_height = static_cast<float>(
            parseJsonNumberProperty(*viewport, "height").value_or(window.viewport_height));
      }
      window.z_index =
          static_cast<int>(parseJsonNumberProperty(*placement, "zIndex").value_or(window.z_index));
    }
    window.selected_player_index = static_cast<uint32_t>(
        std::max(0.0, parseJsonNumberProperty(object, "selected_player_index").value_or(0.0)));
    window.selected_player_id =
        parseJsonStringProperty(object, "selected_player_id").value_or("");
    if (const auto playerId = parseJsonStringProperty(object, "playerId")) {
      window.selected_player_id = *playerId;
      if (const auto parsedPlayerIndex = parseUnsignedIntegerString(*playerId)) {
        window.selected_player_index = *parsedPlayerIndex;
      } else if (const auto uniquePlayerIndex = uniqueIdPlayerIndices.find(*playerId);
                 uniquePlayerIndex != uniqueIdPlayerIndices.end()) {
        window.selected_player_index = uniquePlayerIndex->second;
      }
    }
    window.selected_team_is_team_0 =
        parseJsonBoolProperty(object, "selected_team_is_team_0").value_or(true) ? 1 : 0;
    if (const auto team = parseJsonStringProperty(object, "team")) {
      if (*team == "blue") {
        window.selected_team_is_team_0 = 1;
      } else if (*team == "orange") {
        window.selected_team_is_team_0 = 0;
      }
    }
    window.module_name = parseJsonStringProperty(object, "module_name").value_or("");
    window.module_view = static_cast<int>(
        std::max(0.0, parseJsonNumberProperty(object, "module_view").value_or(0.0)));
    const bool hasEntriesProperty = object.find("\"entries\"") != std::string::npos;
    for (const std::string &statId : parseJsonStringArrayProperty(object, "entries")) {
      if (statsWindowObjectsFromWeb) {
        continue;
      }
      window.entries.push_back(UiStatsWindow::Entry{normalizeUiStatId(statId), ""});
    }
    for (const std::string &entryObject : parseJsonObjectArrayProperty(object, "entries")) {
      std::string statId = statsWindowObjectsFromWeb
                               ? parseJsonStringProperty(entryObject, "statId").value_or("")
                               : parseJsonStringProperty(entryObject, "stat_id").value_or("");
      if (!statsWindowObjectsFromWeb && statId.empty()) {
        statId = parseJsonStringProperty(entryObject, "statId").value_or("");
      }
      if (statId.empty()) {
        continue;
      }
      statId = normalizeUiStatId(statId);
      std::string targetId = statsWindowObjectsFromWeb
                                 ? parseJsonStringProperty(entryObject, "targetId").value_or("")
                                 : parseJsonStringProperty(entryObject, "target_id").value_or("");
      if (!statsWindowObjectsFromWeb && targetId.empty()) {
        targetId = parseJsonStringProperty(entryObject, "targetId").value_or("");
      }
      window.entries.push_back(UiStatsWindow::Entry{statId, targetId});
    }
    if (!statsWindowObjectsFromWeb && !hasEntriesProperty && window.entries.empty() &&
        window.kind != UiStatsWindowKind::StatsModule) {
      initializeStatsWindowEntries(window);
    }
    window.has_placement =
        parseJsonBoolProperty(object, "has_placement").value_or(window.has_placement);
    window.pending_apply_placement = window.has_placement;
    window.x = static_cast<float>(parseJsonNumberProperty(object, "x").value_or(window.x));
    window.y = static_cast<float>(parseJsonNumberProperty(object, "y").value_or(window.y));
    window.width =
        static_cast<float>(parseJsonNumberProperty(object, "width").value_or(window.width));
    window.height =
        static_cast<float>(parseJsonNumberProperty(object, "height").value_or(window.height));
    window.viewport_width = static_cast<float>(
        parseJsonNumberProperty(object, "viewport_width").value_or(window.viewport_width));
    window.viewport_height = static_cast<float>(
        parseJsonNumberProperty(object, "viewport_height").value_or(window.viewport_height));
    window.z_index =
        static_cast<int>(parseJsonNumberProperty(object, "zIndex").value_or(window.z_index));
    const auto [defaultWidth, defaultHeight] = defaultStatsWindowSize(window.kind);
    if (window.width <= 0.0f) {
      window.width = defaultWidth;
    }
    if (window.height <= 0.0f) {
      window.height = defaultHeight;
    }
    nextUiWindowZIndex = std::max(nextUiWindowZIndex, window.z_index + 1);
    uiStatsWindows.push_back(std::move(window));
  }

  if (!uiStatsWindows.empty()) {
    cvarManager->log(std::format(
        "subtr-actor: loaded {} UI stats windows from {}",
        uiStatsWindows.size(),
        sourceLabel));
  }
  focusTopLoadedWindow();
  if (applyPlaybackConfig) {
    applyPlaybackConfigToReplay(sourceLabel);
  }
  lastSavedUiConfigJson = uiConfigJson();
  nextUiConfigAutosave = std::chrono::steady_clock::now() + std::chrono::seconds(2);
}
