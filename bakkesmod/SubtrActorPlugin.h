#pragma once

#include <chrono>
#include <deque>
#include <filesystem>
#include <string>
#include <vector>
#include <windows.h>

#pragma comment(lib, "pluginsdk.lib")

#include "bakkesmod/plugin/bakkesmodplugin.h"
#include "bakkesmod/wrappers/GameObject/BallWrapper.h"
#include "bakkesmod/wrappers/GameObject/CarWrapper.h"
#include "bakkesmod/wrappers/GameObject/CarComponent/BoostWrapper.h"
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

  HMODULE rustLibrary = nullptr;
  SaEngine *engine = nullptr;
  EngineCreate engineCreate = nullptr;
  EngineDestroy engineDestroy = nullptr;
  EngineReset engineReset = nullptr;
  ProcessFrame processFrame = nullptr;
  DrainEvents drainEvents = nullptr;

  uint64_t frameNumber = 0;
  float lastTime = 0.0f;
  float lastBoostAmount = 0.0f;
  bool loaded = false;
  std::deque<OverlayMessage> messages;

  bool loadRustLibrary();
  void unloadRustLibrary();
  void tick(std::string eventName);
  void render(CanvasWrapper canvas);
  void pushEventMessage(const SaMechanicEvent &event);
  SaLiveFrame sampleFrame();
  SaRigidBody sampleRigidBody(ActorWrapper actor);
  SaPlayerFrame sampleLocalPlayer(CarWrapper car);
};
