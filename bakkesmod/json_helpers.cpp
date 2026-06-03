// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
namespace {

void skipJsonWhitespace(const std::string &json, size_t &offset) {
  while (offset < json.size() &&
         std::isspace(static_cast<unsigned char>(json[offset])) != 0) {
    ++offset;
  }
}

std::optional<std::string> parseJsonString(const std::string &json, size_t &offset) {
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

std::optional<std::vector<std::string>> parseJsonStringArrayValue(
    const std::string &json,
    size_t &offset) {
  std::vector<std::string> values;
  skipJsonWhitespace(json, offset);
  if (offset >= json.size() || json[offset] != '[') {
    return std::nullopt;
  }
  ++offset;
  skipJsonWhitespace(json, offset);
  if (offset < json.size() && json[offset] == ']') {
    ++offset;
    return values;
  }

  while (offset < json.size()) {
    auto value = parseJsonString(json, offset);
    if (!value) {
      return std::nullopt;
    }
    values.push_back(*value);
    skipJsonWhitespace(json, offset);
    if (offset < json.size() && json[offset] == ',') {
      ++offset;
      skipJsonWhitespace(json, offset);
      continue;
    }
    if (offset < json.size() && json[offset] == ']') {
      ++offset;
      return values;
    }
    return std::nullopt;
  }
  return std::nullopt;
}

std::vector<std::string> parseJsonStringArray(const std::string &json) {
  size_t offset = 0;
  auto values = parseJsonStringArrayValue(json, offset);
  if (!values) {
    return {};
  }
  skipJsonWhitespace(json, offset);
  return offset == json.size() ? *values : std::vector<std::string>{};
}

bool isAbsoluteWindowsPath(const std::string &path) {
  return path.size() >= 3 && std::isalpha(static_cast<unsigned char>(path[0])) != 0 &&
         path[1] == ':' && (path[2] == '\\' || path[2] == '/');
}

std::optional<std::filesystem::path> existingReplayPathCandidate(
    const std::filesystem::path &path) {
  std::error_code error;
  if (!std::filesystem::exists(path, error)) {
    return std::nullopt;
  }
  const auto canonical = std::filesystem::weakly_canonical(path, error);
  return error ? path : canonical;
}

std::string normalizedReplayPathString(const std::filesystem::path &path) {
  std::error_code error;
  const auto canonical = std::filesystem::weakly_canonical(path, error);
  return (error ? path : canonical).string();
}

std::vector<std::string> parseJsonStringArrayProperty(
    const std::string &json,
    const std::string &propertyName) {
  const std::string needle = std::format("\"{}\"", propertyName);
  size_t offset = json.find(needle);
  if (offset == std::string::npos) {
    return {};
  }
  offset += needle.size();
  skipJsonWhitespace(json, offset);
  if (offset >= json.size() || json[offset] != ':') {
    return {};
  }
  ++offset;
  auto values = parseJsonStringArrayValue(json, offset);
  return values.value_or(std::vector<std::string>{});
}

bool skipJsonValue(const std::string &json, size_t &offset) {
  skipJsonWhitespace(json, offset);
  if (offset >= json.size()) {
    return false;
  }

  if (json[offset] == '"') {
    return parseJsonString(json, offset).has_value();
  }

  if (json[offset] == '{') {
    ++offset;
    skipJsonWhitespace(json, offset);
    if (offset < json.size() && json[offset] == '}') {
      ++offset;
      return true;
    }
    while (offset < json.size()) {
      if (!parseJsonString(json, offset)) {
        return false;
      }
      skipJsonWhitespace(json, offset);
      if (offset >= json.size() || json[offset] != ':') {
        return false;
      }
      ++offset;
      if (!skipJsonValue(json, offset)) {
        return false;
      }
      skipJsonWhitespace(json, offset);
      if (offset < json.size() && json[offset] == ',') {
        ++offset;
        skipJsonWhitespace(json, offset);
        continue;
      }
      if (offset < json.size() && json[offset] == '}') {
        ++offset;
        return true;
      }
      return false;
    }
    return false;
  }

  if (json[offset] == '[') {
    ++offset;
    skipJsonWhitespace(json, offset);
    if (offset < json.size() && json[offset] == ']') {
      ++offset;
      return true;
    }
    while (offset < json.size()) {
      if (!skipJsonValue(json, offset)) {
        return false;
      }
      skipJsonWhitespace(json, offset);
      if (offset < json.size() && json[offset] == ',') {
        ++offset;
        skipJsonWhitespace(json, offset);
        continue;
      }
      if (offset < json.size() && json[offset] == ']') {
        ++offset;
        return true;
      }
      return false;
    }
    return false;
  }

  if (json.compare(offset, 4, "true") == 0) {
    offset += 4;
    return true;
  }
  if (json.compare(offset, 5, "false") == 0) {
    offset += 5;
    return true;
  }
  if (json.compare(offset, 4, "null") == 0) {
    offset += 4;
    return true;
  }

  const size_t start = offset;
  if (json[offset] == '-') {
    ++offset;
  }
  const size_t integerStart = offset;
  while (offset < json.size() &&
         std::isdigit(static_cast<unsigned char>(json[offset])) != 0) {
    ++offset;
  }
  if (offset == integerStart) {
    return false;
  }
  if (offset < json.size() && json[offset] == '.') {
    ++offset;
    const size_t fractionStart = offset;
    while (offset < json.size() &&
           std::isdigit(static_cast<unsigned char>(json[offset])) != 0) {
      ++offset;
    }
    if (offset == fractionStart) {
      return false;
    }
  }
  if (offset < json.size() && (json[offset] == 'e' || json[offset] == 'E')) {
    ++offset;
    if (offset < json.size() && (json[offset] == '+' || json[offset] == '-')) {
      ++offset;
    }
    const size_t exponentStart = offset;
    while (offset < json.size() &&
           std::isdigit(static_cast<unsigned char>(json[offset])) != 0) {
      ++offset;
    }
    if (offset == exponentStart) {
      return false;
    }
  }
  return offset > start;
}

std::vector<std::string> parseJsonObjectKeys(const std::string &json) {
  std::vector<std::string> keys;
  size_t offset = 0;
  skipJsonWhitespace(json, offset);
  if (offset >= json.size() || json[offset] != '{') {
    return {};
  }
  ++offset;
  skipJsonWhitespace(json, offset);
  if (offset < json.size() && json[offset] == '}') {
    ++offset;
    skipJsonWhitespace(json, offset);
    return offset == json.size() ? keys : std::vector<std::string>{};
  }

  while (offset < json.size()) {
    auto key = parseJsonString(json, offset);
    if (!key) {
      return {};
    }
    skipJsonWhitespace(json, offset);
    if (offset >= json.size() || json[offset] != ':') {
      return {};
    }
    ++offset;
    if (!skipJsonValue(json, offset)) {
      return {};
    }
    keys.push_back(*key);
    skipJsonWhitespace(json, offset);
    if (offset < json.size() && json[offset] == ',') {
      ++offset;
      skipJsonWhitespace(json, offset);
      continue;
    }
    if (offset < json.size() && json[offset] == '}') {
      ++offset;
      skipJsonWhitespace(json, offset);
      return offset == json.size() ? keys : std::vector<std::string>{};
    }
    return {};
  }
  return {};
}

std::optional<size_t> parseJsonArrayPropertyElementCount(
    const std::string &json,
    const std::string &propertyName) {
  const std::string needle = std::format("\"{}\"", propertyName);
  size_t offset = json.find(needle);
  if (offset == std::string::npos) {
    return std::nullopt;
  }
  offset += needle.size();
  skipJsonWhitespace(json, offset);
  if (offset >= json.size() || json[offset] != ':') {
    return std::nullopt;
  }
  ++offset;
  skipJsonWhitespace(json, offset);
  if (offset >= json.size() || json[offset] != '[') {
    return std::nullopt;
  }
  ++offset;
  skipJsonWhitespace(json, offset);
  if (offset < json.size() && json[offset] == ']') {
    return 0;
  }

  size_t count = 0;
  while (offset < json.size()) {
    if (!skipJsonValue(json, offset)) {
      return std::nullopt;
    }
    count += 1;
    skipJsonWhitespace(json, offset);
    if (offset < json.size() && json[offset] == ',') {
      ++offset;
      skipJsonWhitespace(json, offset);
      continue;
    }
    if (offset < json.size() && json[offset] == ']') {
      return count;
    }
    return std::nullopt;
  }
  return std::nullopt;
}

std::optional<size_t> parseJsonArrayElementCountAt(const std::string &json, size_t offset) {
  skipJsonWhitespace(json, offset);
  if (offset >= json.size() || json[offset] != '[') {
    return std::nullopt;
  }
  ++offset;
  skipJsonWhitespace(json, offset);
  if (offset < json.size() && json[offset] == ']') {
    return 0;
  }

  size_t count = 0;
  while (offset < json.size()) {
    if (!skipJsonValue(json, offset)) {
      return std::nullopt;
    }
    count += 1;
    skipJsonWhitespace(json, offset);
    if (offset < json.size() && json[offset] == ',') {
      ++offset;
      skipJsonWhitespace(json, offset);
      continue;
    }
    if (offset < json.size() && json[offset] == ']') {
      return count;
    }
    return std::nullopt;
  }
  return std::nullopt;
}

std::string summarizeJsonValueAt(const std::string &json, size_t offset) {
  skipJsonWhitespace(json, offset);
  if (offset >= json.size()) {
    return "--";
  }

  if (json[offset] == '"') {
    auto value = parseJsonString(json, offset);
    if (!value) {
      return "--";
    }
    return value->size() > 96 ? value->substr(0, 93) + "..." : *value;
  }

  if (json[offset] == '[') {
    const auto count = parseJsonArrayElementCountAt(json, offset);
    return count ? std::format("{} item{}", *count, *count == 1 ? "" : "s") : "array";
  }

  if (json[offset] == '{') {
    return "object";
  }

  const size_t start = offset;
  if (!skipJsonValue(json, offset)) {
    return "--";
  }
  return json.substr(start, offset - start);
}

void collectJsonFieldSummaries(
    const std::string &json,
    std::string_view prefix,
    std::vector<JsonFieldSummary> &out,
    size_t maxFields,
    int maxDepth) {
  if (out.size() >= maxFields || maxDepth < 0) {
    return;
  }

  size_t offset = 0;
  skipJsonWhitespace(json, offset);
  if (offset >= json.size() || json[offset] != '{') {
    return;
  }
  ++offset;
  skipJsonWhitespace(json, offset);

  while (offset < json.size() && out.size() < maxFields) {
    if (json[offset] == '}') {
      return;
    }

    auto key = parseJsonString(json, offset);
    if (!key) {
      return;
    }
    skipJsonWhitespace(json, offset);
    if (offset >= json.size() || json[offset] != ':') {
      return;
    }
    ++offset;
    skipJsonWhitespace(json, offset);

    const std::string label =
        prefix.empty() ? *key : std::format("{}.{}", prefix, *key);
    const size_t valueStart = offset;
    if (offset < json.size() && json[offset] == '{' && maxDepth > 0) {
      size_t end = offset;
      if (!skipJsonValue(json, end)) {
        return;
      }
      collectJsonFieldSummaries(
          json.substr(valueStart, end - valueStart),
          label,
          out,
          maxFields,
          maxDepth - 1);
      offset = end;
    } else {
      out.push_back(JsonFieldSummary{label, summarizeJsonValueAt(json, offset)});
      if (!skipJsonValue(json, offset)) {
        return;
      }
    }

    skipJsonWhitespace(json, offset);
    if (offset < json.size() && json[offset] == ',') {
      ++offset;
      skipJsonWhitespace(json, offset);
      continue;
    }
    if (offset < json.size() && json[offset] == '}') {
      return;
    }
    return;
  }
}

void collectJsonLeafStatPaths(
    const std::string &json,
    std::string_view prefix,
    std::vector<std::string> &out,
    int maxDepth) {
  if (maxDepth < 0) {
    return;
  }

  size_t offset = 0;
  skipJsonWhitespace(json, offset);
  if (offset >= json.size() || json[offset] != '{') {
    return;
  }
  ++offset;
  skipJsonWhitespace(json, offset);

  while (offset < json.size()) {
    if (json[offset] == '}') {
      return;
    }

    auto key = parseJsonString(json, offset);
    if (!key) {
      return;
    }
    skipJsonWhitespace(json, offset);
    if (offset >= json.size() || json[offset] != ':') {
      return;
    }
    ++offset;
    skipJsonWhitespace(json, offset);

    const std::string label =
        prefix.empty() ? *key : std::format("{}.{}", prefix, *key);
    const size_t valueStart = offset;
    if (offset < json.size() && json[offset] == '{') {
      size_t end = offset;
      if (!skipJsonValue(json, end)) {
        return;
      }
      collectJsonLeafStatPaths(json.substr(valueStart, end - valueStart), label, out, maxDepth - 1);
      offset = end;
    } else {
      out.push_back(label);
      if (!skipJsonValue(json, offset)) {
        return;
      }
    }

    skipJsonWhitespace(json, offset);
    if (offset < json.size() && json[offset] == ',') {
      ++offset;
      skipJsonWhitespace(json, offset);
      continue;
    }
    if (offset < json.size() && json[offset] == '}') {
      return;
    }
    return;
  }
}

std::string escapeJsonString(std::string_view value) {
  std::string escaped;
  escaped.reserve(value.size() + 8);
  for (const char ch : value) {
    switch (ch) {
    case '"':
      escaped += "\\\"";
      break;
    case '\\':
      escaped += "\\\\";
      break;
    case '\b':
      escaped += "\\b";
      break;
    case '\f':
      escaped += "\\f";
      break;
    case '\n':
      escaped += "\\n";
      break;
    case '\r':
      escaped += "\\r";
      break;
    case '\t':
      escaped += "\\t";
      break;
    default:
      escaped.push_back(ch);
      break;
    }
  }
  return escaped;
}

std::string urlEncode(std::string_view value) {
  constexpr char HEX[] = "0123456789ABCDEF";
  std::string encoded;
  encoded.reserve(value.size());
  for (const unsigned char byte : value) {
    const bool unreserved =
        (byte >= 'A' && byte <= 'Z') || (byte >= 'a' && byte <= 'z') ||
        (byte >= '0' && byte <= '9') || byte == '-' || byte == '_' ||
        byte == '.' || byte == '~';
    if (unreserved) {
      encoded.push_back(static_cast<char>(byte));
      continue;
    }
    encoded.push_back('%');
    encoded.push_back(HEX[byte >> 4]);
    encoded.push_back(HEX[byte & 0x0F]);
  }
  return encoded;
}

int urlHexValue(char ch) {
  if (ch >= '0' && ch <= '9') {
    return ch - '0';
  }
  if (ch >= 'A' && ch <= 'F') {
    return ch - 'A' + 10;
  }
  if (ch >= 'a' && ch <= 'f') {
    return ch - 'a' + 10;
  }
  return -1;
}

std::optional<std::string> urlDecode(std::string_view value) {
  std::string decoded;
  decoded.reserve(value.size());
  for (size_t index = 0; index < value.size(); index += 1) {
    const char ch = value[index];
    if (ch == '+') {
      decoded.push_back(' ');
      continue;
    }
    if (ch != '%') {
      decoded.push_back(ch);
      continue;
    }
    if (index + 2 >= value.size()) {
      return std::nullopt;
    }
    const int high = urlHexValue(value[index + 1]);
    const int low = urlHexValue(value[index + 2]);
    if (high < 0 || low < 0) {
      return std::nullopt;
    }
    decoded.push_back(static_cast<char>((high << 4) | low));
    index += 2;
  }
  return decoded;
}

std::optional<std::string> statsPlayerCfgValueFromClipboard(std::string_view clipboardText) {
  const size_t firstByte = clipboardText.find_first_not_of(" \t\r\n");
  if (firstByte == std::string_view::npos) {
    return std::nullopt;
  }
  if (clipboardText[firstByte] == '{') {
    return std::string{clipboardText.substr(firstByte)};
  }

  const size_t cfgOffset = clipboardText.find("cfg=");
  if (cfgOffset == std::string_view::npos) {
    return std::nullopt;
  }
  const size_t valueStart = cfgOffset + 4;
  size_t valueEnd = clipboardText.find_first_of("&# \t\r\n", valueStart);
  if (valueEnd == std::string_view::npos) {
    valueEnd = clipboardText.size();
  }
  std::optional<std::string> decoded =
      urlDecode(clipboardText.substr(valueStart, valueEnd - valueStart));
  if (!decoded) {
    return std::nullopt;
  }
  const size_t decodedFirstByte = decoded->find_first_not_of(" \t\r\n");
  if (decodedFirstByte == std::string::npos) {
    return std::nullopt;
  }
  return decoded->substr(decodedFirstByte);
}

std::string clippedDisplayText(std::string value, size_t maxBytes = 24000) {
  if (value.size() <= maxBytes) {
    return value;
  }
  const size_t originalSize = value.size();
  value.resize(maxBytes);
  value += std::format(
      "\n\n... truncated {} of {} bytes ...",
      originalSize - maxBytes,
      originalSize);
  return value;
}

std::string formatGraphStatNumber(double value) {
  if (!std::isfinite(value)) {
    return "--";
  }
  if (std::trunc(value) == value) {
    return std::format("{:.0f}", value);
  }

  double rounded = std::round(value * 1000.0) / 1000.0;
  if (rounded == 0.0) {
    rounded = 0.0;
  }

  std::string formatted = std::format("{:.3f}", rounded);
  while (!formatted.empty() && formatted.back() == '0') {
    formatted.pop_back();
  }
  if (!formatted.empty() && formatted.back() == '.') {
    formatted.pop_back();
  }
  return formatted.empty() ? "--" : formatted;
}

std::string formatByteSize(size_t bytes) {
  if (bytes == 0) {
    return "--";
  }

  constexpr std::array<const char *, 4> units{{"B", "KB", "MB", "GB"}};
  double value = static_cast<double>(bytes);
  size_t unitIndex = 0;
  while (value >= 1024.0 && unitIndex + 1 < units.size()) {
    value /= 1024.0;
    unitIndex += 1;
  }
  if (unitIndex == 0) {
    return std::format("{} {}", bytes, units[unitIndex]);
  }
  return value >= 10.0 ? std::format("{:.1f} {}", value, units[unitIndex])
                       : std::format("{:.2f} {}", value, units[unitIndex]);
}

} // namespace
