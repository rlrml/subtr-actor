#include "SubtrActorPlugin.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <format>

BAKKESMOD_PLUGIN(
    SubtrActorPlugin,
    "subtr-actor mechanic overlay",
    "0.1.0",
    PLUGINTYPE_FREEPLAY | PLUGINTYPE_CUSTOM_TRAINING | PLUGINTYPE_REPLAY)

namespace {

constexpr float PI = 3.14159265358979323846f;
constexpr float UNREAL_ROTATOR_TO_RADIANS = (2.0f * PI) / 65536.0f;
constexpr wchar_t RUST_DLL_NAME[] = L"subtr_actor_bakkesmod.dll";

int moduleAnchor = 0;

SaVec3 toSaVec3(Vector value) {
  return SaVec3{value.X, value.Y, value.Z};
}

SaQuat rotatorToQuat(Rotator rotation) {
  const float pitch = rotation.Pitch * UNREAL_ROTATOR_TO_RADIANS;
  const float yaw = rotation.Yaw * UNREAL_ROTATOR_TO_RADIANS;
  const float roll = rotation.Roll * UNREAL_ROTATOR_TO_RADIANS;

  const float cy = std::cos(yaw * 0.5f);
  const float sy = std::sin(yaw * 0.5f);
  const float cp = std::cos(pitch * 0.5f);
  const float sp = std::sin(pitch * 0.5f);
  const float cr = std::cos(roll * 0.5f);
  const float sr = std::sin(roll * 0.5f);

  return SaQuat{
      sr * cp * cy - cr * sp * sy,
      cr * sp * cy + sr * cp * sy,
      cr * cp * sy - sr * sp * cy,
      cr * cp * cy + sr * sp * sy,
  };
}

std::string mechanicLabel(SaMechanicKind kind) {
  switch (kind) {
  case SaMechanicKindSpeedFlip:
    return "Speed flip";
  case SaMechanicKindHalfFlip:
    return "Half flip";
  case SaMechanicKindWavedash:
    return "Wavedash";
  default:
    return "Mechanic";
  }
}

std::filesystem::path currentModuleDirectory() {
  HMODULE module = nullptr;
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

std::vector<std::filesystem::path> rustLibrarySearchPaths(GameWrapper *gameWrapper) {
  std::vector<std::filesystem::path> paths;
  const auto moduleDirectory = currentModuleDirectory();
  if (!moduleDirectory.empty()) {
    paths.push_back(moduleDirectory / RUST_DLL_NAME);
  }
  if (gameWrapper) {
    paths.push_back(gameWrapper->GetDataFolder() / "subtr-actor" / RUST_DLL_NAME);
  }
  paths.emplace_back(RUST_DLL_NAME);
  return paths;
}

} // namespace

void SubtrActorPlugin::onLoad() {
  loaded = loadRustLibrary();
  if (!loaded) {
    cvarManager->log("subtr-actor: failed to load subtr_actor_bakkesmod.dll");
    return;
  }

  gameWrapper->RegisterDrawable([this](CanvasWrapper canvas) { render(canvas); });
  gameWrapper->HookEvent(
      "Function TAGame.Car_TA.SetVehicleInput",
      [this](std::string eventName) { tick(eventName); });

  cvarManager->log("subtr-actor: mechanic overlay loaded");
}

void SubtrActorPlugin::onUnload() {
  gameWrapper->UnregisterDrawables();
  gameWrapper->UnhookEvent("Function TAGame.Car_TA.SetVehicleInput");
  unloadRustLibrary();
}

bool SubtrActorPlugin::loadRustLibrary() {
  for (const auto &dllPath : rustLibrarySearchPaths(gameWrapper.get())) {
    rustLibrary = LoadLibraryW(dllPath.c_str());
    if (rustLibrary) {
      cvarManager->log(std::format("subtr-actor: loaded Rust ABI from {}", dllPath.string()));
      break;
    }
  }
  if (!rustLibrary) {
    cvarManager->log(
        std::format("subtr-actor: LoadLibrary failed with error {}", GetLastError()));
    return false;
  }

  engineCreate = reinterpret_cast<EngineCreate>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_engine_create"));
  engineDestroy = reinterpret_cast<EngineDestroy>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_engine_destroy"));
  engineReset = reinterpret_cast<EngineReset>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_engine_reset"));
  processFrame = reinterpret_cast<ProcessFrame>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_process_frame"));
  drainEvents = reinterpret_cast<DrainEvents>(
      GetProcAddress(rustLibrary, "subtr_actor_bakkesmod_drain_events"));

  if (!engineCreate || !engineDestroy || !engineReset || !processFrame || !drainEvents) {
    unloadRustLibrary();
    return false;
  }

  engine = engineCreate();
  return engine != nullptr;
}

void SubtrActorPlugin::unloadRustLibrary() {
  if (engine && engineDestroy) {
    engineDestroy(engine);
  }
  engine = nullptr;

  if (rustLibrary) {
    FreeLibrary(rustLibrary);
  }
  rustLibrary = nullptr;
  engineCreate = nullptr;
  engineDestroy = nullptr;
  engineReset = nullptr;
  processFrame = nullptr;
  drainEvents = nullptr;
}

void SubtrActorPlugin::tick(std::string) {
  if (!loaded || !engine) {
    return;
  }

  if (!gameWrapper->IsInGame()) {
    if (wasInGame && engineReset) {
      engineReset(engine);
      frameNumber = 0;
      lastTime = 0.0f;
      lastBoostAmounts.clear();
      sampledPlayers.clear();
      messages.clear();
    }
    wasInGame = false;
    return;
  }
  if (!wasInGame && engineReset) {
    engineReset(engine);
    frameNumber = 0;
    lastTime = 0.0f;
    lastBoostAmounts.clear();
    sampledPlayers.clear();
    messages.clear();
  }
  wasInGame = true;

  SaLiveFrame frame = sampleFrame();
  if (processFrame(engine, &frame) != 0) {
    return;
  }

  SaMechanicEvent events[16];
  const size_t count = drainEvents(engine, events, 16);
  for (size_t i = 0; i < count; i += 1) {
    pushEventMessage(events[i]);
  }
}

SaLiveFrame SubtrActorPlugin::sampleFrame() {
  ServerWrapper server = gameWrapper->GetGameEventAsServer();
  CarWrapper car = gameWrapper->GetLocalCar();

  const float now = server.IsNull() ? lastTime : server.GetSecondsElapsed();
  const float dt = frameNumber == 0 ? 0.0f : std::max(0.0f, now - lastTime);
  lastTime = now;

  samplePlayers(server, car);

  SaLiveFrame frame{};
  frame.frame_number = frameNumber++;
  frame.time = now;
  frame.dt = dt;
  frame.live_play = 1;
  frame.ball_has_been_hit = 1;
  frame.has_ball_has_been_hit = 1;
  frame.players = sampledPlayers.empty() ? nullptr : sampledPlayers.data();
  frame.player_count = sampledPlayers.size();

  if (!server.IsNull()) {
    BallWrapper ball = server.GetBall();
    if (!ball.IsNull()) {
      frame.has_ball = 1;
      frame.ball = sampleRigidBody(ball);
    }
  }

  return frame;
}

void SubtrActorPlugin::samplePlayers(ServerWrapper server, CarWrapper localCar) {
  sampledPlayers.clear();

  if (!server.IsNull()) {
    ArrayWrapper<CarWrapper> cars = server.GetCars();
    if (!cars.IsNull()) {
      const int carCount = cars.Count();
      sampledPlayers.reserve(static_cast<size_t>(std::max(0, carCount)));
      for (int i = 0; i < carCount; i += 1) {
        CarWrapper car = cars.Get(i);
        if (!car.IsNull()) {
          sampledPlayers.push_back(samplePlayer(car, static_cast<uint32_t>(i)));
        }
      }
    }
  }

  if (sampledPlayers.empty() && !localCar.IsNull()) {
    sampledPlayers.push_back(samplePlayer(localCar, 0));
  }
}

SaRigidBody SubtrActorPlugin::sampleRigidBody(ActorWrapper actor) {
  SaRigidBody body{};
  if (actor.IsNull()) {
    body.sleeping = 1;
    return body;
  }

  body.location = toSaVec3(actor.GetLocation());
  body.rotation = rotatorToQuat(actor.GetRotation());
  body.linear_velocity = toSaVec3(actor.GetVelocity());
  body.angular_velocity = toSaVec3(actor.GetAngularVelocity());
  body.has_linear_velocity = 1;
  body.has_angular_velocity = 1;
  body.sleeping = 0;
  return body;
}

SaPlayerFrame SubtrActorPlugin::samplePlayer(CarWrapper car, uint32_t playerIndex) {
  SaPlayerFrame player{};
  player.player_index = playerIndex;
  player.is_team_0 = 1;
  if (car.IsNull()) {
    player.has_rigid_body = 0;
    return player;
  }

  PriWrapper pri = car.GetPRI();
  if (!pri.IsNull()) {
    player.is_team_0 = pri.GetTeamNum() == 0 ? 1 : 0;
  }

  player.has_rigid_body = 1;
  player.rigid_body = sampleRigidBody(car);
  player.dodge_active =
      car.GetDodgeComponent().IsNull() ? 0 : car.GetDodgeComponent().GetbActive();
  player.powerslide_active = car.GetbReplicatedHandbrake() != 0;

  BoostWrapper boost = car.GetBoostComponent();
  if (!boost.IsNull()) {
    const auto previousBoost = lastBoostAmounts.find(playerIndex);
    player.boost_amount = boost.GetCurrentBoostAmount();
    player.last_boost_amount =
        previousBoost == lastBoostAmounts.end() ? player.boost_amount : previousBoost->second;
    player.boost_active = player.boost_amount < player.last_boost_amount ? 1 : 0;
    lastBoostAmounts[playerIndex] = player.boost_amount;
  }

  return player;
}

void SubtrActorPlugin::pushEventMessage(const SaMechanicEvent &event) {
  const bool isBlue = event.is_team_0 != 0;
  OverlayMessage message{
      std::format("{} ({:.0f}%)", mechanicLabel(event.kind), event.confidence * 100.0f),
      isBlue ? LinearColor{80, 190, 255, 255} : LinearColor{255, 175, 80, 255},
      std::chrono::steady_clock::now() + std::chrono::seconds(2),
  };
  messages.push_back(message);
}

void SubtrActorPlugin::render(CanvasWrapper canvas) {
  const auto now = std::chrono::steady_clock::now();
  while (!messages.empty() && messages.front().expires_at <= now) {
    messages.pop_front();
  }

  Vector2 position{64, 280};
  for (const OverlayMessage &message : messages) {
    canvas.SetPosition(position);
    canvas.SetColor(message.color);
    canvas.DrawString(message.text, 1.4f, 1.4f, true);
    position.Y += 34;
  }
}
