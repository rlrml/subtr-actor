// Plugin lifecycle, cvars, notifiers, and the replay-frame capture logic.

namespace {

// BakkesMod `Vector` is float Unreal units in the game's own coordinate
// system, which is exactly what the archetype Location fields store: a
// direct 1:1 copy with no axis flip or rescale.
// TODO(phase-3): confirm against typed archetype constructors.
TrVec3 temRecorderVec3(const Vector &value) {
  return TrVec3{value.X, value.Y, value.Z};
}

// BakkesMod `Rotator` is integer Unreal rotator units (65536 = full turn),
// the same units the archetype RotationP/Y/R fields store: direct copy.
// TODO(phase-3): confirm against typed archetype constructors.
TrRotator temRecorderRotator(const Rotator &value) {
  return TrRotator{value.Pitch, value.Yaw, value.Roll};
}

}  // namespace

void TemRecorderPlugin::onLoad() {
  registerCvarsAndNotifiers();

  rustLoaded = loadRustLibrary();
  if (!rustLoaded) {
    setStatus("tem_recorder.dll not found; capture disabled");
    return;
  }

  const std::string defaultName = cvarString("tem_recorder_pack_name", "TEM Recorder Pack");
  const std::string defaultCreator = cvarString("tem_recorder_creator_name", "");
  const std::string outputDir = cvarString("tem_recorder_output_dir", "");
  std::snprintf(packNameBuffer.data(), packNameBuffer.size(), "%s", defaultName.c_str());
  std::snprintf(
      creatorNameBuffer.data(), creatorNameBuffer.size(), "%s", defaultCreator.c_str());
  std::snprintf(outputDirBuffer.data(), outputDirBuffer.size(), "%s", outputDir.c_str());

  newPack();
}

void TemRecorderPlugin::onUnload() {
  unloadRustLibrary();
}

void TemRecorderPlugin::registerCvarsAndNotifiers() {
  cvarManager->registerCvar(
      "tem_recorder_output_dir",
      "",
      // TODO: the real game directory contains a per-account subfolder
      // (<online-id>\\MyTraining); users may need to point this cvar there
      // for the game to list saved packs.
      "Directory to save .Tem files into. Empty resolves to "
      "%USERPROFILE%\\Documents\\My Games\\Rocket League\\TAGame\\Training.",
      true);
  cvarManager->registerCvar(
      "tem_recorder_pack_name",
      "TEM Recorder Pack",
      "Display name written into new training packs.",
      true);
  cvarManager->registerCvar(
      "tem_recorder_creator_name",
      "",
      "Creator name written into new training packs.",
      true);
  cvarManager->registerCvar(
      "tem_recorder_time_limit",
      "8.0",
      "Per-shot time limit in seconds for captured rounds.",
      true,
      true,
      1,
      true,
      120);

  cvarManager->registerNotifier(
      "tem_recorder_capture_shot",
      [this](std::vector<std::string>) { captureShot(); },
      "Captures the current replay frame's ball and car states as a new "
      "shot in the in-memory training pack.",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "tem_recorder_save_pack",
      [this](std::vector<std::string>) { savePack(); },
      "Saves the in-memory training pack as an encrypted .Tem file in "
      "tem_recorder_output_dir.",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "tem_recorder_new_pack",
      [this](std::vector<std::string>) { newPack(); },
      "Discards the in-memory training pack and starts a fresh one.",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "tem_recorder_open_pack",
      [this](std::vector<std::string> params) {
        if (params.size() < 2) {
          setStatus("usage: tem_recorder_open_pack <path-to-.Tem>");
          return;
        }
        openPackFromPath(params[1]);
      },
      "Opens an existing .Tem file so captured shots append to it. "
      "Usage: tem_recorder_open_pack <path>",
      PERMISSION_ALL);
}

void TemRecorderPlugin::newPack() {
  if (!rustLoaded || !packCreate) {
    setStatus("rust library not loaded");
    return;
  }
  if (pack && packDestroy) {
    packDestroy(pack);
  }
  pack = packCreate();
  applyMetadataToPack();
  setStatus(std::format("new pack {}", packGuidHexString()));
}

void TemRecorderPlugin::openPackFromPath(const std::string &path) {
  if (!rustLoaded || !packOpen) {
    setStatus("rust library not loaded");
    return;
  }
  TrPack *opened = packOpen(path.c_str());
  if (!opened) {
    setStatus(std::format("open failed: {}", globalErrorMessage()));
    return;
  }
  if (pack && packDestroy) {
    packDestroy(pack);
  }
  pack = opened;
  const size_t count = packShotCount ? packShotCount(pack) : 0;
  if (packNameLen && packWriteName) {
    const size_t nameLength = packNameLen(pack);
    std::string name(nameLength, '\0');
    const size_t written =
        packWriteName(pack, reinterpret_cast<uint8_t *>(name.data()), name.size());
    name.resize(written);
    if (!name.empty()) {
      std::snprintf(packNameBuffer.data(), packNameBuffer.size(), "%s", name.c_str());
      setCvarString("tem_recorder_pack_name", name);
    }
  }
  if (packDifficulty) {
    difficultyIndex = static_cast<int>(packDifficulty(pack));
  }
  setStatus(std::format("opened {} ({} shots)", path, count));
}

void TemRecorderPlugin::applyMetadataToPack() {
  if (!pack) {
    return;
  }
  const std::string name = cvarString("tem_recorder_pack_name", "TEM Recorder Pack");
  const std::string creator = cvarString("tem_recorder_creator_name", "");
  if (packSetName && packSetName(pack, name.c_str()) != 0) {
    setStatus(std::format("set name failed: {}", packErrorMessage()));
  }
  if (packSetCreatorName) {
    if (packSetCreatorName(pack, creator.empty() ? nullptr : creator.c_str()) != 0) {
      setStatus(std::format("set creator failed: {}", packErrorMessage()));
    }
  }
  if (packSetDifficulty &&
      packSetDifficulty(pack, static_cast<uint32_t>(difficultyIndex)) != 0) {
    setStatus(std::format("set difficulty failed: {}", packErrorMessage()));
  }
}

float TemRecorderPlugin::timeLimitSeconds() {
  auto cvar = cvarManager->getCvar("tem_recorder_time_limit");
  const float value = static_cast<bool>(cvar) ? cvar.getFloatValue() : 8.0f;
  return value > 0.0f ? value : 8.0f;
}

void TemRecorderPlugin::captureShot() {
  if (!rustLoaded || !pack || !packAddShot) {
    setStatus("rust library not loaded");
    return;
  }
  if (!gameWrapper->IsInReplay()) {
    setStatus("capture requires an in-game replay");
    return;
  }
  ReplayServerWrapper server = gameWrapper->GetGameEventAsReplay();
  if (server.IsNull()) {
    setStatus("no replay server");
    return;
  }
  BallWrapper ball = server.GetBall();
  if (ball.IsNull()) {
    setStatus("no ball in the current frame");
    return;
  }

  TrBallState ballState{};
  ballState.location = temRecorderVec3(ball.GetLocation());
  // Ball velocity crosses the ABI as a vector; the Rust side converts it to
  // the direction-rotator + speed encoding the archetype stores.
  ballState.linear_velocity = temRecorderVec3(ball.GetVelocity());
  // Not representable in current .tem archetypes; carried for phase-3.
  ballState.angular_velocity = temRecorderVec3(ball.GetAngularVelocity());

  // The replay camera's current view target marks the primary (IsPC) car.
  // TODO(phase-3): pick the spectated player explicitly; the view target
  // can be the ball or a free camera, in which case the first car wins.
  ActorWrapper viewTarget = server.GetViewTarget();
  const std::uintptr_t viewTargetAddress =
      viewTarget.IsNull() ? 0 : viewTarget.memory_address;

  std::vector<TrCarState> cars;
  ArrayWrapper<CarWrapper> carArray = server.GetCars();
  for (int index = 0; index < carArray.Count(); ++index) {
    CarWrapper car = carArray.Get(index);
    if (car.IsNull()) {
      continue;
    }
    TrCarState state{};
    state.location = temRecorderVec3(car.GetLocation());
    // Rotator ints pass straight through to the archetype RotationP/Y/R.
    state.rotation = temRecorderRotator(car.GetRotation());
    // Not representable in current .tem archetypes; carried for phase-3.
    state.linear_velocity = temRecorderVec3(car.GetVelocity());
    state.angular_velocity = temRecorderVec3(car.GetAngularVelocity());
    // Boost is a 0..1 float from BakkesMod; not representable in current
    // .tem archetypes, carried through the ABI for phase-3.
    BoostWrapper boost = car.GetBoostComponent();
    if (!boost.IsNull()) {
      state.boost_amount = boost.GetCurrentBoostAmount();
    }
    state.team = car.GetTeamNum2();
    state.is_primary =
        (viewTargetAddress != 0 && car.memory_address == viewTargetAddress) ? 1 : 0;
    cars.push_back(state);
  }
  if (!cars.empty()) {
    const bool anyPrimary = std::any_of(
        cars.begin(), cars.end(), [](const TrCarState &car) { return car.is_primary != 0; });
    if (!anyPrimary) {
      cars.front().is_primary = 1;
    }
  }

  applyMetadataToPack();

  TrCapturedShot shot{};
  shot.ball = ballState;
  shot.time_limit = timeLimitSeconds();
  shot.cars = cars.empty() ? nullptr : cars.data();
  shot.car_count = cars.size();
  if (packAddShot(pack, &shot) != 0) {
    setStatus(std::format("capture failed: {}", packErrorMessage()));
    return;
  }
  const size_t count = packShotCount ? packShotCount(pack) : 0;
  setStatus(std::format("captured shot {} at replay time {:.1f}s",
                        count,
                        server.GetReplayTimeElapsed()));
}

std::filesystem::path TemRecorderPlugin::resolveOutputDirectory() {
  const std::string configured = cvarString("tem_recorder_output_dir", "");
  if (!configured.empty()) {
    return std::filesystem::path(configured);
  }
  // TODO: the game actually lists packs from a per-account subfolder,
  // %USERPROFILE%\Documents\My Games\Rocket League\TAGame\Training\
  // <online-id>\MyTraining. Without knowing the account id we default to
  // the Training root; point tem_recorder_output_dir at the MyTraining
  // folder to have packs show up in-game.
  char *profile = nullptr;
  size_t profileLength = 0;
  if (_dupenv_s(&profile, &profileLength, "USERPROFILE") == 0 && profile) {
    std::filesystem::path base(profile);
    std::free(profile);
    return base / "Documents" / "My Games" / "Rocket League" / "TAGame" / "Training";
  }
  return gameWrapper->GetDataFolder() / "tem-recorder";
}

void TemRecorderPlugin::savePack() {
  if (!rustLoaded || !pack || !packSave) {
    setStatus("rust library not loaded");
    return;
  }
  applyMetadataToPack();
  const std::string guidHex = packGuidHexString();
  if (guidHex.empty()) {
    setStatus("could not derive pack GUID");
    return;
  }
  const std::filesystem::path outputPath =
      resolveOutputDirectory() / (guidHex + ".Tem");
  if (packSave(pack, outputPath.string().c_str()) != 0) {
    setStatus(std::format("save failed: {}", packErrorMessage()));
    return;
  }
  setStatus(std::format("saved {}", outputPath.string()));
}

void TemRecorderPlugin::removeShot(size_t index) {
  if (!rustLoaded || !pack || !packRemoveShot) {
    setStatus("rust library not loaded");
    return;
  }
  if (packRemoveShot(pack, index) != 0) {
    setStatus(std::format("remove failed: {}", packErrorMessage()));
    return;
  }
  setStatus(std::format("removed shot {}", index + 1));
}

std::string TemRecorderPlugin::cvarString(const char *name, const std::string &fallback) {
  auto cvar = cvarManager->getCvar(name);
  return static_cast<bool>(cvar) ? cvar.getStringValue() : fallback;
}

void TemRecorderPlugin::setCvarString(const char *name, const std::string &value) {
  auto cvar = cvarManager->getCvar(name);
  if (static_cast<bool>(cvar)) {
    cvar.setValue(value);
  }
}

void TemRecorderPlugin::setStatus(std::string message) {
  statusLine = std::move(message);
  cvarManager->log(std::format("tem-recorder: {}", statusLine));
}
