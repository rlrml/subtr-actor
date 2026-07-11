// Included by StateExportPlugin.cpp; shares the plugin translation unit.
//
// F2 > Plugins settings page. Everything here runs on the render thread; any
// call that mutates game or engine state hops to the game thread via
// gameWrapper->Execute (the SDK convention the other plugins follow).
std::string StateExportPlugin::GetPluginName() {
  return "state-export";
}

void StateExportPlugin::SetImGuiContext(uintptr_t ctx) {
  imguiContext = ctx;
  ImGui::SetCurrentContext(reinterpret_cast<ImGuiContext *>(ctx));
}

void StateExportPlugin::RenderSettings() {
  if (imguiContext != 0) {
    ImGui::SetCurrentContext(reinterpret_cast<ImGuiContext *>(imguiContext));
  }

  if (!rustLoaded) {
    ImGui::TextWrapped(
        "state_export.dll is not loaded. Install it to "
        "bakkesmod\\data\\state-export\\state_export.dll and reload the "
        "plugin.");
    return;
  }

  bool enabled = exportEnabled();
  if (ImGui::Checkbox("Enable state export", &enabled)) {
    auto cvar = cvarManager->getCvar("state_export_enabled");
    if (static_cast<bool>(cvar)) {
      cvar.setValue(enabled ? 1 : 0);
    }
  }

  ImGui::Separator();
  ImGui::Text("Server");
  int port = configuredPort();
  if (ImGui::InputInt("Port", &port)) {
    auto cvar = cvarManager->getCvar("state_export_port");
    if (static_cast<bool>(cvar)) {
      cvar.setValue(std::clamp(port, 0, 65535));
    }
  }
  bool bindAll = bindAllInterfacesEnabled();
  if (ImGui::Checkbox("Bind all interfaces (0.0.0.0)", &bindAll)) {
    auto cvar = cvarManager->getCvar("state_export_bind_all_interfaces");
    if (static_cast<bool>(cvar)) {
      cvar.setValue(bindAll ? 1 : 0);
    }
  }
  if (ImGui::IsItemHovered()) {
    ImGui::SetTooltip(
        "Exposes raw game state to every host that can reach this machine\n"
        "(LAN/VPN). Leave off unless remote consumers need it.");
  }
  if (ImGui::Button("Apply (restart server)")) {
    gameWrapper->Execute([this](GameWrapper *) { restartServer(); });
  }

  ImGui::Separator();
  ImGui::Text("Sampling");
  int intervalMs = cvarInt("state_export_sample_interval_ms", 8);
  if (ImGui::SliderInt("Sample interval (ms)", &intervalMs, 1, 1000)) {
    auto cvar = cvarManager->getCvar("state_export_sample_interval_ms");
    if (static_cast<bool>(cvar)) {
      cvar.setValue(intervalMs);
    }
  }
  bool sampleWhenNoClients = sampleWhenNoClientsEnabled();
  if (ImGui::Checkbox("Sample with no clients connected", &sampleWhenNoClients)) {
    auto cvar = cvarManager->getCvar("state_export_sample_when_no_clients");
    if (static_cast<bool>(cvar)) {
      cvar.setValue(sampleWhenNoClients ? 1 : 0);
    }
  }

  ImGui::Separator();
  // lastStatus is refreshed by the game-thread tick (atomics-only on the
  // Rust side); rendering the cached copy avoids a cross-thread engine call.
  ImGui::Text(
      "Status: %s, port %u, %u client(s), %llu frames sent, %llu dropped",
      seStatusStateLabel(lastStatus.state),
      static_cast<unsigned>(lastStatus.port),
      static_cast<unsigned>(lastStatus.client_count),
      static_cast<unsigned long long>(lastStatus.frames_sent),
      static_cast<unsigned long long>(lastStatus.frames_dropped));
  if (!lastErrorText.empty()) {
    ImGui::PushStyleColor(ImGuiCol_Text, ImVec4(1.0f, 0.45f, 0.35f, 1.0f));
    ImGui::TextWrapped("Last error: %s", lastErrorText.c_str());
    ImGui::PopStyleColor();
  }
  ImGui::TextWrapped(
      "Consumer: ws://127.0.0.1:%u (subtr-actor-live protocol; postcard "
      "default, ?format=json for JSON)",
      static_cast<unsigned>(lastStatus.port));
  ImGui::TextWrapped("%s", buildId().c_str());
  ImGui::TextWrapped("rust core: %s", rustCoreBuildInfo().c_str());
}
