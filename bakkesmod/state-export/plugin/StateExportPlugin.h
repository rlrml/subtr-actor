#pragma once

#include <cstdint>
#include <filesystem>
#include <memory>
#include <optional>
#include <string>
#include <unordered_map>
#include <utility>
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
#include "bakkesmod/wrappers/GameObject/CarComponent/DodgeComponentWrapper.h"
#include "bakkesmod/wrappers/GameObject/GoalWrapper.h"
#include "bakkesmod/wrappers/GameObject/PriWrapper.h"
#include "bakkesmod/wrappers/GameObject/TeamWrapper.h"
#include "bakkesmod/wrappers/GameEvent/GameSettingPlaylistWrapper.h"
#include "bakkesmod/wrappers/GameEvent/ServerWrapper.h"
#include "bakkesmod/wrappers/UniqueIDWrapper.h"
#include "state_export.h"

class StateExportPlugin : public BakkesMod::Plugin::BakkesModPlugin,
                          public BakkesMod::Plugin::PluginSettingsWindow {
public:
  void onLoad() override;
  void onUnload() override;
  void RenderSettings() override;
  std::string GetPluginName() override;
  void SetImGuiContext(uintptr_t ctx) override;

private:
  // Function-pointer table for state_export.dll, mirroring
  // rust/include/state_export.h. The Rust cdylib is loaded at runtime via
  // LoadLibraryW/GetProcAddress (like the existing SubtrActorPlugin); the
  // plugin degrades gracefully when the DLL is missing.
  using EngineCreate = SeEngine *(*)(const SeConfig *);
  using EngineDestroy = void (*)(SeEngine *);
  using EngineRestart = int32_t (*)(SeEngine *, const SeConfig *);
  using PushFrame = int32_t (*)(SeEngine *, const SeFrame *);
  using SetMatchContext = int32_t (*)(SeEngine *, const SeMatchContext *);
  using NotifyMatchEnd = int32_t (*)(SeEngine *);
  using EngineStatus = int32_t (*)(const SeEngine *, SeStatus *);
  using LastErrorLen = size_t (*)(const SeEngine *);
  using WriteLastError = size_t (*)(const SeEngine *, uint8_t *, size_t);
  using BuildInfoLen = size_t (*)();
  using WriteBuildInfo = size_t (*)(uint8_t *, size_t);

  struct PlayerStatSnapshot {
    int shots = 0;
    int saves = 0;
    int assists = 0;
    int demolishes = 0;
  };

  struct TouchAttribution {
    uint32_t player_index = 0;
    uint8_t is_team_0 = 1;
  };

  HMODULE rustLibrary = nullptr;
  bool rustLoaded = false;
  SeEngine *engine = nullptr;

  EngineCreate engineCreate = nullptr;
  EngineDestroy engineDestroy = nullptr;
  EngineRestart engineRestart = nullptr;
  PushFrame pushFrame = nullptr;
  SetMatchContext setMatchContext = nullptr;
  NotifyMatchEnd notifyMatchEnd = nullptr;
  EngineStatus engineStatus = nullptr;
  LastErrorLen lastErrorLen = nullptr;
  WriteLastError writeLastError = nullptr;
  BuildInfoLen rustBuildInfoLen = nullptr;
  WriteBuildInfo rustWriteBuildInfo = nullptr;

  // Export tick bookkeeping. Everything below is only touched from the game
  // thread (SetTimeout callbacks, hooks, and notifiers all run there), so no
  // synchronization is needed; the Rust side owns all cross-thread state.
  std::shared_ptr<bool> exportTickCancelled = std::make_shared<bool>(false);
  bool wasInGame = false;
  uint64_t frameNumber = 0;
  float lastTime = 0.0f;
  std::optional<float> lastProcessedGameTime;
  // Refreshed from state_export_status every tick (atomics-only, so this is
  // effectively free); also read by the settings window.
  SeStatus lastStatus{};
  // Last engine error, cached on the game thread by refreshStatus so the
  // render-thread settings window never calls into the engine concurrently
  // with game-thread FFI writes.
  std::string lastErrorText;

  // Per-tick sampling scratch, cleared and reused so steady-state sampling
  // performs no per-tick heap allocation beyond map growth.
  std::vector<SePlayerFrame> sampledPlayers;
  std::vector<std::string> sampledPlayerNames;
  std::vector<std::string> sampledPlayerEpicIds;
  std::vector<SeTouchEvent> pendingTouches;
  std::vector<SeDodgeRefreshedEvent> pendingDodgeRefreshes;
  std::vector<SeBoostPadEvent> pendingBoostPadEvents;
  std::vector<SeGoalEvent> pendingGoals;
  std::vector<SePlayerStatEvent> pendingPlayerStatEvents;
  std::vector<SeDemolishEvent> pendingDemolishes;
  std::unordered_map<uint32_t, float> lastBoostAmounts;
  std::unordered_map<uintptr_t, uint32_t> carPlayerIndices;
  std::unordered_map<uintptr_t, uint32_t> priPlayerIndices;
  std::unordered_map<std::string, uint32_t> uniqueIdPlayerIndices;
  std::unordered_map<uintptr_t, uint32_t> stablePriPlayerIndices;
  std::unordered_map<uintptr_t, PlayerStatSnapshot> lastPlayerStats;
  std::unordered_map<uintptr_t, PlayerStatSnapshot> suppressedPlayerStatDeltas;
  std::unordered_map<uint32_t, bool> lastCanJump;
  std::unordered_map<uint32_t, uint64_t> lastBallTouchFrames;
  std::unordered_map<uint32_t, int32_t> dodgeRefreshCounters;
  std::unordered_map<uintptr_t, uint32_t> boostPadIds;
  std::unordered_map<uintptr_t, uint8_t> boostPadSequences;
  std::optional<std::pair<int, int>> lastTeamScores;
  std::optional<SeGoalEvent> lastGoalEvent;
  std::optional<TouchAttribution> lastTouch;
  uint32_t nextPlayerIndex = 0;
  uint32_t nextBoostPadId = 1;

  // Last match context pushed to the engine; refreshed while the match GUID
  // is still empty (it becomes available shortly after the map loads).
  std::string pushedMatchGuid;
  std::string pushedMapName;
  int32_t pushedPlaylistId = 0;
  bool pushedHasPlaylistId = false;
  bool matchContextPushed = false;

  uintptr_t imguiContext = 0;

  // rust_bridge.cpp
  std::vector<std::filesystem::path> rustLibrarySearchPaths();
  bool loadRustLibrary();
  void unloadRustLibrary();
  std::string engineErrorMessage();
  std::string rustCoreBuildInfo();

  // plugin_lifecycle.cpp
  std::string buildId() const;
  void logVersion();
  void registerCvarsAndNotifiers();
  SeConfig configFromCvars();
  void createEngine();
  void destroyEngine();
  void restartServer();
  void logStatus();
  bool exportEnabled();
  bool bindAllInterfacesEnabled();
  bool sampleWhenNoClientsEnabled();
  int configuredPort();
  float sampleIntervalSeconds();
  bool cvarBool(const char *name, bool defaultValue) const;
  int cvarInt(const char *name, int defaultValue) const;

  // tick.cpp
  void scheduleExportTick(float delaySeconds = 0.25f);
  void tick();
  void refreshStatus();
  void resetLiveState();

  // sampling.cpp
  SeFrame sampleFrame();
  void samplePlayers(ServerWrapper server, CarWrapper localCar);
  SeRigidBody sampleRigidBody(ActorWrapper actor);
  SePlayerFrame samplePlayer(CarWrapper car, uint32_t playerIndex);
  SePlayerFrame samplePlayer(PriWrapper pri, uint32_t playerIndex);
  void populatePlayerFromPri(SePlayerFrame &player, PriWrapper pri, uint32_t fallbackIndex);
  void sampleControllerInput(SePlayerFrame &player, CarWrapper car);
  void sampleCameraState(SePlayerFrame &player, PriWrapper pri);
  void sampleDodgeState(SePlayerFrame &player, CarWrapper car);
  void sampleRemoteId(SePlayerFrame &player, PriWrapper pri);
  void sampleTeamScores(ServerWrapper server, SeFrame &frame);
  void sampleTeamScores(ServerWrapper server, SeGoalEvent &goal);
  std::optional<bool> scoringTeamFromScoreDelta(const SeGoalEvent &goal) const;
  void rememberTeamScores(const SeFrame &frame);
  void rememberTeamScores(const SeGoalEvent &goal);
  bool goalEventIsDuplicate(const SeGoalEvent &goal) const;
  uint32_t stablePlayerIndexForPri(PriWrapper pri, uint32_t fallbackIndex);
  void clearPendingFrameEvents();
  void commitPendingFrameEvents();
  void attachPendingFrameEvents(SeFrame &frame);
  SeEventTiming currentEventTiming();

  // hooks.cpp
  void hookGameEvents();
  void unhookGameEvents();
  void recordTouch(CarWrapper car);
  void recordDodgeRefreshFromJumpState(CarWrapper car, uint32_t playerIndex, uint8_t isTeam0);
  void recordBoostPadEvent(ActorWrapper pickup, SeBoostPadEventKind kind);
  void recordGoal(ServerWrapper server, GoalWrapper goal, int scoreIndex, int assistIndex);
  void recordDemolish(CarWrapper victim, ActorWrapper demolisher);
  void recordPlayerStatDeltas(PriWrapper pri, uint32_t playerIndex, uint8_t isTeam0);
  void recordExplicitPlayerStat(PriWrapper pri, SePlayerStatEventKind kind);
  std::optional<uint32_t> playerIndexForCar(CarWrapper car);
  std::optional<uint32_t> playerIndexForPri(PriWrapper pri);
  PriWrapper priForScoreIndex(ServerWrapper server, int scoreIndex);
  std::optional<uint32_t> playerIndexForScoreIndex(ServerWrapper server, int scoreIndex);
  std::optional<uint32_t> playerIndexForNearestCar(ActorWrapper actor, float maxDistance);
  uint32_t boostPadId(ActorWrapper pickup);
  void pushMatchContext(ServerWrapper server);
  void maybeRefreshMatchContext();
  void notifyMatchEndAndReset(const char *reason);
};
