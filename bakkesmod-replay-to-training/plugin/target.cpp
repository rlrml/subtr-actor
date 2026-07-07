// Persistent default-save "target": binds the in-memory pack to a specific
// custom-training .Tem under the game's Training root so captures accumulate
// into it non-destructively.
//
// The save flow is inherently append-only because memory is the single
// source of truth: setting a target OPENS the existing file into memory
// (existing rounds included), capture appends to memory, and save writes
// memory back to the target path. See recorder.rs `save_to_target`.
//
// Sanitize/discover logic is adapted from
// rlrml-training-pack-snapshot/plugin/TrainingPackSnapshotPlugin.cpp
// (sanitizeTrainingSaveName / discoverTrainingTargets).

#include <cctype>
#include <sstream>
#include <string_view>
#include <system_error>

namespace {

std::string trimWhitespace(std::string value) {
  const auto notSpace = [](unsigned char ch) { return !std::isspace(ch); };
  value.erase(value.begin(),
              std::find_if(value.begin(), value.end(), notSpace));
  value.erase(std::find_if(value.rbegin(), value.rend(), notSpace).base(),
              value.end());
  return value;
}

std::string toLower(std::string value) {
  std::transform(value.begin(), value.end(), value.begin(),
                 [](unsigned char ch) { return std::tolower(ch); });
  return value;
}

bool hasTemExtension(const std::filesystem::path &path) {
  return toLower(path.extension().string()) == ".tem";
}

// The subfolders the game lists custom training from, under the Training
// root. Targets resolve into one of these.
constexpr std::array<std::string_view, 2> kTargetFolders = {"MyTraining",
                                                            "Downloaded"};

}  // namespace

// Normalizes a user-entered target: trims, converts '/'→'\\', drops a
// trailing .tem/.Tem, and — when the name is `<parent>\<stem>` with a known
// parent (MyTraining/Downloaded) — canonicalizes the parent's case, e.g.
// "mytraining/AEB.tem" -> "MyTraining\\AEB". A bare stem is returned as-is
// (resolveTargetPath defaults it into MyTraining\).
std::string ReplayToTrainingPlugin::sanitizeTargetName(std::string value) {
  value = trimWhitespace(std::move(value));
  std::replace(value.begin(), value.end(), '/', '\\');

  constexpr std::string_view suffix = ".tem";
  if (value.size() >= suffix.size() &&
      toLower(value.substr(value.size() - suffix.size())) == suffix) {
    value.erase(value.size() - suffix.size());
  }

  std::vector<std::string> components;
  std::stringstream stream(value);
  std::string component;
  while (std::getline(stream, component, '\\')) {
    if (!component.empty()) {
      components.push_back(component);
    }
  }
  if (components.size() >= 2) {
    const std::string parent = toLower(components[components.size() - 2]);
    for (const std::string_view folder : kTargetFolders) {
      if (parent == toLower(std::string(folder))) {
        return std::string(folder) + "\\" + components.back();
      }
    }
  }
  return value;
}

// The Training root that holds the MyTraining\ and Downloaded\ subfolders.
// Reuses the same resolution as the output directory (cvar override, then
// %USERPROFILE%\...\TAGame\Training).
std::filesystem::path ReplayToTrainingPlugin::resolveTrainingRoot() {
  return resolveOutputDirectory();
}

// Turns a sanitized target name into an on-disk path
// `<trainingRoot>/<sub>/<stem>.Tem`. A bare stem (no subfolder) defaults
// into MyTraining\, which is where the game lists locally created packs.
std::filesystem::path ReplayToTrainingPlugin::resolveTargetPath(
    const std::string &sanitizedName) {
  if (sanitizedName.empty()) {
    return {};
  }
  std::filesystem::path relative;
  const size_t separator = sanitizedName.find('\\');
  if (separator == std::string::npos) {
    relative = std::filesystem::path("MyTraining") / sanitizedName;
  } else {
    relative = std::filesystem::path(sanitizedName.substr(0, separator)) /
               std::filesystem::path(sanitizedName.substr(separator + 1));
  }
  return resolveTrainingRoot() / relative.replace_extension(".Tem");
}

// Scans MyTraining\ and Downloaded\ under the Training root for .Tem files
// and returns their sanitized `<sub>\<stem>` names (sorted).
std::vector<std::string> ReplayToTrainingPlugin::discoverTargets(
    std::string &error) {
  std::vector<std::string> targets;
  const std::filesystem::path root = resolveTrainingRoot();
  std::error_code ec;
  if (root.empty() || !std::filesystem::exists(root, ec) ||
      !std::filesystem::is_directory(root, ec)) {
    error = "Training root is unavailable: " + root.string() +
            " (set replay_to_training_output_dir to your local Rocket League "
            "Training folder)";
    return targets;
  }

  for (const std::string_view folderView : kTargetFolders) {
    const std::string folder(folderView);
    const std::filesystem::path directory = root / folder;
    if (!std::filesystem::exists(directory, ec) ||
        !std::filesystem::is_directory(directory, ec)) {
      ec.clear();
      continue;
    }
    std::vector<std::string> folderTargets;
    std::filesystem::directory_iterator it(
        directory, std::filesystem::directory_options::skip_permission_denied,
        ec);
    const std::filesystem::directory_iterator end;
    for (; !ec && it != end; it.increment(ec)) {
      if (it->is_regular_file(ec) && hasTemExtension(it->path())) {
        folderTargets.push_back(
            sanitizeTargetName(folder + "\\" + it->path().stem().string()));
      }
    }
    if (ec) {
      error = "Stopped scanning " + folder + " after filesystem error: " +
              ec.message();
      return targets;
    }
    std::sort(folderTargets.begin(), folderTargets.end());
    targets.insert(targets.end(), folderTargets.begin(), folderTargets.end());
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
void ReplayToTrainingPlugin::setTarget(const std::string &requested) {
  const std::string sanitized = sanitizeTargetName(requested);
  if (sanitized.empty()) {
    setStatus("target name is empty after sanitizing");
    return;
  }
  const std::filesystem::path resolved = resolveTargetPath(sanitized);
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
