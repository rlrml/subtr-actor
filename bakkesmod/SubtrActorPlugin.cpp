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


#include "SubtrActorPlugin_ui_constants.cpp"
#include "SubtrActorPlugin_json_helpers.cpp"
#include "SubtrActorPlugin_graph_stat_helpers.cpp"
#include "SubtrActorPlugin_abi_asserts.cpp"
#include "SubtrActorPlugin_event_helpers.cpp"
#include "SubtrActorPlugin_web_config.cpp"
#include "SubtrActorPlugin_apply_ui_config.cpp"
#include "SubtrActorPlugin_ui_config_io.cpp"
#include "SubtrActorPlugin_live_tick.cpp"
#include "SubtrActorPlugin_live_sampling.cpp"
#include "SubtrActorPlugin_graph_dump_commands.cpp"
#include "SubtrActorPlugin_graph_verify.cpp"
#include "SubtrActorPlugin_event_messages.cpp"
#include "SubtrActorPlugin_overlay_layout.cpp"
#include "SubtrActorPlugin_launcher_windows.cpp"
#include "SubtrActorPlugin_review_windows.cpp"
#include "SubtrActorPlugin_control_windows.cpp"
#include "SubtrActorPlugin_window_manager.cpp"
#include "SubtrActorPlugin_stats_windows_core.cpp"
#include "SubtrActorPlugin_stats_window_editor.cpp"
#include "SubtrActorPlugin_stats_window_tables.cpp"
#include "SubtrActorPlugin_json_inspector_windows.cpp"
