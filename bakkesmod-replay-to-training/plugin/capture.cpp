// Plugin lifecycle, cvars, notifiers, and the replay-frame capture logic.

namespace {

// BakkesMod `Vector` is float Unreal units in the game's own coordinate
// system, which is exactly what the archetype Location fields store: a
// direct 1:1 copy with no axis flip or rescale (confirmed against the typed
// archetype constructors in subtr-actor-training, which document the
// Location fields as floats in uu).
TrVec3 replayToTrainingVec3(const Vector &value) {
  return TrVec3{value.X, value.Y, value.Z};
}

// BakkesMod `Rotator` is integer Unreal rotator units (65536 = full turn),
// the same units the archetype RotationP/Y/R fields store: direct copy
// (confirmed against the typed archetype constructors in
// subtr-actor-training, which document RotationP/Y/R as integer UE rotator
// units).
TrRotator replayToTrainingRotator(const Rotator &value) {
  return TrRotator{value.Pitch, value.Yaw, value.Roll};
}

}  // namespace

std::string ReplayToTrainingPlugin::buildId() const {
  return std::format(
      "replay-to-training plugin {} build={} dirty={} commit_date={}",
      REPLAY_TO_TRAINING_PLUGIN_VERSION,
      REPLAY_TO_TRAINING_GIT_HASH,
      REPLAY_TO_TRAINING_GIT_DIRTY ? 1 : 0,
      REPLAY_TO_TRAINING_COMMIT_DATE);
}

// Logs both halves of the shipped DLL pair so a mismatched
// plugin/rust-core install is immediately visible.
void ReplayToTrainingPlugin::logVersion() {
  cvarManager->log(buildId());
  cvarManager->log(std::format("rust core: {}", rustCoreBuildInfo()));
}

void ReplayToTrainingPlugin::onLoad() {
  registerCvarsAndNotifiers();

  rustLoaded = loadRustLibrary();
  logVersion();
  if (!rustLoaded) {
    setStatus("replay_to_training.dll not found; capture disabled");
    return;
  }

  const std::string defaultName = cvarString("replay_to_training_pack_name", "Replay To Training Pack");
  const std::string defaultCreator = cvarString("replay_to_training_creator_name", "");
  const std::string outputDir = cvarString("replay_to_training_output_dir", "");
  const std::string persistedTarget =
      cvarString("replay_to_training_target_save_name", "");
  std::snprintf(packNameBuffer.data(), packNameBuffer.size(), "%s", defaultName.c_str());
  std::snprintf(
      creatorNameBuffer.data(), creatorNameBuffer.size(), "%s", defaultCreator.c_str());
  std::snprintf(outputDirBuffer.data(), outputDirBuffer.size(), "%s", outputDir.c_str());
  std::snprintf(targetBuffer.data(), targetBuffer.size(), "%s", persistedTarget.c_str());

  if (persistedTarget.empty()) {
    newPack();
  } else {
    // Restore the persisted target: this loads the bound .Tem into memory
    // (existing rounds included) so a session resumes appending to it.
    setTarget(persistedTarget);
  }
}

void ReplayToTrainingPlugin::onUnload() {
  unloadRustLibrary();
}

void ReplayToTrainingPlugin::registerCvarsAndNotifiers() {
  cvarManager->registerCvar(
      "replay_to_training_output_dir",
      "",
      // Also serves as the Training root scanned by
      // replay_to_training_list_targets and used to resolve targets into
      // MyTraining\ / Downloaded\.
      "Training root for saving/scanning .Tem files. Empty resolves to "
      "%USERPROFILE%\\Documents\\My Games\\Rocket League\\TAGame\\Training. "
      "Set a target to save into MyTraining\\ (what the game lists).",
      true);
  cvarManager->registerCvar(
      "replay_to_training_pack_name",
      "Replay To Training Pack",
      "Display name written into new training packs.",
      true);
  cvarManager->registerCvar(
      "replay_to_training_creator_name",
      "",
      "Creator name written into new training packs.",
      true);
  cvarManager->registerCvar(
      "replay_to_training_target_save_name",
      "",
      "Persistent default-save target, e.g. MyTraining\\<name> or "
      "Downloaded\\<name>. When set, captures append into that .Tem and "
      "save writes back to it (non-destructively) instead of a random GUID "
      "file. Empty = auto <GUID>.Tem in replay_to_training_output_dir.",
      true);
  cvarManager->registerCvar(
      "replay_to_training_autosave",
      "1",
      "Autosave after each captured shot: runs the normal save flow (to the "
      "target when one is set, else an auto <GUID>.Tem) so captures land on "
      "disk immediately. 0 keeps captures in memory until "
      "replay_to_training_save_pack.",
      true);
  cvarManager->registerCvar(
      "replay_to_training_time_limit",
      "8.0",
      "Per-shot time limit in seconds for captured rounds.",
      true,
      true,
      1,
      true,
      120);
  cvarManager->registerCvar(
      "replay_to_training_mirror_by_team",
      "1",
      "Auto-mirror captures 180 degrees about field center when the "
      "captured player's team does not match the training convention "
      "(striker scenarios attack +Y, goalie scenarios defend -Y; the "
      "training player is blue-oriented, so orange captures are flipped). "
      "0 captures the raw replay orientation.",
      true);
  cvarManager->registerCvar(
      "replay_to_training_capture_momentum",
      "1",
      "Write the captured car's forward speed (velocity projected onto its "
      "facing) into the spawn point's VelocityStartSpeed so the training "
      "car starts moving like it was in the replay. 0 spawns the car "
      "stationary.",
      true);

  cvarManager->registerNotifier(
      "replay_to_training_capture_shot",
      [this](std::vector<std::string>) { captureShot(CaptureMode::Shot); },
      "Captures the current replay frame as an OFFENSIVE (striker) shot in "
      "the in-memory training pack. The first capture into a fresh pack "
      "sets the pack type to Striker.",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "replay_to_training_capture_save",
      [this](std::vector<std::string>) { captureShot(CaptureMode::Save); },
      "Captures the current replay frame as a DEFENSIVE (goalie) save in "
      "the in-memory training pack. The first capture into a fresh pack "
      "sets the pack type to Goalie.",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "replay_to_training_window",
      [this](std::vector<std::string>) {
        cvarManager->executeCommand("togglemenu " + GetMenuName());
      },
      "Toggles the replay-to-training capture window (same as `togglemenu "
      "replaytotraining`).",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "replay_to_training_save_pack",
      [this](std::vector<std::string>) { savePack(); },
      "Saves the in-memory training pack as an encrypted .Tem file in "
      "replay_to_training_output_dir.",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "replay_to_training_new_pack",
      [this](std::vector<std::string>) { newPack(); },
      "Discards the in-memory training pack and starts a fresh one.",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "replay_to_training_open_pack",
      [this](std::vector<std::string> params) {
        if (params.size() < 2) {
          setStatus("usage: replay_to_training_open_pack <path-to-.Tem>");
          return;
        }
        openPackFromPath(params[1]);
      },
      "Opens an existing .Tem file so captured shots append to it. "
      "Usage: replay_to_training_open_pack <path>",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "replay_to_training_version",
      [this](std::vector<std::string>) { logVersion(); },
      "Log the loaded plugin and Rust core build identifiers.",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "replay_to_training_target",
      [this](std::vector<std::string> args) { targetCommand(args); },
      "Set or show the persistent default-save target. Setting one loads "
      "that .Tem into memory so captures append to it. "
      "Usage: replay_to_training_target [MyTraining\\<name>]",
      PERMISSION_ALL);
  cvarManager->registerNotifier(
      "replay_to_training_list_targets",
      [this](std::vector<std::string>) { listTargetsCommand(); },
      "List local custom-training .Tem targets under MyTraining\\ and "
      "Downloaded\\.",
      PERMISSION_ALL);
}

void ReplayToTrainingPlugin::newPack() {
  if (!rustLoaded || !packCreate) {
    setStatus("rust library not loaded");
    return;
  }
  if (pack && packDestroy) {
    packDestroy(pack);
  }
  pack = packCreate();
  // Guardrail (a): a fresh pack is never bound to a target, so it falls back
  // to the auto-GUID save flow and cannot overwrite a target file. setTarget
  // re-binds after calling this.
  clearTarget();
  applyMetadataToPack();
  // The fresh pack's training type is unset until the first capture
  // (capture_shot -> Striker, capture_save -> Goalie) or a manual override.
  setStatus(std::format("new pack {} (type unset; first capture decides)",
                        packGuidHexString()));
}

void ReplayToTrainingPlugin::openPackFromPath(const std::string &path) {
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
      setCvarString("replay_to_training_pack_name", name);
    }
  }
  if (packDifficulty) {
    difficultyIndex = static_cast<int>(packDifficulty(pack));
  }
  setStatus(std::format("opened {} ({} shots)", path, count));
}

void ReplayToTrainingPlugin::applyMetadataToPack() {
  if (!pack) {
    return;
  }
  const std::string name = cvarString("replay_to_training_pack_name", "Replay To Training Pack");
  const std::string creator = cvarString("replay_to_training_creator_name", "");
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

float ReplayToTrainingPlugin::timeLimitSeconds() {
  auto cvar = cvarManager->getCvar("replay_to_training_time_limit");
  const float value = static_cast<bool>(cvar) ? cvar.getFloatValue() : 8.0f;
  return value > 0.0f ? value : 8.0f;
}

void ReplayToTrainingPlugin::captureShot(CaptureMode mode) {
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
  ballState.location = replayToTrainingVec3(ball.GetLocation());
  // Ball velocity crosses the ABI as a vector; the Rust side converts it to
  // the direction-rotator + speed encoding the archetype stores.
  ballState.linear_velocity = replayToTrainingVec3(ball.GetVelocity());
  // Not representable in .tem archetypes; carried across the ABI anyway.
  ballState.angular_velocity = replayToTrainingVec3(ball.GetAngularVelocity());

  // The replay camera's current view target marks the primary (IsPC) car.
  // TODO(in-game): pick the spectated player explicitly; the view target
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
    state.location = replayToTrainingVec3(car.GetLocation());
    // Rotator ints pass straight through to the archetype RotationP/Y/R.
    state.rotation = replayToTrainingRotator(car.GetRotation());
    // Not representable in .tem archetypes; carried across the ABI anyway.
    state.linear_velocity = replayToTrainingVec3(car.GetVelocity());
    state.angular_velocity = replayToTrainingVec3(car.GetAngularVelocity());
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
  // Per-capture options: the mode comes from which zero-arg command was
  // used; everything else is a persisted cvar default (no command
  // parameters, per the plugin's UX convention).
  shot.mode = static_cast<uint8_t>(mode);
  shot.mirror_by_team = mirrorByTeamEnabled() ? 1 : 0;
  shot.capture_momentum = captureMomentumEnabled() ? 1 : 0;
  const int32_t added = packAddShot(pack, &shot);
  if (added == 1) {
    setStatus(std::format("capture failed: {}", packErrorMessage()));
    return;
  }
  const char *modeLabel = mode == CaptureMode::Save ? "save" : "shot";
  // Return code 2: the capture landed, but its mode conflicts with the
  // pack's already-assigned training type. The .tem format has no
  // per-round type, so warn rather than refuse.
  const std::string mismatchWarning =
      added == 2 ? std::format(
                       " WARNING: {} capture in a {} pack (type is "
                       "pack-level; round kept)",
                       modeLabel,
                       trainingTypeLabel(packTrainingTypeIndex()))
                 : "";
  const size_t count = packShotCount ? packShotCount(pack) : 0;
  if (autosaveEnabled()) {
    // Run the normal save flow (target-bound with its clobber guardrails
    // when a target is set, else auto-GUID) so the capture is on disk
    // immediately.
    std::string saveMessage;
    if (savePackInternal(saveMessage)) {
      // saveMessage reads "saved ..." -> "captured shot N; autosaved ...".
      setStatus(std::format("captured {} {}; auto{}{}", modeLabel, count,
                            saveMessage, mismatchWarning));
    } else {
      setStatus(std::format("captured {} {}; autosave failed: {}{}",
                            modeLabel, count, saveMessage, mismatchWarning));
    }
    return;
  }
  setStatus(std::format(
      "captured {} {} at replay time {:.1f}s (unsaved; "
      "replay_to_training_save_pack writes it){}",
      modeLabel,
      count,
      server.GetReplayTimeElapsed(),
      mismatchWarning));
}

bool ReplayToTrainingPlugin::autosaveEnabled() {
  auto cvar = cvarManager->getCvar("replay_to_training_autosave");
  return static_cast<bool>(cvar) ? cvar.getBoolValue() : true;
}

bool ReplayToTrainingPlugin::mirrorByTeamEnabled() {
  auto cvar = cvarManager->getCvar("replay_to_training_mirror_by_team");
  return static_cast<bool>(cvar) ? cvar.getBoolValue() : true;
}

bool ReplayToTrainingPlugin::captureMomentumEnabled() {
  auto cvar = cvarManager->getCvar("replay_to_training_capture_momentum");
  return static_cast<bool>(cvar) ? cvar.getBoolValue() : true;
}

// ABI training-type encoding (replay_to_training.h): 0 None, 1 Aerial,
// 2 Goalie, 3 Striker, 4 unset (first capture decides), 5 unmodeled.
uint32_t ReplayToTrainingPlugin::packTrainingTypeIndex() {
  if (!pack || !packTrainingType) {
    return 4;
  }
  return packTrainingType(pack);
}

const char *ReplayToTrainingPlugin::trainingTypeLabel(uint32_t index) {
  switch (index) {
    case 0: return "None";
    case 1: return "Aerial";
    case 2: return "Goalie";
    case 3: return "Striker";
    case 4: return "unset (first capture decides)";
    default: return "other";
  }
}

// Manual override (window dropdown), incl. Aerial/None for publishing
// metadata; marks the type assigned so later mismatched-mode captures warn.
void ReplayToTrainingPlugin::overridePackTrainingType(uint32_t index) {
  if (!rustLoaded || !pack || !packSetTrainingType) {
    setStatus("rust library not loaded");
    return;
  }
  if (packSetTrainingType(pack, index) != 0) {
    setStatus(std::format("set pack type failed: {}", packErrorMessage()));
    return;
  }
  setStatus(std::format("pack type set to {}", trainingTypeLabel(index)));
}

std::filesystem::path ReplayToTrainingPlugin::resolveOutputDirectory() {
  const std::string configured = cvarString("replay_to_training_output_dir", "");
  if (!configured.empty()) {
    return std::filesystem::path(configured);
  }
  // The game lists locally created packs from <account>\MyTraining\ under
  // this Training root (the per-account directory is inserted by the game;
  // a root-level MyTraining\ also works on some setups). Target resolution
  // and the auto-GUID default save dir handle the account directory in the
  // Rust core; this only picks the root.
  char *profile = nullptr;
  size_t profileLength = 0;
  if (_dupenv_s(&profile, &profileLength, "USERPROFILE") == 0 && profile) {
    std::filesystem::path base(profile);
    std::free(profile);
    return base / "Documents" / "My Games" / "Rocket League" / "TAGame" / "Training";
  }
  return gameWrapper->GetDataFolder() / "replay-to-training";
}

// The directory untargeted auto-GUID saves land in. The Rust core redirects
// into `<root>\<account>\MyTraining\` when the Training root contains
// exactly one account directory (so the pack shows up under Training >
// Custom Training in-game); otherwise the root itself, matching the old
// behavior.
std::filesystem::path ReplayToTrainingPlugin::defaultSaveDirectory() {
  const std::filesystem::path root = resolveTrainingRoot();
  if (!defaultSaveDir) {
    return root;
  }
  const std::string rootString = root.string();
  std::string directory(4096, '\0');
  const size_t written = defaultSaveDir(
      rootString.c_str(), reinterpret_cast<uint8_t *>(directory.data()),
      directory.size());
  if (written == 0) {
    return root;
  }
  directory.resize(written);
  return std::filesystem::path(directory);
}

void ReplayToTrainingPlugin::savePack() {
  std::string message;
  savePackInternal(message);
  setStatus(std::move(message));
}

bool ReplayToTrainingPlugin::savePackInternal(std::string &message) {
  if (!rustLoaded || !pack || !packSave) {
    message = "rust library not loaded";
    return false;
  }
  applyMetadataToPack();

  // Target flow: write the in-memory pack back to the bound target path.
  // Because memory was seeded from that file when the target was set, this
  // is inherently append/non-destructive; the Rust side additionally refuses
  // to clobber a different pack that ended up at the path (guardrail (b)).
  if (!activeTargetPath.empty() && packSaveToTarget) {
    const int outcome =
        packSaveToTarget(pack, activeTargetPath.string().c_str());
    switch (outcome) {
      case 0:
        message = std::format("saved (created) target {}",
                              activeTargetPath.string());
        return true;
      case 1:
        message = std::format("saved (appended) target {}",
                              activeTargetPath.string());
        return true;
      case 2:
        // Refused: last-error explains ("target already contains a different
        // pack; ..."). Nothing was written.
        message = std::format("save refused: {}", packErrorMessage());
        return false;
      default:
        message = std::format("save failed: {}", packErrorMessage());
        return false;
    }
  }

  // No target: auto <GUID>.Tem in the default save directory (the sole
  // account's MyTraining\ when one exists, else the Training root).
  const std::string guidHex = packGuidHexString();
  if (guidHex.empty()) {
    message = "could not derive pack GUID";
    return false;
  }
  const std::filesystem::path outputPath =
      defaultSaveDirectory() / (guidHex + ".Tem");
  if (packSave(pack, outputPath.string().c_str()) != 0) {
    message = std::format("save failed: {}", packErrorMessage());
    return false;
  }
  message = std::format("saved {}", outputPath.string());
  return true;
}

void ReplayToTrainingPlugin::removeShot(size_t index) {
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

std::string ReplayToTrainingPlugin::cvarString(const char *name, const std::string &fallback) {
  auto cvar = cvarManager->getCvar(name);
  return static_cast<bool>(cvar) ? cvar.getStringValue() : fallback;
}

void ReplayToTrainingPlugin::setCvarString(const char *name, const std::string &value) {
  auto cvar = cvarManager->getCvar(name);
  if (static_cast<bool>(cvar)) {
    cvar.setValue(value);
  }
}

void ReplayToTrainingPlugin::setStatus(std::string message) {
  statusLine = std::move(message);
  cvarManager->log(std::format("replay-to-training: {}", statusLine));
}
