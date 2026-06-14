// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
bool SubtrActorPlugin::eventPlaylistSourceEnabled(const UiEventRecord &event) const {
  if (eventPlaylistUsesDefaultSources(eventPlaylistSourceFilter)) {
    return defaultEventPlaylistSourceAllows(event.category, event.type);
  }
  return eventFilterAllows(eventPlaylistSourceFilter, event.category, event.type);
}

std::string SubtrActorPlugin::mechanicsReviewKey(const UiEventRecord &event) const {
  return std::format(
      "{}:{}:{}:{}",
      event.category,
      event.type,
      event.frame_number,
      event.actor);
}

const char *SubtrActorPlugin::mechanicsReviewDecisionLabel(const UiEventRecord &event) const {
  const auto decision = mechanicsReviewDecisions.find(mechanicsReviewKey(event));
  if (decision == mechanicsReviewDecisions.end()) {
    return "unreviewed";
  }
  if (decision->second == 1) {
    return "confirmed";
  }
  if (decision->second == 2) {
    return "rejected";
  }
  if (decision->second == 3) {
    return "uncertain";
  }
  return "unreviewed";
}

void SubtrActorPlugin::renderEventPlaylistWindow() {
  if (!uiEventPlaylistOpen) {
    return;
  }

  applySingletonWindowPlacement(eventPlaylistPlacement);
  if (!ImGui::Begin(
          "Event playlist##subtr-actor",
          &uiEventPlaylistOpen,
          UI_FLOATING_WINDOW_FLAGS)) {
    ImGui::End();
    return;
  }
  captureWindowPlacement(eventPlaylistPlacement);
  if (renderSingletonWindowHeader("Event playlist", uiEventPlaylistOpen)) {
    ImGui::End();
    return;
  }

  const std::string currentFilter = eventPlaylistSourceFilter;
  const bool allEventSourcesEnabled = allEventSourcesSelected(currentFilter);
  std::vector<std::string> selectedSources = selectedEventSourceTokens(currentFilter);

  auto sourceHasEnabledPlaylistGroup = [&](std::string_view source) {
    for (const UiEventRecord &event : recentUiEvents) {
      if (eventFilterAllows(source, event.category, event.type) &&
          eventPlaylistSourceEnabled(event)) {
        return true;
      }
    }
    return false;
  };

  struct PlaylistSource {
    const EventFilterOption *option = nullptr;
    size_t count = 0;
    bool enabled = false;
  };

  std::vector<PlaylistSource> playlistSources;
  playlistSources.reserve(EVENT_FILTER_OPTIONS.size());
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
    const bool selected = containsString(selectedSources, option.value);
    if (count == 0 && (allEventSourcesEnabled || !selected)) {
      continue;
    }
    playlistSources.push_back(
        PlaylistSource{&option, count, selected && sourceHasEnabledPlaylistGroup(option.value)});
  }
  auto playlistSourceRank = [](const EventFilterOption &option) {
    const std::string_view group{option.group};
    const int groupRank = group == "Replay"     ? 0
                          : group == "Mechanics" ? 1
                          : group == "Stats"     ? 2
                                                  : 3;
    if (group == "Replay" && std::string_view{option.value} == "goal") {
      return std::tuple{groupRank, 0, std::string_view{option.label}};
    }
    return std::tuple{groupRank, 1, std::string_view{option.label}};
  };
  std::sort(
      playlistSources.begin(),
      playlistSources.end(),
      [&](const PlaylistSource &left, const PlaylistSource &right) {
        return playlistSourceRank(*left.option) < playlistSourceRank(*right.option);
      });

  const size_t selectedSourceCount = static_cast<size_t>(std::count_if(
      playlistSources.begin(),
      playlistSources.end(),
      [](const PlaylistSource &source) { return source.enabled; }));

  std::vector<size_t> playlistEventIndexes;
  playlistEventIndexes.reserve(recentUiEvents.size());
  for (size_t index = 0; index < recentUiEvents.size(); index += 1) {
    const UiEventRecord &event = recentUiEvents[index];
    if (eventPlaylistSourceEnabled(event)) {
      playlistEventIndexes.push_back(index);
    }
  }
  std::sort(
      playlistEventIndexes.begin(),
      playlistEventIndexes.end(),
      [&](size_t left, size_t right) {
        const UiEventRecord &leftEvent = recentUiEvents[left];
        const UiEventRecord &rightEvent = recentUiEvents[right];
        if (leftEvent.time != rightEvent.time) {
          return leftEvent.time < rightEvent.time;
        }
        if (leftEvent.label != rightEvent.label) {
          return leftEvent.label < rightEvent.label;
        }
        return left < right;
      });

  if (playlistSources.empty()) {
    eventPlaylistLastActiveKey.clear();
    ImGui::TextDisabled(
        recentUiEvents.empty() && replayAnnotations == nullptr ? "Load a replay to see events."
                                                               : "No events loaded.");
    ImGui::End();
    return;
  }

  const std::string filterSummary = std::format(
      "Filters {}/{}##event-playlist-filter",
      selectedSourceCount,
      playlistSources.size());
  const bool filtersOpen = ImGui::TreeNode(filterSummary.c_str());
  ImGui::SameLine();
  if (ImGui::Checkbox("Auto-follow", &eventPlaylistAutoFollow)) {
    eventPlaylistLastActiveKey.clear();
    scheduleUiConfigAutosave();
  }

  if (filtersOpen) {
    if (ImGui::Button("All##event-playlist-sources-all")) {
      selectedSources.clear();
      selectedSources.reserve(playlistSources.size());
      for (const PlaylistSource &source : playlistSources) {
        selectedSources.emplace_back(source.option->value);
      }
      eventPlaylistSourceFilter = eventFilterFromSelectedSources(selectedSources);
      eventPlaylistLastActiveKey.clear();
      scheduleUiConfigAutosave();
    }
    ImGui::SameLine();
    if (ImGui::Button("None##event-playlist-sources-none")) {
      eventPlaylistSourceFilter = "none";
      eventPlaylistLastActiveKey.clear();
      scheduleUiConfigAutosave();
    }

    ImGui::BeginChild("event-playlist-source-list", ImVec2{0.0f, 170.0f}, true);
    std::string_view currentGroup;
    for (const PlaylistSource &source : playlistSources) {
      const EventFilterOption &option = *source.option;
      const std::string_view optionGroup{option.group};
      if (currentGroup != optionGroup) {
        if (!currentGroup.empty()) {
          ImGui::Separator();
        }
        ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "%s", option.group);
        currentGroup = optionGroup;
      }

      ImGui::PushID(option.value);
      const std::string label = std::format("{} ({})", option.label, source.count);
      bool enabled = source.enabled;
      if (ImGui::Checkbox(label.c_str(), &enabled)) {
        if (!enabled) {
          selectedSources.erase(
              std::remove(selectedSources.begin(), selectedSources.end(), std::string{option.value}),
              selectedSources.end());
        } else {
          appendUniqueFilterToken(selectedSources, option.value);
        }
        eventPlaylistSourceFilter = eventFilterFromSelectedSources(selectedSources);
        eventPlaylistLastActiveKey.clear();
        scheduleUiConfigAutosave();
      }
      ImGui::PopID();
    }
    ImGui::EndChild();
    ImGui::TreePop();
  }

  ImGui::Separator();

  ReplayServerWrapper replayServer = gameWrapper->GetGameEventAsReplay();
  const bool hasReplayServer = !replayServer.IsNull();
  const float currentPlaybackTime =
      hasReplayServer ? replayServer.GetReplayTimeElapsed() : playbackCurrentTime;
  auto eventSeekTime = [](const UiEventRecord &event) {
    const float leadSeconds = event.category == "goal_context" || event.type == "goal"
                                  ? GOAL_WATCH_LEAD_SECONDS
                                  : 2.0f;
    return std::max(0.0f, event.time - leadSeconds);
  };

  std::optional<size_t> activeEventIndex;
  float activeEventDistance = std::numeric_limits<float>::infinity();
  for (const size_t index : playlistEventIndexes) {
    const UiEventRecord &event = recentUiEvents[index];
    const float distance = std::abs(event.time - currentPlaybackTime);
    if (distance < activeEventDistance) {
      activeEventDistance = distance;
      activeEventIndex = index;
    }
  }
  const std::string activeEventKey =
      activeEventIndex ? mechanicsReviewKey(recentUiEvents[*activeEventIndex]) : "";
  auto sourceLabelForEvent = [&](const UiEventRecord &event) -> std::string {
    for (const PlaylistSource &source : playlistSources) {
      if (source.enabled && eventFilterAllows(source.option->value, event.category, event.type)) {
        return source.option->label;
      }
    }
    return eventTypeDisplayLabel(event.type);
  };
  auto renderEventPlaylistItem = [&](const std::string &timeLabel,
                                     const std::string &eventLabel,
                                     const std::string &metaLabel,
                                     const ImVec4 &eventColor,
                                     bool active) {
    constexpr float timeColumnWidth = 54.0f;
    constexpr float rowPaddingX = 10.0f;
    constexpr float rowPaddingY = 8.0f;
    constexpr float colorRailWidth = 4.0f;
    constexpr float columnGap = 10.0f;
    const float rowWidth = ImGui::GetContentRegionAvail().x;
    const float rowHeight = ImGui::GetTextLineHeight() * 2.0f + rowPaddingY * 2.0f + 2.0f;
    const std::string buttonId = std::format("##event-playlist-item-{}", eventLabel);
    const bool clicked = ImGui::InvisibleButton(buttonId.c_str(), ImVec2{rowWidth, rowHeight});
    const bool hovered = ImGui::IsItemHovered();
    const ImVec2 rowMin = ImGui::GetItemRectMin();
    const ImVec2 rowMax = ImGui::GetItemRectMax();
    ImDrawList *drawList = ImGui::GetWindowDrawList();
    const ImU32 eventColorU32 = ImGui::ColorConvertFloat4ToU32(eventColor);
    const ImVec4 rowBg = (active || hovered)
                             ? ImVec4{eventColor.x, eventColor.y, eventColor.z, 0.15f}
                             : ImVec4{1.0f, 1.0f, 1.0f, 0.035f};
    const ImVec4 border = (active || hovered)
                              ? ImVec4{eventColor.x, eventColor.y, eventColor.z, 0.48f}
                              : ImVec4{1.0f, 1.0f, 1.0f, 0.09f};
    drawList->AddRectFilled(rowMin, rowMax, ImGui::ColorConvertFloat4ToU32(rowBg), 6.0f);
    drawList->AddRect(rowMin, rowMax, ImGui::ColorConvertFloat4ToU32(border), 6.0f);
    drawList->AddRectFilled(
        rowMin,
        ImVec2{rowMin.x + colorRailWidth, rowMax.y},
        eventColorU32,
        6.0f);

    const float timeX = rowMin.x + rowPaddingX + colorRailWidth;
    const float mainX = timeX + timeColumnWidth + columnGap;
    const float titleY = rowMin.y + rowPaddingY;
    const float metaY = titleY + ImGui::GetTextLineHeight() + 3.0f;
    drawList->AddText(
        ImVec2{timeX, titleY},
        IM_COL32(137, 164, 186, 255),
        timeLabel.c_str());
    drawList->PushClipRect(
        ImVec2{mainX, rowMin.y},
        ImVec2{rowMax.x - rowPaddingX, rowMax.y},
        true);
    drawList->AddText(ImVec2{mainX, titleY}, IM_COL32(237, 245, 250, 255), eventLabel.c_str());
    if (!metaLabel.empty()) {
      drawList->AddText(ImVec2{mainX, metaY}, IM_COL32(137, 164, 186, 255), metaLabel.c_str());
    }
    drawList->PopClipRect();
    return clicked;
  };

  ImGui::BeginChild("event-playlist-list", ImVec2{0.0f, 0.0f}, true);
  for (const size_t index : playlistEventIndexes) {
    const UiEventRecord &event = recentUiEvents[index];

    ImGui::PushID(static_cast<int>(index));
    const ImVec4 color =
        toImVec4(event.has_player != 0 ? eventPlaylistPlayerColor(event.player_index)
                                       : event.color);
    const bool active = activeEventIndex && *activeEventIndex == index;
    const float seekTime = eventSeekTime(event);
    const std::string timeLabel = formatEventPlaylistTime(event.time);
    const std::string sourceLabel = sourceLabelForEvent(event);
    const std::string eventLabel = event.label.empty() ? sourceLabel : event.label;
    std::vector<std::string> metaParts;
    if (!event.actor.empty()) {
      metaParts.push_back(event.actor);
    }
    if (event.frame_number != 0) {
      metaParts.push_back(std::format("frame {}", event.frame_number));
    }
    if (!sourceLabel.empty()) {
      metaParts.push_back(sourceLabel);
    }
    const std::string metaLabel = joinStrings(metaParts, " · ");
    const bool selected =
        renderEventPlaylistItem(timeLabel, eventLabel, metaLabel, color, active);
    if (selected) {
      mechanicsReviewClipActive = false;
      playbackCurrentTime = seekTime;
      playbackSkipPostGoalTransitions = false;
      playbackSkipKickoffs = false;
      showSingletonWindow(uiPlaybackControlsOpen, playbackControlsPlacement);
      if (hasReplayServer) {
        replayServer.SkipToTime(seekTime);
      }
    }
    if (active && eventPlaylistAutoFollow && activeEventKey != eventPlaylistLastActiveKey) {
      ImGui::SetScrollHereY(0.5f);
    }
    ImGui::PopID();
  }
  eventPlaylistLastActiveKey = activeEventKey;
  if (playlistEventIndexes.empty()) {
    ImGui::TextDisabled("No event types selected.");
  }
  ImGui::EndChild();
  ImGui::End();
}

void SubtrActorPlugin::renderMechanicsReviewWindow() {
  if (!uiMechanicsReviewOpen) {
    return;
  }

  applySingletonWindowPlacement(mechanicsReviewPlacement);
  if (!ImGui::Begin(
          "Mechanics review##subtr-actor",
          &uiMechanicsReviewOpen,
          UI_FLOATING_WINDOW_FLAGS)) {
    ImGui::End();
    return;
  }
  captureWindowPlacement(mechanicsReviewPlacement);
  if (renderSingletonWindowHeader("Mechanics review", uiMechanicsReviewOpen)) {
    ImGui::End();
    return;
  }

  std::vector<size_t> candidates;
  candidates.reserve(recentUiEvents.size());
  for (size_t index = 0; index < recentUiEvents.size(); index += 1) {
    const UiEventRecord &event = recentUiEvents[index];
    if (eventPlaylistSourceEnabled(event) && uiEventVisible(event)) {
      candidates.push_back(index);
    }
  }
  if (candidates.empty()) {
    mechanicsReviewIndex = 0;
  } else {
    mechanicsReviewIndex = std::clamp(
        mechanicsReviewIndex,
        0,
        static_cast<int>(candidates.size()) - 1);
  }

  const UiEventRecord *current = candidates.empty()
                                     ? nullptr
                                     : &recentUiEvents[candidates[static_cast<size_t>(
                                           mechanicsReviewIndex)]];
  const std::string currentKey = current == nullptr ? std::string{} : mechanicsReviewKey(*current);
  const float clipStart =
      current == nullptr ? 0.0f : std::max(0.0f, current->time - mechanicsReviewClipLeadSeconds);
  const float clipEnd = current == nullptr ? 0.0f : current->time + mechanicsReviewClipTrailSeconds;
  auto mechanicsReviewItemTitle = [](const UiEventRecord &event, size_t index) {
    if (!event.label.empty()) {
      return event.label;
    }
    if (!event.type.empty()) {
      return eventTypeDisplayLabel(event.type);
    }
    return std::format("Review item {}", index + 1);
  };
  const std::string statusReadout =
      mechanicsReviewClipActive
          ? std::format(
                "Playing clip {:.2f}s to {:.2f}s",
                mechanicsReviewClipStartSeconds,
                mechanicsReviewClipEndSeconds)
      : !mechanicsReviewStatus.empty() ? mechanicsReviewStatus
      : candidates.empty()             ? "Load a review playlist."
                                       : "Loaded review playlist.";
  ImGui::TextWrapped("%s", statusReadout.c_str());
  ImGui::Separator();
  ImGui::Text("%d / %zu", current == nullptr ? 0 : mechanicsReviewIndex + 1, candidates.size());
  const std::string currentTitle =
      current == nullptr ? "No candidate selected"
                         : mechanicsReviewItemTitle(
                               *current,
                               static_cast<size_t>(mechanicsReviewIndex));
  ImGui::TextWrapped("%s", currentTitle.c_str());
  const std::string clipReadout =
      current == nullptr
          ? "--"
          : std::format(
                "{:.2f}s to {:.2f}s · {:.1f}s clip · {:.1f}s preroll · {:.1f}s postroll",
                clipStart,
                clipEnd,
                std::max(0.0f, clipEnd - clipStart),
                std::max(0.0f, current->time - clipStart),
                std::max(0.0f, clipEnd - current->time));
  const std::string eventReadout = [&]() {
    if (current == nullptr) {
      return std::string{"--"};
    }
    if (current->frame_number == 0) {
      return std::format("{:.2f}s", current->time);
    }
    return std::format(
        "{:.2f}s · frame {}",
        current->time,
        static_cast<unsigned long long>(current->frame_number));
  }();
  const std::string reasonReadout =
      current == nullptr || current->details.empty() ? "--" : current->details;
  ImGui::Columns(2, "mechanics-review-fields", false);
  const std::string mechanicReadout =
      current == nullptr || current->type.empty() ? "--" : eventTypeDisplayLabel(current->type);
  renderWebDetailGridCell("Mechanic", mechanicReadout);
  ImGui::NextColumn();
  renderWebDetailGridCell(
      "Player",
      current == nullptr || current->actor.empty() ? "--" : current->actor);
  ImGui::NextColumn();
  renderWebDetailGridCell("Clip", clipReadout);
  ImGui::NextColumn();
  renderWebDetailGridCell("Event", eventReadout);
  ImGui::Columns(1);
  ImGui::Spacing();
  ImGui::TextColored(ImVec4{0.54f, 0.64f, 0.73f, 1.0f}, "Reason");
  ImGui::PushStyleColor(ImGuiCol_Text, ImVec4{0.93f, 0.96f, 0.98f, 1.0f});
  ImGui::TextWrapped("%s", reasonReadout.c_str());
  ImGui::PopStyleColor();

  auto mechanicsReviewButton = [](const char *label,
                                  bool disabled,
                                  float width,
                                  std::optional<ImVec4> borderColor = std::nullopt) {
    if (disabled) {
      ImGui::PushStyleVar(ImGuiStyleVar_Alpha, ImGui::GetStyle().Alpha * 0.45f);
    }
    if (borderColor) {
      ImGui::PushStyleVar(ImGuiStyleVar_FrameBorderSize, 1.0f);
      ImGui::PushStyleColor(ImGuiCol_Border, *borderColor);
    }
    const bool clicked = ImGui::Button(label, ImVec2{width, 0.0f});
    if (borderColor) {
      ImGui::PopStyleColor();
      ImGui::PopStyleVar();
    }
    if (disabled) {
      ImGui::PopStyleVar();
    }
    return clicked && !disabled;
  };
  const bool prevDisabled = current == nullptr || mechanicsReviewIndex <= 0;
  const bool replayDisabled = current == nullptr;
  const bool nextDisabled =
      current == nullptr || mechanicsReviewIndex >= static_cast<int>(candidates.size()) - 1;
  const bool decisionDisabled = current == nullptr;
  const float actionGap = ImGui::GetStyle().ItemSpacing.x;
  const float actionButtonWidth =
      std::max(72.0f, (ImGui::GetContentRegionAvail().x - actionGap * 2.0f) / 3.0f);

  if (mechanicsReviewButton("Prev", prevDisabled, actionButtonWidth)) {
    mechanicsReviewIndex -= 1;
    scheduleUiConfigAutosave();
  }
  ImGui::SameLine(0.0f, actionGap);
  if (mechanicsReviewButton("Replay clip", replayDisabled, actionButtonWidth)) {
    playbackCurrentTime = clipStart;
    playbackPlaying = true;
    playbackSkipPostGoalTransitions = false;
    playbackSkipKickoffs = false;
    mechanicsReviewClipActive = true;
    mechanicsReviewClipStartSeconds = clipStart;
    mechanicsReviewClipEndSeconds = clipEnd;
    mechanicsReviewStatus = std::format("Playing clip {:.2f}s to {:.2f}s", clipStart, clipEnd);
    showSingletonWindow(uiPlaybackControlsOpen, playbackControlsPlacement);

    ReplayServerWrapper replayServer = gameWrapper->GetGameEventAsReplay();
    if (!replayServer.IsNull()) {
      replayServer.StartPlaybackAtTime(clipStart);
    } else {
      mechanicsReviewClipActive = false;
      playbackPlaying = false;
      mechanicsReviewStatus = "Open a Rocket League replay to seek this clip";
      cvarManager->log(
          "subtr-actor: replay clip selected; open a replay to seek in Rocket League");
    }
    scheduleUiConfigAutosave();
  }
  ImGui::SameLine(0.0f, actionGap);
  if (mechanicsReviewButton("Next", nextDisabled, actionButtonWidth)) {
    mechanicsReviewIndex += 1;
    scheduleUiConfigAutosave();
  }

  if (mechanicsReviewButton(
          "Confirm",
          decisionDisabled,
          actionButtonWidth,
          ImVec4{0.30f, 0.69f, 0.47f, 0.52f})) {
    mechanicsReviewDecisions[currentKey] = 1;
  }
  ImGui::SameLine(0.0f, actionGap);
  if (mechanicsReviewButton(
          "Reject",
          decisionDisabled,
          actionButtonWidth,
          ImVec4{0.86f, 0.37f, 0.37f, 0.58f})) {
    mechanicsReviewDecisions[currentKey] = 2;
  }
  ImGui::SameLine(0.0f, actionGap);
  if (mechanicsReviewButton("Uncertain", decisionDisabled, actionButtonWidth)) {
    mechanicsReviewDecisions[currentKey] = 3;
  }

  ImGui::Separator();
  ImGui::Text("Replays");
  ImGui::SameLine();
  ImGui::TextDisabled("%s", replayAnnotations ? "1 replay" : "0 replays");

  ImGui::Separator();
  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "PLAYLIST");
  ImGui::SameLine();
  ImGui::TextDisabled("%zu %s", candidates.size(), candidates.size() == 1 ? "item" : "items");
  auto renderMechanicsReviewItem = [](const std::string &title,
                                      const std::string &meta,
                                      bool active) {
    const ImGuiStyle &style = ImGui::GetStyle();
    const float rowWidth = ImGui::GetContentRegionAvail().x;
    const float rowHeight = std::max(32.0f, ImGui::GetTextLineHeight() + 14.0f);
    const bool clicked =
        ImGui::InvisibleButton("##mechanics-review-item", ImVec2{rowWidth, rowHeight});
    const bool hovered = ImGui::IsItemHovered();
    const ImVec2 rowMin = ImGui::GetItemRectMin();
    const ImVec2 rowMax = ImGui::GetItemRectMax();
    ImDrawList *drawList = ImGui::GetWindowDrawList();
    const ImVec4 bg = (active || hovered) ? ImVec4{0.29f, 0.58f, 1.0f, 0.16f}
                                          : ImVec4{1.0f, 1.0f, 1.0f, 0.045f};
    const ImVec4 border = active ? ImVec4{0.29f, 0.58f, 1.0f, 0.42f}
                                 : ImVec4{1.0f, 1.0f, 1.0f, 0.09f};
    drawList->AddRectFilled(rowMin, rowMax, ImGui::ColorConvertFloat4ToU32(bg), 6.0f);
    drawList->AddRect(rowMin, rowMax, ImGui::ColorConvertFloat4ToU32(border), 6.0f);

    const std::string metaText = meta.empty() ? "--" : meta;
    const ImVec2 metaSize = ImGui::CalcTextSize(metaText.c_str());
    const float textY =
        rowMin.y + std::max(0.0f, (rowMax.y - rowMin.y - ImGui::GetTextLineHeight()) * 0.5f);
    const float titleX = rowMin.x + style.FramePadding.x;
    const float metaX = rowMax.x - style.FramePadding.x - metaSize.x;
    drawList->PushClipRect(
        ImVec2{titleX, rowMin.y},
        ImVec2{std::max(titleX, metaX - 10.0f), rowMax.y},
        true);
    drawList->AddText(ImVec2{titleX, textY}, IM_COL32(237, 245, 250, 255), title.c_str());
    drawList->PopClipRect();
    drawList->AddText(ImVec2{metaX, textY}, IM_COL32(137, 164, 186, 255), metaText.c_str());
    return clicked;
  };
  ImGui::BeginChild("mechanics-review-list", ImVec2{0.0f, 150.0f}, true);
  if (candidates.empty()) {
    ImGui::TextDisabled("No review playlist loaded.");
  }
  for (size_t i = 0; i < candidates.size(); i += 1) {
    const UiEventRecord &event = recentUiEvents[candidates[i]];
    ImGui::PushID(static_cast<int>(i));
    const bool active = i == static_cast<size_t>(mechanicsReviewIndex);
    const std::string title = mechanicsReviewItemTitle(event, i);
    std::vector<std::string> metaParts;
    if (!event.type.empty()) {
      metaParts.push_back(eventTypeDisplayLabel(event.type));
    }
    if (!event.actor.empty()) {
      metaParts.push_back(event.actor);
    }
    metaParts.push_back(mechanicsReviewDecisionLabel(event));
    const std::string meta = joinStrings(metaParts, " · ");
    if (renderMechanicsReviewItem(title, meta, active)) {
      mechanicsReviewIndex = static_cast<int>(i);
      scheduleUiConfigAutosave();
    }
    ImGui::PopID();
  }
  ImGui::EndChild();

  ImGui::End();
}

void SubtrActorPlugin::renderReplayLoadingWindow() {
  if (!uiReplayLoadingOpen) {
    return;
  }

  applySingletonWindowPlacement(replayLoadingPlacement);
  if (!ImGui::Begin(
          "Replay loading##subtr-actor",
          &uiReplayLoadingOpen,
          UI_FLOATING_WINDOW_FLAGS)) {
    ImGui::End();
    return;
  }
  captureWindowPlacement(replayLoadingPlacement);
  if (renderSingletonWindowHeader("Replay loading", uiReplayLoadingOpen)) {
    ImGui::End();
    return;
  }

  const bool annotationsEnabled = replayAnnotationsEnabled();
  const bool inReplay = gameWrapper->IsInReplay();
  ReplayServerWrapper replayServer = gameWrapper->GetGameEventAsReplay();
  const bool hasReplayServer = !replayServer.IsNull();
  const std::optional<std::string> replayPath =
      hasReplayServer ? currentReplayPath(replayServer) : std::nullopt;
  std::string rawReplayPath;
  if (hasReplayServer) {
    ReplayWrapper replay = replayServer.GetReplay();
    if (!replay.IsNull()) {
      rawReplayPath = replay.GetFilePath().ToString();
    }
  }
  const size_t annotationCount =
      replayAnnotations && replayAnnotationCount ? replayAnnotationCount(replayAnnotations) : 0;
  std::vector<SaReplayPlayerInfo> annotationPlayers;
  if (replayAnnotations && replayAnnotationPlayerCount && writeReplayAnnotationPlayers) {
    annotationPlayers.resize(replayAnnotationPlayerCount(replayAnnotations));
    const size_t copied = writeReplayAnnotationPlayers(
        replayAnnotations,
        annotationPlayers.data(),
        annotationPlayers.size());
    annotationPlayers.resize(copied);
  }
  const char *status = !annotationsEnabled
                           ? "Disabled"
                           : !inReplay       ? "Pending"
                           : replayAnnotations ? "Loaded"
                           : replayAnnotationLoadFailed ? "Failed"
                                                        : "Loading";
  const bool hasReplaySource =
      replayPath || !rawReplayPath.empty() || !replayAnnotationPath.empty();
  const std::string replaySummary = hasReplaySource ? "1 replay" : "0 replays";
  const char *activeSummary = !hasReplaySource         ? "No playlist"
                              : replayAnnotations      ? "Complete"
                              : replayAnnotationLoadFailed ? "1 failed"
                              : annotationsEnabled && inReplay ? "1 active, 0 pending"
                                                               : status;

  ImGui::Text("%s", replaySummary.c_str());
  ImGui::SameLine();
  ImGui::TextDisabled("%s", activeSummary);

  ImGui::Separator();
  ImGui::BeginChild("replay-loading-list", ImVec2{0.0f, 112.0f}, true);
  if (!hasReplaySource) {
    ImGui::TextDisabled("No replay sources.");
  } else {
    const std::string titlePath = replayPath ? *replayPath
                                  : !rawReplayPath.empty() ? rawReplayPath
                                                           : replayAnnotationPath;
    const std::string title = replaySourceDisplayLabel(titlePath);
    const float replayLoadProgress = replayAnnotations ? 1.0f : 0.0f;
    const ImVec4 statusColor =
        replayAnnotations ? ImVec4{0.50f, 0.86f, 0.62f, 1.0f}
                          : replayAnnotationLoadFailed
                              ? ImVec4{0.95f, 0.45f, 0.45f, 1.0f}
                              : ImVec4{0.72f, 0.78f, 0.86f, 1.0f};
    std::vector<std::string> replayMeta;
    if (!rawReplayPath.empty()) {
      replayMeta.push_back(std::format("raw: {}", rawReplayPath));
    }
    if (!replayAnnotationPath.empty() && replayAnnotationPath != titlePath) {
      replayMeta.push_back(std::format("processed: {}", replayAnnotationPath));
    }
    if (annotationCount > 0) {
      replayMeta.push_back(std::format("{} events", annotationCount));
    }
    if (!annotationPlayers.empty()) {
      replayMeta.push_back(std::format("{} players", annotationPlayers.size()));
    }
    const std::string meta = joinStrings(replayMeta, " · ");
    const float rowWidth = ImGui::GetContentRegionAvail().x;
    const float rowHeight = 62.0f;
    ImGui::Dummy(ImVec2{rowWidth, rowHeight});
    const ImVec2 rowMin = ImGui::GetItemRectMin();
    const ImVec2 rowMax = ImGui::GetItemRectMax();
    ImDrawList *drawList = ImGui::GetWindowDrawList();
    const ImVec4 rowBorder =
        replayAnnotations ? ImVec4{0.30f, 0.69f, 0.47f, 0.42f}
                          : replayAnnotationLoadFailed
                              ? ImVec4{0.86f, 0.37f, 0.37f, 0.58f}
                              : ImVec4{1.0f, 1.0f, 1.0f, 0.08f};
    drawList->AddRectFilled(
        rowMin,
        rowMax,
        ImGui::ColorConvertFloat4ToU32(ImVec4{1.0f, 1.0f, 1.0f, 0.035f}),
        6.0f);
    drawList->AddRect(rowMin, rowMax, ImGui::ColorConvertFloat4ToU32(rowBorder), 6.0f);

    constexpr float rowPadding = 8.0f;
    const float statusWidth = ImGui::CalcTextSize(status).x;
    const float statusX = rowMax.x - rowPadding - statusWidth;
    const float titleY = rowMin.y + rowPadding;
    const float metaY = titleY + ImGui::GetTextLineHeight() + 3.0f;
    drawList->PushClipRect(
        ImVec2{rowMin.x + rowPadding, rowMin.y},
        ImVec2{std::max(rowMin.x + rowPadding, statusX - 10.0f), rowMax.y},
        true);
    drawList->AddText(
        ImVec2{rowMin.x + rowPadding, titleY},
        IM_COL32(237, 245, 250, 255),
        title.c_str());
    if (!meta.empty()) {
      drawList->AddText(
          ImVec2{rowMin.x + rowPadding, metaY},
          IM_COL32(137, 164, 186, 255),
          meta.c_str());
    }
    drawList->PopClipRect();
    drawList->AddText(
        ImVec2{statusX, titleY},
        ImGui::ColorConvertFloat4ToU32(statusColor),
        status);

    const float progressMinX = rowMin.x + rowPadding;
    const float progressMaxX = rowMax.x - rowPadding;
    const float progressY = rowMax.y - rowPadding - 4.0f;
    drawList->AddRectFilled(
        ImVec2{progressMinX, progressY},
        ImVec2{progressMaxX, progressY + 4.0f},
        IM_COL32(255, 255, 255, 20),
        999.0f);
    drawList->AddRectFilled(
        ImVec2{progressMinX, progressY},
        ImVec2{progressMinX + (progressMaxX - progressMinX) * replayLoadProgress, progressY + 4.0f},
        ImGui::ColorConvertFloat4ToU32(
            replayAnnotations ? ImVec4{0.30f, 0.69f, 0.47f, 1.0f}
                              : replayAnnotationLoadFailed
                                  ? ImVec4{0.86f, 0.37f, 0.37f, 1.0f}
                                  : ImVec4{0.47f, 0.66f, 1.0f, 1.0f}),
        999.0f);
  }
  ImGui::EndChild();

  ImGui::End();
}
