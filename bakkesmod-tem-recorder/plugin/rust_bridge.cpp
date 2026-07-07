// Runtime loading of tem_recorder.dll (the Rust cdylib built from
// bakkesmod-tem-recorder/rust). Mirrors the SubtrActorPlugin approach:
// LoadLibraryW over a small search-path list, GetProcAddress for every
// exported symbol, all-or-nothing with cvarManager->log diagnostics, and
// graceful degradation when the DLL is missing.

namespace {

constexpr wchar_t TEM_RECORDER_RUST_DLL_NAME[] = L"tem_recorder.dll";

std::filesystem::path temRecorderModuleDirectory() {
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

std::vector<std::filesystem::path> TemRecorderPlugin::rustLibrarySearchPaths() {
  std::vector<std::filesystem::path> paths;
  const auto moduleDirectory = temRecorderModuleDirectory();
  if (!moduleDirectory.empty()) {
    paths.push_back(moduleDirectory / TEM_RECORDER_RUST_DLL_NAME);
  }
  if (gameWrapper) {
    paths.push_back(gameWrapper->GetDataFolder() / "tem-recorder" / TEM_RECORDER_RUST_DLL_NAME);
  }
  paths.emplace_back(TEM_RECORDER_RUST_DLL_NAME);
  return paths;
}

bool TemRecorderPlugin::loadRustLibrary() {
  for (const auto &dllPath : rustLibrarySearchPaths()) {
    rustLibrary = LoadLibraryW(dllPath.c_str());
    if (rustLibrary) {
      cvarManager->log(
          std::format("tem-recorder: loaded Rust ABI from {}", dllPath.string()));
      break;
    }
  }
  if (!rustLibrary) {
    cvarManager->log(
        std::format("tem-recorder: LoadLibrary failed with error {}", GetLastError()));
    return false;
  }

  packCreate = reinterpret_cast<PackCreate>(
      GetProcAddress(rustLibrary, "tem_recorder_pack_create"));
  packOpen = reinterpret_cast<PackOpen>(
      GetProcAddress(rustLibrary, "tem_recorder_pack_open"));
  packDestroy = reinterpret_cast<PackDestroy>(
      GetProcAddress(rustLibrary, "tem_recorder_pack_destroy"));
  packSetName = reinterpret_cast<PackSetString>(
      GetProcAddress(rustLibrary, "tem_recorder_pack_set_name"));
  packSetCode = reinterpret_cast<PackSetString>(
      GetProcAddress(rustLibrary, "tem_recorder_pack_set_code"));
  packSetCreatorName = reinterpret_cast<PackSetString>(
      GetProcAddress(rustLibrary, "tem_recorder_pack_set_creator_name"));
  packSetMapName = reinterpret_cast<PackSetString>(
      GetProcAddress(rustLibrary, "tem_recorder_pack_set_map_name"));
  packSetDifficulty = reinterpret_cast<PackSetDifficulty>(
      GetProcAddress(rustLibrary, "tem_recorder_pack_set_difficulty"));
  packDifficulty = reinterpret_cast<PackDifficulty>(
      GetProcAddress(rustLibrary, "tem_recorder_pack_difficulty"));
  packNameLen = reinterpret_cast<PackStringLen>(
      GetProcAddress(rustLibrary, "tem_recorder_pack_name_len"));
  packWriteName = reinterpret_cast<PackWriteString>(
      GetProcAddress(rustLibrary, "tem_recorder_pack_write_name"));
  packAddShot = reinterpret_cast<PackAddShot>(
      GetProcAddress(rustLibrary, "tem_recorder_pack_add_shot"));
  packRemoveShot = reinterpret_cast<PackRemoveShot>(
      GetProcAddress(rustLibrary, "tem_recorder_pack_remove_shot"));
  packShotCount = reinterpret_cast<PackShotCount>(
      GetProcAddress(rustLibrary, "tem_recorder_pack_shot_count"));
  packShotSummaryLen = reinterpret_cast<PackShotSummaryLen>(
      GetProcAddress(rustLibrary, "tem_recorder_pack_shot_summary_len"));
  packWriteShotSummary = reinterpret_cast<PackWriteShotSummary>(
      GetProcAddress(rustLibrary, "tem_recorder_pack_write_shot_summary"));
  packGuidHex = reinterpret_cast<PackWriteString>(
      GetProcAddress(rustLibrary, "tem_recorder_pack_guid_hex"));
  packSave = reinterpret_cast<PackSave>(
      GetProcAddress(rustLibrary, "tem_recorder_pack_save"));
  packLastErrorLen = reinterpret_cast<PackStringLen>(
      GetProcAddress(rustLibrary, "tem_recorder_pack_last_error_len"));
  packWriteLastError = reinterpret_cast<PackWriteString>(
      GetProcAddress(rustLibrary, "tem_recorder_pack_write_last_error"));
  globalLastErrorLen = reinterpret_cast<GlobalErrorLen>(
      GetProcAddress(rustLibrary, "tem_recorder_last_error_len"));
  globalWriteLastError = reinterpret_cast<GlobalWriteError>(
      GetProcAddress(rustLibrary, "tem_recorder_write_last_error"));

  const bool complete = packCreate && packOpen && packDestroy && packSetName &&
      packSetCode && packSetCreatorName && packSetMapName && packSetDifficulty &&
      packDifficulty && packNameLen && packWriteName && packAddShot &&
      packRemoveShot && packShotCount && packShotSummaryLen &&
      packWriteShotSummary && packGuidHex && packSave && packLastErrorLen &&
      packWriteLastError && globalLastErrorLen && globalWriteLastError;
  if (!complete) {
    cvarManager->log(
        "tem-recorder: tem_recorder.dll is missing expected exports; "
        "the DLL likely does not match this plugin build");
    unloadRustLibrary();
    return false;
  }
  return true;
}

void TemRecorderPlugin::unloadRustLibrary() {
  if (pack && packDestroy) {
    packDestroy(pack);
  }
  pack = nullptr;
  packCreate = nullptr;
  packOpen = nullptr;
  packDestroy = nullptr;
  packSetName = nullptr;
  packSetCode = nullptr;
  packSetCreatorName = nullptr;
  packSetMapName = nullptr;
  packSetDifficulty = nullptr;
  packDifficulty = nullptr;
  packNameLen = nullptr;
  packWriteName = nullptr;
  packAddShot = nullptr;
  packRemoveShot = nullptr;
  packShotCount = nullptr;
  packShotSummaryLen = nullptr;
  packWriteShotSummary = nullptr;
  packGuidHex = nullptr;
  packSave = nullptr;
  packLastErrorLen = nullptr;
  packWriteLastError = nullptr;
  globalLastErrorLen = nullptr;
  globalWriteLastError = nullptr;
  if (rustLibrary) {
    FreeLibrary(rustLibrary);
    rustLibrary = nullptr;
  }
  rustLoaded = false;
}

std::string TemRecorderPlugin::packErrorMessage() {
  if (!pack || !packLastErrorLen || !packWriteLastError) {
    return "rust library not loaded";
  }
  const size_t length = packLastErrorLen(pack);
  if (length == 0) {
    return "unknown error";
  }
  std::string message(length, '\0');
  const size_t written =
      packWriteLastError(pack, reinterpret_cast<uint8_t *>(message.data()), message.size());
  message.resize(written);
  return message;
}

std::string TemRecorderPlugin::globalErrorMessage() {
  if (!globalLastErrorLen || !globalWriteLastError) {
    return "rust library not loaded";
  }
  const size_t length = globalLastErrorLen();
  if (length == 0) {
    return "unknown error";
  }
  std::string message(length, '\0');
  const size_t written =
      globalWriteLastError(reinterpret_cast<uint8_t *>(message.data()), message.size());
  message.resize(written);
  return message;
}

std::string TemRecorderPlugin::packGuidHexString() {
  if (!pack || !packGuidHex) {
    return {};
  }
  std::string hex(32, '\0');
  const size_t written =
      packGuidHex(pack, reinterpret_cast<uint8_t *>(hex.data()), hex.size());
  hex.resize(written);
  return hex;
}

std::string TemRecorderPlugin::shotSummary(size_t index) {
  if (!pack || !packShotSummaryLen || !packWriteShotSummary) {
    return {};
  }
  const size_t length = packShotSummaryLen(pack, index);
  if (length == 0) {
    return {};
  }
  std::string summary(length, '\0');
  const size_t written = packWriteShotSummary(
      pack, index, reinterpret_cast<uint8_t *>(summary.data()), summary.size());
  summary.resize(written);
  return summary;
}
