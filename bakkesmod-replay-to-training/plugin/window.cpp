// Standalone in-game capture window (BakkesMod::Plugin::PluginWindow).
//
// BakkesMod discovers the PluginWindow interface on the plugin class at
// load time and binds it to `togglemenu <GetMenuName()>`; OnOpen/OnClose
// track BakkesMod's open state, Render() draws only while open, and
// closing the window through its ImGui X button reports back through the
// same `togglemenu` command so the two stay in sync (the standard SDK
// plugin-window pattern).
//
// This is the one-stop capture HUD meant to float over a replay: every
// control is backed by its persisted cvar (commands stay zero-arg), and
// the building blocks are shared with the F2 settings page
// (settings_window.cpp). Render() runs on the render thread; the shared
// helpers hop to the game thread via gameWrapper->Execute for anything
// that mutates game or pack state.

namespace {

// `togglemenu` name (lowercase, no spaces) and the window title bar text.
constexpr const char *REPLAY_TO_TRAINING_MENU_NAME = "replaytotraining";
constexpr const char *REPLAY_TO_TRAINING_MENU_TITLE = "Replay to Training";

}  // namespace

std::string ReplayToTrainingPlugin::GetMenuName() {
  return REPLAY_TO_TRAINING_MENU_NAME;
}

std::string ReplayToTrainingPlugin::GetMenuTitle() {
  return REPLAY_TO_TRAINING_MENU_TITLE;
}

void ReplayToTrainingPlugin::OnOpen() {
  captureWindowOpen = true;
}

void ReplayToTrainingPlugin::OnClose() {
  captureWindowOpen = false;
}

// Block game input only while the window is actually being interacted
// with, so the replay stays scrubable around it.
bool ReplayToTrainingPlugin::ShouldBlockInput() {
  const ImGuiIO &io = ImGui::GetIO();
  return io.WantCaptureMouse || io.WantCaptureKeyboard;
}

// A real interactive window, not a passive overlay.
bool ReplayToTrainingPlugin::IsActiveOverlay() {
  return true;
}

void ReplayToTrainingPlugin::Render() {
  if (imguiContext != 0) {
    ImGui::SetCurrentContext(reinterpret_cast<ImGuiContext *>(imguiContext));
  }
  if (!captureWindowOpen) {
    return;
  }

  ImGui::SetNextWindowSize(ImVec2(460, 560), ImGuiCond_FirstUseEver);
  if (!ImGui::Begin(REPLAY_TO_TRAINING_MENU_TITLE, &captureWindowOpen)) {
    ImGui::End();
    // Collapsed windows still count as open; only the X button below
    // reports a close.
    return;
  }

  if (!rustLoaded) {
    ImGui::TextWrapped(
        "replay_to_training.dll is not loaded. Install it to "
        "bakkesmod\\data\\replay-to-training\\replay_to_training.dll and "
        "reload the plugin.");
    renderStatusLine();
    ImGui::End();
    return;
  }

  // Compact one-stop capture layout: type, tunables, metadata, target,
  // actions, shot list, status.
  renderPackTypeControls();
  ImGui::Separator();
  renderCaptureToggles();
  ImGui::Separator();
  renderPackMetadataControls();
  ImGui::Separator();
  renderTargetControls();
  ImGui::Separator();
  renderPackActions();
  ImGui::Separator();
  renderShotList();
  ImGui::Separator();
  renderStatusLine();

  ImGui::End();

  if (!captureWindowOpen) {
    // The X button was clicked: run the toggle so BakkesMod's own
    // open-state (and OnClose) stay in sync with ours.
    cvarManager->executeCommand("togglemenu " +
                                std::string(REPLAY_TO_TRAINING_MENU_NAME));
  }
}
