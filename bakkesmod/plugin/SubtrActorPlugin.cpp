#include "SubtrActorPlugin.h"

#include <algorithm>
#include <array>
#include <cctype>
#include <cmath>
#include <cstddef>
#include <cstdlib>
#include <fstream>
#include <format>
#include <initializer_list>
#include <iterator>
#include <limits>
#include <sstream>
#include <tuple>
#include <type_traits>
#include <unordered_set>

#include "imgui/imgui.h"

BAKKESMOD_PLUGIN(
    SubtrActorPlugin,
    "subtr-actor mechanic overlay",
    "0.1.0",
    PLUGINTYPE_FREEPLAY | PLUGINTYPE_CUSTOM_TRAINING | PLUGINTYPE_REPLAY)


#include "ui_constants.cpp"
#include "json_helpers.cpp"
#include "graph_stat_helpers.cpp"
#include "abi_asserts.cpp"
#include "event_helpers.cpp"
#include "web_config.cpp"
#include "apply_ui_config.cpp"
#include "ui_config_io.cpp"
#include "live_tick.cpp"
#include "live_sampling.cpp"
#include "graph_dump_commands.cpp"
#include "product_dump_commands.cpp"
#include "graph_verify.cpp"
#include "event_messages.cpp"
#include "overlay_layout.cpp"
#include "launcher_windows.cpp"
#include "review_windows.cpp"
#include "control_windows.cpp"
#include "window_manager.cpp"
#include "stats_windows_core.cpp"
#include "stats_window_editor.cpp"
#include "stats_window_tables.cpp"
#include "json_inspector_windows.cpp"
