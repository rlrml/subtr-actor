// Included by StateExportPlugin.cpp; shares the plugin translation unit.
//
// Plugin lifecycle: cvar/notifier registration, Rust DLL loading, engine
// create/destroy/restart (the engine MUST be destroyed before the DLL is
// freed in onUnload), and the cvar accessors the tick reads.
std::string StateExportPlugin::buildId() const {
  return std::format(
      "state-export plugin {} build={} dirty={} commit_date={}",
      STATE_EXPORT_PLUGIN_VERSION,
      STATE_EXPORT_GIT_HASH,
      STATE_EXPORT_GIT_DIRTY ? 1 : 0,
      STATE_EXPORT_COMMIT_DATE);
}

// Logs both halves of the shipped DLL pair so a mismatched
// plugin/rust-core install is immediately visible.
void StateExportPlugin::logVersion() {
  cvarManager->log(buildId());
  cvarManager->log(std::format("rust core: {}", rustCoreBuildInfo()));
}

void StateExportPlugin::onLoad() {
  registerCvarsAndNotifiers();

  rustLoaded = loadRustLibrary();
  logVersion();
  if (!rustLoaded) {
    cvarManager->log("state-export: state_export.dll not found; export disabled");
    return;
  }

  createEngine();
  hookGameEvents();
  exportTickCancelled = std::make_shared<bool>(false);
  scheduleExportTick();
}

void StateExportPlugin::onUnload() {
  if (exportTickCancelled) {
    *exportTickCancelled = true;
  }
  unhookGameEvents();
  // unloadRustLibrary destroys the engine (joining the server threads that
  // live inside state_export.dll) before FreeLibrary unmaps the DLL.
  unloadRustLibrary();
}

void StateExportPlugin::registerCvarsAndNotifiers() {
  cvarManager->registerCvar(
      "state_export_enabled",
      "1",
      "Enable live game-state export over the WebSocket server.",
      true,
      true,
      0,
      true,
      1);
  cvarManager->registerCvar(
      "state_export_port",
      "49109",
      "TCP port the state-export WebSocket server listens on (0 binds an "
      "ephemeral port; read the bound port back via state_export_status). "
      "Applied by state_export_restart_server or the settings-page Apply.",
      true,
      true,
      0,
      true,
      65535);
  cvarManager->registerCvar(
      "state_export_bind_all_interfaces",
      "0",
      "Bind the export server to 0.0.0.0 instead of 127.0.0.1, exposing raw "
      "game state to every host that can reach this machine (LAN/VPN). Leave "
      "off unless remote consumers need it. Applied on server restart.",
      true,
      true,
      0,
      true,
      1);
  cvarManager->registerCvar(
      "state_export_sample_interval_ms",
      "8",
      "Minimum elapsed game time between exported frame samples.",
      true,
      true,
      1,
      true,
      1000);
  cvarManager->registerCvar(
      "state_export_sample_when_no_clients",
      "0",
      "Sample and enqueue frames even when no WebSocket client is connected "
      "(normally sampling is skipped so an idle server costs nothing).",
      true,
      true,
      0,
      true,
      1);

  cvarManager->registerNotifier(
      "state_export_restart_server",
      [this](std::vector<std::string>) { restartServer(); },
      "Restarts the export WebSocket server with the current "
      "state_export_port / state_export_bind_all_interfaces cvar values.",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "state_export_status",
      [this](std::vector<std::string>) { logStatus(); },
      "Logs the export server status (state, port, clients, frames "
      "sent/dropped) and both build identifiers to the console.",
      PERMISSION_ALL);
}

SeConfig StateExportPlugin::configFromCvars() {
  SeConfig config{};
  config.server_name = nullptr;
  config.max_queued_frames = 0;
  config.max_client_queue = 0;
  config.port = static_cast<uint16_t>(std::clamp(configuredPort(), 0, 65535));
  config.bind_any_interface = bindAllInterfacesEnabled() ? 1 : 0;
  return config;
}

void StateExportPlugin::createEngine() {
  if (!engineCreate) {
    return;
  }

  const SeConfig config = configFromCvars();
  // engine_create never returns null: a bind failure comes back as an
  // engine in the error state with the message in last-error.
  engine = engineCreate(&config);
  refreshStatus();
  if (lastStatus.state == SE_STATE_ERROR) {
    cvarManager->log(std::format(
        "state-export: server failed to start: {}", engineErrorMessage()));
  } else {
    cvarManager->log(std::format(
        "state-export: server listening on port {}", lastStatus.port));
  }
}

void StateExportPlugin::destroyEngine() {
  if (engine && engineDestroy) {
    engineDestroy(engine);
  }
  engine = nullptr;
  lastStatus = SeStatus{};
}

void StateExportPlugin::restartServer() {
  if (!engine || !engineRestart) {
    cvarManager->log("state-export: rust library not loaded");
    return;
  }

  const SeConfig config = configFromCvars();
  const int32_t result = engineRestart(engine, &config);
  refreshStatus();
  if (result != 0) {
    cvarManager->log(std::format(
        "state-export: server restart failed: {}", engineErrorMessage()));
    return;
  }
  cvarManager->log(std::format(
      "state-export: server restarted on port {}{}",
      lastStatus.port,
      bindAllInterfacesEnabled() ? " (all interfaces)" : ""));
}

void StateExportPlugin::logStatus() {
  if (!engine) {
    cvarManager->log("state-export: no engine (rust library not loaded)");
    return;
  }

  refreshStatus();
  cvarManager->log(std::format(
      "state-export: {} port={} clients={} frames_sent={} frames_dropped={}",
      seStatusStateLabel(lastStatus.state),
      lastStatus.port,
      lastStatus.client_count,
      lastStatus.frames_sent,
      lastStatus.frames_dropped));
  const std::string error = engineErrorMessage();
  if (!error.empty()) {
    cvarManager->log(std::format("state-export: last error: {}", error));
  }
  logVersion();
}

bool StateExportPlugin::exportEnabled() {
  return cvarBool("state_export_enabled", true);
}

bool StateExportPlugin::bindAllInterfacesEnabled() {
  return cvarBool("state_export_bind_all_interfaces", false);
}

bool StateExportPlugin::sampleWhenNoClientsEnabled() {
  return cvarBool("state_export_sample_when_no_clients", false);
}

int StateExportPlugin::configuredPort() {
  return cvarInt("state_export_port", SE_DEFAULT_STATE_EXPORT_PORT);
}

float StateExportPlugin::sampleIntervalSeconds() {
  const float intervalMs = std::clamp(
      static_cast<float>(cvarInt("state_export_sample_interval_ms", 8)), 1.0f, 1000.0f);
  return intervalMs / 1000.0f;
}

bool StateExportPlugin::cvarBool(const char *name, bool defaultValue) const {
  auto cvar = cvarManager->getCvar(name);
  return static_cast<bool>(cvar) ? cvar.getBoolValue() : defaultValue;
}

int StateExportPlugin::cvarInt(const char *name, int defaultValue) const {
  auto cvar = cvarManager->getCvar(name);
  return static_cast<bool>(cvar) ? cvar.getIntValue() : defaultValue;
}
