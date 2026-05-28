#pragma once

#include <cstddef>
#include <optional>
#include <string>
#include <vector>

namespace subtr_actor_plugin::graph_json {

std::vector<std::string> parseJsonStringArray(const std::string &json);
std::vector<std::string> parseJsonStringArrayProperty(
    const std::string &json,
    const std::string &propertyName);
std::vector<std::string> parseJsonObjectKeys(const std::string &json);
std::optional<size_t> parseJsonArrayPropertyElementCount(
    const std::string &json,
    const std::string &propertyName);

} // namespace subtr_actor_plugin::graph_json
