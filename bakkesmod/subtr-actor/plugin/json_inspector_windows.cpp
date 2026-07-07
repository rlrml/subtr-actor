// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
void renderJsonFieldTable(const char *tableId, const std::vector<JsonFieldSummary> &fields) {
  ImGui::Columns(2, tableId, false);
  for (const JsonFieldSummary &field : fields) {
    ImGui::TextWrapped("%s", field.label.c_str());
    ImGui::NextColumn();
    ImGui::TextWrapped("%s", field.value.c_str());
    ImGui::NextColumn();
  }
  ImGui::Columns(1);
}

void renderStatsModuleFrameOverview(const std::string &json, const std::string &id) {
  bool renderedAny = false;
  auto renderSnapshot = [&](const char *heading,
                            const char *propertyName,
                            const ImVec4 &color,
                            size_t maxFields) {
    const auto object = parseJsonObjectProperty(json, propertyName);
    if (!object) {
      return;
    }
    std::vector<JsonFieldSummary> fields;
    collectJsonFieldSummaries(*object, "", fields, maxFields, 2);
    if (fields.empty()) {
      return;
    }
    renderedAny = true;
    ImGui::TextColored(color, "%s", heading);
    renderJsonFieldTable(std::format("{}-{}-fields", id, propertyName).c_str(), fields);
    ImGui::Spacing();
  };

  renderSnapshot("Blue team", "team_zero", ImVec4{0.31f, 0.74f, 1.0f, 1.0f}, 14);
  renderSnapshot("Orange team", "team_one", ImVec4{1.0f, 0.69f, 0.31f, 1.0f}, 14);

  const std::vector<std::string> playerStats = parseJsonObjectArrayProperty(json, "player_stats");
  if (!playerStats.empty()) {
    renderedAny = true;
    ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "Players");
    for (size_t index = 0; index < playerStats.size(); index += 1) {
      const std::string &player = playerStats[index];
      const auto stats = parseJsonObjectProperty(player, "stats");
      if (!stats) {
        continue;
      }
      std::vector<JsonFieldSummary> fields;
      collectJsonFieldSummaries(*stats, "", fields, 12, 2);
      if (fields.empty()) {
        continue;
      }

      std::string label =
          parseJsonStringProperty(player, "name").value_or(std::format("Player {}", index + 1));
      const std::optional<bool> isTeam0 = parseJsonBoolProperty(player, "is_team_0");
      const ImVec4 color = isTeam0.value_or(true) ? ImVec4{0.31f, 0.74f, 1.0f, 1.0f}
                                                  : ImVec4{1.0f, 0.69f, 0.31f, 1.0f};
      ImGui::PushID(static_cast<int>(index));
      ImGui::TextColored(color, "%s", label.c_str());
      renderJsonFieldTable("player-module-fields", fields);
      ImGui::Spacing();
      ImGui::PopID();
    }
  }

  if (!renderedAny) {
    ImGui::TextWrapped("No team or player module frame fields are available.");
  }
}

void SubtrActorPlugin::renderJsonSummary(const std::string &json) {
  bool renderedAny = false;
  auto renderObjectSection = [&](const char *label, const char *propertyName, size_t maxFields) {
    const auto object = parseJsonObjectProperty(json, propertyName);
    if (!object) {
      return;
    }
    renderedAny = true;
    std::vector<JsonFieldSummary> fields;
    collectJsonFieldSummaries(*object, "", fields, maxFields, 2);
    if (ImGui::TreeNode(label)) {
      if (fields.empty()) {
        ImGui::Text("No scalar fields.");
      } else {
        renderJsonFieldTable(std::format("{}-fields", propertyName).c_str(), fields);
      }
      ImGui::TreePop();
    }
  };

  const std::array<const char *, 4> arrayProperties{
      "events",
      "timeline",
      "ledger_events",
      "state_events",
  };
  std::vector<JsonFieldSummary> counts;
  for (const char *propertyName : arrayProperties) {
    const auto count = parseJsonArrayPropertyElementCount(json, propertyName);
    if (count) {
      counts.push_back(JsonFieldSummary{
          propertyName,
          std::format("{} item{}", *count, *count == 1 ? "" : "s"),
      });
    }
  }
  if (!counts.empty()) {
    renderedAny = true;
    if (ImGui::TreeNode("Event collections")) {
      renderJsonFieldTable("module-event-counts", counts);
      ImGui::TreePop();
    }
  }

  renderObjectSection("Team zero", "team_zero", 16);
  renderObjectSection("Team one", "team_one", 16);
  renderObjectSection("Stats", "stats", 24);

  const std::vector<std::string> playerStats = parseJsonObjectArrayProperty(json, "player_stats");
  if (!playerStats.empty()) {
    renderedAny = true;
    if (ImGui::TreeNode(std::format("Player stats ({})", playerStats.size()).c_str())) {
      for (size_t index = 0; index < playerStats.size(); index += 1) {
        ImGui::PushID(static_cast<int>(index));
        const auto playerId = parseJsonObjectProperty(playerStats[index], "player_id");
        const std::string playerLabel =
            playerId ? clippedDisplayText(*playerId, 96) : std::format("Player {}", index + 1);
        if (ImGui::TreeNode(playerLabel.c_str())) {
          const auto stats = parseJsonObjectProperty(playerStats[index], "stats");
          if (stats) {
            std::vector<JsonFieldSummary> fields;
            collectJsonFieldSummaries(*stats, "", fields, 18, 2);
            renderJsonFieldTable("player-stats-fields", fields);
          } else {
            ImGui::Text("No stats object.");
          }
          ImGui::TreePop();
        }
        ImGui::PopID();
      }
      ImGui::TreePop();
    }
  }

  if (!renderedAny) {
    std::vector<JsonFieldSummary> fields;
    collectJsonFieldSummaries(json, "", fields, 32, 2);
    if (fields.empty()) {
      ImGui::TextWrapped("No structured summary is available for this JSON shape.");
    } else {
      renderJsonFieldTable("module-top-level-fields", fields);
    }
  }
}

void SubtrActorPlugin::renderJsonInspectorPayload(
    const char *id,
    const std::string &label,
    const std::string &json) {
  if (json.empty()) {
    ImGui::TextWrapped("%s is not available from the live graph yet.", label.c_str());
    return;
  }

  ImGui::Text("%s (%zu bytes)", label.c_str(), json.size());
  ImGui::SameLine();
  if (ImGui::SmallButton(std::format("Copy##{}-json", id).c_str())) {
    ImGui::SetClipboardText(json.c_str());
  }

  renderJsonSummary(json);
  ImGui::Separator();

  const std::string display = clippedDisplayText(json);
  if (ImGui::TreeNode(std::format("Raw JSON##{}-raw", id).c_str())) {
    ImGui::BeginChild(
        std::format("{}-json", id).c_str(),
        ImVec2{0.0f, 220.0f},
        true,
        ImGuiWindowFlags_HorizontalScrollbar);
    ImGui::TextUnformatted(display.c_str(), display.c_str() + display.size());
    ImGui::EndChild();
    ImGui::TreePop();
  }
}

void SubtrActorPlugin::renderStatsModuleWindow(UiStatsWindow &window) {
  if (!loaded || !engine) {
    ImGui::TextWrapped("Load the graph runtime to inspect graph-backed stats modules.");
    return;
  }

  const std::vector<std::string> moduleNames = availableStatsModuleNames();
  if (window.module_name.empty() && !moduleNames.empty()) {
    window.module_name = moduleNames.front();
  }
  if (window.module_name.empty()) {
    ImGui::TextWrapped("No builtin stats modules are available yet.");
    return;
  }

  const char *viewLabel = "frame";
  std::string json;
  if (window.module_view == 1) {
    viewLabel = "module";
    json = readNamedJsonBuffer(statsModuleJsonLen, writeStatsModuleJson, window.module_name);
  } else if (window.module_view == 2) {
    viewLabel = "config";
    json = readNamedJsonBuffer(
        statsModuleConfigJsonLen,
        writeStatsModuleConfigJson,
        window.module_name);
  } else {
    window.module_view = 0;
    json = readNamedJsonBuffer(
        statsModuleFrameJsonLen,
        writeStatsModuleFrameJson,
        window.module_name);
    if (json.empty()) {
      json = replayStatsModuleFrameJson(currentReplayFrameJson(), window.module_name);
      if (!json.empty()) {
        viewLabel = "replay frame";
      }
    }
  }

  if (json.empty()) {
    ImGui::TextWrapped(
        "The '%s' %s JSON is not available from the live graph yet.",
        window.module_name.c_str(),
        viewLabel);
    return;
  }

  ImGui::Text(
      "%s %s JSON (%zu bytes)",
      window.module_name.c_str(),
      viewLabel,
      json.size());
  ImGui::SameLine();
  if (ImGui::SmallButton(std::format("Copy##module-json-{}", window.id).c_str())) {
    ImGui::SetClipboardText(json.c_str());
  }

  if (window.module_view == 0) {
    renderStatsModuleFrameOverview(json, std::format("module-frame-{}", window.id));
    ImGui::Separator();
  }
  renderJsonSummary(json);
  ImGui::Separator();

  const std::string display = clippedDisplayText(std::move(json));
  if (ImGui::TreeNode(std::format("Raw JSON##module-raw-{}", window.id).c_str())) {
    ImGui::BeginChild(
        std::format("module-json-{}", window.id).c_str(),
        ImVec2{0.0f, 220.0f},
        true,
        ImGuiWindowFlags_HorizontalScrollbar);
    ImGui::TextUnformatted(display.c_str(), display.c_str() + display.size());
    ImGui::EndChild();
    ImGui::TreePop();
  }
}

void SubtrActorPlugin::render(CanvasWrapper canvas) {
  auto overlayEnabledCvar = cvarManager->getCvar("subtr_actor_overlay_enabled");
  const bool overlayEnabled =
      !static_cast<bool>(overlayEnabledCvar) || overlayEnabledCvar.getBoolValue();
  auto statusOverlayEnabledCvar = cvarManager->getCvar("subtr_actor_status_overlay_enabled");
  const bool statusOverlayEnabled = !static_cast<bool>(statusOverlayEnabledCvar) ||
                                    statusOverlayEnabledCvar.getBoolValue();
  const float scale = overlayScale();
  const int lineHeight = static_cast<int>(std::round(24.0f * scale));
  const int messageLineHeight =
      static_cast<int>(std::round(static_cast<float>(lineHeight) * 1.25f));
  const Vector2 panelPosition{overlayX(), overlayY()};

  if (overlayEnabled) {
    const auto now = std::chrono::steady_clock::now();
    while (!messages.empty() && messages.front().expires_at <= now) {
      messages.pop_front();
    }
  }

  std::optional<std::pair<std::string, LinearColor>> statusLine;

  if (statusOverlayEnabled) {
    const bool processingEnabled = liveProcessingEnabled();
    const bool replayAnnotationActive = replayAnnotationsEnabled() && replayAnnotations != nullptr;
    const bool inGame = gameWrapper->IsInGame();
    const float intervalMs = sampleIntervalSeconds() * 1000.0f;
    const std::string status =
        replayAnnotationActive
            ? std::format(
                  "subtr-actor REPLAY | annotations={}",
                  replayAnnotationCount ? replayAnnotationCount(replayAnnotations) : 0)
            : !processingEnabled
                  ? "subtr-actor OFF"
                  : inGame ? std::format(
                                 "subtr-actor LIVE | frames={} | interval={:.0f}ms",
                                 frameNumber,
                                 intervalMs)
                           : "subtr-actor ON | waiting for game";
    statusLine = std::pair{
        status,
        (processingEnabled || replayAnnotationActive) ? LinearColor{80, 255, 150, 255}
                                                      : LinearColor{180, 180, 180, 255}};
  }

  if (!statusLine && (!overlayEnabled || messages.empty())) {
    return;
  }

  float panelWidth = 0.0f;
  int panelHeight = 0;
  if (statusLine) {
    panelWidth =
        std::max(panelWidth, canvas.GetStringSize(statusLine->first, scale, scale).X);
    panelHeight += lineHeight;
  }
  if (overlayEnabled) {
    for (const OverlayMessage &message : messages) {
      panelWidth = std::max(
          panelWidth,
          canvas.GetStringSize(message.text, scale * 1.25f, scale * 1.25f).X);
      panelHeight += messageLineHeight;
    }
  }

  constexpr float panelPaddingX = 12.0f;
  constexpr float panelPaddingY = 10.0f;
  canvas.SetPosition(Vector2F{
      static_cast<float>(panelPosition.X) - panelPaddingX,
      static_cast<float>(panelPosition.Y) - panelPaddingY});
  canvas.SetColor(LinearColor{8, 12, 16, 180});
  canvas.FillBox(Vector2F{
      panelWidth + panelPaddingX * 2.0f,
      static_cast<float>(panelHeight) + panelPaddingY * 2.0f});

  Vector2 position = panelPosition;
  if (statusLine) {
    canvas.SetPosition(position);
    canvas.SetColor(statusLine->second);
    canvas.DrawString(statusLine->first, scale, scale, true);
    position.Y += lineHeight;
  }

  if (!overlayEnabled) {
    return;
  }

  for (const OverlayMessage &message : messages) {
    canvas.SetPosition(position);
    canvas.SetColor(message.color);
    canvas.DrawString(message.text, scale * 1.25f, scale * 1.25f, true);
    position.Y += messageLineHeight;
  }
}
