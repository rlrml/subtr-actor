// Persistent default-save "target": binds the in-memory pack to a specific
// custom-training .Tem under the game's Training root so captures accumulate
// into it non-destructively.
//
// The save flow is inherently append-only because memory is the single
// source of truth: setting a target OPENS the existing file into memory
// (existing rounds included), capture appends to memory, and save writes
// memory back to the target path. See recorder.rs `save_to_target`.
//
// All path logic (sanitizing, discovery, resolution, default save dir) lives
// in the Rust core (rust/src/targets.rs) behind the ABI, where it is
// unit-tested; this file only marshals strings across. The Rust side is
// account-directory aware: the game keeps the listing folders under
// `<root>\<account>\MyTraining` etc. (e.g. `Training\0000000000000000\
// MyTraining\*.Tem`), with a root-level `MyTraining` also scanned for
// robustness.

#include <system_error>

namespace {

// Reads a `size_t (const char*, uint8_t*, size_t)`-shaped ABI string
// function into a std::string, given the exact byte length.
template <typename WriteFn>
std::string readAbiString(size_t length, WriteFn &&write) {
  if (length == 0) {
    return {};
  }
  std::string text(length, '\0');
  const size_t written =
      write(reinterpret_cast<uint8_t *>(text.data()), text.size());
  text.resize(written);
  return text;
}

}  // namespace

// Normalizes a user-entered target via the Rust core: trims, '/' -> '\\',
// drops a trailing .tem/.Tem, canonicalizes a known folder's case, and
// collapses a pasted full path to `<account>\<Folder>\<stem>`. Empty when
// the name sanitizes to nothing or the Rust ABI is unavailable.
std::string ReplayToTrainingPlugin::sanitizeTargetName(std::string value) {
  if (!sanitizeTarget) {
    return {};
  }
  // Sanitizing only ever removes or same-length-rewrites characters, so the
  // input size bounds the output size.
  return readAbiString(value.size(), [&](uint8_t *buffer, size_t capacity) {
    return sanitizeTarget(value.c_str(), buffer, capacity);
  });
}

// The Training root that holds the account directories and the MyTraining\
// and Downloaded\ subfolders. Reuses the same resolution as the output
// directory (cvar override, then %USERPROFILE%\...\TAGame\Training).
std::filesystem::path ReplayToTrainingPlugin::resolveTrainingRoot() {
  return resolveOutputDirectory();
}

// Scans the Training root (root-level and per-account MyTraining\ and
// Downloaded\) for .Tem files via the Rust core and returns their target
// names, sorted. Duplicate stems across accounts come back qualified as
// `<account>\<Folder>\<stem>`.
std::vector<std::string> ReplayToTrainingPlugin::discoverTargets(
    std::string &error) {
  std::vector<std::string> targets;
  if (!targetsLen || !writeTargets) {
    error = "rust library not loaded";
    return targets;
  }
  const std::filesystem::path root = resolveTrainingRoot();
  std::error_code ec;
  if (root.empty() || !std::filesystem::exists(root, ec) ||
      !std::filesystem::is_directory(root, ec)) {
    error = "Training root is unavailable: " + root.string() +
            " (set replay_to_training_output_dir to your local Rocket League "
            "Training folder)";
    return targets;
  }

  const std::string rootString = root.string();
  const std::string joined = readAbiString(
      targetsLen(rootString.c_str()), [&](uint8_t *buffer, size_t capacity) {
        return writeTargets(rootString.c_str(), buffer, capacity);
      });
  size_t begin = 0;
  while (begin < joined.size()) {
    size_t end = joined.find('\n', begin);
    if (end == std::string::npos) {
      end = joined.size();
    }
    if (end > begin) {
      targets.emplace_back(joined.substr(begin, end - begin));
    }
    begin = end + 1;
  }
  return targets;
}

void ReplayToTrainingPlugin::clearTarget() {
  activeTargetPath.clear();
  std::snprintf(targetBuffer.data(), targetBuffer.size(), "%s", "");
  setCvarString("replay_to_training_target_save_name", "");
}

// Setting a target binds the in-memory pack to a specific .Tem: if the file
// exists it is OPENED into memory (existing rounds shown in the shot list,
// so capture appends to them); otherwise a fresh pack is started but the
// target path is remembered so the first save creates the file there.
//
// Resolution is account-directory aware (Rust core): an unqualified
// `MyTraining\<stem>` binds to the single location where that file exists
// (root-level or any account dir), or into the sole account dir for a new
// name. When the same stem exists under several accounts the set is refused
// with the qualified `<account>\MyTraining\<stem>` candidates, which are
// also accepted here directly.
void ReplayToTrainingPlugin::setTarget(const std::string &requested) {
  if (!resolveTarget) {
    setStatus("rust library not loaded");
    return;
  }
  const std::string sanitized = sanitizeTargetName(requested);
  if (sanitized.empty()) {
    setStatus("target name is empty after sanitizing");
    return;
  }
  const std::string rootString = resolveTrainingRoot().string();
  std::string resolvedBuffer(4096, '\0');
  const int32_t outcome = resolveTarget(
      rootString.c_str(), sanitized.c_str(),
      reinterpret_cast<uint8_t *>(resolvedBuffer.data()),
      resolvedBuffer.size());
  if (outcome < 0) {
    // -2 = ambiguous across accounts (message lists the qualified
    // candidates to use), -1 = invalid name.
    setStatus(std::format("target not set: {}", globalErrorMessage()));
    return;
  }
  resolvedBuffer.resize(static_cast<size_t>(outcome));
  const std::filesystem::path resolved(resolvedBuffer);
  std::snprintf(targetBuffer.data(), targetBuffer.size(), "%s",
                sanitized.c_str());
  setCvarString("replay_to_training_target_save_name", sanitized);

  std::error_code ec;
  if (std::filesystem::exists(resolved, ec)) {
    // openPackFromPath swaps the in-memory pack to the opened file and, on
    // the Rust side, records it as `loaded_from` so a later save recognizes
    // it as this pack (append, not clobber).
    openPackFromPath(resolved.string());
    activeTargetPath = resolved;
    setStatus(std::format("target {} ({} shots; captures append here)",
                          sanitized, packShotCount ? packShotCount(pack) : 0));
  } else {
    // No file yet: start clean but remember the bound path so the first save
    // creates it there.
    newPack();
    activeTargetPath = resolved;
    setStatus(std::format(
        "target {} (new; first save creates it)", sanitized));
  }
}

void ReplayToTrainingPlugin::targetCommand(
    const std::vector<std::string> &args) {
  if (args.size() >= 2) {
    std::string requested;
    for (size_t index = 1; index < args.size(); ++index) {
      if (index > 1) {
        requested += ' ';
      }
      requested += args[index];
    }
    setTarget(requested);
    return;
  }
  if (activeTargetPath.empty()) {
    setStatus("no target set. Use: replay_to_training_target MyTraining\\<name>");
  } else {
    setStatus(std::format("target {} -> {}",
                          cvarString("replay_to_training_target_save_name", ""),
                          activeTargetPath.string()));
  }
}

void ReplayToTrainingPlugin::listTargetsCommand() {
  std::string error;
  const std::vector<std::string> targets = discoverTargets(error);
  cvarManager->log(std::format("replay-to-training targets under {}",
                               resolveTrainingRoot().string()));
  for (const std::string &target : targets) {
    cvarManager->log("  " + target);
  }
  if (!error.empty()) {
    setStatus(error);
    return;
  }
  setStatus(std::format("listed {} training target(s); set one with "
                        "replay_to_training_target <name>",
                        targets.size()));
}
