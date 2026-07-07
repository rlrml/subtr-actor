// F2 > Plugins settings page, plus the shared ImGui building blocks it has
// in common with the standalone capture window (window.cpp): pack metadata,
// pack type, capture toggles, target controls, capture/save buttons, and
// the captured-shot list.
//
// Everything here runs on the render thread; any call that mutates game or
// pack state hops to the game thread via gameWrapper->Execute (the SDK
// convention this plugin already follows).

std::string ReplayToTrainingPlugin::GetPluginName() {
  return "replay-to-training";
}

void ReplayToTrainingPlugin::SetImGuiContext(uintptr_t ctx) {
  imguiContext = ctx;
  ImGui::SetCurrentContext(reinterpret_cast<ImGuiContext *>(ctx));
}

// --- shared building blocks ---

void ReplayToTrainingPlugin::renderPackMetadataControls() {
  if (ImGui::InputText("Pack name", packNameBuffer.data(), packNameBuffer.size())) {
    setCvarString("replay_to_training_pack_name", packNameBuffer.data());
  }
  if (ImGui::InputText(
          "Creator name", creatorNameBuffer.data(), creatorNameBuffer.size())) {
    setCvarString("replay_to_training_creator_name", creatorNameBuffer.data());
  }
  static const char *difficultyLabels[] = {"Easy", "Medium", "Hard"};
  ImGui::Combo("Difficulty", &difficultyIndex, difficultyLabels, 3);
}

// Pack training type display + manual override. The type is normally set
// by the FIRST capture's mode (shot -> Striker, save -> Goalie); the
// dropdown overrides it, including Aerial/None for publishing metadata.
void ReplayToTrainingPlugin::renderPackTypeControls() {
  const uint32_t typeIndex = packTrainingTypeIndex();
  ImGui::Text("Pack type: %s", trainingTypeLabel(typeIndex));
  // ABI encoding: the first four values are the settable types, in order.
  static const char *settableTypes[] = {"None", "Aerial", "Goalie", "Striker"};
  // Unset/other show an empty combo preview (-1) until an override is picked.
  int overrideIndex = typeIndex <= 3 ? static_cast<int>(typeIndex) : -1;
  if (ImGui::Combo("Type override", &overrideIndex, settableTypes, 4)) {
    const uint32_t selected = static_cast<uint32_t>(overrideIndex);
    gameWrapper->Execute(
        [this, selected](GameWrapper *) { overridePackTrainingType(selected); });
  }
}

// The persisted capture-mode selection (striker vs goalie), shown in both
// UIs. The generic capture command and the window's Capture buttons follow
// it; the explicit capture_shot/capture_save shortcuts and an activated
// typed pack update it (the bound pack's type is authoritative, so with a
// typed pack active the dropdown effectively tracks the pack).
void ReplayToTrainingPlugin::renderCaptureModeSelector() {
  static const char *modeLabels[] = {"Striker (shots)", "Goalie (saves)"};
  int modeIndex = selectedCaptureMode() == CaptureMode::Save ? 1 : 0;
  if (ImGui::Combo("Capture mode", &modeIndex, modeLabels, 2)) {
    setSelectedCaptureMode(modeIndex == 1 ? CaptureMode::Save
                                          : CaptureMode::Shot);
  }
  if (ImGui::IsItemHovered()) {
    ImGui::SetTooltip(
        "Mode used by replay_to_training_capture (bindable single key).\n"
        "capture_shot/capture_save set it; opening or targeting a typed\n"
        "pack syncs it. Captures that contradict the active pack's type\n"
        "are refused.");
  }
}

// Cvar-backed capture tunables (the plugin's commands are all zero-arg, so
// these persisted defaults are the only way capture behavior is tuned).
void ReplayToTrainingPlugin::renderCaptureToggles() {
  renderCaptureModeSelector();
  float timeLimit = timeLimitSeconds();
  if (ImGui::SliderFloat("Time limit (s)", &timeLimit, 1.0f, 120.0f, "%.0f")) {
    auto cvar = cvarManager->getCvar("replay_to_training_time_limit");
    if (static_cast<bool>(cvar)) {
      cvar.setValue(timeLimit);
    }
  }

  bool mirror = mirrorByTeamEnabled();
  if (ImGui::Checkbox("Mirror by team", &mirror)) {
    setCvarString("replay_to_training_mirror_by_team", mirror ? "1" : "0");
  }
  if (ImGui::IsItemHovered()) {
    ImGui::SetTooltip(
        "Flip captures 180 degrees about field center when the captured\n"
        "player's team does not match the training convention (striker\n"
        "shots attack +Y, goalie saves defend -Y; orange captures flip).");
  }
  ImGui::SameLine();
  bool momentum = captureMomentumEnabled();
  if (ImGui::Checkbox("Capture momentum", &momentum)) {
    setCvarString("replay_to_training_capture_momentum", momentum ? "1" : "0");
  }
  if (ImGui::IsItemHovered()) {
    ImGui::SetTooltip(
        "Write the car's forward speed into the spawn point so the\n"
        "training car starts moving like it did in the replay.");
  }
  ImGui::SameLine();
  bool autosave = autosaveEnabled();
  if (ImGui::Checkbox("Autosave", &autosave)) {
    setCvarString("replay_to_training_autosave", autosave ? "1" : "0");
  }
  if (!autosave) {
    ImGui::TextWrapped(
        "Autosave is off: captured shots stay in memory until you save.");
  }

  renderMomentumWarningControls();
}

// Compact "Momentum warning" group: the three thresholds that gate the
// capture-time momentum-loss diagnostic (shared into both the F2 settings
// page and the standalone capture window via renderCaptureToggles).
void ReplayToTrainingPlugin::renderMomentumWarningControls() {
  if (!ImGui::CollapsingHeader("Momentum warning")) {
    return;
  }
  float minSpeed = momentumWarnMinSpeed();
  if (ImGui::SliderFloat("Min speed (uu/s)", &minSpeed, 0.0f, 3000.0f, "%.0f")) {
    auto cvar = cvarManager->getCvar("replay_to_training_momentum_warn_min_speed");
    if (static_cast<bool>(cvar)) {
      cvar.setValue(minSpeed);
    }
  }
  if (ImGui::IsItemHovered()) {
    ImGui::SetTooltip(
        "Total speed below which no warning is raised. Set this above the\n"
        "fastest speed you capture at to disable the warning entirely.");
  }
  float minLost = momentumWarnMinLost();
  if (ImGui::SliderFloat("Min lost (uu/s)", &minLost, 0.0f, 3000.0f, "%.0f")) {
    auto cvar = cvarManager->getCvar("replay_to_training_momentum_warn_min_lost");
    if (static_cast<bool>(cvar)) {
      cvar.setValue(minLost);
    }
  }
  if (ImGui::IsItemHovered()) {
    ImGui::SetTooltip(
        "Unrepresentable (lost) velocity magnitude above which a warning\n"
        "is raised, whatever the off-axis angle.");
  }
  float maxAngle = momentumWarnMaxAngle();
  if (ImGui::SliderFloat("Max off-axis (deg)", &maxAngle, 1.0f, 90.0f, "%.0f")) {
    auto cvar = cvarManager->getCvar("replay_to_training_momentum_warn_max_angle");
    if (static_cast<bool>(cvar)) {
      cvar.setValue(maxAngle);
    }
  }
  if (ImGui::IsItemHovered()) {
    ImGui::SetTooltip(
        "Angle between velocity and facing above which a warning is\n"
        "raised, even when the lost magnitude is modest.");
  }
}

// Target (persistent default save) display, set/clear, and the
// discovered-target pick list.
void ReplayToTrainingPlugin::renderTargetControls() {
  const bool hasTarget = !activeTargetPath.empty();
  if (hasTarget) {
    ImGui::TextWrapped("Active target: %s -> %s",
                       targetBuffer.data(),
                       activeTargetPath.string().c_str());
    ImGui::TextWrapped(
        "Captures append here; Save writes back non-destructively.");
  } else {
    ImGui::TextWrapped(
        "No target set. Saves go to an auto <GUID>.Tem in the account's "
        "MyTraining folder (or the Training root).");
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
}

// New pack / capture (shot and save) / save buttons. The two capture
// buttons mirror the two zero-arg notifiers: they set the persisted mode
// selection and capture with it (offense vs defense).
void ReplayToTrainingPlugin::renderPackActions() {
  if (ImGui::Button("New pack")) {
    gameWrapper->Execute([this](GameWrapper *) { newPack(); });
  }
  ImGui::SameLine();
  const bool inReplay = gameWrapper->IsInReplay();
  // The SDK bundles ImGui 1.75 (no BeginDisabled); emulate by branching.
  if (ImGui::Button(inReplay ? "Capture shot" : "Capture shot (needs replay)")) {
    if (inReplay) {
      // Like the capture_shot notifier: update the mode selection, then
      // capture. Capture touches game wrappers, so hop to the game thread.
      gameWrapper->Execute([this](GameWrapper *) {
        setSelectedCaptureMode(CaptureMode::Shot);
        captureShot(CaptureMode::Shot);
      });
    } else {
      setStatus("capture requires an in-game replay");
    }
  }
  ImGui::SameLine();
  if (ImGui::Button(inReplay ? "Capture save" : "Capture save (needs replay)")) {
    if (inReplay) {
      // Like the capture_save notifier: selection follows the button.
      gameWrapper->Execute([this](GameWrapper *) {
        setSelectedCaptureMode(CaptureMode::Save);
        captureShot(CaptureMode::Save);
      });
    } else {
      setStatus("capture requires an in-game replay");
    }
  }
  ImGui::SameLine();
  // The save button reflects target-vs-GUID so the destination is clear.
  const char *saveLabel =
      activeTargetPath.empty() ? "Save pack (auto GUID)" : "Save to target";
  if (ImGui::Button(saveLabel)) {
    gameWrapper->Execute([this](GameWrapper *) { savePack(); });
  }
}

void ReplayToTrainingPlugin::renderShotList() {
  const size_t count = packShotCount ? packShotCount(pack) : 0;
  if (activeTargetPath.empty()) {
    ImGui::Text("Shots: %zu (pack %s)", count, packGuidHexString().c_str());
  } else {
    ImGui::Text("Shots: %zu (target %s, pack %s)", count, targetBuffer.data(),
                packGuidHexString().c_str());
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
}

void ReplayToTrainingPlugin::renderStatusLine() {
  // Dedicated momentum-warning line: sticks around (unlike the transient
  // status text) until the next warning-free capture or a new pack, so a
  // capture whose car velocity was mostly unrepresentable stays visible.
  // PushStyleColor rather than TextColored so the text still wraps.
  if (!momentumWarningLine.empty()) {
    ImGui::PushStyleColor(ImGuiCol_Text, ImVec4(1.0f, 0.65f, 0.15f, 1.0f));
    ImGui::TextWrapped("Momentum warning (last capture): %s",
                       momentumWarningLine.c_str());
    ImGui::PopStyleColor();
  }
  ImGui::TextWrapped("Status: %s", statusLine.c_str());
}

// --- F2 > Plugins settings page ---

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

  ImGui::TextWrapped(
      "Tip: `togglemenu replaytotraining` (or the "
      "replay_to_training_window command) opens the standalone capture "
      "window for use while watching a replay.");

  ImGui::Separator();
  ImGui::Text("Training pack");
  renderPackMetadataControls();
  renderPackTypeControls();
  if (ImGui::InputText(
          "Output directory", outputDirBuffer.data(), outputDirBuffer.size())) {
    setCvarString("replay_to_training_output_dir", outputDirBuffer.data());
  }
  ImGui::TextWrapped(
      "Empty output directory / training root resolves to Documents\\My "
      "Games\\Rocket League\\TAGame\\Training. Saves land in your account's "
      "MyTraining\\ under that root, which is what the game lists.");

  ImGui::Separator();
  ImGui::Text("Target (default save)");
  renderTargetControls();

  ImGui::Separator();
  ImGui::Text("Capture");
  renderCaptureToggles();
  renderPackActions();

  ImGui::Separator();
  ImGui::InputText("Open path", openPathBuffer.data(), openPathBuffer.size());
  ImGui::SameLine();
  if (ImGui::Button("Open pack")) {
    const std::string path = openPathBuffer.data();
    gameWrapper->Execute([this, path](GameWrapper *) { openPackFromPath(path); });
  }

  ImGui::Separator();
  renderShotList();

  ImGui::Separator();
  renderStatusLine();
}
