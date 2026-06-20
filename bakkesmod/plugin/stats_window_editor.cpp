// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
void SubtrActorPlugin::renderAdHocTargetSelector(
    UiStatsWindow &window,
    UiStatsWindow::Entry &entry,
    std::string_view statId,
    size_t index) {
  const UiStatDefinition *definition = uiStatDefinition(statId);
  const auto graphStat = parseGraphStatId(normalizeUiStatId(statId));
  const bool playerScoped =
      definition ? definition->player : (graphStat && graphStat->scope == "player");
  const bool teamScoped =
      definition ? definition->team : (graphStat && graphStat->scope == "team");
  if (!playerScoped && !teamScoped) {
    ImGui::TextDisabled("-");
    return;
  }

  auto pushAdHocTargetSelectorStyle = [](std::optional<LinearColor> teamColor) {
    ImGui::SetNextItemWidth(std::min(112.0f, ImGui::GetContentRegionAvail().x));
    if (!teamColor) {
      return 0;
    }

    const ImVec4 accent = toImVec4(*teamColor);
    ImGui::PushStyleColor(ImGuiCol_Border, accent);
    ImGui::PushStyleColor(
        ImGuiCol_FrameBg,
        ImVec4{accent.x * 0.18f, accent.y * 0.18f, accent.z * 0.18f, 0.58f});
    ImGui::PushStyleColor(
        ImGuiCol_FrameBgHovered,
        ImVec4{accent.x * 0.24f, accent.y * 0.24f, accent.z * 0.24f, 0.74f});
    ImGui::PushStyleColor(
        ImGuiCol_FrameBgActive,
        ImVec4{accent.x * 0.30f, accent.y * 0.30f, accent.z * 0.30f, 0.88f});
    return 4;
  };

  if (playerScoped) {
    const SaPlayerFrame *selected = nullptr;
    if (const std::optional<uint32_t> selectedPlayerIndex =
            playerIndexForTargetId(entry.target_id)) {
      selected = sampledPlayerByIndex(*selectedPlayerIndex);
    }
    const std::string selectedLabel =
        selected ? playerLabel(selected->player_index, selected->is_team_0) : "Select player";
    const std::optional<LinearColor> selectedColor =
        selected ? std::make_optional(selected->is_team_0 != 0 ? LinearColor{80, 190, 255, 255}
                                                              : LinearColor{255, 175, 80, 255})
                 : std::nullopt;
    const int selectorStyleColors = pushAdHocTargetSelectorStyle(selectedColor);
    const bool comboOpen = ImGui::BeginCombo(
        std::format("##ad-hoc-target-{}-{}", window.id, index).c_str(),
        selectedLabel.c_str());
    if (selectorStyleColors > 0) {
      ImGui::PopStyleColor(selectorStyleColors);
    }
    if (comboOpen) {
      for (uint8_t isTeam0 : {uint8_t{1}, uint8_t{0}}) {
        const bool hasTeamPlayers = std::any_of(
            sampledPlayers.begin(),
            sampledPlayers.end(),
            [isTeam0](const SaPlayerFrame &player) { return player.is_team_0 == isTeam0; });
        if (!hasTeamPlayers) {
          continue;
        }

        const LinearColor color =
            isTeam0 != 0 ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255};
        ImGui::TextColored(toImVec4(color), "%s team", teamLabel(isTeam0).c_str());
        for (const SaPlayerFrame &player : sampledPlayers) {
          if (player.is_team_0 != isTeam0) {
            continue;
          }
          const std::string nextTarget = webPlayerIdForIndex(player.player_index);
          const bool isSelected = statsWindowTargetsEqual(statId, entry.target_id, nextTarget);
          if (ImGui::Selectable(playerLabel(player.player_index, player.is_team_0).c_str(),
                                isSelected) &&
              !statsWindowHasStat(window, statId, nextTarget)) {
            entry.target_id = nextTarget;
            scheduleUiConfigAutosave();
          }
        }
        ImGui::Separator();
      }
      if (sampledPlayers.empty()) {
        ImGui::TextDisabled("Waiting for sampled players.");
      }
      ImGui::EndCombo();
    }
    return;
  }

  const char *selectedTeam = entry.target_id == "orange" ? "Orange" : "Blue";
  const std::optional<LinearColor> selectedColor =
      entry.target_id == "orange" ? std::make_optional(LinearColor{255, 175, 80, 255})
                                  : std::make_optional(LinearColor{80, 190, 255, 255});
  const int selectorStyleColors = pushAdHocTargetSelectorStyle(selectedColor);
  const bool comboOpen = ImGui::BeginCombo(
      std::format("##ad-hoc-target-{}-{}", window.id, index).c_str(),
      selectedTeam);
  if (selectorStyleColors > 0) {
    ImGui::PopStyleColor(selectorStyleColors);
  }
  if (comboOpen) {
    for (const auto &[label, targetId, isTeam0] : {
             std::tuple<const char *, const char *, uint8_t>{"Blue", "blue", uint8_t{1}},
             std::tuple<const char *, const char *, uint8_t>{"Orange", "orange", uint8_t{0}},
         }) {
      const LinearColor color =
          isTeam0 != 0 ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255};
      ImGui::PushStyleColor(ImGuiCol_Text, toImVec4(color));
      if (ImGui::Selectable(label, entry.target_id == targetId) &&
          !statsWindowHasStat(window, statId, targetId)) {
        entry.target_id = targetId;
        scheduleUiConfigAutosave();
      }
      ImGui::PopStyleColor();
    }
    ImGui::EndCombo();
  }
}

void SubtrActorPlugin::renderStatsWindow(UiStatsWindow &window, size_t /*stackIndex*/) {
  applyStatsWindowPlacement(window);
  if (window.pending_focus) {
    ImGui::SetNextWindowFocus();
    window.pending_focus = false;
  }
  const std::string title = statsWindowTitle(window);
  if (!ImGui::Begin(title.c_str(), &window.open, UI_FLOATING_WINDOW_FLAGS)) {
    ImGui::End();
    return;
  }
  captureStatsWindowPlacement(window);

  const bool scopeHeaderOnly = statsWindowKindHasScopeSelector(window.kind);
  if (!scopeHeaderOnly) {
    const std::string headerLabel = uppercaseHeaderLabel(statsWindowKindLabel(window.kind));
    ImGui::TextColored(
        ImVec4{0.53f, 0.69f, 0.83f, 1.0f},
        "%s",
        headerLabel.c_str());
    ImGui::SameLine();
  }
  const std::string hideLabel = std::format("Hide##stats-window-hide-{}", window.id);
  const float buttonPadding = ImGui::GetStyle().FramePadding.x * 2.0f;
  const float hideWidth =
      ImGui::CalcTextSize("Hide").x + buttonPadding;
  const float rightAlignedX = ImGui::GetWindowContentRegionMax().x - hideWidth;
  if (rightAlignedX > ImGui::GetCursorPosX()) {
    ImGui::SetCursorPosX(rightAlignedX);
  }
  ImGui::PushStyleVar(ImGuiStyleVar_FrameRounding, 6.0f);
  const bool hideClicked = ImGui::Button(hideLabel.c_str());
  ImGui::PopStyleVar();
  if (hideClicked) {
    hideStatsWindow(window);
    ImGui::End();
    return;
  }
  ImGui::Separator();

  renderStatsWindowScopeSelector(window);
  renderStatsWindowAddControl(window);
  renderStatsWindowEntries(window);
  ImGui::End();
}

void SubtrActorPlugin::renderStatsWindowScopeSelector(UiStatsWindow &window) {
  auto pushStatsScopeSelectorStyle = [](std::optional<LinearColor> teamColor) {
    ImGui::SetNextItemWidth(std::min(208.0f, ImGui::GetContentRegionAvail().x));
    if (!teamColor) {
      return 0;
    }

    const ImVec4 accent = toImVec4(*teamColor);
    ImGui::PushStyleColor(ImGuiCol_Border, accent);
    ImGui::PushStyleColor(
        ImGuiCol_FrameBg,
        ImVec4{accent.x * 0.18f, accent.y * 0.18f, accent.z * 0.18f, 0.58f});
    ImGui::PushStyleColor(
        ImGuiCol_FrameBgHovered,
        ImVec4{accent.x * 0.24f, accent.y * 0.24f, accent.z * 0.24f, 0.74f});
    ImGui::PushStyleColor(
        ImGuiCol_FrameBgActive,
        ImVec4{accent.x * 0.30f, accent.y * 0.30f, accent.z * 0.30f, 0.88f});
    return 4;
  };

  if (window.kind == UiStatsWindowKind::Player) {
    resolveStatsWindowPlayerSelection(window);
    const SaPlayerFrame *selected = sampledPlayerByIndex(window.selected_player_index);
    const std::string selectedLabel =
        selected ? playerLabel(selected->player_index, selected->is_team_0) : "Select player";
    const std::optional<LinearColor> selectedColor =
        selected ? std::make_optional(selected->is_team_0 != 0 ? LinearColor{80, 190, 255, 255}
                                                              : LinearColor{255, 175, 80, 255})
                 : std::nullopt;
    const int selectorStyleColors = pushStatsScopeSelectorStyle(selectedColor);
    const bool comboOpen = ImGui::BeginCombo(
        std::format("##stats-window-player-scope-{}", window.id).c_str(),
        selectedLabel.c_str());
    if (selectorStyleColors > 0) {
      ImGui::PopStyleColor(selectorStyleColors);
    }
    if (comboOpen) {
      for (uint8_t isTeam0 : {uint8_t{1}, uint8_t{0}}) {
        const bool hasTeamPlayers = std::any_of(
            sampledPlayers.begin(),
            sampledPlayers.end(),
            [isTeam0](const SaPlayerFrame &player) { return player.is_team_0 == isTeam0; });
        if (!hasTeamPlayers) {
          continue;
        }

        const LinearColor color =
            isTeam0 != 0 ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255};
        ImGui::TextColored(toImVec4(color), "%s team", teamLabel(isTeam0).c_str());
        for (const SaPlayerFrame &player : sampledPlayers) {
          if (player.is_team_0 != isTeam0) {
            continue;
          }
          const std::string label = playerLabel(player.player_index, player.is_team_0);
          const bool isSelected = player.player_index == window.selected_player_index;
          if (ImGui::Selectable(label.c_str(), isSelected)) {
            window.selected_player_index = player.player_index;
            window.selected_player_id = webPlayerIdForIndex(window.selected_player_index);
            scheduleUiConfigAutosave();
          }
        }
        ImGui::Separator();
      }
      if (sampledPlayers.empty()) {
        ImGui::TextDisabled("Waiting for sampled players.");
      }
      ImGui::EndCombo();
    }
    return;
  }

  if (window.kind == UiStatsWindowKind::Team) {
    const char *selectedTeam = window.selected_team_is_team_0 != 0 ? "Blue" : "Orange";
    const std::optional<LinearColor> selectedColor =
        window.selected_team_is_team_0 != 0 ? std::make_optional(LinearColor{80, 190, 255, 255})
                                           : std::make_optional(LinearColor{255, 175, 80, 255});
    const int selectorStyleColors = pushStatsScopeSelectorStyle(selectedColor);
    const bool comboOpen = ImGui::BeginCombo(
        std::format("##stats-window-team-scope-{}", window.id).c_str(),
        selectedTeam);
    if (selectorStyleColors > 0) {
      ImGui::PopStyleColor(selectorStyleColors);
    }
    if (comboOpen) {
      for (uint8_t isTeam0 : {uint8_t{1}, uint8_t{0}}) {
        const LinearColor color =
            isTeam0 != 0 ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255};
        const std::string label = teamLabel(isTeam0);
        const bool selected = (window.selected_team_is_team_0 != 0) == (isTeam0 != 0);
        ImGui::PushStyleColor(ImGuiCol_Text, toImVec4(color));
        if (ImGui::Selectable(label.c_str(), selected)) {
          window.selected_team_is_team_0 = isTeam0;
          scheduleUiConfigAutosave();
        }
        ImGui::PopStyleColor();
      }
      ImGui::EndCombo();
    }
    return;
  }

  if (window.kind == UiStatsWindowKind::StatsModule) {
    const std::vector<std::string> moduleNames = availableStatsModuleNames();
    const char *selectedModule =
        window.module_name.empty() ? "Select module" : window.module_name.c_str();
    if (ImGui::BeginCombo("Module", selectedModule)) {
      for (const std::string &moduleName : moduleNames) {
        const bool selected = moduleName == window.module_name;
        if (ImGui::Selectable(moduleName.c_str(), selected)) {
          window.module_name = moduleName;
          scheduleUiConfigAutosave();
        }
      }
      ImGui::EndCombo();
    }
    ImGui::Separator();
  }
}

void renderStatsWindowEmpty(std::string_view message) {
  const std::string messageString{message};
  ImGui::PushStyleColor(ImGuiCol_Text, ImVec4{0.62f, 0.71f, 0.78f, 1.0f});
  ImGui::TextWrapped("%s", messageString.c_str());
  ImGui::PopStyleColor();
}

void SubtrActorPlugin::renderStatsWindowAddControl(UiStatsWindow &window) {
  if (window.kind == UiStatsWindowKind::StatsModule) {
    if (ImGui::RadioButton(
            std::format("Frame##module-view-{}", window.id).c_str(),
            &window.module_view,
            0)) {
      scheduleUiConfigAutosave();
    }
    ImGui::SameLine();
    if (ImGui::RadioButton(
            std::format("Module##module-view-{}", window.id).c_str(),
            &window.module_view,
            1)) {
      scheduleUiConfigAutosave();
    }
    ImGui::SameLine();
    if (ImGui::RadioButton(
            std::format("Config##module-view-{}", window.id).c_str(),
            &window.module_view,
            2)) {
      scheduleUiConfigAutosave();
    }
    ImGui::Separator();
    return;
  }

  if (!statsWindowKindHasStatPicker(window.kind)) {
    ImGui::Separator();
    return;
  }

  const std::string addButton = std::format("+##add-stat-{}", window.id);
  const bool hasScopeSelector = statsWindowKindHasScopeSelector(window.kind);
  const float addButtonSize = ImGui::GetFrameHeight();
  auto renderAddButton = [&]() {
    if (window.picker_open) {
      ImGui::PushStyleColor(ImGuiCol_Button, ImVec4{1.0f, 1.0f, 1.0f, 0.10f});
      ImGui::PushStyleColor(ImGuiCol_ButtonHovered, ImVec4{1.0f, 1.0f, 1.0f, 0.14f});
      ImGui::PushStyleColor(ImGuiCol_ButtonActive, ImVec4{1.0f, 1.0f, 1.0f, 0.18f});
    }
    const bool clicked = ImGui::Button(addButton.c_str(), ImVec2{addButtonSize, 0.0f});
    if (window.picker_open) {
      ImGui::PopStyleColor(3);
    }
    return clicked;
  };
  if (hasScopeSelector) {
    ImGui::SameLine();
  } else {
    const float addButtonX =
        std::max(ImGui::GetCursorPosX(), ImGui::GetWindowContentRegionMax().x - addButtonSize);
    ImGui::SetCursorPosX(addButtonX);
  }
  if (renderAddButton()) {
    window.picker_open = !window.picker_open;
  }
  if (ImGui::IsItemHovered()) {
    ImGui::SetTooltip("Add stat");
  }

  if (!window.picker_open) {
    ImGui::Separator();
    return;
  }

  ImGui::PushStyleVar(ImGuiStyleVar_ChildRounding, 6.0f);
  ImGui::PushStyleVar(ImGuiStyleVar_WindowPadding, ImVec2{12.0f, 10.0f});
  ImGui::PushStyleColor(ImGuiCol_ChildBg, ImVec4{1.0f, 1.0f, 1.0f, 0.04f});
  ImGui::PushStyleColor(ImGuiCol_Border, ImVec4{1.0f, 1.0f, 1.0f, 0.08f});
  ImGui::BeginChild(
      std::format("stat-picker-{}", window.id).c_str(),
      ImVec2{0.0f, 190.0f},
      true);
  auto endStatsPickerPanel = [&]() {
    ImGui::EndChild();
    ImGui::PopStyleColor(2);
    ImGui::PopStyleVar(2);
  };
  auto renderStatsPickerItem = [&](std::string_view label,
                                   std::string_view meta,
                                   std::string_view id,
                                   bool disabled = false) {
    const ImGuiStyle &style = ImGui::GetStyle();
    const std::string buttonId = std::format("##stats-picker-item-{}-{}", window.id, id);
    const float rowWidth = ImGui::GetContentRegionAvail().x;
    ImGui::PushStyleVar(ImGuiStyleVar_FrameRounding, 6.0f);
    ImGui::PushStyleVar(ImGuiStyleVar_FramePadding, ImVec2{8.0f, 5.0f});
    ImGui::PushStyleColor(
        ImGuiCol_Button,
        disabled ? ImVec4{1.0f, 1.0f, 1.0f, 0.03f} : ImVec4{1.0f, 1.0f, 1.0f, 0.06f});
    ImGui::PushStyleColor(
        ImGuiCol_ButtonHovered,
        disabled ? ImVec4{1.0f, 1.0f, 1.0f, 0.03f} : ImVec4{1.0f, 1.0f, 1.0f, 0.10f});
    ImGui::PushStyleColor(
        ImGuiCol_ButtonActive,
        disabled ? ImVec4{1.0f, 1.0f, 1.0f, 0.03f} : ImVec4{1.0f, 1.0f, 1.0f, 0.14f});
    const bool clicked = ImGui::Button(buttonId.c_str(), ImVec2{rowWidth, 0.0f});
    ImGui::PopStyleColor(3);
    ImGui::PopStyleVar(2);

    const ImVec2 rowMin = ImGui::GetItemRectMin();
    const ImVec2 rowMax = ImGui::GetItemRectMax();
    const float textY =
        rowMin.y + std::max(0.0f, (rowMax.y - rowMin.y - ImGui::GetTextLineHeight()) * 0.5f);
    const std::string labelString{label};
    const std::string metaString = uppercaseHeaderLabel(meta);
    const ImVec2 metaSize = ImGui::CalcTextSize(metaString.c_str());
    const float rightX = rowMax.x - style.FramePadding.x - metaSize.x;
    ImDrawList *drawList = ImGui::GetWindowDrawList();
    const ImU32 labelColor =
        disabled ? IM_COL32(115, 132, 146, 255) : IM_COL32(237, 245, 250, 255);
    drawList->PushClipRect(
        ImVec2{rowMin.x + style.FramePadding.x, rowMin.y},
        ImVec2{rightX - 8.0f, rowMax.y},
        true);
    drawList->AddText(
        ImVec2{rowMin.x + style.FramePadding.x, textY},
        labelColor,
        labelString.c_str());
    drawList->PopClipRect();
    drawList->AddText(
        ImVec2{rightX, textY},
        disabled ? IM_COL32(107, 133, 156, 255) : IM_COL32(135, 175, 212, 255),
        metaString.c_str());
    return clicked && !disabled;
  };

  std::array<char, 128> queryBuffer{};
  const size_t querySize = std::min(window.picker_query.size(), queryBuffer.size() - 1);
  std::copy_n(window.picker_query.data(), querySize, queryBuffer.data());
  ImGui::TextDisabled("Search stats");
  ImGui::SetNextItemWidth(-1.0f);
  if (ImGui::InputText(
          std::format("##stats-window-search-{}", window.id).c_str(),
          queryBuffer.data(),
          queryBuffer.size())) {
    window.picker_query = queryBuffer.data();
  }

  std::vector<UiStatDefinitionCandidate> definitions;
  std::unordered_set<std::string> definitionIds;
  for (size_t index = 0; index < UI_STAT_DEFINITIONS.size(); index += 1) {
    const UiStatDefinition &definition = UI_STAT_DEFINITIONS[index];
    definitions.push_back(UiStatDefinitionCandidate{
        definition.id,
        definition.label,
        definition.category,
        definition.player,
        definition.team,
        definition.event,
    });
    definitionIds.insert(definition.id);
  }
  for (UiStatDefinitionCandidate &definition :
       graphStatDefinitionsFromStatsJson(currentStatsJson())) {
    if (definitionIds.insert(definition.id).second) {
      definitions.push_back(std::move(definition));
    }
  }
  for (UiStatDefinitionCandidate &definition :
       graphStatDefinitionsFromReplayFrameJson(currentReplayFrameJson())) {
    if (definitionIds.insert(definition.id).second) {
      definitions.push_back(std::move(definition));
    }
  }

  std::vector<UiStatDefinitionMatch> matches;
  for (size_t index = 0; index < definitions.size(); index += 1) {
    const UiStatDefinitionCandidate &definition = definitions[index];
    if (!statsWindowSupportsStat(window, definition.id)) {
      continue;
    }
    const auto score = statDefinitionSearchScore(definition, window.picker_query);
    if (!score) {
      continue;
    }
    matches.push_back(UiStatDefinitionMatch{definition, *score, index});
  }
  std::sort(matches.begin(), matches.end(), [](const auto &left, const auto &right) {
    return left.score == right.score ? left.index < right.index : left.score < right.score;
  });

  std::vector<std::pair<std::string, int>> categoryCounts;
  for (const UiStatDefinitionMatch &match : matches) {
    auto found = std::find_if(
        categoryCounts.begin(),
        categoryCounts.end(),
        [&](const auto &entry) { return entry.first == match.definition.category; });
    if (found == categoryCounts.end()) {
      categoryCounts.emplace_back(match.definition.category, 1);
    } else {
      found->second += 1;
    }
  }

  for (const auto &[category, count] : categoryCounts) {
    if (count < 2) {
      continue;
    }
    if (renderStatsPickerItem(
            std::format("Add all {}", category),
            std::to_string(count),
            std::format("all-{}", category))) {
      bool added = false;
      for (const UiStatDefinitionMatch &match : matches) {
        if (category != match.definition.category) {
          continue;
        }
        const std::string targetId =
            window.kind == UiStatsWindowKind::AdHoc ? defaultAdHocTargetId(match.definition.id)
                                                    : "";
        if (!statsWindowHasStat(window, match.definition.id, targetId)) {
          window.entries.push_back(UiStatsWindow::Entry{match.definition.id, targetId});
          added = true;
        }
      }
      if (added) {
        scheduleUiConfigAutosave();
      }
    }
  }

  if (matches.empty()) {
    renderStatsWindowEmpty("No matching stats.");
    endStatsPickerPanel();
    ImGui::Separator();
    return;
  }

  for (const UiStatDefinitionMatch &match : matches) {
    const UiStatDefinitionCandidate &definition = match.definition;
    const bool alreadySelected = statsWindowHasStat(window, definition.id);
    const bool disabled = alreadySelected && window.kind != UiStatsWindowKind::AdHoc;
    if (renderStatsPickerItem(
            definition.label,
            uiStatScopeLabel(definition),
            definition.id,
            disabled)) {
      if (window.kind == UiStatsWindowKind::AdHoc) {
        const std::string targetId = defaultAdHocTargetId(definition.id);
        if (!statsWindowHasStat(window, definition.id, targetId)) {
          window.entries.push_back(UiStatsWindow::Entry{definition.id, targetId});
          scheduleUiConfigAutosave();
        }
      } else {
        window.entries.push_back(UiStatsWindow::Entry{definition.id, ""});
        scheduleUiConfigAutosave();
      }
    }
  }
  endStatsPickerPanel();
  ImGui::Separator();
}

void SubtrActorPlugin::renderStatsWindowEntries(UiStatsWindow &window) {
  if (window.kind == UiStatsWindowKind::StatsModule) {
    renderStatsModuleWindow(window);
    return;
  }

  if (window.kind == UiStatsWindowKind::GoalsOverview) {
    renderGoalsOverviewStats(window);
    return;
  }

  const bool hasSupportedEntries = std::any_of(
      window.entries.begin(),
      window.entries.end(),
      [&](const UiStatsWindow::Entry &entry) {
        return statsWindowSupportsStat(window, entry.stat_id);
      });
  if (!hasSupportedEntries) {
    renderStatsWindowEmpty("No stats added.");
    return;
  }

  switch (window.kind) {
  case UiStatsWindowKind::Player:
    if (const SaPlayerFrame *player = sampledPlayerByIndex(window.selected_player_index)) {
      renderPlayerStatsTable(window, *player);
    } else {
      renderMissingStatsRows(window);
    }
    break;
  case UiStatsWindowKind::Team:
    if (sampledPlayers.empty() && currentStatsJson().empty()) {
      renderStatsWindowEmpty("Load a replay to show stats.");
      break;
    }
    renderTeamStatsTable(window, window.selected_team_is_team_0);
    break;
  case UiStatsWindowKind::AllPlayers:
    renderAllPlayersStatsTable(window);
    break;
  case UiStatsWindowKind::AllTeams:
    if (sampledPlayers.empty() && currentStatsJson().empty()) {
      renderStatsWindowEmpty("Load a replay to show stats.");
      break;
    }
    renderAllTeamsStatsTable(window);
    break;
  case UiStatsWindowKind::KickoffOverview:
    break;
  case UiStatsWindowKind::GoalsOverview:
    break;
  case UiStatsWindowKind::AdHoc:
    renderAdHocStatsWindow(window);
    break;
  case UiStatsWindowKind::StatsModule:
    break;
  }
}

bool SubtrActorPlugin::renderStatsWindowValueRow(
    UiStatsWindow &window,
    size_t entryIndex,
    std::string_view label,
    std::string_view value,
    std::string_view idSuffix) {
  const std::string valueString{value};
  const float removeWidth = ImGui::CalcTextSize("x").x + ImGui::GetStyle().FramePadding.x * 2.0f;
  const float valueWidth =
      std::max(48.0f, ImGui::CalcTextSize(valueString.c_str()).x + 18.0f);
  const float removeX =
      std::max(ImGui::GetCursorPosX(), ImGui::GetWindowContentRegionMax().x - removeWidth);
  const float valueX = std::max(
      ImGui::GetCursorPosX(),
      removeX - valueWidth - 12.0f);
  const std::string labelString{label};
  ImGui::AlignTextToFramePadding();
  ImGui::TextColored(ImVec4{0.62f, 0.71f, 0.78f, 1.0f}, "%s", labelString.c_str());
  ImGui::SameLine(valueX);
  ImGui::AlignTextToFramePadding();
  ImGui::TextColored(ImVec4{0.93f, 0.96f, 0.98f, 1.0f}, "%s", valueString.c_str());
  ImGui::SameLine(removeX);
  const std::string removeLabel = idSuffix.empty()
                                      ? std::format("x##remove-stat-{}-{}", window.id, entryIndex)
                                      : std::format(
                                            "x##remove-stat-{}-{}-{}",
                                            window.id,
                                            entryIndex,
                                            idSuffix);
  ImGui::PushStyleVar(ImGuiStyleVar_FramePadding, ImVec2{5.0f, 2.0f});
  ImGui::PushStyleVar(ImGuiStyleVar_FrameRounding, 4.0f);
  if (ImGui::SmallButton(removeLabel.c_str())) {
    ImGui::PopStyleVar(2);
    window.entries.erase(window.entries.begin() + static_cast<std::ptrdiff_t>(entryIndex));
    scheduleUiConfigAutosave();
    return true;
  }
  ImGui::PopStyleVar(2);
  if (ImGui::IsItemHovered()) {
    ImGui::SetTooltip("Remove stat");
  }
  return false;
}

void SubtrActorPlugin::renderMissingStatsRows(UiStatsWindow &window) {
  for (size_t i = 0; i < window.entries.size();) {
    const std::string &statId = window.entries[i].stat_id;
    if (!statsWindowSupportsStat(window, statId)) {
      ++i;
      continue;
    }

    if (renderStatsWindowValueRow(window, i, uiStatLabel(statId), "--")) {
      return;
    }
    ++i;
  }
}
