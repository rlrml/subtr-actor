// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
void SubtrActorPlugin::verifyGraphRuntime(std::vector<std::string> params) {
  if (!loaded || !engine) {
    cvarManager->log("subtr-actor: graph verification requested before engine was loaded");
    return;
  }

  const bool shouldFinish =
      std::find_if(params.begin(), params.end(), [](const std::string &param) {
        return param == "finish" || param == "finalize";
      }) != params.end();
  const bool requireEventHistory = wantsRequiredEventHistory(params);
  const bool requireGraphEvents = wantsRequiredGraphEvents(params);
  if (shouldFinish) {
    if (!engineFinish) {
      cvarManager->log(
          "subtr-actor: graph verification requested finish but finish ABI is unavailable");
      return;
    }
    const int32_t finishResult = engineFinish(engine);
    if (finishResult != 0) {
      cvarManager->log(
          std::format("subtr-actor: graph finish failed before verification: {}", finishResult));
      return;
    }
    drainPendingEvents();
  }

  bool ok = true;
  const std::string graphInfoJson =
      readNamedJsonBuffer(graphOutputJsonLen, writeGraphOutputJson, "graph_info");
  std::vector<std::string> outputNames =
      parseJsonStringArrayProperty(graphInfoJson, "graph_output_names");
  if (outputNames.empty()) {
    if (!graphInfoJson.empty()) {
      ok = false;
      cvarManager->log(
          "subtr-actor: graph verification could not read graph output names from graph_info");
    }
    outputNames.assign(VERIFY_GRAPH_OUTPUTS.begin(), VERIFY_GRAPH_OUTPUTS.end());
  }
  bool missingRequiredGraphOutput = false;
  for (const char *outputName : VERIFY_GRAPH_OUTPUTS) {
    if (!containsString(outputNames, outputName)) {
      ok = false;
      missingRequiredGraphOutput = true;
      cvarManager->log(std::format(
          "subtr-actor: graph verification graph_info missing required graph output '{}'",
          outputName));
    }
  }
  if (!missingRequiredGraphOutput) {
    cvarManager->log(std::format(
        "subtr-actor: graph_info declares all {} required graph outputs",
        VERIFY_GRAPH_OUTPUTS.size()));
  }
  std::vector<std::string> graphEventFieldNames =
      parseJsonStringArrayProperty(graphInfoJson, "graph_event_field_names");
  if (graphEventFieldNames.empty()) {
    if (!graphInfoJson.empty()) {
      ok = false;
      cvarManager->log(
          "subtr-actor: graph verification could not read graph event field names from graph_info");
    }
    graphEventFieldNames = defaultGraphEventFields();
  }
  bool missingKnownGraphEventField = false;
  for (const char *fieldName : GRAPH_EVENT_FIELDS) {
    if (!containsString(graphEventFieldNames, fieldName)) {
      ok = false;
      missingKnownGraphEventField = true;
      cvarManager->log(std::format(
          "subtr-actor: graph verification graph_info missing required graph event field '{}'",
          fieldName));
    }
  }
  if (!missingKnownGraphEventField) {
    cvarManager->log(std::format(
        "subtr-actor: graph_info declares all {} known graph event fields",
        GRAPH_EVENT_FIELDS.size()));
  }
  std::vector<std::string> requiredGraphEventFieldNames =
      parseJsonStringArrayProperty(graphInfoJson, "required_graph_event_field_names");
  if (requiredGraphEventFieldNames.empty()) {
    if (!graphInfoJson.empty()) {
      ok = false;
      cvarManager->log(
          "subtr-actor: graph verification could not read required graph event field names from graph_info");
    }
    requiredGraphEventFieldNames = defaultRequiredGraphEventFields();
  }
  bool missingKnownRequiredGraphEventField = false;
  for (const char *fieldName : REQUIRED_GRAPH_EVENT_FIELDS) {
    if (!containsString(requiredGraphEventFieldNames, fieldName)) {
      ok = false;
      missingKnownRequiredGraphEventField = true;
      cvarManager->log(std::format(
          "subtr-actor: graph verification graph_info missing strict graph event field '{}'",
          fieldName));
    }
  }
  if (!missingKnownRequiredGraphEventField) {
    cvarManager->log(std::format(
        "subtr-actor: graph_info declares all {} strict graph event fields",
        REQUIRED_GRAPH_EVENT_FIELDS.size()));
  }
  bool requiredGraphEventFieldNotDeclared = false;
  for (const std::string &fieldName : requiredGraphEventFieldNames) {
    if (!containsString(graphEventFieldNames, fieldName)) {
      ok = false;
      requiredGraphEventFieldNotDeclared = true;
      cvarManager->log(std::format(
          "subtr-actor: graph verification required graph event field '{}' is not declared",
          fieldName));
    }
  }
  if (!requiredGraphEventFieldNotDeclared) {
    cvarManager->log(std::format(
        "subtr-actor: graph_info declares {} graph event fields and {} required graph event fields",
        graphEventFieldNames.size(),
        requiredGraphEventFieldNames.size()));
  }
  std::vector<std::string> eventHistoryFieldNames =
      parseJsonStringArrayProperty(graphInfoJson, "event_history_field_names");
  if (eventHistoryFieldNames.empty()) {
    if (!graphInfoJson.empty()) {
      ok = false;
      cvarManager->log(
          "subtr-actor: graph verification could not read event_history field names from graph_info");
    }
    eventHistoryFieldNames = defaultEventHistoryFields();
  }
  bool missingKnownEventHistoryField = false;
  for (const char *fieldName : FRAME_EVENTS_STATE_EVENT_FIELDS) {
    if (!containsString(eventHistoryFieldNames, fieldName)) {
      ok = false;
      missingKnownEventHistoryField = true;
      cvarManager->log(std::format(
          "subtr-actor: graph verification graph_info missing required event_history field '{}'",
          fieldName));
    }
  }
  if (!missingKnownEventHistoryField) {
    cvarManager->log(std::format(
        "subtr-actor: graph_info declares all {} known event_history fields",
        FRAME_EVENTS_STATE_EVENT_FIELDS.size()));
  }
  std::vector<std::string> requiredEventHistoryFieldNames =
      parseJsonStringArrayProperty(graphInfoJson, "required_event_history_field_names");
  if (requiredEventHistoryFieldNames.empty()) {
    if (!graphInfoJson.empty()) {
      ok = false;
      cvarManager->log(
          "subtr-actor: graph verification could not read required event_history field names from graph_info");
    }
    requiredEventHistoryFieldNames = defaultRequiredEventHistoryFields();
  }
  bool missingKnownRequiredEventHistoryField = false;
  for (const char *fieldName : REQUIRED_EVENT_HISTORY_FIELDS) {
    if (!containsString(requiredEventHistoryFieldNames, fieldName)) {
      ok = false;
      missingKnownRequiredEventHistoryField = true;
      cvarManager->log(std::format(
          "subtr-actor: graph verification graph_info missing strict cumulative event_history field '{}'",
          fieldName));
    }
  }
  if (!missingKnownRequiredEventHistoryField) {
    cvarManager->log(std::format(
        "subtr-actor: graph_info declares all {} strict cumulative event_history fields",
        REQUIRED_EVENT_HISTORY_FIELDS.size()));
  }
  bool requiredEventHistoryFieldNotDeclared = false;
  for (const std::string &fieldName : requiredEventHistoryFieldNames) {
    if (!containsString(eventHistoryFieldNames, fieldName)) {
      ok = false;
      requiredEventHistoryFieldNotDeclared = true;
      cvarManager->log(std::format(
          "subtr-actor: graph verification required event_history field '{}' is not declared",
          fieldName));
    }
  }
  if (!requiredEventHistoryFieldNotDeclared) {
    cvarManager->log(std::format(
        "subtr-actor: graph_info declares {} event_history fields and {} required cumulative event fields",
        eventHistoryFieldNames.size(),
        requiredEventHistoryFieldNames.size()));
  }

  std::string analysisNodesJson;
  std::string graphEventsJson;
  std::string eventHistoryJson;
  for (const std::string &outputName : outputNames) {
    const std::string outputJson =
        readNamedJsonBuffer(graphOutputJsonLen, writeGraphOutputJson, outputName);
    if (outputJson.empty()) {
      ok = false;
      cvarManager->log(std::format(
          "subtr-actor: graph verification missing graph output '{}'", outputName));
      continue;
    }
    cvarManager->log(std::format(
        "subtr-actor: graph output '{}' callable ({} bytes)",
        outputName,
        outputJson.size()));
    std::string fixedOutputJson;
    if (outputName == "events") {
      fixedOutputJson = readJsonBuffer(eventsJsonLen, writeEventsJson);
    } else if (outputName == "frame") {
      fixedOutputJson = readJsonBuffer(frameJsonLen, writeFrameJson);
    } else if (outputName == "timeline") {
      fixedOutputJson = readJsonBuffer(timelineJsonLen, writeTimelineJson);
    } else if (outputName == "stats") {
      fixedOutputJson = readJsonBuffer(statsJsonLen, writeStatsJson);
    } else if (outputName == "graph_info") {
      fixedOutputJson = readJsonBuffer(graphInfoJsonLen, writeGraphInfoJson);
    }
    if (!fixedOutputJson.empty()) {
      if (fixedOutputJson != outputJson) {
        ok = false;
        cvarManager->log(std::format(
            "subtr-actor: graph verification fixed ABI output '{}' differs from named output",
            outputName));
      } else {
        cvarManager->log(std::format(
            "subtr-actor: graph output '{}' matches fixed ABI",
            outputName));
      }
    }
    if (outputName == "events") {
      graphEventsJson = outputJson;
    } else if (outputName == "analysis_nodes") {
      analysisNodesJson = outputJson;
    } else if (outputName == "event_history") {
      eventHistoryJson = outputJson;
    }
  }
  if (!outputNames.empty()) {
    cvarManager->log(std::format(
        "subtr-actor: verified {} graph outputs by name",
        outputNames.size()));
  }

  std::vector<std::string> graphEventKeys = parseJsonObjectKeys(graphEventsJson);
  if (graphEventKeys.empty()) {
    ok = false;
    cvarManager->log(
        "subtr-actor: graph verification could not inspect events graph output fields");
  } else {
    std::sort(graphEventKeys.begin(), graphEventKeys.end());
    bool missingGraphEventField = false;
    bool missingRequiredGraphEvent = false;
    for (const std::string &fieldName : graphEventFieldNames) {
      if (!std::binary_search(graphEventKeys.begin(), graphEventKeys.end(), fieldName)) {
        ok = false;
        missingGraphEventField = true;
        if (requireGraphEvents && containsString(requiredGraphEventFieldNames, fieldName)) {
          missingRequiredGraphEvent = true;
        }
        cvarManager->log(std::format(
            "subtr-actor: graph verification events output missing graph event field '{}'",
            fieldName));
        continue;
      }
      const auto eventCount = parseJsonArrayPropertyElementCount(graphEventsJson, fieldName);
      if (!eventCount) {
        ok = false;
        if (requireGraphEvents && containsString(requiredGraphEventFieldNames, fieldName)) {
          missingRequiredGraphEvent = true;
        }
        cvarManager->log(std::format(
            "subtr-actor: graph verification events output field '{}' is not an array",
            fieldName));
        continue;
      }
      cvarManager->log(std::format(
          "subtr-actor: events output field '{}' has {} entries",
          fieldName,
          *eventCount));
      if (requireGraphEvents && containsString(requiredGraphEventFieldNames, fieldName) &&
          *eventCount == 0) {
        ok = false;
        missingRequiredGraphEvent = true;
        cvarManager->log(std::format(
            "subtr-actor: graph verification events required graph event field '{}' has no entries",
            fieldName));
      }
    }
    if (!missingGraphEventField) {
      cvarManager->log(std::format(
          "subtr-actor: events output exposes {} graph event fields",
          graphEventFieldNames.size()));
    }
    if (requireGraphEvents && !requiredGraphEventFieldNotDeclared &&
        !missingRequiredGraphEvent && !missingGraphEventField) {
      cvarManager->log(
          "subtr-actor: events required graph event fields are nonzero");
    }
  }

  const std::vector<std::string> moduleNames =
      parseJsonStringArrayProperty(graphInfoJson, "builtin_stats_module_names");
  if (moduleNames.empty()) {
    ok = false;
    cvarManager->log(
        "subtr-actor: graph verification could not read builtin stats module names from graph_info");
  }
  for (const std::string &moduleName : moduleNames) {
    const std::string moduleJson =
        readNamedJsonBuffer(statsModuleJsonLen, writeStatsModuleJson, moduleName);
    const std::string frameJson =
        readNamedJsonBuffer(statsModuleFrameJsonLen, writeStatsModuleFrameJson, moduleName);
    const std::string configJson =
        readNamedJsonBuffer(statsModuleConfigJsonLen, writeStatsModuleConfigJson, moduleName);
    if (moduleJson.empty() || frameJson.empty() || configJson.empty()) {
      ok = false;
      cvarManager->log(std::format(
          "subtr-actor: graph verification missing stats module '{}' output: module={} frame={} config={}",
          moduleName,
          moduleJson.size(),
          frameJson.size(),
          configJson.size()));
      continue;
    }
    cvarManager->log(std::format(
        "subtr-actor: stats module '{}' callable (module={} frame={} config={} bytes)",
        moduleName,
        moduleJson.size(),
        frameJson.size(),
        configJson.size()));
  }
  if (!moduleNames.empty()) {
    cvarManager->log(std::format(
        "subtr-actor: verified {} builtin stats modules by name",
        moduleNames.size()));
  }

  const std::string nodeNamesJson =
      readJsonBuffer(analysisNodeNamesJsonLen, writeAnalysisNodeNamesJson);
  const std::vector<std::string> nodeNames = parseJsonStringArray(nodeNamesJson);
  const std::vector<std::string> graphInfoNodeNames =
      parseJsonStringArrayProperty(graphInfoJson, "callable_analysis_node_names");
  const std::vector<std::string> resolvedGraphNodeNames =
      parseJsonStringArrayProperty(graphInfoJson, "node_names");
  if (nodeNames.empty()) {
    ok = false;
    cvarManager->log(
        "subtr-actor: graph verification could not read callable analysis node names");
  }
  if (graphInfoNodeNames.empty()) {
    ok = false;
    cvarManager->log(
        "subtr-actor: graph verification could not read callable analysis node names from graph_info");
  } else if (!nodeNames.empty() && graphInfoNodeNames != nodeNames) {
    ok = false;
    cvarManager->log(std::format(
        "subtr-actor: graph verification callable analysis node registry mismatch: graph_info={} names_abi={}",
        graphInfoNodeNames.size(),
        nodeNames.size()));
  } else if (!nodeNames.empty()) {
    cvarManager->log(
        "subtr-actor: callable analysis node registry matches graph_info");
  }

  if (resolvedGraphNodeNames.empty()) {
    ok = false;
    cvarManager->log(
        "subtr-actor: graph verification could not read resolved graph node names from graph_info");
  } else if (!nodeNames.empty()) {
    std::vector<std::string> sortedNodeNames = nodeNames;
    std::sort(sortedNodeNames.begin(), sortedNodeNames.end());
    bool missingResolvedNode = false;
    for (const std::string &resolvedNodeName : resolvedGraphNodeNames) {
      if (!std::binary_search(
              sortedNodeNames.begin(), sortedNodeNames.end(), resolvedNodeName)) {
        ok = false;
        missingResolvedNode = true;
        cvarManager->log(std::format(
            "subtr-actor: graph verification resolved node '{}' is not callable by name",
            resolvedNodeName));
      }
    }
    if (!missingResolvedNode) {
      cvarManager->log(std::format(
          "subtr-actor: all {} resolved analysis graph nodes are callable by name",
          resolvedGraphNodeNames.size()));
    }
  }

  if (analysisNodesJson.empty()) {
    ok = false;
    cvarManager->log(
        "subtr-actor: graph verification could not inspect analysis_nodes output");
  } else if (!nodeNames.empty()) {
    std::vector<std::string> analysisNodeKeys = parseJsonObjectKeys(analysisNodesJson);
    if (analysisNodeKeys.empty()) {
      ok = false;
      cvarManager->log(
          "subtr-actor: graph verification could not parse analysis_nodes output keys");
    } else {
      std::vector<std::string> sortedNodeNames = nodeNames;
      std::sort(sortedNodeNames.begin(), sortedNodeNames.end());
      std::sort(analysisNodeKeys.begin(), analysisNodeKeys.end());
      bool nodeSetMismatch = false;
      for (const std::string &nodeName : nodeNames) {
        if (!std::binary_search(analysisNodeKeys.begin(), analysisNodeKeys.end(), nodeName)) {
          ok = false;
          nodeSetMismatch = true;
          cvarManager->log(std::format(
              "subtr-actor: graph verification analysis_nodes output missing callable node '{}'",
              nodeName));
        }
      }
      for (const std::string &nodeName : analysisNodeKeys) {
        if (!std::binary_search(sortedNodeNames.begin(), sortedNodeNames.end(), nodeName)) {
          ok = false;
          nodeSetMismatch = true;
          cvarManager->log(std::format(
              "subtr-actor: graph verification analysis_nodes output has unexpected node '{}'",
              nodeName));
        }
      }
      if (!nodeSetMismatch) {
        cvarManager->log(std::format(
            "subtr-actor: analysis_nodes output contains {} callable analysis nodes exactly",
            nodeNames.size()));
      }
    }
  }

  for (const std::string &nodeName : nodeNames) {
    const std::string nodeJson =
        readNamedJsonBuffer(analysisNodeJsonLen, writeAnalysisNodeJson, nodeName);
    if (nodeJson.empty()) {
      ok = false;
      cvarManager->log(std::format(
          "subtr-actor: graph verification missing analysis node '{}'", nodeName));
      continue;
    }
    cvarManager->log(std::format(
        "subtr-actor: analysis node '{}' callable ({} bytes)",
        nodeName,
        nodeJson.size()));
  }
  if (!nodeNames.empty()) {
    cvarManager->log(std::format(
        "subtr-actor: verified {} callable analysis nodes by name",
        nodeNames.size()));
  }

  const std::string frameEventsJson =
      readNamedJsonBuffer(analysisNodeJsonLen, writeAnalysisNodeJson, FRAME_EVENTS_STATE_NODE);
  std::vector<std::string> frameEventKeys = parseJsonObjectKeys(frameEventsJson);
  if (frameEventKeys.empty()) {
    ok = false;
    cvarManager->log(
        "subtr-actor: graph verification could not inspect frame_events_state event fields");
  } else {
    std::sort(frameEventKeys.begin(), frameEventKeys.end());
    bool missingEventField = false;
    for (const std::string &fieldName : eventHistoryFieldNames) {
      if (!std::binary_search(frameEventKeys.begin(), frameEventKeys.end(), fieldName)) {
        ok = false;
        missingEventField = true;
        cvarManager->log(std::format(
            "subtr-actor: graph verification frame_events_state missing event field '{}'",
            fieldName));
        continue;
      }
      const auto eventCount = parseJsonArrayPropertyElementCount(frameEventsJson, fieldName);
      if (!eventCount) {
        ok = false;
        cvarManager->log(std::format(
            "subtr-actor: graph verification frame_events_state event field '{}' is not an array",
            fieldName));
        continue;
      }
      cvarManager->log(std::format(
          "subtr-actor: frame_events_state event field '{}' has {} entries",
          fieldName,
          *eventCount));
    }
    if (!missingEventField) {
      cvarManager->log(std::format(
          "subtr-actor: frame_events_state exposes {} live event fields",
          eventHistoryFieldNames.size()));
    }
  }

  std::vector<std::string> eventHistoryKeys = parseJsonObjectKeys(eventHistoryJson);
  if (eventHistoryKeys.empty()) {
    ok = false;
    cvarManager->log(
        "subtr-actor: graph verification could not inspect event_history event fields");
  } else {
    std::sort(eventHistoryKeys.begin(), eventHistoryKeys.end());
    bool missingEventHistoryField = false;
    bool missingRequiredEventHistory = false;
    for (const std::string &fieldName : eventHistoryFieldNames) {
      if (!std::binary_search(eventHistoryKeys.begin(), eventHistoryKeys.end(), fieldName)) {
        ok = false;
        missingEventHistoryField = true;
        if (requireEventHistory && containsString(requiredEventHistoryFieldNames, fieldName)) {
          missingRequiredEventHistory = true;
        }
        cvarManager->log(std::format(
            "subtr-actor: graph verification event_history missing event field '{}'",
            fieldName));
        continue;
      }
      const auto eventCount = parseJsonArrayPropertyElementCount(eventHistoryJson, fieldName);
      if (!eventCount) {
        ok = false;
        if (requireEventHistory && containsString(requiredEventHistoryFieldNames, fieldName)) {
          missingRequiredEventHistory = true;
        }
        cvarManager->log(std::format(
            "subtr-actor: graph verification event_history event field '{}' is not an array",
            fieldName));
        continue;
      }
      cvarManager->log(std::format(
          "subtr-actor: event_history event field '{}' has {} cumulative entries",
          fieldName,
          *eventCount));
      if (requireEventHistory && containsString(requiredEventHistoryFieldNames, fieldName) &&
          *eventCount == 0) {
        ok = false;
        missingRequiredEventHistory = true;
        cvarManager->log(std::format(
            "subtr-actor: graph verification event_history required event field '{}' has no cumulative entries",
            fieldName));
      }
    }
    if (!missingEventHistoryField) {
      cvarManager->log(std::format(
          "subtr-actor: event_history exposes {} cumulative live event fields",
          eventHistoryFieldNames.size()));
    }
    if (requireEventHistory && !requiredEventHistoryFieldNotDeclared &&
        !missingRequiredEventHistory && !missingEventHistoryField) {
      cvarManager->log(
          "subtr-actor: event_history required cumulative event fields are nonzero");
    }
  }

  cvarManager->log(ok
                       ? "subtr-actor: graph verification passed"
                       : "subtr-actor: graph verification failed; enter gameplay/replay and try again");
}

void SubtrActorPlugin::selfTestGraphRuntime(std::vector<std::string> params) {
  if (!loaded || !engineCreate || !engineDestroy || !processFrame || !engineFinish ||
      !graphOutputJsonLen || !writeGraphOutputJson) {
    cvarManager->log("subtr-actor: graph self-test requested before ABI was loaded");
    return;
  }
  const bool shouldDump =
      std::find_if(params.begin(), params.end(), [](const std::string &param) {
        return param == "dump" || param == "write_dump" || param == "write-dump";
      }) != params.end();

  SaEngine *selfTestEngine = engineCreate();
  if (!selfTestEngine) {
    cvarManager->log("subtr-actor: graph self-test failed to create temporary engine");
    return;
  }

  std::array<SaPlayerFrame, 2> players{
      syntheticPlayer(0, "self-test-blue", 1, 0.0f, 0.0f, 92.75f),
      syntheticPlayer(1, "self-test-orange", 0, 120.0f, 0.0f, 92.75f),
  };
  std::array<SaTouchEvent, 1> touches{SaTouchEvent{
      syntheticTiming(1, 0.1f),
      0,
      1,
      1,
      12.0f,
      1,
  }};
  std::array<SaDodgeRefreshedEvent, 1> dodgeRefreshes{SaDodgeRefreshedEvent{
      syntheticTiming(1, 0.1f),
      0,
      1,
      1,
  }};
  std::array<SaBoostPadEvent, 1> boostPadEvents{SaBoostPadEvent{
      syntheticTiming(1, 0.1f),
      34,
      SaBoostPadEventKindPickedUp,
      1,
      0,
      1,
  }};
  std::array<SaGoalEvent, 1> goals{SaGoalEvent{
      syntheticTiming(1, 0.1f),
      1,
      0,
      1,
      1,
      1,
      0,
      1,
  }};
  const SaRigidBody shotBall = syntheticRigidBody(300.0f, 100.0f, 120.0f, 1000.0f, 500.0f, 100.0f);
  const SaRigidBody shotPlayer = syntheticRigidBody(240.0f, 90.0f, 92.75f, 800.0f, 300.0f, 0.0f);
  std::array<SaPlayerStatEvent, 3> playerStatEvents{
      SaPlayerStatEvent{
          syntheticTiming(1, 0.1f),
          0,
          1,
          SaPlayerStatEventKindShot,
          1,
          shotBall,
          1,
          shotPlayer,
      },
      SaPlayerStatEvent{
          syntheticTiming(1, 0.1f),
          1,
          0,
          SaPlayerStatEventKindSave,
          0,
          SaRigidBody{},
          0,
          SaRigidBody{},
      },
      SaPlayerStatEvent{
          syntheticTiming(1, 0.1f),
          0,
          1,
          SaPlayerStatEventKindAssist,
          0,
          SaRigidBody{},
          0,
          SaRigidBody{},
      },
  };
  std::array<SaDemolishEvent, 1> demolishes{SaDemolishEvent{
      syntheticTiming(1, 0.1f),
      0,
      1,
      SaVec3{2300.0f, 0.0f, 0.0f},
      SaVec3{0.0f, 0.0f, 0.0f},
      SaVec3{120.0f, 0.0f, 92.75f},
      DEMO_ACTIVE_DURATION_SECONDS,
  }};

  std::array<SaLiveFrame, 3> frames{};
  for (size_t index = 0; index < frames.size(); index += 1) {
    const uint64_t frameNumber = static_cast<uint64_t>(index + 1);
    SaLiveFrame &frame = frames[index];
    frame.frame_number = frameNumber;
    frame.time = 0.1f * static_cast<float>(frameNumber);
    frame.dt = index == 0 ? 0.0f : 0.1f;
    frame.seconds_remaining = 300;
    frame.has_seconds_remaining = 1;
    frame.ball_has_been_hit = 1;
    frame.has_ball_has_been_hit = 1;
    frame.team_zero_score = 1;
    frame.has_team_zero_score = 1;
    frame.team_one_score = 0;
    frame.has_team_one_score = 1;
    frame.possession_team_is_team_0 = 1;
    frame.has_possession_team = 1;
    frame.scored_on_team_is_team_0 = 0;
    frame.has_scored_on_team = 1;
    frame.live_play = 1;
    frame.has_live_play = 1;
    frame.has_ball = 1;
    frame.ball = syntheticRigidBody(25.0f * static_cast<float>(frameNumber), 0.0f, 120.0f);
    frame.players = players.data();
    frame.player_count = players.size();
  }
  frames[0].touches = touches.data();
  frames[0].touch_count = touches.size();
  frames[0].dodge_refreshes = dodgeRefreshes.data();
  frames[0].dodge_refresh_count = dodgeRefreshes.size();
  frames[0].boost_pad_events = boostPadEvents.data();
  frames[0].boost_pad_event_count = boostPadEvents.size();
  frames[0].goals = goals.data();
  frames[0].goal_count = goals.size();
  frames[0].player_stat_events = playerStatEvents.data();
  frames[0].player_stat_event_count = playerStatEvents.size();
  frames[0].demolishes = demolishes.data();
  frames[0].demolish_count = demolishes.size();

  SaEngine *liveEngine = engine;
  const auto liveMessages = messages;
  engine = selfTestEngine;
  bool processed = true;
  for (const SaLiveFrame &frame : frames) {
    const int32_t result = processFrame(engine, &frame);
    if (result != 0) {
      processed = false;
      cvarManager->log(std::format(
          "subtr-actor: graph self-test frame {} failed: {}",
          frame.frame_number,
          result));
      break;
    }
  }

  if (processed) {
    const std::string eventHistoryJson =
        readNamedJsonBuffer(graphOutputJsonLen, writeGraphOutputJson, "event_history");
    const auto activeDemoCount =
        parseJsonArrayPropertyElementCount(eventHistoryJson, "active_demos");
    if (!activeDemoCount || *activeDemoCount == 0) {
      processed = false;
      cvarManager->log(
          "subtr-actor: graph self-test failed to derive active_demos from demolish event");
    } else {
      cvarManager->log(std::format(
          "subtr-actor: graph self-test derived active_demos from demolish event ({} entries)",
          *activeDemoCount));
    }
  }

  if (processed) {
    cvarManager->log(
        "subtr-actor: graph self-test fed every required event family");
    verifyGraphRuntime({"finish", "require_event_history", "require_graph_events"});
    if (shouldDump) {
      cvarManager->log("subtr-actor: graph self-test writing synthetic graph dump");
      dumpGraphJson({"subtr_actor_dump_graph", "finish"});
    }
  }
  messages = liveMessages;
  engine = liveEngine;
  engineDestroy(selfTestEngine);
}
