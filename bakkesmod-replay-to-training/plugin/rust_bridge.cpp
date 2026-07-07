// Runtime loading of replay_to_training.dll (the Rust cdylib built from
// bakkesmod-replay-to-training/rust). Mirrors the SubtrActorPlugin approach:
// LoadLibraryW over a small search-path list, GetProcAddress for every
// exported symbol, all-or-nothing with cvarManager->log diagnostics, and
// graceful degradation when the DLL is missing.

namespace {

constexpr wchar_t REPLAY_TO_TRAINING_RUST_DLL_NAME[] = L"replay_to_training.dll";

std::filesystem::path replayToTrainingModuleDirectory() {
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

std::vector<std::filesystem::path> ReplayToTrainingPlugin::rustLibrarySearchPaths() {
  std::vector<std::filesystem::path> paths;
  const auto moduleDirectory = replayToTrainingModuleDirectory();
  if (!moduleDirectory.empty()) {
    paths.push_back(moduleDirectory / REPLAY_TO_TRAINING_RUST_DLL_NAME);
  }
  if (gameWrapper) {
    paths.push_back(gameWrapper->GetDataFolder() / "replay-to-training" / REPLAY_TO_TRAINING_RUST_DLL_NAME);
  }
  paths.emplace_back(REPLAY_TO_TRAINING_RUST_DLL_NAME);
  return paths;
}

bool ReplayToTrainingPlugin::loadRustLibrary() {
  for (const auto &dllPath : rustLibrarySearchPaths()) {
    rustLibrary = LoadLibraryW(dllPath.c_str());
    if (rustLibrary) {
      cvarManager->log(
          std::format("replay-to-training: loaded Rust ABI from {}", dllPath.string()));
      break;
    }
  }
  if (!rustLibrary) {
    cvarManager->log(
        std::format("replay-to-training: LoadLibrary failed with error {}", GetLastError()));
    return false;
  }

  packCreate = reinterpret_cast<PackCreate>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_create"));
  packOpen = reinterpret_cast<PackOpen>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_open"));
  packDestroy = reinterpret_cast<PackDestroy>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_destroy"));
  packSetName = reinterpret_cast<PackSetString>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_set_name"));
  packSetCode = reinterpret_cast<PackSetString>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_set_code"));
  packSetCreatorName = reinterpret_cast<PackSetString>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_set_creator_name"));
  packSetMapName = reinterpret_cast<PackSetString>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_set_map_name"));
  packSetDifficulty = reinterpret_cast<PackSetDifficulty>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_set_difficulty"));
  packDifficulty = reinterpret_cast<PackDifficulty>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_difficulty"));
  packSetTrainingType = reinterpret_cast<PackSetTrainingType>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_set_training_type"));
  packTrainingType = reinterpret_cast<PackTrainingType>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_training_type"));
  packCaptureModeSync = reinterpret_cast<PackCaptureModeSync>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_capture_mode_sync"));
  packNameLen = reinterpret_cast<PackStringLen>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_name_len"));
  packWriteName = reinterpret_cast<PackWriteString>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_write_name"));
  packAddShot = reinterpret_cast<PackAddShot>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_add_shot"));
  momentumNote = reinterpret_cast<MomentumNote>(
      GetProcAddress(rustLibrary, "replay_to_training_momentum_note"));
  packRemoveShot = reinterpret_cast<PackRemoveShot>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_remove_shot"));
  packShotCount = reinterpret_cast<PackShotCount>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_shot_count"));
  packShotSummaryLen = reinterpret_cast<PackShotSummaryLen>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_shot_summary_len"));
  packWriteShotSummary = reinterpret_cast<PackWriteShotSummary>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_write_shot_summary"));
  packGuidHex = reinterpret_cast<PackWriteString>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_guid_hex"));
  packSave = reinterpret_cast<PackSave>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_save"));
  packSaveToTarget = reinterpret_cast<PackSaveToTarget>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_save_to_target"));
  fileGuidHex = reinterpret_cast<FileGuidHex>(
      GetProcAddress(rustLibrary, "replay_to_training_file_guid_hex"));
  sanitizeTarget = reinterpret_cast<SanitizeTarget>(
      GetProcAddress(rustLibrary, "replay_to_training_sanitize_target"));
  targetsLen = reinterpret_cast<TargetsLen>(
      GetProcAddress(rustLibrary, "replay_to_training_targets_len"));
  writeTargets = reinterpret_cast<WriteTargets>(
      GetProcAddress(rustLibrary, "replay_to_training_write_targets"));
  resolveTarget = reinterpret_cast<ResolveTarget>(
      GetProcAddress(rustLibrary, "replay_to_training_resolve_target"));
  defaultSaveDir = reinterpret_cast<DefaultSaveDir>(
      GetProcAddress(rustLibrary, "replay_to_training_default_save_dir"));
  packLastErrorLen = reinterpret_cast<PackStringLen>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_last_error_len"));
  packWriteLastError = reinterpret_cast<PackWriteString>(
      GetProcAddress(rustLibrary, "replay_to_training_pack_write_last_error"));
  globalLastErrorLen = reinterpret_cast<GlobalErrorLen>(
      GetProcAddress(rustLibrary, "replay_to_training_last_error_len"));
  globalWriteLastError = reinterpret_cast<GlobalWriteError>(
      GetProcAddress(rustLibrary, "replay_to_training_write_last_error"));
  rustBuildInfoLen = reinterpret_cast<GlobalErrorLen>(
      GetProcAddress(rustLibrary, "replay_to_training_build_info_len"));
  rustWriteBuildInfo = reinterpret_cast<GlobalWriteError>(
      GetProcAddress(rustLibrary, "replay_to_training_write_build_info"));

  const bool complete = packCreate && packOpen && packDestroy && packSetName &&
      packSetCode && packSetCreatorName && packSetMapName && packSetDifficulty &&
      packDifficulty && packSetTrainingType && packTrainingType &&
      packCaptureModeSync &&
      packNameLen && packWriteName && packAddShot && momentumNote &&
      packRemoveShot && packShotCount && packShotSummaryLen &&
      packWriteShotSummary && packGuidHex && packSave && packSaveToTarget &&
      fileGuidHex && sanitizeTarget && targetsLen && writeTargets &&
      resolveTarget && defaultSaveDir && packLastErrorLen &&
      packWriteLastError && globalLastErrorLen && globalWriteLastError &&
      rustBuildInfoLen && rustWriteBuildInfo;
  if (!complete) {
    cvarManager->log(
        "replay-to-training: replay_to_training.dll is missing expected exports; "
        "the DLL likely does not match this plugin build");
    unloadRustLibrary();
    return false;
  }
  return true;
}

void ReplayToTrainingPlugin::unloadRustLibrary() {
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
  packSetTrainingType = nullptr;
  packTrainingType = nullptr;
  packCaptureModeSync = nullptr;
  packNameLen = nullptr;
  packWriteName = nullptr;
  packAddShot = nullptr;
  momentumNote = nullptr;
  packRemoveShot = nullptr;
  packShotCount = nullptr;
  packShotSummaryLen = nullptr;
  packWriteShotSummary = nullptr;
  packGuidHex = nullptr;
  packSave = nullptr;
  packSaveToTarget = nullptr;
  fileGuidHex = nullptr;
  sanitizeTarget = nullptr;
  targetsLen = nullptr;
  writeTargets = nullptr;
  resolveTarget = nullptr;
  defaultSaveDir = nullptr;
  packLastErrorLen = nullptr;
  packWriteLastError = nullptr;
  globalLastErrorLen = nullptr;
  globalWriteLastError = nullptr;
  rustBuildInfoLen = nullptr;
  rustWriteBuildInfo = nullptr;
  if (rustLibrary) {
    FreeLibrary(rustLibrary);
    rustLibrary = nullptr;
  }
  rustLoaded = false;
}

std::string ReplayToTrainingPlugin::packErrorMessage() {
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

std::string ReplayToTrainingPlugin::globalErrorMessage() {
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

std::string ReplayToTrainingPlugin::packGuidHexString() {
  if (!pack || !packGuidHex) {
    return {};
  }
  std::string hex(32, '\0');
  const size_t written =
      packGuidHex(pack, reinterpret_cast<uint8_t *>(hex.data()), hex.size());
  hex.resize(written);
  return hex;
}

std::string ReplayToTrainingPlugin::shotSummary(size_t index) {
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

std::string ReplayToTrainingPlugin::fileGuidHexString(const std::string &path) {
  if (!fileGuidHex) {
    return {};
  }
  std::string hex(32, '\0');
  const size_t written =
      fileGuidHex(path.c_str(), reinterpret_cast<uint8_t *>(hex.data()), hex.size());
  hex.resize(written);
  return hex;
}

std::string ReplayToTrainingPlugin::momentumNoteString(const TrCarState &car) {
  if (!momentumNote) {
    return {};
  }
  // Generously sized fixed buffer: the note is one short sentence, and a
  // 0 return means "no warning" (never a partial write worth retrying).
  std::string note(512, '\0');
  const size_t written = momentumNote(
      &car,
      momentumWarnMinSpeed(),
      momentumWarnMinLost(),
      momentumWarnMaxAngle(),
      reinterpret_cast<uint8_t *>(note.data()),
      note.size());
  note.resize(written);
  return note;
}

std::string ReplayToTrainingPlugin::rustCoreBuildInfo() {
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
