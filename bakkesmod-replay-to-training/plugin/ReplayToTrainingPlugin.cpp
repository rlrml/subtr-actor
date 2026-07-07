#include "ReplayToTrainingPlugin.h"

#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <format>

#include "imgui/imgui.h"

// Build identification, normally injected by CMake (see
// target_compile_definitions in ../CMakeLists.txt).
#ifndef REPLAY_TO_TRAINING_PLUGIN_VERSION
#define REPLAY_TO_TRAINING_PLUGIN_VERSION "0.1.0"
#endif

#ifndef REPLAY_TO_TRAINING_GIT_HASH
#define REPLAY_TO_TRAINING_GIT_HASH "unknown"
#endif

#ifndef REPLAY_TO_TRAINING_GIT_DIRTY
#define REPLAY_TO_TRAINING_GIT_DIRTY 0
#endif

#ifndef REPLAY_TO_TRAINING_COMMIT_DATE
#define REPLAY_TO_TRAINING_COMMIT_DATE "unknown"
#endif

BAKKESMOD_PLUGIN(
    ReplayToTrainingPlugin,
    "Replay-to-training pack recorder",
    REPLAY_TO_TRAINING_PLUGIN_VERSION,
    PLUGINTYPE_REPLAY)

// Unity build: every other plugin .cpp is compiled through this single
// translation unit (mirroring bakkesmod/plugin/SubtrActorPlugin.cpp), so
// none of the included files may define colliding file-scope names.
#include "rust_bridge.cpp"
#include "capture.cpp"
#include "target.cpp"
#include "settings_window.cpp"
