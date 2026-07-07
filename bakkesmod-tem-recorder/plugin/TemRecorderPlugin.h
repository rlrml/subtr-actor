#pragma once

#include <array>
#include <cstdint>
#include <filesystem>
#include <string>
#include <vector>
#include <windows.h>

#pragma comment(lib, "pluginsdk.lib")

#include "bakkesmod/plugin/bakkesmodplugin.h"
#include "bakkesmod/plugin/PluginSettingsWindow.h"
#include "bakkesmod/wrappers/arraywrapper.h"
#include "bakkesmod/wrappers/Engine/ActorWrapper.h"
#include "bakkesmod/wrappers/GameObject/BallWrapper.h"
#include "bakkesmod/wrappers/GameObject/CarWrapper.h"
#include "bakkesmod/wrappers/GameObject/CarComponent/BoostWrapper.h"
#include "bakkesmod/wrappers/GameEvent/ServerWrapper.h"
#include "bakkesmod/wrappers/ReplayServerWrapper.h"
#include "tem_recorder.h"

class TemRecorderPlugin : public BakkesMod::Plugin::BakkesModPlugin,
                          public BakkesMod::Plugin::PluginSettingsWindow {
public:
  void onLoad() override;
  void onUnload() override;
  void RenderSettings() override;
  std::string GetPluginName() override;
  void SetImGuiContext(uintptr_t ctx) override;

private:
  // Function-pointer table for tem_recorder.dll, mirroring
  // rust/include/tem_recorder.h. The Rust cdylib is loaded at runtime via
  // LoadLibraryW/GetProcAddress (like the existing SubtrActorPlugin); the
  // plugin degrades gracefully when the DLL is missing.
  using PackCreate = TrPack *(*)();
  using PackOpen = TrPack *(*)(const char *);
  using PackDestroy = void (*)(TrPack *);
  using PackSetString = int32_t (*)(TrPack *, const char *);
  using PackSetDifficulty = int32_t (*)(TrPack *, uint32_t);
  using PackDifficulty = uint32_t (*)(const TrPack *);
  using PackStringLen = size_t (*)(const TrPack *);
  using PackWriteString = size_t (*)(const TrPack *, uint8_t *, size_t);
  using PackAddShot = int32_t (*)(TrPack *, const TrCapturedShot *);
  using PackRemoveShot = int32_t (*)(TrPack *, size_t);
  using PackShotCount = size_t (*)(const TrPack *);
  using PackShotSummaryLen = size_t (*)(const TrPack *, size_t);
  using PackWriteShotSummary = size_t (*)(const TrPack *, size_t, uint8_t *, size_t);
  using PackSave = int32_t (*)(TrPack *, const char *);
  using GlobalErrorLen = size_t (*)();
  using GlobalWriteError = size_t (*)(uint8_t *, size_t);

  HMODULE rustLibrary = nullptr;
  bool rustLoaded = false;
  TrPack *pack = nullptr;

  PackCreate packCreate = nullptr;
  PackOpen packOpen = nullptr;
  PackDestroy packDestroy = nullptr;
  PackSetString packSetName = nullptr;
  PackSetString packSetCode = nullptr;
  PackSetString packSetCreatorName = nullptr;
  PackSetString packSetMapName = nullptr;
  PackSetDifficulty packSetDifficulty = nullptr;
  PackDifficulty packDifficulty = nullptr;
  PackStringLen packNameLen = nullptr;
  PackWriteString packWriteName = nullptr;
  PackAddShot packAddShot = nullptr;
  PackRemoveShot packRemoveShot = nullptr;
  PackShotCount packShotCount = nullptr;
  PackShotSummaryLen packShotSummaryLen = nullptr;
  PackWriteShotSummary packWriteShotSummary = nullptr;
  PackWriteString packGuidHex = nullptr;
  PackSave packSave = nullptr;
  PackStringLen packLastErrorLen = nullptr;
  PackWriteString packWriteLastError = nullptr;
  GlobalErrorLen globalLastErrorLen = nullptr;
  GlobalWriteError globalWriteLastError = nullptr;

  uintptr_t imguiContext = 0;
  std::string statusLine = "tem-recorder: loading";
  std::array<char, 128> packNameBuffer{};
  std::array<char, 128> creatorNameBuffer{};
  std::array<char, 512> outputDirBuffer{};
  std::array<char, 512> openPathBuffer{};
  int difficultyIndex = 1;

  // rust_bridge.cpp
  std::vector<std::filesystem::path> rustLibrarySearchPaths();
  bool loadRustLibrary();
  void unloadRustLibrary();
  std::string packErrorMessage();
  std::string globalErrorMessage();
  std::string packGuidHexString();
  std::string shotSummary(size_t index);

  // capture.cpp
  void registerCvarsAndNotifiers();
  void newPack();
  void openPackFromPath(const std::string &path);
  void applyMetadataToPack();
  void captureShot();
  void savePack();
  void removeShot(size_t index);
  float timeLimitSeconds();
  std::filesystem::path resolveOutputDirectory();
  std::string cvarString(const char *name, const std::string &fallback);
  void setCvarString(const char *name, const std::string &value);
  void setStatus(std::string message);
};
