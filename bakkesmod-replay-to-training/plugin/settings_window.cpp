// F2 > Plugins settings page: pack metadata, capture/save buttons, and the
// captured-shot list.

std::string ReplayToTrainingPlugin::GetPluginName() {
  return "replay-to-training";
}

void ReplayToTrainingPlugin::SetImGuiContext(uintptr_t ctx) {
  imguiContext = ctx;
  ImGui::SetCurrentContext(reinterpret_cast<ImGuiContext *>(ctx));
}

void ReplayToTrainingPlugin::RenderSettings() {
  if (imguiContext != 0) {
    ImGui::SetCurrentContext(reinterpret_cast<ImGuiContext *>(imguiContext));
  }

  if (!rustLoaded) {
    ImGui::TextWrapped(
        "replay_to_training.dll is not loaded. Install it to "
        "bakkesmod\\data\\replay-to-training\\replay_to_training.dll and reload the "
        "plugin.");
    ImGui::TextWrapped("Status: %s", statusLine.c_str());
    return;
  }

  ImGui::Text("Training pack");
  if (ImGui::InputText("Pack name", packNameBuffer.data(), packNameBuffer.size())) {
    setCvarString("replay_to_training_pack_name", packNameBuffer.data());
  }
  if (ImGui::InputText(
          "Creator name", creatorNameBuffer.data(), creatorNameBuffer.size())) {
    setCvarString("replay_to_training_creator_name", creatorNameBuffer.data());
  }
  static const char *difficultyLabels[] = {"Easy", "Medium", "Hard"};
  ImGui::Combo("Difficulty", &difficultyIndex, difficultyLabels, 3);
  if (ImGui::InputText(
          "Output directory", outputDirBuffer.data(), outputDirBuffer.size())) {
    setCvarString("replay_to_training_output_dir", outputDirBuffer.data());
  }
  ImGui::TextWrapped(
      "Empty output directory / training root resolves to Documents\\My "
      "Games\\Rocket League\\TAGame\\Training. Set a target below to save "
      "into MyTraining\\, which is what the game lists.");

  ImGui::Separator();

  // --- Target (persistent default save) ---
  ImGui::Text("Target (default save)");
  const bool hasTarget = !activeTargetPath.empty();
  if (hasTarget) {
    ImGui::TextWrapped("Active target: %s -> %s",
                       targetBuffer.data(),
                       activeTargetPath.string().c_str());
    ImGui::TextWrapped(
        "Captures append here; Save writes back non-destructively.");
  } else {
    ImGui::TextWrapped(
        "No target set. Captures go to an auto <GUID>.Tem on save.");
  }
  ImGui::InputText("Target name", targetBuffer.data(), targetBuffer.size());
  ImGui::SameLine();
  if (ImGui::Button("Set target")) {
    const std::string requested = targetBuffer.data();
    gameWrapper->Execute([this, requested](GameWrapper *) { setTarget(requested); });
  }
  if (hasTarget) {
    ImGui::SameLine();
    if (ImGui::Button("Clear target")) {
      gameWrapper->Execute([this](GameWrapper *) {
        clearTarget();
        setStatus("target cleared; saves use auto <GUID>.Tem");
      });
    }
  }
  if (ImGui::Button("Refresh target list")) {
    gameWrapper->Execute([this](GameWrapper *) {
      std::string error;
      discoveredTargets = discoverTargets(error);
      if (!error.empty()) {
        setStatus(error);
      } else {
        setStatus(std::format("found {} target(s)", discoveredTargets.size()));
      }
    });
  }
  for (size_t index = 0; index < discoveredTargets.size(); ++index) {
    ImGui::PushID(static_cast<int>(index) + 10000);
    if (ImGui::SmallButton("Pick")) {
      const std::string requested = discoveredTargets[index];
      gameWrapper->Execute([this, requested](GameWrapper *) { setTarget(requested); });
    }
    ImGui::SameLine();
    ImGui::Text("%s", discoveredTargets[index].c_str());
    ImGui::PopID();
  }

  ImGui::Separator();

  if (ImGui::Button("New pack")) {
    gameWrapper->Execute([this](GameWrapper *) { newPack(); });
  }
  ImGui::SameLine();
  const bool inReplay = gameWrapper->IsInReplay();
  // The SDK bundles ImGui 1.75 (no BeginDisabled); emulate by branching.
  if (ImGui::Button(inReplay ? "Capture shot" : "Capture shot (needs replay)")) {
    if (inReplay) {
      // Capture touches game wrappers, so hop to the game thread.
      gameWrapper->Execute([this](GameWrapper *) { captureShot(); });
    } else {
      setStatus("capture requires an in-game replay");
    }
  }
  ImGui::SameLine();
  // The save button reflects target-vs-GUID so the destination is clear.
  const char *saveLabel =
      hasTarget ? "Save to target" : "Save pack (auto GUID)";
  if (ImGui::Button(saveLabel)) {
    gameWrapper->Execute([this](GameWrapper *) { savePack(); });
  }

  ImGui::Separator();
  ImGui::InputText("Open path", openPathBuffer.data(), openPathBuffer.size());
  ImGui::SameLine();
  if (ImGui::Button("Open pack")) {
    const std::string path = openPathBuffer.data();
    gameWrapper->Execute([this, path](GameWrapper *) { openPackFromPath(path); });
  }

  ImGui::Separator();
  const size_t count = packShotCount ? packShotCount(pack) : 0;
  if (hasTarget) {
    ImGui::Text("Shots: %zu (target %s, pack %s)", count, targetBuffer.data(),
                packGuidHexString().c_str());
  } else {
    ImGui::Text("Shots: %zu (pack %s)", count, packGuidHexString().c_str());
  }
  for (size_t index = 0; index < count; ++index) {
    const std::string summary = shotSummary(index);
    ImGui::Text("%zu. %s", index + 1, summary.c_str());
    ImGui::SameLine();
    ImGui::PushID(static_cast<int>(index));
    if (ImGui::SmallButton("Remove")) {
      gameWrapper->Execute([this, index](GameWrapper *) { removeShot(index); });
    }
    ImGui::PopID();
  }

  ImGui::Separator();
  ImGui::TextWrapped("Status: %s", statusLine.c_str());
}
