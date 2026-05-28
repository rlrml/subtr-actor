#include "SubtrActorPluginGraphJson.h"

#include <cctype>

namespace subtr_actor_plugin::graph_json {

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

} // namespace subtr_actor_plugin::graph_json
