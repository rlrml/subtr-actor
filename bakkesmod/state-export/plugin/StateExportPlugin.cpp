#include "StateExportPlugin.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstddef>
#include <cstdio>
#include <format>
#include <type_traits>

#include "imgui/imgui.h"

// Build identification, normally injected by CMake (see
// target_compile_definitions in ../CMakeLists.txt).
#ifndef STATE_EXPORT_PLUGIN_VERSION
#define STATE_EXPORT_PLUGIN_VERSION "0.1.0"
#endif

#ifndef STATE_EXPORT_GIT_HASH
#define STATE_EXPORT_GIT_HASH "unknown"
#endif

#ifndef STATE_EXPORT_GIT_DIRTY
#define STATE_EXPORT_GIT_DIRTY 0
#endif

#ifndef STATE_EXPORT_COMMIT_DATE
#define STATE_EXPORT_COMMIT_DATE "unknown"
#endif

BAKKESMOD_PLUGIN(
    StateExportPlugin,
    "State export",
    STATE_EXPORT_PLUGIN_VERSION,
    PLUGINTYPE_FREEPLAY)

// Unity build: every other plugin .cpp is compiled through this single
// translation unit (mirroring bakkesmod/subtr-actor/plugin/SubtrActorPlugin.cpp), so
// none of the included files may define colliding file-scope names.
#include "constants.cpp"
#include "abi_asserts.cpp"
#include "rust_bridge.cpp"
#include "sampling.cpp"
#include "hooks.cpp"
#include "tick.cpp"
#include "plugin_lifecycle.cpp"
#include "settings_window.cpp"
