// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
void SubtrActorPlugin::renderPlayerStatsTable(
    UiStatsWindow &window,
    const SaPlayerFrame &player) {
  for (size_t i = 0; i < window.entries.size();) {
    const std::string &statId = window.entries[i].stat_id;
    if (!statsWindowSupportsStat(window, statId)) {
      ++i;
      continue;
    }
    const std::string statLabel = uiStatLabel(statId);
    const std::string statValue = playerStatValue(player, statId);
    if (renderStatsWindowValueRow(window, i, statLabel, statValue)) {
      return;
    }
    ++i;
  }
}

void SubtrActorPlugin::renderTeamStatsTable(UiStatsWindow &window, uint8_t isTeam0) {
  for (size_t i = 0; i < window.entries.size();) {
    const std::string &statId = window.entries[i].stat_id;
    if (!statsWindowSupportsStat(window, statId)) {
      ++i;
      continue;
    }
    const std::string statLabel = uiStatLabel(statId);
    const std::string statValue = teamStatValue(isTeam0, statId);
    if (renderStatsWindowValueRow(window, i, statLabel, statValue)) {
      return;
    }
    ++i;
  }
}

void SubtrActorPlugin::renderAllPlayersStatsTable(UiStatsWindow &window) {
  if (sampledPlayers.empty()) {
    renderStatsWindowEmpty("Load a replay to show stats.");
    return;
  }

  auto renderTeamGroup = [&](uint8_t isTeam0) {
    const LinearColor color =
        isTeam0 != 0 ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255};
    const ImVec4 teamColor = toImVec4(color);
    const size_t playerCount = static_cast<size_t>(std::count_if(
        sampledPlayers.begin(),
        sampledPlayers.end(),
        [isTeam0](const SaPlayerFrame &player) { return player.is_team_0 == isTeam0; }));
    if (playerCount == 0) {
      return false;
    }

    ImGui::Spacing();
    const std::string teamTitle = std::format("{} team", teamLabel(isTeam0));
    const std::string teamMeta =
        std::format("{} player{}", playerCount, playerCount == 1 ? "" : "s");
    ImGui::TextColored(teamColor, "%s", teamTitle.c_str());
    const float metaX = std::max(
        ImGui::GetCursorPosX(),
        ImGui::GetWindowContentRegionMax().x - ImGui::CalcTextSize(teamMeta.c_str()).x);
    ImGui::SameLine(metaX);
    ImGui::TextColored(teamColor, "%s", teamMeta.c_str());
    ImGui::PushStyleColor(ImGuiCol_Separator, teamColor);
    ImGui::Separator();
    ImGui::PopStyleColor();
    for (const SaPlayerFrame &player : sampledPlayers) {
      if (player.is_team_0 != isTeam0) {
        continue;
      }

      ImGui::PushID(static_cast<int>(player.player_index));
      const std::string playerName = playerLabel(player.player_index, player.is_team_0);
      ImDrawList *drawList = ImGui::GetWindowDrawList();
      const ImVec2 entityStart = ImGui::GetCursorScreenPos();
      drawList->AddRectFilled(
          ImVec2{entityStart.x, entityStart.y + 2.0f},
          ImVec2{entityStart.x + 2.0f, entityStart.y + ImGui::GetTextLineHeight() + 2.0f},
          ImGui::GetColorU32(teamColor));
      ImGui::Indent(8.0f);
      ImGui::TextColored(teamColor, "%s", playerName.c_str());
      for (size_t i = 0; i < window.entries.size();) {
        const std::string &statId = window.entries[i].stat_id;
        if (!statsWindowSupportsStat(window, statId)) {
          ++i;
          continue;
        }
        const std::string statLabel = uiStatLabel(statId);
        const std::string statValue = playerStatValue(player, statId);
        if (renderStatsWindowValueRow(
                window, i, statLabel, statValue, std::format("player-{}", player.player_index))) {
          ImGui::Unindent(8.0f);
          ImGui::PopID();
          return true;
        }
        ++i;
      }
      ImGui::Unindent(8.0f);
      ImGui::Spacing();
      ImGui::PopID();
    }
    return false;
  };

  if (renderTeamGroup(1)) {
    return;
  }
  if (renderTeamGroup(0)) {
    return;
  }
}

void SubtrActorPlugin::renderAllTeamsStatsTable(UiStatsWindow &window) {
  for (const uint8_t isTeam0 : {static_cast<uint8_t>(1), static_cast<uint8_t>(0)}) {
    const LinearColor color =
        isTeam0 != 0 ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255};
    const ImVec4 teamColor = toImVec4(color);
    ImDrawList *drawList = ImGui::GetWindowDrawList();
    const ImVec2 entityStart = ImGui::GetCursorScreenPos();
    drawList->AddRectFilled(
        ImVec2{entityStart.x, entityStart.y + 2.0f},
        ImVec2{entityStart.x + 2.0f, entityStart.y + ImGui::GetTextLineHeight() + 2.0f},
        ImGui::GetColorU32(teamColor));
    ImGui::Indent(8.0f);
    ImGui::TextColored(teamColor, "%s", teamLabel(isTeam0).c_str());
    for (size_t i = 0; i < window.entries.size();) {
      const std::string &statId = window.entries[i].stat_id;
      if (!statsWindowSupportsStat(window, statId)) {
        ++i;
        continue;
      }
      const std::string statLabel = uiStatLabel(statId);
      const std::string statValue = teamStatValue(isTeam0, statId);
      if (renderStatsWindowValueRow(
              window, i, statLabel, statValue, std::format("team-{}", isTeam0))) {
        ImGui::Unindent(8.0f);
        return;
      }
      ++i;
    }
    ImGui::Unindent(8.0f);
    ImGui::Spacing();
  }
}

void SubtrActorPlugin::renderGoalsOverviewStats(UiStatsWindow &window) {
  (void)window;
  auto isGoalTagEvent = [](const UiEventRecord &event) {
    if (event.category != "mechanics") {
      return false;
    }
    return event.type == "aerial_goal" || event.type == "high_aerial_goal" ||
           event.type == "long_distance_goal" || event.type == "own_half_goal" ||
           event.type == "empty_net_goal" || event.type == "counter_attack_goal" ||
           event.type == "flick_goal" || event.type == "double_tap_goal" ||
           event.type == "one_timer_goal" || event.type == "passing_goal" ||
           event.type == "air_dribble_goal" || event.type == "flip_reset_goal" ||
           event.type == "half_volley_goal";
  };
  struct GoalTagChip {
    std::string label;
    float confidence = 1.0f;
    std::string text;
  };
  auto goalTagChip = [](const UiEventRecord &event) {
    std::string label = event.label;
    if (!event.actor.empty()) {
      const std::string actorPrefix = event.actor + " ";
      if (label.rfind(actorPrefix, 0) == 0) {
        label = label.substr(actorPrefix.size());
      }
    }

    std::string confidence = "100%";
    float confidenceValue = 1.0f;
    const size_t parenthesizedConfidence = label.rfind(" (");
    if (parenthesizedConfidence != std::string::npos && !label.empty() &&
        label.back() == ')' &&
        label.find('%', parenthesizedConfidence) != std::string::npos) {
      confidence = label.substr(
          parenthesizedConfidence + 2,
          label.size() - parenthesizedConfidence - 3);
      label = label.substr(0, parenthesizedConfidence);
    } else if (const size_t percent = event.details.find('%');
               percent != std::string::npos) {
      const size_t start = event.details.rfind(' ', percent);
      confidence = event.details.substr(start == std::string::npos ? 0 : start + 1, percent + 1);
    }
    try {
      confidenceValue = std::clamp(std::stof(confidence) / 100.0f, 0.0f, 1.0f);
    } catch (const std::exception &) {
      confidenceValue = 1.0f;
    }

    return GoalTagChip{label, confidenceValue, std::format("{} {}", label, confidence)};
  };
  auto goalTagsForEvent = [&](const UiEventRecord &goalEvent) {
    std::vector<GoalTagChip> tags;
    for (const UiEventRecord &candidate : recentUiEvents) {
      if (!isGoalTagEvent(candidate)) {
        continue;
      }
      const bool samePlayer = goalEvent.has_player == 0 || candidate.has_player == 0 ||
                              goalEvent.player_index == candidate.player_index;
      const bool sameFrame = candidate.frame_number == goalEvent.frame_number;
      const bool nearbyTime = std::fabs(candidate.time - goalEvent.time) <= 0.25f;
      if (samePlayer && (sameFrame || nearbyTime)) {
        tags.push_back(goalTagChip(candidate));
      }
    }
    std::sort(tags.begin(), tags.end(), [](const GoalTagChip &left, const GoalTagChip &right) {
      if (left.label == right.label) {
        return left.confidence > right.confidence;
      }
      return left.label < right.label;
    });
    std::vector<std::string> tagTexts;
    for (const GoalTagChip &tag : tags) {
      if (!containsString(tagTexts, tag.text)) {
        tagTexts.push_back(tag.text);
      }
    }
    return tagTexts;
  };

  std::vector<size_t> goalEventIndexes;
  goalEventIndexes.reserve(recentUiEvents.size());
  for (size_t index = 0; index < recentUiEvents.size(); index += 1) {
    const UiEventRecord &event = recentUiEvents[index];
    if (event.category == "goal_context" || event.type == "goal") {
      goalEventIndexes.push_back(index);
    }
  }
  for (size_t index = 0; index < recentUiEvents.size(); index += 1) {
    const UiEventRecord &event = recentUiEvents[index];
    if (!isGoalTagEvent(event)) {
      continue;
    }
    const bool hasMatchingGoalEvent = std::any_of(
        goalEventIndexes.begin(),
        goalEventIndexes.end(),
        [&](size_t goalIndex) {
          const UiEventRecord &goalEvent = recentUiEvents[goalIndex];
          const bool samePlayer = goalEvent.has_player == 0 || event.has_player == 0 ||
                                  goalEvent.player_index == event.player_index;
          const bool sameFrame = goalEvent.frame_number == event.frame_number;
          const bool nearbyTime = std::fabs(goalEvent.time - event.time) <= 0.25f;
          return samePlayer && (sameFrame || nearbyTime);
        });
    if (!hasMatchingGoalEvent) {
      goalEventIndexes.push_back(index);
    }
  }
  std::sort(goalEventIndexes.begin(), goalEventIndexes.end(), [&](size_t left, size_t right) {
    const UiEventRecord &leftEvent = recentUiEvents[left];
    const UiEventRecord &rightEvent = recentUiEvents[right];
    if (leftEvent.time == rightEvent.time) {
      return leftEvent.frame_number < rightEvent.frame_number;
    }
    return leftEvent.time < rightEvent.time;
  });

  ReplayServerWrapper replayServer = gameWrapper->GetGameEventAsReplay();
  const bool hasReplayServer = !replayServer.IsNull();
  auto renderGoalTagChip = [](std::string_view text, bool empty, size_t chipIndex) {
    const ImVec4 background = empty ? ImVec4{0.28f, 0.31f, 0.35f, 0.72f}
                                    : ImVec4{0.12f, 0.29f, 0.46f, 0.82f};
    const ImVec4 foreground = empty ? ImVec4{0.75f, 0.80f, 0.84f, 1.0f}
                                    : ImVec4{0.81f, 0.90f, 1.0f, 1.0f};
    const std::string label = std::format("{}##goal-tag-chip-{}", text, chipIndex);
    ImGui::PushStyleColor(ImGuiCol_Button, background);
    ImGui::PushStyleColor(ImGuiCol_ButtonHovered, background);
    ImGui::PushStyleColor(ImGuiCol_ButtonActive, background);
    ImGui::PushStyleColor(ImGuiCol_Text, foreground);
    ImGui::SmallButton(label.c_str());
    ImGui::PopStyleColor(4);
  };
  for (size_t ordinal = 0; ordinal < goalEventIndexes.size(); ordinal += 1) {
    const size_t index = goalEventIndexes[ordinal];
    const UiEventRecord &event = recentUiEvents[index];
    const float seekTime = std::max(0.0f, event.time - GOAL_WATCH_LEAD_SECONDS);
    ImGui::PushID(static_cast<int>(index));
    const float buttonPadding = ImGui::GetStyle().FramePadding.x * 2.0f;
    const float watchWidth = ImGui::CalcTextSize("Watch").x + buttonPadding;
    const float cueWidth = ImGui::CalcTextSize("Cue").x + buttonPadding;
    const float actionsX = std::max(
        ImGui::GetCursorPosX(),
        ImGui::GetWindowContentRegionMax().x - watchWidth - cueWidth -
            ImGui::GetStyle().ItemSpacing.x);
    ImGui::TextColored(toImVec4(event.color), "Goal %zu", ordinal + 1);
    ImGui::SameLine(actionsX);
    const bool watchClicked = ImGui::SmallButton("Watch");
    ImGui::SameLine();
    const bool cueClicked = ImGui::SmallButton("Cue");
    ImGui::TextDisabled(
        "%s · %s",
        formatEventPlaylistTime(event.time).c_str(),
        event.actor.empty() ? "Unknown scorer" : event.actor.c_str());
    const std::vector<std::string> tags = goalTagsForEvent(event);
    if (tags.empty()) {
      renderGoalTagChip("Unlabeled", true, 0);
    } else {
      for (size_t tagIndex = 0; tagIndex < tags.size(); tagIndex += 1) {
        if (tagIndex > 0) {
          ImGui::SameLine();
        }
        renderGoalTagChip(tags[tagIndex], false, tagIndex);
      }
    }
    if (watchClicked) {
      mechanicsReviewClipActive = false;
      playbackCurrentTime = seekTime;
      playbackPlaying = true;
      playbackSkipPostGoalTransitions = false;
      playbackSkipKickoffs = false;
      showSingletonWindow(uiPlaybackControlsOpen, playbackControlsPlacement);
      if (hasReplayServer) {
        replayServer.StartPlaybackAtTime(seekTime);
      } else {
        cvarManager->log(std::format(
          "subtr-actor: selected goal at {:.2f}s; open a replay to seek",
          seekTime));
      }
    }
    if (cueClicked) {
      mechanicsReviewClipActive = false;
      playbackCurrentTime = seekTime;
      playbackPlaying = false;
      playbackSkipPostGoalTransitions = false;
      playbackSkipKickoffs = false;
      showSingletonWindow(uiPlaybackControlsOpen, playbackControlsPlacement);
      if (hasReplayServer) {
        replayServer.SkipToTime(seekTime);
        ReplayWrapper replay = replayServer.GetReplay();
        if (!replay.IsNull()) {
          replay.StopPlayback();
        }
      } else {
        cvarManager->log(std::format(
            "subtr-actor: selected goal at {:.2f}s; open a replay to seek",
            seekTime));
      }
    }
    ImGui::Spacing();
    ImGui::PopID();
  }
  if (goalEventIndexes.empty()) {
    if (!replayAnnotations && (!gameWrapper || !gameWrapper->IsInReplay())) {
      renderStatsWindowEmpty("Load a replay to show goal labels.");
    } else {
      renderStatsWindowEmpty("No goals loaded.");
    }
  }
}

void SubtrActorPlugin::renderAdHocStatsWindow(UiStatsWindow &window) {
  for (size_t i = 0; i < window.entries.size();) {
    UiStatsWindow::Entry &entry = window.entries[i];
    const std::string &statId = entry.stat_id;
    if (!statsWindowSupportsStat(window, statId)) {
      ++i;
      continue;
    }
    const std::string statLabel = uiStatLabel(statId);
    const std::string statValue = adHocStatValue(statId, entry.target_id);
    const float removeWidth = ImGui::CalcTextSize("x").x + ImGui::GetStyle().FramePadding.x * 2.0f;
    const float valueWidth = std::max(48.0f, ImGui::CalcTextSize(statValue.c_str()).x + 18.0f);
    const float removeX =
        std::max(ImGui::GetCursorPosX(), ImGui::GetWindowContentRegionMax().x - removeWidth);
    const float valueX = std::max(
        ImGui::GetCursorPosX(),
        removeX - valueWidth - 12.0f);
    const float targetX = std::max(ImGui::GetCursorPosX(), valueX - 148.0f);

    ImGui::Text("%s", statLabel.c_str());
    ImGui::SameLine(targetX);
    ImGui::SetNextItemWidth(132.0f);
    renderAdHocTargetSelector(window, entry, statId, i);
    ImGui::SameLine(valueX);
    ImGui::Text("%s", statValue.c_str());
    ImGui::SameLine(removeX);
    if (ImGui::SmallButton(std::format("x##remove-stat-{}-{}", window.id, i).c_str())) {
      window.entries.erase(window.entries.begin() + static_cast<std::ptrdiff_t>(i));
      scheduleUiConfigAutosave();
      return;
    }
    if (ImGui::IsItemHovered()) {
      ImGui::SetTooltip("Remove stat");
    }
    ++i;
  }
}
