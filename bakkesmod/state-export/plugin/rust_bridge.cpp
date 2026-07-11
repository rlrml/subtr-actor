// Included by StateExportPlugin.cpp; shares the plugin translation unit.
//
// Runtime loading of state_export.dll (the Rust cdylib built from
// bakkesmod/state-export/rust). Mirrors the replay-to-training approach:
// LoadLibraryW over a small search-path list, GetProcAddress for every
// exported symbol, all-or-nothing with cvarManager->log diagnostics, and
// graceful degradation when the DLL is missing.

namespace {

constexpr wchar_t STATE_EXPORT_RUST_DLL_NAME[] = L"state_export.dll";

std::filesystem::path stateExportModuleDirectory() {
  HMODULE module = nullptr;
  static int moduleAnchor = 0;
  const BOOL foundModule = GetModuleHandleExW(
      GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS |
          GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
      reinterpret_cast<LPCWSTR>(&moduleAnchor),
      &module);
  if (!foundModule || !module) {
    return {};
  }

  std::array<wchar_t, 32768> pathBuffer{};
  const DWORD length =
      GetModuleFileNameW(module, pathBuffer.data(), static_cast<DWORD>(pathBuffer.size()));
  if (length == 0 || length >= pathBuffer.size()) {
    return {};
  }

  return std::filesystem::path(pathBuffer.data()).parent_path();
}

}  // namespace

std::vector<std::filesystem::path> StateExportPlugin::rustLibrarySearchPaths() {
  std::vector<std::filesystem::path> paths;
  const auto moduleDirectory = stateExportModuleDirectory();
  if (!moduleDirectory.empty()) {
    paths.push_back(moduleDirectory / STATE_EXPORT_RUST_DLL_NAME);
  }
  if (gameWrapper) {
    paths.push_back(gameWrapper->GetDataFolder() / "state-export" / STATE_EXPORT_RUST_DLL_NAME);
  }
  paths.emplace_back(STATE_EXPORT_RUST_DLL_NAME);
  return paths;
}

bool StateExportPlugin::loadRustLibrary() {
  for (const auto &dllPath : rustLibrarySearchPaths()) {
    rustLibrary = LoadLibraryW(dllPath.c_str());
    if (rustLibrary) {
      cvarManager->log(
          std::format("state-export: loaded Rust ABI from {}", dllPath.string()));
      break;
    }
  }
  if (!rustLibrary) {
    cvarManager->log(
        std::format("state-export: LoadLibrary failed with error {}", GetLastError()));
    return false;
  }

  engineCreate = reinterpret_cast<EngineCreate>(
      GetProcAddress(rustLibrary, "state_export_engine_create"));
  engineDestroy = reinterpret_cast<EngineDestroy>(
      GetProcAddress(rustLibrary, "state_export_engine_destroy"));
  engineRestart = reinterpret_cast<EngineRestart>(
      GetProcAddress(rustLibrary, "state_export_engine_restart"));
  pushFrame = reinterpret_cast<PushFrame>(
      GetProcAddress(rustLibrary, "state_export_push_frame"));
  setMatchContext = reinterpret_cast<SetMatchContext>(
      GetProcAddress(rustLibrary, "state_export_set_match_context"));
  notifyMatchEnd = reinterpret_cast<NotifyMatchEnd>(
      GetProcAddress(rustLibrary, "state_export_notify_match_end"));
  engineStatus = reinterpret_cast<EngineStatus>(
      GetProcAddress(rustLibrary, "state_export_status"));
  lastErrorLen = reinterpret_cast<LastErrorLen>(
      GetProcAddress(rustLibrary, "state_export_last_error_len"));
  writeLastError = reinterpret_cast<WriteLastError>(
      GetProcAddress(rustLibrary, "state_export_write_last_error"));
  rustBuildInfoLen = reinterpret_cast<BuildInfoLen>(
      GetProcAddress(rustLibrary, "state_export_build_info_len"));
  rustWriteBuildInfo = reinterpret_cast<WriteBuildInfo>(
      GetProcAddress(rustLibrary, "state_export_write_build_info"));

  const bool complete = engineCreate && engineDestroy && engineRestart && pushFrame &&
      setMatchContext && notifyMatchEnd && engineStatus && lastErrorLen &&
      writeLastError && rustBuildInfoLen && rustWriteBuildInfo;
  if (!complete) {
    cvarManager->log(
        "state-export: state_export.dll is missing expected exports; "
        "the DLL likely does not match this plugin build");
    unloadRustLibrary();
    return false;
  }
  return true;
}

void StateExportPlugin::unloadRustLibrary() {
  // The engine owns server threads inside the Rust DLL; it MUST be destroyed
  // before FreeLibrary unmaps their code.
  destroyEngine();
  engineCreate = nullptr;
  engineDestroy = nullptr;
  engineRestart = nullptr;
  pushFrame = nullptr;
  setMatchContext = nullptr;
  notifyMatchEnd = nullptr;
  engineStatus = nullptr;
  lastErrorLen = nullptr;
  writeLastError = nullptr;
  rustBuildInfoLen = nullptr;
  rustWriteBuildInfo = nullptr;
  if (rustLibrary) {
    FreeLibrary(rustLibrary);
    rustLibrary = nullptr;
  }
  rustLoaded = false;
}

std::string StateExportPlugin::engineErrorMessage() {
  if (!engine || !lastErrorLen || !writeLastError) {
    return "rust library not loaded";
  }
  const size_t length = lastErrorLen(engine);
  if (length == 0) {
    return {};
  }
  std::string message(length, '\0');
  const size_t written =
      writeLastError(engine, reinterpret_cast<uint8_t *>(message.data()), message.size());
  message.resize(written);
  return message;
}

std::string StateExportPlugin::rustCoreBuildInfo() {
  if (!rustBuildInfoLen || !rustWriteBuildInfo) {
    return "not loaded";
  }
  const size_t length = rustBuildInfoLen();
  if (length == 0) {
    return "unknown";
  }
  std::string info(length, '\0');
  const size_t written =
      rustWriteBuildInfo(reinterpret_cast<uint8_t *>(info.data()), info.size());
  info.resize(written);
  return info;
}
