// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
bool SubtrActorPlugin::finishAndDrainPendingEvents(std::string_view context) {
  if (!engine || !engineFinish) {
    return false;
  }

  const int32_t finishResult = engineFinish(engine);
  if (finishResult != 0) {
    cvarManager->log(std::format(
        "subtr-actor: live graph finalization failed during {}: {}",
        context,
        finishResult));
    return false;
  }

  drainPendingEvents();
  return true;
}

void SubtrActorPlugin::drainPendingEvents() {
  if (!engine || !drainEvents || !drainTeamEvents || !drainGoalContextEvents) {
    return;
  }

  SaMechanicEvent events[16];
  size_t count = 0;
  do {
    count = drainEvents(engine, events, 16);
    for (size_t i = 0; i < count; i += 1) {
      pushEventMessage(events[i]);
    }
  } while (count == 16);

  SaTeamEvent teamEvents[16];
  do {
    count = drainTeamEvents(engine, teamEvents, 16);
    for (size_t i = 0; i < count; i += 1) {
      pushTeamEventMessage(teamEvents[i]);
    }
  } while (count == 16);

  SaGoalContextEvent goalContextEvents[16];
  do {
    count = drainGoalContextEvents(engine, goalContextEvents, 16);
    for (size_t i = 0; i < count; i += 1) {
      pushGoalContextEventMessage(goalContextEvents[i]);
    }
  } while (count == 16);
}

bool SubtrActorPlugin::overlayCategoryEnabled(std::string_view category) {
  const std::string normalizedCategory = normalizeEventFilterToken(category);
  if (normalizedCategory == "mechanics") {
    auto cvar = cvarManager->getCvar("subtr_actor_overlay_mechanics_enabled");
    if (static_cast<bool>(cvar) && !cvar.getBoolValue()) {
      return false;
    }
  } else if (normalizedCategory == "team") {
    auto cvar = cvarManager->getCvar("subtr_actor_overlay_team_events_enabled");
    if (static_cast<bool>(cvar) && !cvar.getBoolValue()) {
      return false;
    }
  } else if (normalizedCategory == "goal_context") {
    auto cvar = cvarManager->getCvar("subtr_actor_overlay_goal_context_enabled");
    if (static_cast<bool>(cvar) && !cvar.getBoolValue()) {
      return false;
    }
  }

  auto filterCvar = cvarManager->getCvar("subtr_actor_overlay_event_types");
  const std::string filter =
      static_cast<bool>(filterCvar) ? filterCvar.getStringValue() : "all";
  return eventFilterAllows(filter, normalizedCategory, normalizedCategory);
}

bool SubtrActorPlugin::overlayMechanicEnabled(SaMechanicKind kind) {
  auto cvar = cvarManager->getCvar("subtr_actor_overlay_mechanics_enabled");
  if (static_cast<bool>(cvar) && !cvar.getBoolValue()) {
    return false;
  }
  auto filterCvar = cvarManager->getCvar("subtr_actor_overlay_event_types");
  const std::string filter =
      static_cast<bool>(filterCvar) ? filterCvar.getStringValue() : "all";
  return eventFilterAllows(filter, "mechanics", mechanicToken(kind));
}

std::string SubtrActorPlugin::teamLabel(uint8_t isTeam0) const {
  return isTeam0 != 0 ? "Blue" : "Orange";
}

std::string SubtrActorPlugin::playerLabel(uint32_t playerIndex, uint8_t isTeam0) const {
  const auto name = playerNamesByIndex.find(playerIndex);
  if (name != playerNamesByIndex.end() && !name->second.empty()) {
    return name->second;
  }
  const auto team = playerTeamsByIndex.find(playerIndex);
  const uint8_t labelTeam = team == playerTeamsByIndex.end() ? isTeam0 : team->second;
  return std::format("{} #{}", teamLabel(labelTeam), playerIndex + 1);
}

void SubtrActorPlugin::appendUiEvent(UiEventRecord event) {
  recentUiEvents.push_front(std::move(event));
  while (recentUiEvents.size() > MAX_RECENT_UI_EVENTS) {
    mechanicsReviewDecisions.erase(mechanicsReviewKey(recentUiEvents.back()));
    recentUiEvents.pop_back();
  }
}

bool SubtrActorPlugin::uiEventVisible(const UiEventRecord &event) {
  if (event.category == "mechanics" &&
      !cvarBool("subtr_actor_overlay_mechanics_enabled", true)) {
    return false;
  }
  if (event.category == "team" && !cvarBool("subtr_actor_overlay_team_events_enabled", true)) {
    return false;
  }
  if (event.category == "goal_context" &&
      !cvarBool("subtr_actor_overlay_goal_context_enabled", true)) {
    return false;
  }
  return eventFilterAllows(cvarString("subtr_actor_overlay_event_types", "all"), event.category, event.type);
}

void SubtrActorPlugin::pushGoalEventMessage(const SaGoalEvent &event) {
  const bool isBlue = event.scoring_team_is_team_0 != 0;
  const std::string actor =
      event.has_player != 0 ? playerLabel(event.player_index, event.scoring_team_is_team_0)
                            : teamLabel(event.scoring_team_is_team_0);
  std::string details = teamLabel(event.scoring_team_is_team_0);
  if (event.has_team_zero_score != 0 && event.has_team_one_score != 0) {
    details = std::format("Blue {} - {} Orange", event.team_zero_score, event.team_one_score);
  }
  appendUiEvent(UiEventRecord{
      "goal",
      "goal",
      actor,
      std::format("{} goal", actor),
      details,
      isBlue ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255},
      event.timing.frame_number,
      event.timing.time,
      event.has_player,
      event.player_index,
  });
}

void SubtrActorPlugin::pushEventMessage(const SaMechanicEvent &event) {
  const bool isBlue = event.is_team_0 != 0;
  const bool isCorePlayerStat = corePlayerStatMechanicKind(event.kind);
  const std::string action = event.confidence < 0.999f
                                 ? std::format(
                                       "{} ({:.0f}%)",
                                       mechanicLabel(event.kind),
                                       event.confidence * 100.0f)
                                 : mechanicLabel(event.kind);
  const std::string label =
      std::format("{} {}", playerLabel(event.player_index, event.is_team_0), action);
  appendUiEvent(UiEventRecord{
      isCorePlayerStat ? "core" : "mechanics",
      mechanicToken(event.kind),
      playerLabel(event.player_index, event.is_team_0),
      label,
      isCorePlayerStat
          ? "Shots, saves, assists"
          : event.confidence < 0.999f
                ? std::format("{:.0f}% confidence", event.confidence * 100.0f)
                : "high confidence",
      isBlue ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255},
      event.frame_number,
      event.time,
      1,
      event.player_index,
  });

  if (isCorePlayerStat) {
    return;
  }
  if (!overlayMechanicEnabled(event.kind)) {
    return;
  }

  OverlayMessage message{
      label,
      isBlue ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255},
      std::chrono::steady_clock::now() +
          std::chrono::duration_cast<std::chrono::steady_clock::duration>(
              std::chrono::duration<float>(overlayMessageSeconds())),
  };
  messages.push_back(message);
  while (messages.size() > static_cast<size_t>(overlayMaxMessages())) {
    messages.pop_front();
  }
}

void SubtrActorPlugin::pushTeamEventMessage(const SaTeamEvent &event) {
  const bool isBlue = event.is_team_0 != 0;
  const std::string title = event.kind == SaTeamEventKindRush
                                ? std::format(
                                      "{} rush {}v{}",
                                      teamLabel(event.is_team_0),
                                      event.attackers,
                                      event.defenders)
                                : std::format(
                                      "{} {}",
                                      teamLabel(event.is_team_0),
                                      teamEventLabel(event));
  const std::string action = event.confidence < 0.999f
                                 ? std::format("{} ({:.0f}%)", title, event.confidence * 100.0f)
                                 : title;
  appendUiEvent(UiEventRecord{
      "team",
      "rush",
      teamLabel(event.is_team_0),
      title,
      std::format("{:.1f}s - {:.1f}s", event.start_time, event.end_time),
      isBlue ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255},
      event.start_frame,
      event.start_time,
      0,
      0,
  });

  if (!overlayCategoryEnabled("team")) {
    return;
  }

  OverlayMessage message{
      action,
      isBlue ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255},
      std::chrono::steady_clock::now() +
          std::chrono::duration_cast<std::chrono::steady_clock::duration>(
              std::chrono::duration<float>(overlayMessageSeconds())),
  };
  messages.push_back(message);
  while (messages.size() > static_cast<size_t>(overlayMaxMessages())) {
    messages.pop_front();
  }
}

void SubtrActorPlugin::pushGoalContextEventMessage(const SaGoalContextEvent &event) {
  const bool isBlue = event.scoring_team_is_team_0 != 0;
  const std::string actor =
      event.has_scorer != 0
          ? playerLabel(event.scorer_index, event.scoring_team_is_team_0)
          : teamLabel(event.scoring_team_is_team_0);
  appendUiEvent(UiEventRecord{
      "goal_context",
      "goal_context",
      actor,
      std::format("{} {}", actor, goalContextLabel(event)),
      teamLabel(event.scoring_team_is_team_0),
      isBlue ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255},
      event.frame_number,
      event.time,
      event.has_scorer,
      event.scorer_index,
  });

  if (!overlayCategoryEnabled("goal_context")) {
    return;
  }

  OverlayMessage message{
      std::format("{}: {}", actor, goalContextLabel(event)),
      isBlue ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255},
      std::chrono::steady_clock::now() +
          std::chrono::duration_cast<std::chrono::steady_clock::duration>(
              std::chrono::duration<float>(overlayMessageSeconds())),
  };
  messages.push_back(message);
  while (messages.size() > static_cast<size_t>(overlayMaxMessages())) {
    messages.pop_front();
  }
}

void SubtrActorPlugin::pushPlayerStatEventMessage(const SaPlayerStatEvent &event) {
  const bool isBlue = event.is_team_0 != 0;
  const std::string actor = playerLabel(event.player_index, event.is_team_0);
  const std::string label =
      std::format("{} {}", actor, playerStatEventLabel(event.kind));
  appendUiEvent(UiEventRecord{
      "core",
      std::string{playerStatEventType(event.kind)},
      actor,
      label,
      "Shots, saves, assists",
      isBlue ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255},
      event.timing.frame_number,
      event.timing.time,
      1,
      event.player_index,
  });
}
