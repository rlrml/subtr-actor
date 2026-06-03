// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
namespace {

bool findJsonPropertyValueOffset(
    const std::string &json,
    const std::string &propertyName,
    size_t &offset) {
  const std::string needle = std::format("\"{}\"", propertyName);
  offset = json.find(needle);
  if (offset == std::string::npos) {
    return false;
  }
  offset += needle.size();
  skipJsonWhitespace(json, offset);
  if (offset >= json.size() || json[offset] != ':') {
    return false;
  }
  ++offset;
  skipJsonWhitespace(json, offset);
  return offset < json.size();
}

bool jsonPropertyExists(const std::string &json, const std::string &propertyName) {
  size_t offset = 0;
  return findJsonPropertyValueOffset(json, propertyName, offset);
}

bool jsonPropertyIsNull(const std::string &json, const std::string &propertyName) {
  size_t offset = 0;
  return findJsonPropertyValueOffset(json, propertyName, offset) &&
         json.compare(offset, 4, "null") == 0;
}

std::optional<std::string> parseJsonStringProperty(
    const std::string &json,
    const std::string &propertyName) {
  size_t offset = 0;
  if (!findJsonPropertyValueOffset(json, propertyName, offset)) {
    return std::nullopt;
  }
  return parseJsonString(json, offset);
}

std::optional<bool> parseJsonBoolProperty(
    const std::string &json,
    const std::string &propertyName) {
  size_t offset = 0;
  if (!findJsonPropertyValueOffset(json, propertyName, offset)) {
    return std::nullopt;
  }
  if (json.compare(offset, 4, "true") == 0) {
    return true;
  }
  if (json.compare(offset, 5, "false") == 0) {
    return false;
  }
  return std::nullopt;
}

std::optional<double> parseJsonNumberProperty(
    const std::string &json,
    const std::string &propertyName) {
  size_t offset = 0;
  if (!findJsonPropertyValueOffset(json, propertyName, offset)) {
    return std::nullopt;
  }
  const size_t start = offset;
  if (offset < json.size() && json[offset] == '-') {
    ++offset;
  }
  while (offset < json.size() &&
         std::isdigit(static_cast<unsigned char>(json[offset])) != 0) {
    ++offset;
  }
  if (offset < json.size() && json[offset] == '.') {
    ++offset;
    while (offset < json.size() &&
           std::isdigit(static_cast<unsigned char>(json[offset])) != 0) {
      ++offset;
    }
  }
  if (offset == start) {
    return std::nullopt;
  }
  try {
    return std::stod(json.substr(start, offset - start));
  } catch (...) {
    return std::nullopt;
  }
}

std::vector<std::string> parseJsonObjectArrayProperty(
    const std::string &json,
    const std::string &propertyName) {
  std::vector<std::string> objects;
  size_t offset = 0;
  if (!findJsonPropertyValueOffset(json, propertyName, offset) || json[offset] != '[') {
    return objects;
  }
  ++offset;
  while (offset < json.size()) {
    skipJsonWhitespace(json, offset);
    if (offset < json.size() && json[offset] == ']') {
      break;
    }
    if (offset >= json.size() || json[offset] != '{') {
      return {};
    }
    const size_t start = offset;
    size_t end = offset;
    if (!skipJsonValue(json, end)) {
      return {};
    }
    objects.push_back(json.substr(start, end - start));
    offset = end;
    skipJsonWhitespace(json, offset);
    if (offset < json.size() && json[offset] == ',') {
      ++offset;
      continue;
    }
    if (offset < json.size() && json[offset] == ']') {
      break;
    }
  }
  return objects;
}

std::optional<std::string> parseJsonObjectProperty(
    const std::string &json,
    const std::string &propertyName) {
  size_t offset = 0;
  if (!findJsonPropertyValueOffset(json, propertyName, offset) || json[offset] != '{') {
    return std::nullopt;
  }
  const size_t start = offset;
  size_t end = offset;
  if (!skipJsonValue(json, end)) {
    return std::nullopt;
  }
  return json.substr(start, end - start);
}

std::optional<std::string> parseJsonPropertyValue(
    const std::string &json,
    std::string_view propertyName) {
  size_t offset = 0;
  if (!findJsonPropertyValueOffset(json, std::string{propertyName}, offset)) {
    return std::nullopt;
  }
  const size_t start = offset;
  size_t end = offset;
  if (!skipJsonValue(json, end)) {
    return std::nullopt;
  }
  return json.substr(start, end - start);
}

std::vector<std::string_view> dotPathSegments(std::string_view path) {
  std::vector<std::string_view> segments;
  size_t offset = 0;
  while (offset < path.size()) {
    const size_t end = path.find('.', offset);
    segments.emplace_back(
        path.data() + offset,
        (end == std::string_view::npos ? path.size() : end) - offset);
    if (end == std::string_view::npos) {
      break;
    }
    offset = end + 1;
  }
  return segments;
}

std::string formatGraphStatJsonValueAt(const std::string &json, size_t offset) {
  skipJsonWhitespace(json, offset);
  if (offset >= json.size()) {
    return "--";
  }

  if (json.compare(offset, 4, "null") == 0) {
    return "--";
  }
  if (json.compare(offset, 4, "true") == 0) {
    return "true";
  }
  if (json.compare(offset, 5, "false") == 0) {
    return "false";
  }

  if (json[offset] == '"') {
    auto value = parseJsonString(json, offset);
    return value ? clippedDisplayText(*value, 240) : "--";
  }

  if (json[offset] == '[') {
    const auto count = parseJsonArrayElementCountAt(json, offset);
    if (count && *count == 0) {
      return "[]";
    }
    const size_t start = offset;
    size_t end = offset;
    if (!skipJsonValue(json, end)) {
      return "--";
    }
    return clippedDisplayText(json.substr(start, end - start), 240);
  }

  if (json[offset] == '{') {
    return "object";
  }

  const size_t start = offset;
  size_t end = offset;
  if (!skipJsonValue(json, end)) {
    return "--";
  }
  try {
    return formatGraphStatNumber(std::stod(json.substr(start, end - start)));
  } catch (...) {
    return "--";
  }
}

std::optional<std::string> jsonDisplayValueAtPath(
    const std::string &json,
    std::string_view path) {
  std::string current = json;
  const std::vector<std::string_view> segments = dotPathSegments(path);
  if (segments.empty()) {
    return std::nullopt;
  }
  for (size_t index = 0; index < segments.size(); index += 1) {
    const auto value = parseJsonPropertyValue(current, segments[index]);
    if (!value) {
      return std::nullopt;
    }
    if (index + 1 == segments.size()) {
      return formatGraphStatJsonValueAt(*value, 0);
    }
    size_t offset = 0;
    skipJsonWhitespace(*value, offset);
    if (offset >= value->size() || (*value)[offset] != '{') {
      return std::nullopt;
    }
    current = *value;
  }
  return std::nullopt;
}

std::optional<double> parseJsonNumberValue(const std::string &json) {
  size_t offset = 0;
  skipJsonWhitespace(json, offset);
  const size_t start = offset;
  if (offset < json.size() && json[offset] == '-') {
    ++offset;
  }
  while (offset < json.size() &&
         std::isdigit(static_cast<unsigned char>(json[offset])) != 0) {
    ++offset;
  }
  if (offset < json.size() && json[offset] == '.') {
    ++offset;
    while (offset < json.size() &&
           std::isdigit(static_cast<unsigned char>(json[offset])) != 0) {
      ++offset;
    }
  }
  if (offset == start) {
    return std::nullopt;
  }
  try {
    return std::stod(json.substr(start, offset - start));
  } catch (...) {
    return std::nullopt;
  }
}

struct GraphStatId {
  std::string_view scope;
  std::string_view module;
  std::string_view path;
};

std::optional<GraphStatId> parseGraphStatId(std::string_view statId) {
  const size_t scopeEnd = statId.find(':');
  if (scopeEnd == std::string_view::npos) {
    return std::nullopt;
  }
  const size_t moduleEnd = statId.find('.', scopeEnd + 1);
  if (moduleEnd == std::string_view::npos || moduleEnd + 1 >= statId.size()) {
    return std::nullopt;
  }
  return GraphStatId{
      statId.substr(0, scopeEnd),
      statId.substr(scopeEnd + 1, moduleEnd - scopeEnd - 1),
      statId.substr(moduleEnd + 1),
  };
}

bool jsonPlayerIdMatchesIndex(const std::string &playerIdJson, uint32_t playerIndex) {
  const auto splitScreenValue = parseJsonPropertyValue(playerIdJson, "SplitScreen");
  if (!splitScreenValue) {
    return false;
  }
  const auto parsedIndex = parseJsonNumberValue(*splitScreenValue);
  return parsedIndex && static_cast<uint32_t>(*parsedIndex) == playerIndex;
}

std::string graphStatLabel(const GraphStatId &stat) {
  return std::format("{}.{}", stat.module, stat.path);
}

std::vector<UiStatDefinitionCandidate> graphStatDefinitionsFromStatsJson(
    const std::string &statsJson) {
  std::vector<UiStatDefinitionCandidate> definitions;
  const auto frame = parseJsonObjectProperty(statsJson, "frame");
  if (!frame) {
    return definitions;
  }
  const auto modules = parseJsonObjectProperty(*frame, "modules");
  if (!modules) {
    return definitions;
  }

  std::unordered_set<std::string> seenIds;
  for (const std::string &moduleName : parseJsonObjectKeys(*modules)) {
    const auto module = parseJsonObjectProperty(*modules, moduleName);
    if (!module) {
      continue;
    }

    auto appendStatsObject = [&](const std::string &statsObject, const char *scope) {
      std::vector<std::string> paths;
      collectJsonLeafStatPaths(statsObject, "", paths, 8);
      for (const std::string &path : paths) {
        const std::string id = std::format("{}:{}.{}", scope, moduleName, path);
        if (!seenIds.insert(id).second || normalizeUiStatId(id) != id) {
          continue;
        }
        definitions.push_back(UiStatDefinitionCandidate{
            id,
            std::format("{}.{}", moduleName, path),
            moduleName,
            std::string_view{scope} == "player",
            std::string_view{scope} == "team",
            false,
        });
      }
    };

    const std::vector<std::string> playerStats =
        parseJsonObjectArrayProperty(*module, "player_stats");
    if (!playerStats.empty()) {
      if (const auto stats = parseJsonObjectProperty(playerStats.front(), "stats")) {
        appendStatsObject(*stats, "player");
      }
    }
    if (const auto team = parseJsonObjectProperty(*module, "team_zero")) {
      appendStatsObject(*team, "team");
    } else if (const auto teamOne = parseJsonObjectProperty(*module, "team_one")) {
      appendStatsObject(*teamOne, "team");
    }
  }

  return definitions;
}

std::vector<UiStatDefinitionCandidate> graphStatDefinitionsFromReplayFrameJson(
    const std::string &frameJson) {
  std::vector<UiStatDefinitionCandidate> definitions;
  std::unordered_set<std::string> seenIds;

  auto appendStatsObject =
      [&](const std::string &statsObject, const std::string &moduleName, const char *scope) {
        std::vector<std::string> paths;
        collectJsonLeafStatPaths(statsObject, "", paths, 8);
        for (const std::string &path : paths) {
          const std::string id = std::format("{}:{}.{}", scope, moduleName, path);
          if (!seenIds.insert(id).second || normalizeUiStatId(id) != id) {
            continue;
          }
          definitions.push_back(UiStatDefinitionCandidate{
              id,
              std::format("{}.{}", moduleName, path),
              moduleName,
              std::string_view{scope} == "player",
              std::string_view{scope} == "team",
              false,
          });
        }
      };

  if (const auto teamZero = parseJsonObjectProperty(frameJson, "team_zero")) {
    for (const std::string &moduleName : parseJsonObjectKeys(*teamZero)) {
      if (const auto module = parseJsonObjectProperty(*teamZero, moduleName)) {
        appendStatsObject(*module, moduleName, "team");
      }
    }
  } else if (const auto teamOne = parseJsonObjectProperty(frameJson, "team_one")) {
    for (const std::string &moduleName : parseJsonObjectKeys(*teamOne)) {
      if (const auto module = parseJsonObjectProperty(*teamOne, moduleName)) {
        appendStatsObject(*module, moduleName, "team");
      }
    }
  }

  const std::vector<std::string> players =
      parseJsonObjectArrayProperty(frameJson, "players");
  if (!players.empty()) {
    for (const std::string &moduleName : parseJsonObjectKeys(players.front())) {
      if (moduleName == "player_id" || moduleName == "name" || moduleName == "is_team_0") {
        continue;
      }
      if (const auto module = parseJsonObjectProperty(players.front(), moduleName)) {
        appendStatsObject(*module, moduleName, "player");
      }
    }
  }

  return definitions;
}

std::vector<std::string> replayStatsModuleNamesFromFrameJson(const std::string &frameJson) {
  std::vector<std::string> names;
  std::unordered_set<std::string> seen;
  auto appendName = [&](const std::string &moduleName) {
    if (moduleName == "player_id" || moduleName == "name" || moduleName == "is_team_0") {
      return;
    }
    if (seen.insert(moduleName).second) {
      names.push_back(moduleName);
    }
  };

  if (const auto teamZero = parseJsonObjectProperty(frameJson, "team_zero")) {
    for (const std::string &moduleName : parseJsonObjectKeys(*teamZero)) {
      appendName(moduleName);
    }
  }
  if (const auto teamOne = parseJsonObjectProperty(frameJson, "team_one")) {
    for (const std::string &moduleName : parseJsonObjectKeys(*teamOne)) {
      appendName(moduleName);
    }
  }
  for (const std::string &player : parseJsonObjectArrayProperty(frameJson, "players")) {
    for (const std::string &moduleName : parseJsonObjectKeys(player)) {
      appendName(moduleName);
    }
  }
  return names;
}

std::string replayStatsModuleFrameJson(
    const std::string &frameJson,
    const std::string &moduleName) {
  std::optional<std::string> teamZeroModule;
  if (const auto teamZero = parseJsonObjectProperty(frameJson, "team_zero")) {
    teamZeroModule = parseJsonObjectProperty(*teamZero, moduleName);
  }
  std::optional<std::string> teamOneModule;
  if (const auto teamOne = parseJsonObjectProperty(frameJson, "team_one")) {
    teamOneModule = parseJsonObjectProperty(*teamOne, moduleName);
  }

  std::string json = "{";
  json += "\"team_zero\":";
  json += teamZeroModule.value_or("null");
  json += ",\"team_one\":";
  json += teamOneModule.value_or("null");
  json += ",\"player_stats\":[";
  bool firstPlayer = true;
  for (const std::string &player : parseJsonObjectArrayProperty(frameJson, "players")) {
    const auto playerId = parseJsonPropertyValue(player, "player_id");
    const auto playerName = parseJsonPropertyValue(player, "name");
    const auto isTeam0 = parseJsonPropertyValue(player, "is_team_0");
    const auto stats = parseJsonObjectProperty(player, moduleName);
    if (!playerId || !stats) {
      continue;
    }
    if (!firstPlayer) {
      json += ",";
    }
    firstPlayer = false;
    json += "{\"player_id\":";
    json += *playerId;
    if (playerName) {
      json += ",\"name\":";
      json += *playerName;
    }
    if (isTeam0) {
      json += ",\"is_team_0\":";
      json += *isTeam0;
    }
    json += ",\"stats\":";
    json += *stats;
    json += "}";
  }
  json += "]}";
  if (!teamZeroModule && !teamOneModule && firstPlayer) {
    return {};
  }
  return json;
}

} // namespace
