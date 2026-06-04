#include "subtr_actor_bakkesmod.h"

#include <array>
#include <cstdint>
#include <iostream>
#include <optional>
#include <stdexcept>
#include <string>
#include <string_view>
#include <vector>

#ifdef _WIN32
#include <windows.h>
#else
#error "verify-rust-dll-runtime.cpp must be built as a Windows executable"
#endif

namespace {

using EngineCreate = SaEngine *(*)();
using EngineDestroy = void (*)(SaEngine *);
using EngineFinish = int32_t (*)(SaEngine *);
using ProcessFrame = int32_t (*)(SaEngine *, const SaLiveFrame *);
using JsonLen = size_t (*)(const SaEngine *);
using WriteJson = size_t (*)(const SaEngine *, uint8_t *, size_t);
using NamedJsonLen = size_t (*)(const SaEngine *, const char *);
using WriteNamedJson = size_t (*)(const SaEngine *, const char *, uint8_t *, size_t);
using DrainEvents = size_t (*)(SaEngine *, SaMechanicEvent *, size_t);
using DrainTeamEvents = size_t (*)(SaEngine *, SaTeamEvent *, size_t);
using DrainGoalContextEvents = size_t (*)(SaEngine *, SaGoalContextEvent *, size_t);

struct RustAbi {
  HMODULE library = nullptr;
  EngineCreate engineCreate = nullptr;
  EngineDestroy engineDestroy = nullptr;
  EngineFinish finish = nullptr;
  ProcessFrame processFrame = nullptr;
  JsonLen graphInfoJsonLen = nullptr;
  WriteJson writeGraphInfoJson = nullptr;
  JsonLen analysisNodeNamesJsonLen = nullptr;
  WriteJson writeAnalysisNodeNamesJson = nullptr;
  JsonLen eventsJsonLen = nullptr;
  WriteJson writeEventsJson = nullptr;
  JsonLen frameJsonLen = nullptr;
  WriteJson writeFrameJson = nullptr;
  JsonLen timelineJsonLen = nullptr;
  WriteJson writeTimelineJson = nullptr;
  JsonLen statsJsonLen = nullptr;
  WriteJson writeStatsJson = nullptr;
  NamedJsonLen graphOutputJsonLen = nullptr;
  WriteNamedJson writeGraphOutputJson = nullptr;
  NamedJsonLen analysisNodeJsonLen = nullptr;
  WriteNamedJson writeAnalysisNodeJson = nullptr;
  NamedJsonLen statsModuleJsonLen = nullptr;
  WriteNamedJson writeStatsModuleJson = nullptr;
  NamedJsonLen statsModuleFrameJsonLen = nullptr;
  WriteNamedJson writeStatsModuleFrameJson = nullptr;
  NamedJsonLen statsModuleConfigJsonLen = nullptr;
  WriteNamedJson writeStatsModuleConfigJson = nullptr;
  DrainEvents drainEvents = nullptr;
  DrainTeamEvents drainTeamEvents = nullptr;
  DrainGoalContextEvents drainGoalContextEvents = nullptr;
};

template <typename T>
T loadSymbol(HMODULE library, const char *name) {
  auto *address = GetProcAddress(library, name);
  if (!address) {
    throw std::runtime_error(std::string("missing symbol ") + name);
  }
  return reinterpret_cast<T>(address);
}

std::wstring widenPath(std::string_view path) {
  if (path.empty()) {
    return {};
  }
  const int length = MultiByteToWideChar(
      CP_UTF8, MB_ERR_INVALID_CHARS, path.data(), static_cast<int>(path.size()), nullptr, 0);
  if (length <= 0) {
    throw std::runtime_error("failed to decode DLL path as UTF-8");
  }
  std::wstring widePath(static_cast<size_t>(length), L'\0');
  MultiByteToWideChar(
      CP_UTF8,
      MB_ERR_INVALID_CHARS,
      path.data(),
      static_cast<int>(path.size()),
      widePath.data(),
      length);
  return widePath;
}

RustAbi loadAbi(std::string_view dllPath) {
  RustAbi abi;
  const std::wstring widePath = widenPath(dllPath);
  abi.library = LoadLibraryW(widePath.c_str());
  if (!abi.library) {
    throw std::runtime_error("LoadLibraryW failed with error " + std::to_string(GetLastError()));
  }
  abi.engineCreate =
      loadSymbol<EngineCreate>(abi.library, "subtr_actor_bakkesmod_engine_create");
  abi.engineDestroy =
      loadSymbol<EngineDestroy>(abi.library, "subtr_actor_bakkesmod_engine_destroy");
  abi.finish = loadSymbol<EngineFinish>(abi.library, "subtr_actor_bakkesmod_finish");
  abi.processFrame =
      loadSymbol<ProcessFrame>(abi.library, "subtr_actor_bakkesmod_process_frame");
  abi.graphInfoJsonLen =
      loadSymbol<JsonLen>(abi.library, "subtr_actor_bakkesmod_graph_info_json_len");
  abi.writeGraphInfoJson =
      loadSymbol<WriteJson>(abi.library, "subtr_actor_bakkesmod_write_graph_info_json");
  abi.analysisNodeNamesJsonLen =
      loadSymbol<JsonLen>(abi.library, "subtr_actor_bakkesmod_analysis_node_names_json_len");
  abi.writeAnalysisNodeNamesJson = loadSymbol<WriteJson>(
      abi.library, "subtr_actor_bakkesmod_write_analysis_node_names_json");
  abi.eventsJsonLen =
      loadSymbol<JsonLen>(abi.library, "subtr_actor_bakkesmod_events_json_len");
  abi.writeEventsJson =
      loadSymbol<WriteJson>(abi.library, "subtr_actor_bakkesmod_write_events_json");
  abi.frameJsonLen =
      loadSymbol<JsonLen>(abi.library, "subtr_actor_bakkesmod_frame_json_len");
  abi.writeFrameJson =
      loadSymbol<WriteJson>(abi.library, "subtr_actor_bakkesmod_write_frame_json");
  abi.timelineJsonLen =
      loadSymbol<JsonLen>(abi.library, "subtr_actor_bakkesmod_timeline_json_len");
  abi.writeTimelineJson =
      loadSymbol<WriteJson>(abi.library, "subtr_actor_bakkesmod_write_timeline_json");
  abi.statsJsonLen =
      loadSymbol<JsonLen>(abi.library, "subtr_actor_bakkesmod_stats_json_len");
  abi.writeStatsJson =
      loadSymbol<WriteJson>(abi.library, "subtr_actor_bakkesmod_write_stats_json");
  abi.graphOutputJsonLen =
      loadSymbol<NamedJsonLen>(abi.library, "subtr_actor_bakkesmod_graph_output_json_len");
  abi.writeGraphOutputJson = loadSymbol<WriteNamedJson>(
      abi.library, "subtr_actor_bakkesmod_write_graph_output_json");
  abi.analysisNodeJsonLen = loadSymbol<NamedJsonLen>(
      abi.library, "subtr_actor_bakkesmod_analysis_node_json_len");
  abi.writeAnalysisNodeJson = loadSymbol<WriteNamedJson>(
      abi.library, "subtr_actor_bakkesmod_write_analysis_node_json");
  abi.statsModuleJsonLen = loadSymbol<NamedJsonLen>(
      abi.library, "subtr_actor_bakkesmod_stats_module_json_len");
  abi.writeStatsModuleJson = loadSymbol<WriteNamedJson>(
      abi.library, "subtr_actor_bakkesmod_write_stats_module_json");
  abi.statsModuleFrameJsonLen = loadSymbol<NamedJsonLen>(
      abi.library, "subtr_actor_bakkesmod_stats_module_frame_json_len");
  abi.writeStatsModuleFrameJson = loadSymbol<WriteNamedJson>(
      abi.library, "subtr_actor_bakkesmod_write_stats_module_frame_json");
  abi.statsModuleConfigJsonLen = loadSymbol<NamedJsonLen>(
      abi.library, "subtr_actor_bakkesmod_stats_module_config_json_len");
  abi.writeStatsModuleConfigJson = loadSymbol<WriteNamedJson>(
      abi.library, "subtr_actor_bakkesmod_write_stats_module_config_json");
  abi.drainEvents =
      loadSymbol<DrainEvents>(abi.library, "subtr_actor_bakkesmod_drain_events");
  abi.drainTeamEvents =
      loadSymbol<DrainTeamEvents>(abi.library, "subtr_actor_bakkesmod_drain_team_events");
  abi.drainGoalContextEvents = loadSymbol<DrainGoalContextEvents>(
      abi.library, "subtr_actor_bakkesmod_drain_goal_context_events");
  return abi;
}

std::string readJson(const SaEngine *engine, JsonLen jsonLen, WriteJson writeJson) {
  const size_t length = jsonLen(engine);
  if (length == 0) {
    throw std::runtime_error("JSON length function returned zero");
  }
  std::string json(length, '\0');
  const size_t written =
      writeJson(engine, reinterpret_cast<uint8_t *>(json.data()), json.size());
  if (written != length) {
    throw std::runtime_error("JSON write length mismatch");
  }
  return json;
}

std::string readNamedJson(
    const SaEngine *engine,
    NamedJsonLen jsonLen,
    WriteNamedJson writeJson,
    const std::string &name) {
  const size_t length = jsonLen(engine, name.c_str());
  if (length == 0) {
    throw std::runtime_error("named JSON length function returned zero for " + name);
  }
  std::string json(length, '\0');
  const size_t written =
      writeJson(engine, name.c_str(), reinterpret_cast<uint8_t *>(json.data()), json.size());
  if (written != length) {
    throw std::runtime_error("named JSON write length mismatch for " + name);
  }
  return json;
}

void skipJsonWhitespace(std::string_view json, size_t &offset) {
  while (offset < json.size()) {
    const char ch = json[offset];
    if (ch != ' ' && ch != '\n' && ch != '\r' && ch != '\t') {
      return;
    }
    ++offset;
  }
}

std::optional<std::string> parseJsonString(std::string_view json, size_t &offset) {
  skipJsonWhitespace(json, offset);
  if (offset >= json.size() || json[offset] != '"') {
    return std::nullopt;
  }
  ++offset;
  std::string value;
  while (offset < json.size()) {
    const char ch = json[offset++];
    if (ch == '"') {
      return value;
    }
    if (ch != '\\') {
      value.push_back(ch);
      continue;
    }
    if (offset >= json.size()) {
      return std::nullopt;
    }
    const char escaped = json[offset++];
    switch (escaped) {
    case '"':
    case '\\':
    case '/':
      value.push_back(escaped);
      break;
    case 'b':
      value.push_back('\b');
      break;
    case 'f':
      value.push_back('\f');
      break;
    case 'n':
      value.push_back('\n');
      break;
    case 'r':
      value.push_back('\r');
      break;
    case 't':
      value.push_back('\t');
      break;
    default:
      return std::nullopt;
    }
  }
  return std::nullopt;
}

std::vector<std::string> parseStringArray(std::string_view json, size_t &offset) {
  skipJsonWhitespace(json, offset);
  if (offset >= json.size() || json[offset] != '[') {
    throw std::runtime_error("expected JSON string array");
  }
  ++offset;
  std::vector<std::string> values;
  skipJsonWhitespace(json, offset);
  if (offset < json.size() && json[offset] == ']') {
    ++offset;
    return values;
  }
  while (offset < json.size()) {
    auto value = parseJsonString(json, offset);
    if (!value) {
      throw std::runtime_error("expected string in JSON string array");
    }
    values.push_back(*value);
    skipJsonWhitespace(json, offset);
    if (offset < json.size() && json[offset] == ',') {
      ++offset;
      continue;
    }
    if (offset < json.size() && json[offset] == ']') {
      ++offset;
      return values;
    }
    throw std::runtime_error("expected comma or end of JSON string array");
  }
  throw std::runtime_error("unterminated JSON string array");
}

std::vector<std::string> parseStringArrayProperty(
    std::string_view json,
    std::string_view propertyName) {
  size_t offset = 0;
  skipJsonWhitespace(json, offset);
  if (offset >= json.size() || json[offset] != '{') {
    throw std::runtime_error("expected JSON object");
  }
  ++offset;
  while (offset < json.size()) {
    skipJsonWhitespace(json, offset);
    if (offset < json.size() && json[offset] == '}') {
      break;
    }
    auto key = parseJsonString(json, offset);
    if (!key) {
      throw std::runtime_error("expected JSON object key");
    }
    skipJsonWhitespace(json, offset);
    if (offset >= json.size() || json[offset] != ':') {
      throw std::runtime_error("expected ':' after JSON object key");
    }
    ++offset;
    if (*key == propertyName) {
      return parseStringArray(json, offset);
    }
    int depth = 0;
    bool inString = false;
    bool escaped = false;
    while (offset < json.size()) {
      const char ch = json[offset++];
      if (inString) {
        if (escaped) {
          escaped = false;
        } else if (ch == '\\') {
          escaped = true;
        } else if (ch == '"') {
          inString = false;
        }
        continue;
      }
      if (ch == '"') {
        inString = true;
      } else if (ch == '[' || ch == '{') {
        ++depth;
      } else if (ch == ']' || ch == '}') {
        if (depth == 0) {
          --offset;
          break;
        }
        --depth;
      } else if (ch == ',' && depth == 0) {
        break;
      }
    }
  }
  throw std::runtime_error("missing JSON property " + std::string(propertyName));
}

SaVec3 vec3(float x, float y, float z) {
  return SaVec3{x, y, z};
}

SaRigidBody rigidBody(SaVec3 location, SaVec3 linearVelocity = SaVec3{}) {
  SaRigidBody body{};
  body.location = location;
  body.rotation = SaQuat{};
  body.linear_velocity = linearVelocity;
  body.angular_velocity = SaVec3{};
  body.has_linear_velocity = 1;
  body.has_angular_velocity = 1;
  return body;
}

SaEventTiming timing(uint64_t frameNumber) {
  return SaEventTiming{frameNumber, static_cast<float>(frameNumber) * 0.1f, 299, 1, 1};
}

SaPlayerFrame player(uint32_t index, bool isTeam0, SaVec3 location) {
  SaPlayerFrame value{};
  value.player_index = index;
  value.player_name = nullptr;
  value.is_team_0 = isTeam0 ? 1 : 0;
  value.has_rigid_body = 1;
  value.rigid_body = rigidBody(location);
  value.boost_amount = 33.0f;
  value.last_boost_amount = 33.0f;
  value.car_body_id = 23;
  value.has_car_body_id = 1;
  value.has_match_stats = 1;
  value.match_goals = static_cast<int32_t>(index);
  value.match_assists = static_cast<int32_t>(index + 1);
  value.match_saves = static_cast<int32_t>(index + 2);
  value.match_shots = static_cast<int32_t>(index + 3);
  value.match_score = static_cast<int32_t>(index + 100);
  return value;
}

SaLiveFrame liveFrame(uint64_t frameNumber, const std::vector<SaPlayerFrame> &players) {
  SaLiveFrame frame{};
  frame.frame_number = frameNumber;
  frame.time = static_cast<float>(frameNumber) * 0.1f;
  frame.dt = 0.1f;
  frame.seconds_remaining = 299;
  frame.has_seconds_remaining = 1;
  frame.ball_has_been_hit = 1;
  frame.has_ball_has_been_hit = 1;
  frame.live_play = 1;
  frame.has_live_play = 1;
  frame.has_ball = 1;
  frame.ball = rigidBody(vec3(static_cast<float>(frameNumber) * 25.0f, 0.0f, 120.0f));
  frame.players = players.data();
  frame.player_count = players.size();
  return frame;
}

struct SyntheticFrames {
  std::vector<SaPlayerFrame> players;
  std::vector<SaTouchEvent> touches;
  std::vector<SaDodgeRefreshedEvent> dodgeRefreshes;
  std::vector<SaBoostPadEvent> boostPadEvents;
  std::vector<SaGoalEvent> goals;
  std::vector<SaPlayerStatEvent> playerStatEvents;
  std::vector<SaDemolishEvent> demolishes;
  std::vector<SaLiveFrame> frames;
};

SyntheticFrames buildSyntheticFrames() {
  SyntheticFrames fixture;
  fixture.players = {
      player(0, true, vec3(0.0f, 0.0f, 92.75f)),
      player(1, false, vec3(120.0f, 0.0f, 92.75f)),
  };
  auto shotBall = rigidBody(vec3(300.0f, 100.0f, 120.0f), vec3(1000.0f, 500.0f, 100.0f));
  auto shotPlayer = rigidBody(vec3(240.0f, 90.0f, 92.75f), vec3(800.0f, 300.0f, 0.0f));
  fixture.touches = {SaTouchEvent{timing(1), 0, 1, 1, 12.0f, 1}};
  fixture.dodgeRefreshes = {SaDodgeRefreshedEvent{timing(1), 0, 1, 1}};
  fixture.boostPadEvents = {SaBoostPadEvent{timing(1), 34, SaBoostPadEventKindPickedUp, 1, 0, 1}};
  fixture.goals = {SaGoalEvent{timing(1), 1, 0, 1, 1, 1, 0, 1}};
  fixture.playerStatEvents = {
      SaPlayerStatEvent{timing(1), 0, 1, SaPlayerStatEventKindShot, 1, shotBall, 1, shotPlayer},
      SaPlayerStatEvent{timing(1), 1, 0, SaPlayerStatEventKindSave, 0, SaRigidBody{}, 0, SaRigidBody{}},
      SaPlayerStatEvent{timing(1), 0, 1, SaPlayerStatEventKindAssist, 0, SaRigidBody{}, 0, SaRigidBody{}},
  };
  fixture.demolishes = {SaDemolishEvent{
      timing(1),
      0,
      1,
      vec3(2300.0f, 0.0f, 0.0f),
      SaVec3{},
      vec3(120.0f, 0.0f, 92.75f),
      0.25f,
  }};
  for (uint64_t frameNumber = 1; frameNumber <= 3; ++frameNumber) {
    fixture.frames.push_back(liveFrame(frameNumber, fixture.players));
  }
  fixture.frames[0].touches = fixture.touches.data();
  fixture.frames[0].touch_count = fixture.touches.size();
  fixture.frames[0].dodge_refreshes = fixture.dodgeRefreshes.data();
  fixture.frames[0].dodge_refresh_count = fixture.dodgeRefreshes.size();
  fixture.frames[0].boost_pad_events = fixture.boostPadEvents.data();
  fixture.frames[0].boost_pad_event_count = fixture.boostPadEvents.size();
  fixture.frames[0].goals = fixture.goals.data();
  fixture.frames[0].goal_count = fixture.goals.size();
  fixture.frames[0].player_stat_events = fixture.playerStatEvents.data();
  fixture.frames[0].player_stat_event_count = fixture.playerStatEvents.size();
  fixture.frames[0].demolishes = fixture.demolishes.data();
  fixture.frames[0].demolish_count = fixture.demolishes.size();
  return fixture;
}

bool jsonArrayPropertyHasEntries(std::string_view json, std::string_view propertyName) {
  const auto needle = "\"" + std::string(propertyName) + "\":";
  const size_t propertyOffset = json.find(needle);
  if (propertyOffset == std::string_view::npos) {
    throw std::runtime_error("missing JSON property " + std::string(propertyName));
  }
  size_t offset = propertyOffset + needle.size();
  skipJsonWhitespace(json, offset);
  if (offset >= json.size() || json[offset] != '[') {
    throw std::runtime_error("JSON property is not an array: " + std::string(propertyName));
  }
  ++offset;
  skipJsonWhitespace(json, offset);
  return offset < json.size() && json[offset] != ']';
}

int run(std::string_view dllPath) {
  auto abi = loadAbi(dllPath);
  SaEngine *engine = abi.engineCreate();
  if (!engine) {
    throw std::runtime_error("engine_create returned null");
  }
  try {
    auto fixture = buildSyntheticFrames();
    for (const auto &frame : fixture.frames) {
      const int32_t status = abi.processFrame(engine, &frame);
      if (status != 0) {
        throw std::runtime_error("process_frame failed");
      }
    }
    if (abi.finish(engine) != 0) {
      throw std::runtime_error("finish failed");
    }

    const std::string graphInfo = readJson(engine, abi.graphInfoJsonLen, abi.writeGraphInfoJson);
    const auto graphOutputs = parseStringArrayProperty(graphInfo, "graph_output_names");
    const auto analysisNodes = parseStringArrayProperty(graphInfo, "callable_analysis_node_names");
    const std::string analysisNodeNamesJson =
        readJson(engine, abi.analysisNodeNamesJsonLen, abi.writeAnalysisNodeNamesJson);
    size_t analysisNodeNamesOffset = 0;
    const auto analysisNodeNames = parseStringArray(analysisNodeNamesJson, analysisNodeNamesOffset);
    const auto statsModules = parseStringArrayProperty(graphInfo, "builtin_stats_module_names");
    if (analysisNodes != analysisNodeNames) {
      throw std::runtime_error("analysis_node_names does not match graph_info");
    }

    for (const auto &outputName : graphOutputs) {
      readNamedJson(engine, abi.graphOutputJsonLen, abi.writeGraphOutputJson, outputName);
    }
    for (const auto &nodeName : analysisNodes) {
      readNamedJson(engine, abi.analysisNodeJsonLen, abi.writeAnalysisNodeJson, nodeName);
    }
    for (const auto &moduleName : statsModules) {
      readNamedJson(engine, abi.statsModuleJsonLen, abi.writeStatsModuleJson, moduleName);
      readNamedJson(engine, abi.statsModuleFrameJsonLen, abi.writeStatsModuleFrameJson, moduleName);
      readNamedJson(engine, abi.statsModuleConfigJsonLen, abi.writeStatsModuleConfigJson, moduleName);
    }
    readJson(engine, abi.eventsJsonLen, abi.writeEventsJson);
    readJson(engine, abi.frameJsonLen, abi.writeFrameJson);
    readJson(engine, abi.timelineJsonLen, abi.writeTimelineJson);
    readJson(engine, abi.statsJsonLen, abi.writeStatsJson);

    const std::string eventHistory =
        readNamedJson(engine, abi.graphOutputJsonLen, abi.writeGraphOutputJson, "event_history");
    const std::string events =
        readNamedJson(engine, abi.graphOutputJsonLen, abi.writeGraphOutputJson, "events");
    for (const std::string &fieldName :
         parseStringArrayProperty(graphInfo, "required_event_history_field_names")) {
      if (!jsonArrayPropertyHasEntries(eventHistory, fieldName)) {
        throw std::runtime_error("event_history field is empty: " + fieldName);
      }
    }
    for (const std::string &fieldName :
         parseStringArrayProperty(graphInfo, "required_graph_event_field_names")) {
      if (!jsonArrayPropertyHasEntries(events, fieldName)) {
        throw std::runtime_error("events field is empty: " + fieldName);
      }
    }

    std::array<SaMechanicEvent, 64> mechanicEvents{};
    std::array<SaTeamEvent, 64> teamEvents{};
    std::array<SaGoalContextEvent, 64> goalContextEvents{};
    const size_t mechanicCount = abi.drainEvents(engine, mechanicEvents.data(), mechanicEvents.size());
    const size_t teamCount = abi.drainTeamEvents(engine, teamEvents.data(), teamEvents.size());
    const size_t goalContextCount =
        abi.drainGoalContextEvents(engine, goalContextEvents.data(), goalContextEvents.size());
    if (mechanicCount == 0) {
      throw std::runtime_error("expected at least one drainable player-owned event");
    }
    if (goalContextCount == 0) {
      throw std::runtime_error("expected at least one drainable goal-context event");
    }

    std::cout << "Loaded " << dllPath << "; processed " << fixture.frames.size()
              << " frames, called " << analysisNodes.size() << " analysis nodes, "
              << statsModules.size() << " stats modules, drained " << mechanicCount
              << " player events, " << teamCount << " team events, " << goalContextCount
              << " goal-context events\n";
    abi.engineDestroy(engine);
    FreeLibrary(abi.library);
    return 0;
  } catch (...) {
    abi.engineDestroy(engine);
    FreeLibrary(abi.library);
    throw;
  }
}

} // namespace

int main(int argc, char **argv) {
  if (argc < 2) {
    std::cerr << "usage: verify-rust-dll-runtime.exe <subtr_actor_bakkesmod.dll>...\n";
    return 2;
  }
  int status = 0;
  for (int index = 1; index < argc; ++index) {
    try {
      status |= run(argv[index]);
    } catch (const std::exception &error) {
      std::cerr << "ERROR: " << argv[index] << ": " << error.what() << "\n";
      status = 1;
    }
  }
  return status;
}
