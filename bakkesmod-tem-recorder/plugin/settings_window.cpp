// F2 > Plugins settings page: pack metadata, capture/save buttons, and the
// captured-shot list.

std::string TemRecorderPlugin::GetPluginName() {
  return "tem-recorder";
}

void TemRecorderPlugin::SetImGuiContext(uintptr_t ctx) {
  imguiContext = ctx;
  ImGui::SetCurrentContext(reinterpret_cast<ImGuiContext *>(ctx));
}

void TemRecorderPlugin::RenderSettings() {
  if (imguiContext != 0) {
    ImGui::SetCurrentContext(reinterpret_cast<ImGuiContext *>(imguiContext));
  }

  if (!rustLoaded) {
    ImGui::TextWrapped(
        "tem_recorder.dll is not loaded. Install it to "
        "bakkesmod\\data\\tem-recorder\\tem_recorder.dll and reload the "
        "plugin.");
    ImGui::TextWrapped("Status: %s", statusLine.c_str());
    return;
  }

  ImGui::Text("Training pack");
  if (ImGui::InputText("Pack name", packNameBuffer.data(), packNameBuffer.size())) {
    setCvarString("tem_recorder_pack_name", packNameBuffer.data());
  }
  if (ImGui::InputText(
          "Creator name", creatorNameBuffer.data(), creatorNameBuffer.size())) {
    setCvarString("tem_recorder_creator_name", creatorNameBuffer.data());
  }
  static const char *difficultyLabels[] = {"Easy", "Medium", "Hard"};
  ImGui::Combo("Difficulty", &difficultyIndex, difficultyLabels, 3);
  if (ImGui::InputText(
          "Output directory", outputDirBuffer.data(), outputDirBuffer.size())) {
    setCvarString("tem_recorder_output_dir", outputDirBuffer.data());
  }
  ImGui::TextWrapped(
      "Empty output directory saves to Documents\\My Games\\Rocket League\\"
      "TAGame\\Training (the game lists packs from a per-account "
      "<online-id>\\MyTraining subfolder; point the override there).");

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
  if (ImGui::Button("Save pack")) {
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
  ImGui::Text("Shots: %zu (pack %s)", count, packGuidHexString().c_str());
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
