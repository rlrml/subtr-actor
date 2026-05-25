#pragma once

#include <chrono>
#include <deque>
#include <filesystem>
#include <optional>
#include <string>
#include <unordered_map>
#include <vector>
#include <windows.h>

#pragma comment(lib, "pluginsdk.lib")

#include "bakkesmod/plugin/bakkesmodplugin.h"
#include "bakkesmod/wrappers/arraywrapper.h"
#include "bakkesmod/wrappers/Engine/ActorWrapper.h"
#include "bakkesmod/wrappers/GameObject/BallWrapper.h"
#include "bakkesmod/wrappers/GameObject/CarWrapper.h"
#include "bakkesmod/wrappers/GameObject/CarComponent/BoostWrapper.h"
#include "bakkesmod/wrappers/GameObject/GoalWrapper.h"
#include "bakkesmod/wrappers/GameObject/PriWrapper.h"
#include "bakkesmod/wrappers/GameObject/TeamWrapper.h"
#include "bakkesmod/wrappers/GameEvent/ServerWrapper.h"
#include "bakkesmod/wrappers/canvaswrapper.h"
#include "subtr_actor_bakkesmod.h"

class SubtrActorPlugin : public BakkesMod::Plugin::BakkesModPlugin {
public:
  void onLoad() override;
  void onUnload() override;

private:
  using EngineCreate = SaEngine *(*)();
  using EngineDestroy = void (*)(SaEngine *);
  using EngineReset = void (*)(SaEngine *);
  using ProcessFrame = int32_t (*)(SaEngine *, const SaLiveFrame *);
  using DrainEvents = size_t (*)(SaEngine *, SaMechanicEvent *, size_t);

  struct OverlayMessage {
    std::string text;
    LinearColor color;
    std::chrono::steady_clock::time_point expires_at;
  };

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
  SaEngine *engine = nullptr;
  EngineCreate engineCreate = nullptr;
  EngineDestroy engineDestroy = nullptr;
  EngineReset engineReset = nullptr;
  ProcessFrame processFrame = nullptr;
  DrainEvents drainEvents = nullptr;

  uint64_t frameNumber = 0;
  float lastTime = 0.0f;
  bool loaded = false;
  bool wasInGame = false;
  std::vector<SaPlayerFrame> sampledPlayers;
  std::vector<SaTouchEvent> pendingTouches;
  std::vector<SaDodgeRefreshedEvent> pendingDodgeRefreshes;
  std::vector<SaBoostPadEvent> pendingBoostPadEvents;
  std::vector<SaGoalEvent> pendingGoals;
  std::vector<SaPlayerStatEvent> pendingPlayerStatEvents;
  std::vector<SaDemolishEvent> pendingDemolishes;
  std::unordered_map<uint32_t, float> lastBoostAmounts;
  std::unordered_map<uintptr_t, uint32_t> carPlayerIndices;
  std::unordered_map<uintptr_t, uint32_t> priPlayerIndices;
  std::unordered_map<uintptr_t, PlayerStatSnapshot> lastPlayerStats;
  std::unordered_map<uintptr_t, uint32_t> boostPadIds;
  std::unordered_map<uintptr_t, uint8_t> boostPadSequences;
  std::optional<TouchAttribution> lastTouch;
  uint32_t nextBoostPadId = 1;
  std::deque<OverlayMessage> messages;

  bool loadRustLibrary();
  void unloadRustLibrary();
  void tick(std::string eventName);
  void render(CanvasWrapper canvas);
  void pushEventMessage(const SaMechanicEvent &event);
  SaLiveFrame sampleFrame();
  void samplePlayers(ServerWrapper server, CarWrapper localCar);
  SaRigidBody sampleRigidBody(ActorWrapper actor);
  SaPlayerFrame samplePlayer(CarWrapper car, uint32_t playerIndex);
  void hookGameEvents();
  void unhookGameEvents();
  void resetLiveState();
  void clearPendingFrameEvents();
  void attachPendingFrameEvents(SaLiveFrame &frame);
  void recordTouch(CarWrapper car);
  void recordBoostPadEvent(ActorWrapper pickup, SaBoostPadEventKind kind);
  void recordGoal(ServerWrapper server, GoalWrapper goal);
  void recordDemolish(CarWrapper victim, ActorWrapper demolisher);
  void recordPlayerStatDeltas(PriWrapper pri, uint32_t playerIndex, uint8_t isTeam0);
  std::optional<uint32_t> playerIndexForCar(CarWrapper car);
  std::optional<uint32_t> playerIndexForPri(PriWrapper pri);
  std::optional<uint32_t> playerIndexForNearestCar(ActorWrapper actor, float maxDistance);
  uint32_t boostPadId(uintptr_t pickupAddress);
  void sampleTeamScores(ServerWrapper server, SaGoalEvent &goal);
};
