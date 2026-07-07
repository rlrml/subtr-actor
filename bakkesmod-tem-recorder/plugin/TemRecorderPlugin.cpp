#include "TemRecorderPlugin.h"

#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <format>

#include "imgui/imgui.h"

BAKKESMOD_PLUGIN(
    TemRecorderPlugin,
    "TEM training pack recorder",
    "0.1.0",
    PLUGINTYPE_REPLAY)

// Unity build: every other plugin .cpp is compiled through this single
// translation unit (mirroring bakkesmod/plugin/SubtrActorPlugin.cpp), so
// none of the included files may define colliding file-scope names.
#include "rust_bridge.cpp"
#include "capture.cpp"
#include "settings_window.cpp"
