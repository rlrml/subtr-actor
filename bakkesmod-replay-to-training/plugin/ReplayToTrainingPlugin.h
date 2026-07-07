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
#include "bakkesmod/plugin/pluginwindow.h"
#include "bakkesmod/wrappers/arraywrapper.h"
#include "bakkesmod/wrappers/Engine/ActorWrapper.h"
#include "bakkesmod/wrappers/GameObject/BallWrapper.h"
#include "bakkesmod/wrappers/GameObject/CarWrapper.h"
#include "bakkesmod/wrappers/GameObject/CarComponent/BoostWrapper.h"
#include "bakkesmod/wrappers/GameEvent/ServerWrapper.h"
#include "bakkesmod/wrappers/ReplayServerWrapper.h"
#include "replay_to_training.h"

class ReplayToTrainingPlugin : public BakkesMod::Plugin::BakkesModPlugin,
                               public BakkesMod::Plugin::PluginSettingsWindow,
                               public BakkesMod::Plugin::PluginWindow {
public:
  void onLoad() override;
  void onUnload() override;
  void RenderSettings() override;
  std::string GetPluginName() override;
  // Shared by PluginSettingsWindow and PluginWindow (same signature, so one
  // override satisfies both bases).
  void SetImGuiContext(uintptr_t ctx) override;

  // BakkesMod::Plugin::PluginWindow — the standalone capture HUD, opened
  // in-game with `togglemenu replaytotraining` (window.cpp).
  void Render() override;
  std::string GetMenuName() override;
  std::string GetMenuTitle() override;
  bool ShouldBlockInput() override;
  bool IsActiveOverlay() override;
  void OnOpen() override;
  void OnClose() override;

private:
  // Which zero-arg capture command was used; selects the training-pack
  // semantic (mirroring convention + the type the first capture assigns).
  // Values match the ABI TrCapturedShot::mode encoding.
  enum class CaptureMode : uint8_t { Shot = 0, Save = 1 };
  // Function-pointer table for replay_to_training.dll, mirroring
  // rust/include/replay_to_training.h. The Rust cdylib is loaded at runtime via
  // LoadLibraryW/GetProcAddress (like the existing SubtrActorPlugin); the
  // plugin degrades gracefully when the DLL is missing.
  using PackCreate = TrPack *(*)();
  using PackOpen = TrPack *(*)(const char *);
  using PackDestroy = void (*)(TrPack *);
  using PackSetString = int32_t (*)(TrPack *, const char *);
  using PackSetDifficulty = int32_t (*)(TrPack *, uint32_t);
  using PackDifficulty = uint32_t (*)(const TrPack *);
  using PackSetTrainingType = int32_t (*)(TrPack *, uint32_t);
  using PackTrainingType = uint32_t (*)(const TrPack *);
  using PackCaptureModeSync = int32_t (*)(const TrPack *);
  using PackStringLen = size_t (*)(const TrPack *);
  using PackWriteString = size_t (*)(const TrPack *, uint8_t *, size_t);
  using PackAddShot = int32_t (*)(TrPack *, const TrCapturedShot *);
  using PackRemoveShot = int32_t (*)(TrPack *, size_t);
  using PackShotCount = size_t (*)(const TrPack *);
  using PackShotSummaryLen = size_t (*)(const TrPack *, size_t);
  using PackWriteShotSummary = size_t (*)(const TrPack *, size_t, uint8_t *, size_t);
  using PackSave = int32_t (*)(TrPack *, const char *);
  using PackSaveToTarget = int32_t (*)(TrPack *, const char *);
  using FileGuidHex = size_t (*)(const char *, uint8_t *, size_t);
  using SanitizeTarget = size_t (*)(const char *, uint8_t *, size_t);
  using TargetsLen = size_t (*)(const char *);
  using WriteTargets = size_t (*)(const char *, uint8_t *, size_t);
  using ResolveTarget = int32_t (*)(const char *, const char *, uint8_t *, size_t);
  using DefaultSaveDir = size_t (*)(const char *, uint8_t *, size_t);
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
  PackSetTrainingType packSetTrainingType = nullptr;
  PackTrainingType packTrainingType = nullptr;
  PackCaptureModeSync packCaptureModeSync = nullptr;
  PackStringLen packNameLen = nullptr;
  PackWriteString packWriteName = nullptr;
  PackAddShot packAddShot = nullptr;
  PackRemoveShot packRemoveShot = nullptr;
  PackShotCount packShotCount = nullptr;
  PackShotSummaryLen packShotSummaryLen = nullptr;
  PackWriteShotSummary packWriteShotSummary = nullptr;
  PackWriteString packGuidHex = nullptr;
  PackSave packSave = nullptr;
  PackSaveToTarget packSaveToTarget = nullptr;
  FileGuidHex fileGuidHex = nullptr;
  SanitizeTarget sanitizeTarget = nullptr;
  TargetsLen targetsLen = nullptr;
  WriteTargets writeTargets = nullptr;
  ResolveTarget resolveTarget = nullptr;
  DefaultSaveDir defaultSaveDir = nullptr;
  PackStringLen packLastErrorLen = nullptr;
  PackWriteString packWriteLastError = nullptr;
  GlobalErrorLen globalLastErrorLen = nullptr;
  GlobalWriteError globalWriteLastError = nullptr;
  GlobalErrorLen rustBuildInfoLen = nullptr;
  GlobalWriteError rustWriteBuildInfo = nullptr;

  uintptr_t imguiContext = 0;
  std::string statusLine = "replay-to-training: loading";
  std::array<char, 128> packNameBuffer{};
  std::array<char, 128> creatorNameBuffer{};
  std::array<char, 512> outputDirBuffer{};
  std::array<char, 512> openPathBuffer{};
  std::array<char, 256> targetBuffer{};
  int difficultyIndex = 1;
  // Open-state of the standalone PluginWindow (window.cpp); BakkesMod
  // drives it through OnOpen/OnClose via `togglemenu replaytotraining`.
  bool captureWindowOpen = false;

  // The active target's resolved on-disk path, when a target is set (empty
  // otherwise). Save writes here instead of the auto-GUID path; new_pack
  // clears it. Kept as the source of truth for "is a target active" so it
  // stays consistent with the in-memory pack even if the cvar text drifts.
  std::filesystem::path activeTargetPath;
  // Cached discovery results for the settings-window pick list, refreshed on
  // demand (a filesystem scan is too heavy to run every frame).
  std::vector<std::string> discoveredTargets;

  // rust_bridge.cpp
  std::vector<std::filesystem::path> rustLibrarySearchPaths();
  bool loadRustLibrary();
  void unloadRustLibrary();
  std::string packErrorMessage();
  std::string globalErrorMessage();
  std::string packGuidHexString();
  std::string shotSummary(size_t index);
  std::string rustCoreBuildInfo();
  std::string fileGuidHexString(const std::string &path);

  // capture.cpp
  std::string buildId() const;
  void logVersion();
  void registerCvarsAndNotifiers();
  void newPack();
  void openPackFromPath(const std::string &path);
  void applyMetadataToPack();
  void captureShot(CaptureMode mode);
  void savePack();
  // Runs the save flow (target-bound when a target is set, auto-GUID
  // otherwise) without touching the status line; the human-readable outcome
  // goes into `message`. Shared by the manual save button/notifier and the
  // capture autosave.
  bool savePackInternal(std::string &message);
  bool autosaveEnabled();
  bool mirrorByTeamEnabled();
  bool captureMomentumEnabled();
  // The persisted capture-mode selection (cvar
  // replay_to_training_capture_mode: "striker" | "goalie"), used by the
  // generic replay_to_training_capture notifier; the explicit
  // capture_shot/capture_save shortcuts set it before capturing, and an
  // activated pack with an assigned Striker/Goalie type auto-syncs it
  // (the pack's type is authoritative).
  CaptureMode selectedCaptureMode();
  void setSelectedCaptureMode(CaptureMode mode);
  void syncSelectionFromPackType();
  // Current pack training type in the ABI encoding (0 None, 1 Aerial,
  // 2 Goalie, 3 Striker, 4 unset, 5 other), and its display label.
  uint32_t packTrainingTypeIndex();
  static const char *trainingTypeLabel(uint32_t index);
  void overridePackTrainingType(uint32_t index);
  void removeShot(size_t index);
  float timeLimitSeconds();
  std::filesystem::path resolveOutputDirectory();
  // Where untargeted auto-GUID saves land: the Rust core redirects into the
  // sole account's MyTraining\ under the Training root when there is exactly
  // one account directory, else the root itself.
  std::filesystem::path defaultSaveDirectory();
  std::string cvarString(const char *name, const std::string &fallback);
  void setCvarString(const char *name, const std::string &value);
  void setStatus(std::string message);

  // settings_window.cpp — shared ImGui building blocks used by both the F2
  // settings page (RenderSettings) and the standalone capture window
  // (Render). All run on the render thread; anything touching game state
  // hops to the game thread via gameWrapper->Execute.
  void renderPackMetadataControls();
  void renderPackTypeControls();
  void renderCaptureModeSelector();
  void renderCaptureToggles();
  void renderTargetControls();
  void renderPackActions();
  void renderShotList();
  void renderStatusLine();

  // target.cpp — persistent default-save target. Path logic (sanitizing,
  // discovery, resolution) lives in the Rust core (rust/src/targets.rs)
  // behind the ABI so it is unit-testable; these are thin wrappers.
  std::filesystem::path resolveTrainingRoot();
  std::string sanitizeTargetName(std::string value);
  std::vector<std::string> discoverTargets(std::string &error);
  void setTarget(const std::string &requested);
  void clearTarget();
  void targetCommand(const std::vector<std::string> &args);
  void listTargetsCommand();
};
