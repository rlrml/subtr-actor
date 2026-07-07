// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
std::string SubtrActorPlugin::readNamedJsonBuffer(
    NamedJsonLen len,
    WriteNamedJson write,
    const std::string &name) const {
  if (!engine || !len || !write) {
    return {};
  }

  const size_t byteCount = len(engine, name.c_str());
  if (byteCount == 0) {
    return {};
  }

  std::string buffer(byteCount, '\0');
  const size_t written =
      write(engine, name.c_str(), reinterpret_cast<uint8_t *>(buffer.data()), buffer.size());
  buffer.resize(written);
  return buffer;
}

namespace {

std::string jsonEscaped(std::string_view value) {
  std::string escaped;
  escaped.reserve(value.size() + 8);
  for (const unsigned char character : value) {
    switch (character) {
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
      if (character < 0x20) {
        escaped += std::format("\\u{:04x}", static_cast<unsigned int>(character));
      } else {
        escaped.push_back(static_cast<char>(character));
      }
      break;
    }
  }
  return escaped;
}

std::string jsonString(std::string_view value) {
  return std::format("\"{}\"", jsonEscaped(value));
}

std::string unrealString(UnrealStringWrapper value) {
  return static_cast<bool>(value) ? value.ToString() : std::string{};
}

std::string productSlotLabel(ProductSlotWrapper slot) {
  return static_cast<bool>(slot) ? unrealString(slot.GetLabel()) : std::string{};
}

std::string productSlotPluralLabel(ProductSlotWrapper slot) {
  return static_cast<bool>(slot) ? unrealString(slot.GetPluralLabel()) : std::string{};
}

int productSlotIndex(ProductSlotWrapper slot) {
  return static_cast<bool>(slot) ? slot.GetSlotIndex() : -1;
}

bool productSlotIsBody(ProductSlotWrapper slot) {
  const std::string label = productSlotLabel(slot);
  const std::string pluralLabel = productSlotPluralLabel(slot);
  return label == "Body" || pluralLabel == "Bodies";
}

bool paramsContain(const std::vector<std::string> &params, std::string_view needle) {
  return std::find(params.begin(), params.end(), needle) != params.end();
}

} // namespace

void SubtrActorPlugin::dumpGraphJson(std::vector<std::string> params) {
  if (!loaded || !engine) {
    cvarManager->log("subtr-actor: graph dump requested before engine was loaded");
    return;
  }

  const bool shouldFinish =
      std::find_if(params.begin(), params.end(), [](const std::string &param) {
        return param == "finish" || param == "finalize";
      }) != params.end();
  if (shouldFinish) {
    if (!engineFinish) {
      cvarManager->log("subtr-actor: graph dump requested finish but finish ABI is unavailable");
      return;
    }
    const int32_t finishResult = engineFinish(engine);
    if (finishResult != 0) {
      cvarManager->log(
          std::format("subtr-actor: graph finish failed before dump: {}", finishResult));
      return;
    }
    drainPendingEvents();
  }

  const std::filesystem::path outputDirectory =
      gameWrapper->GetDataFolder() / "subtr-actor";
  std::error_code error;
  std::filesystem::create_directories(outputDirectory, error);
  if (error) {
    cvarManager->log(
        std::format("subtr-actor: failed to create graph dump directory: {}", error.message()));
    return;
  }

  const std::string eventsJson = readJsonBuffer(eventsJsonLen, writeEventsJson);
  const std::string frameJson = readJsonBuffer(frameJsonLen, writeFrameJson);
  const std::string timelineJson = readJsonBuffer(timelineJsonLen, writeTimelineJson);
  const std::string statsJson = readJsonBuffer(statsJsonLen, writeStatsJson);
  const std::string analysisNodesJson =
      readNamedJsonBuffer(graphOutputJsonLen, writeGraphOutputJson, "analysis_nodes");
  const std::string eventHistoryJson =
      readNamedJsonBuffer(graphOutputJsonLen, writeGraphOutputJson, "event_history");
  const std::string graphInfoJson = readJsonBuffer(graphInfoJsonLen, writeGraphInfoJson);
  const std::filesystem::path eventsPath = outputDirectory / "graph-events.json";
  const std::filesystem::path framePath = outputDirectory / "graph-frame.json";
  const std::filesystem::path timelinePath = outputDirectory / "graph-timeline.json";
  const std::filesystem::path statsPath = outputDirectory / "graph-stats.json";
  const std::filesystem::path analysisNodesPath = outputDirectory / "graph-analysis-nodes.json";
  const std::filesystem::path eventHistoryPath = outputDirectory / "graph-event-history.json";
  const std::filesystem::path graphInfoPath = outputDirectory / "graph-info.json";

  std::ofstream eventsFile(eventsPath, std::ios::binary);
  eventsFile.write(eventsJson.data(), static_cast<std::streamsize>(eventsJson.size()));
  std::ofstream frameFile(framePath, std::ios::binary);
  frameFile.write(frameJson.data(), static_cast<std::streamsize>(frameJson.size()));
  std::ofstream timelineFile(timelinePath, std::ios::binary);
  timelineFile.write(timelineJson.data(), static_cast<std::streamsize>(timelineJson.size()));
  std::ofstream statsFile(statsPath, std::ios::binary);
  statsFile.write(statsJson.data(), static_cast<std::streamsize>(statsJson.size()));
  std::ofstream analysisNodesFile(analysisNodesPath, std::ios::binary);
  analysisNodesFile.write(
      analysisNodesJson.data(), static_cast<std::streamsize>(analysisNodesJson.size()));
  std::ofstream eventHistoryFile(eventHistoryPath, std::ios::binary);
  eventHistoryFile.write(
      eventHistoryJson.data(), static_cast<std::streamsize>(eventHistoryJson.size()));
  std::ofstream graphInfoFile(graphInfoPath, std::ios::binary);
  graphInfoFile.write(graphInfoJson.data(), static_cast<std::streamsize>(graphInfoJson.size()));

  if (!eventsFile || !frameFile || !timelineFile || !statsFile || !analysisNodesFile ||
      !eventHistoryFile || !graphInfoFile) {
    cvarManager->log("subtr-actor: failed to write graph JSON snapshots");
    return;
  }

  cvarManager->log(std::format(
      "subtr-actor: wrote graph JSON snapshots{}: {} ({} bytes), {} ({} bytes), "
      "{} ({} bytes), {} ({} bytes), {} ({} bytes), {} ({} bytes), {} ({} bytes)",
      shouldFinish ? " after finish" : "",
      eventsPath.string(),
      eventsJson.size(),
      framePath.string(),
      frameJson.size(),
      timelinePath.string(),
      timelineJson.size(),
      statsPath.string(),
      statsJson.size(),
      analysisNodesPath.string(),
      analysisNodesJson.size(),
      eventHistoryPath.string(),
      eventHistoryJson.size(),
      graphInfoPath.string(),
      graphInfoJson.size()));
}

void SubtrActorPlugin::dumpStatsModuleJson(std::vector<std::string> params) {
  if (!loaded || !engine) {
    cvarManager->log("subtr-actor: stats module dump requested before engine was loaded");
    return;
  }
  if (params.size() < 2) {
    cvarManager->log("subtr-actor: usage: subtr_actor_dump_stats_module <module_name> [finish]");
    return;
  }

  const std::string moduleName = params[1];
  const bool shouldFinish =
      std::find_if(params.begin() + 2, params.end(), [](const std::string &param) {
        return param == "finish" || param == "finalize";
      }) != params.end();
  if (shouldFinish) {
    if (!engineFinish) {
      cvarManager->log("subtr-actor: stats module dump requested finish but finish ABI is unavailable");
      return;
    }
    const int32_t finishResult = engineFinish(engine);
    if (finishResult != 0) {
      cvarManager->log(
          std::format("subtr-actor: graph finish failed before stats module dump: {}", finishResult));
      return;
    }
    drainPendingEvents();
  }

  const std::string moduleJson =
      readNamedJsonBuffer(statsModuleJsonLen, writeStatsModuleJson, moduleName);
  if (moduleJson.empty()) {
    cvarManager->log(std::format(
        "subtr-actor: stats module '{}' was unavailable or produced empty JSON", moduleName));
    return;
  }

  const std::filesystem::path outputDirectory =
      gameWrapper->GetDataFolder() / "subtr-actor";
  std::error_code error;
  std::filesystem::create_directories(outputDirectory, error);
  if (error) {
    cvarManager->log(std::format(
        "subtr-actor: failed to create stats module dump directory: {}", error.message()));
    return;
  }

  const std::filesystem::path modulePath =
      outputDirectory / std::format("graph-module-{}.json", safeModuleFileStem(moduleName));
  std::ofstream moduleFile(modulePath, std::ios::binary);
  moduleFile.write(moduleJson.data(), static_cast<std::streamsize>(moduleJson.size()));
  if (!moduleFile) {
    cvarManager->log("subtr-actor: failed to write stats module JSON snapshot");
    return;
  }

  cvarManager->log(std::format(
      "subtr-actor: wrote stats module '{}' JSON{}: {} ({} bytes)",
      moduleName,
      shouldFinish ? " after finish" : "",
      modulePath.string(),
      moduleJson.size()));
}

void SubtrActorPlugin::dumpStatsModuleFrameJson(std::vector<std::string> params) {
  if (!loaded || !engine) {
    cvarManager->log("subtr-actor: stats module frame dump requested before engine was loaded");
    return;
  }
  if (params.size() < 2) {
    cvarManager->log(
        "subtr-actor: usage: subtr_actor_dump_stats_module_frame <module_name> [finish]");
    return;
  }

  const std::string moduleName = params[1];
  const bool shouldFinish =
      std::find_if(params.begin() + 2, params.end(), [](const std::string &param) {
        return param == "finish" || param == "finalize";
      }) != params.end();
  if (shouldFinish) {
    if (!engineFinish) {
      cvarManager->log(
          "subtr-actor: stats module frame dump requested finish but finish ABI is unavailable");
      return;
    }
    const int32_t finishResult = engineFinish(engine);
    if (finishResult != 0) {
      cvarManager->log(std::format(
          "subtr-actor: graph finish failed before stats module frame dump: {}",
          finishResult));
      return;
    }
    drainPendingEvents();
  }

  const std::string moduleJson =
      readNamedJsonBuffer(statsModuleFrameJsonLen, writeStatsModuleFrameJson, moduleName);
  if (moduleJson.empty()) {
    cvarManager->log(std::format(
        "subtr-actor: stats module '{}' frame was unavailable or produced empty JSON",
        moduleName));
    return;
  }

  const std::filesystem::path outputDirectory =
      gameWrapper->GetDataFolder() / "subtr-actor";
  std::error_code error;
  std::filesystem::create_directories(outputDirectory, error);
  if (error) {
    cvarManager->log(std::format(
        "subtr-actor: failed to create stats module frame dump directory: {}",
        error.message()));
    return;
  }

  const std::filesystem::path modulePath =
      outputDirectory / std::format("graph-module-frame-{}.json", safeModuleFileStem(moduleName));
  std::ofstream moduleFile(modulePath, std::ios::binary);
  moduleFile.write(moduleJson.data(), static_cast<std::streamsize>(moduleJson.size()));
  if (!moduleFile) {
    cvarManager->log("subtr-actor: failed to write stats module frame JSON snapshot");
    return;
  }

  cvarManager->log(std::format(
      "subtr-actor: wrote stats module '{}' frame JSON{}: {} ({} bytes)",
      moduleName,
      shouldFinish ? " after finish" : "",
      modulePath.string(),
      moduleJson.size()));
}

void SubtrActorPlugin::dumpStatsModuleConfigJson(std::vector<std::string> params) {
  if (!loaded || !engine) {
    cvarManager->log("subtr-actor: stats module config dump requested before engine was loaded");
    return;
  }
  if (params.size() < 2) {
    cvarManager->log(
        "subtr-actor: usage: subtr_actor_dump_stats_module_config <module_name> [finish]");
    return;
  }

  const std::string moduleName = params[1];
  const bool shouldFinish =
      std::find_if(params.begin() + 2, params.end(), [](const std::string &param) {
        return param == "finish" || param == "finalize";
      }) != params.end();
  if (shouldFinish) {
    if (!engineFinish) {
      cvarManager->log(
          "subtr-actor: stats module config dump requested finish but finish ABI is unavailable");
      return;
    }
    const int32_t finishResult = engineFinish(engine);
    if (finishResult != 0) {
      cvarManager->log(std::format(
          "subtr-actor: graph finish failed before stats module config dump: {}",
          finishResult));
      return;
    }
    drainPendingEvents();
  }

  const std::string moduleJson =
      readNamedJsonBuffer(statsModuleConfigJsonLen, writeStatsModuleConfigJson, moduleName);
  if (moduleJson.empty()) {
    cvarManager->log(std::format(
        "subtr-actor: stats module '{}' config was unavailable or produced empty JSON",
        moduleName));
    return;
  }

  const std::filesystem::path outputDirectory =
      gameWrapper->GetDataFolder() / "subtr-actor";
  std::error_code error;
  std::filesystem::create_directories(outputDirectory, error);
  if (error) {
    cvarManager->log(std::format(
        "subtr-actor: failed to create stats module config dump directory: {}",
        error.message()));
    return;
  }

  const std::filesystem::path modulePath =
      outputDirectory / std::format("graph-module-config-{}.json", safeModuleFileStem(moduleName));
  std::ofstream moduleFile(modulePath, std::ios::binary);
  moduleFile.write(moduleJson.data(), static_cast<std::streamsize>(moduleJson.size()));
  if (!moduleFile) {
    cvarManager->log("subtr-actor: failed to write stats module config JSON snapshot");
    return;
  }

  cvarManager->log(std::format(
      "subtr-actor: wrote stats module '{}' config JSON{}: {} ({} bytes)",
      moduleName,
      shouldFinish ? " after finish" : "",
      modulePath.string(),
      moduleJson.size()));
}

void SubtrActorPlugin::dumpGraphOutputJson(std::vector<std::string> params) {
  if (!loaded || !engine) {
    cvarManager->log("subtr-actor: graph output dump requested before engine was loaded");
    return;
  }
  if (params.size() < 2) {
    cvarManager->log(std::format("subtr-actor: usage: {}", GRAPH_OUTPUT_USAGE));
    return;
  }

  const std::string outputName = params[1];
  const bool shouldFinish =
      std::find_if(params.begin() + 2, params.end(), [](const std::string &param) {
        return param == "finish" || param == "finalize";
      }) != params.end();
  if (shouldFinish) {
    if (!engineFinish) {
      cvarManager->log("subtr-actor: graph output dump requested finish but finish ABI is unavailable");
      return;
    }
    const int32_t finishResult = engineFinish(engine);
    if (finishResult != 0) {
      cvarManager->log(
          std::format("subtr-actor: graph finish failed before output dump: {}", finishResult));
      return;
    }
    drainPendingEvents();
  }

  const std::string outputJson =
      readNamedJsonBuffer(graphOutputJsonLen, writeGraphOutputJson, outputName);
  if (outputJson.empty()) {
    cvarManager->log(std::format(
        "subtr-actor: graph output '{}' was unavailable or produced empty JSON", outputName));
    return;
  }

  const std::filesystem::path outputDirectory =
      gameWrapper->GetDataFolder() / "subtr-actor";
  std::error_code error;
  std::filesystem::create_directories(outputDirectory, error);
  if (error) {
    cvarManager->log(std::format(
        "subtr-actor: failed to create graph output dump directory: {}", error.message()));
    return;
  }

  const std::filesystem::path outputPath =
      outputDirectory / std::format("graph-output-{}.json", safeModuleFileStem(outputName));
  std::ofstream outputFile(outputPath, std::ios::binary);
  outputFile.write(outputJson.data(), static_cast<std::streamsize>(outputJson.size()));
  if (!outputFile) {
    cvarManager->log("subtr-actor: failed to write graph output JSON snapshot");
    return;
  }

  cvarManager->log(std::format(
      "subtr-actor: wrote graph output '{}' JSON{}: {} ({} bytes)",
      outputName,
      shouldFinish ? " after finish" : "",
      outputPath.string(),
      outputJson.size()));
}

void SubtrActorPlugin::dumpAnalysisNodeJson(std::vector<std::string> params) {
  if (!loaded || !engine) {
    cvarManager->log("subtr-actor: analysis node dump requested before engine was loaded");
    return;
  }
  if (params.size() < 2) {
    cvarManager->log(
        "subtr-actor: usage: subtr_actor_dump_analysis_node <node_name> [finish]");
    return;
  }

  const std::string nodeName = params[1];
  const bool shouldFinish =
      std::find_if(params.begin() + 2, params.end(), [](const std::string &param) {
        return param == "finish" || param == "finalize";
      }) != params.end();
  if (shouldFinish) {
    if (!engineFinish) {
      cvarManager->log(
          "subtr-actor: analysis node dump requested finish but finish ABI is unavailable");
      return;
    }
    const int32_t finishResult = engineFinish(engine);
    if (finishResult != 0) {
      cvarManager->log(std::format(
          "subtr-actor: graph finish failed before analysis node dump: {}",
          finishResult));
      return;
    }
    drainPendingEvents();
  }

  const std::string nodeJson =
      readNamedJsonBuffer(analysisNodeJsonLen, writeAnalysisNodeJson, nodeName);
  if (nodeJson.empty()) {
    cvarManager->log(std::format(
        "subtr-actor: analysis node '{}' was unavailable or produced empty JSON",
        nodeName));
    return;
  }

  const std::filesystem::path outputDirectory =
      gameWrapper->GetDataFolder() / "subtr-actor";
  std::error_code error;
  std::filesystem::create_directories(outputDirectory, error);
  if (error) {
    cvarManager->log(std::format(
        "subtr-actor: failed to create analysis node dump directory: {}",
        error.message()));
    return;
  }

  const std::filesystem::path nodePath =
      outputDirectory / std::format("graph-node-{}.json", safeModuleFileStem(nodeName));
  std::ofstream nodeFile(nodePath, std::ios::binary);
  nodeFile.write(nodeJson.data(), static_cast<std::streamsize>(nodeJson.size()));
  if (!nodeFile) {
    cvarManager->log("subtr-actor: failed to write analysis node JSON snapshot");
    return;
  }

  cvarManager->log(std::format(
      "subtr-actor: wrote analysis node '{}' JSON{}: {} ({} bytes)",
      nodeName,
      shouldFinish ? " after finish" : "",
      nodePath.string(),
      nodeJson.size()));
}

void SubtrActorPlugin::dumpProductsJson(std::vector<std::string> params) {
  const bool includeAllSlots = paramsContain(params, "all");
  const std::filesystem::path outputDirectory =
      gameWrapper->GetDataFolder() / "subtr-actor";
  std::error_code error;
  std::filesystem::create_directories(outputDirectory, error);
  if (error) {
    cvarManager->log(std::format(
        "subtr-actor: failed to create product dump directory: {}", error.message()));
    return;
  }

  ItemsWrapper items = gameWrapper->GetItemsWrapper();
  if (!static_cast<bool>(items)) {
    cvarManager->log("subtr-actor: product dump requested before BakkesMod items are available");
    return;
  }

  ArrayWrapper<ProductWrapper> products = items.GetAllProducts();
  if (products.IsNull()) {
    cvarManager->log("subtr-actor: BakkesMod returned a null product list");
    return;
  }

  const std::filesystem::path outputPath =
      outputDirectory / (includeAllSlots ? "products-all.json" : "body-products.json");
  std::ofstream file(outputPath, std::ios::binary);
  if (!file) {
    cvarManager->log(
        std::format("subtr-actor: failed to open product dump file {}", outputPath.string()));
    return;
  }

  file << "{\n";
  file << "  \"source\": \"BakkesMod ItemsWrapper::GetAllProducts\",\n";
  file << "  \"filter\": " << jsonString(includeAllSlots ? "all" : "body") << ",\n";
  file << "  \"products\": [\n";

  int totalCount = 0;
  int writtenCount = 0;
  const int productCount = products.Count();
  for (int index = 0; index < productCount; ++index) {
    ProductWrapper product = products.Get(index);
    if (!static_cast<bool>(product) || product.IsNull()) {
      continue;
    }
    ++totalCount;

    ProductSlotWrapper slot = product.GetSlot();
    if (!includeAllSlots && !productSlotIsBody(slot)) {
      continue;
    }

    if (writtenCount > 0) {
      file << ",\n";
    }
    ++writtenCount;

    file << "    {\n";
    file << "      \"id\": " << product.GetID() << ",\n";
    file << "      \"label\": " << jsonString(unrealString(product.GetLabel())) << ",\n";
    file << "      \"ascii_label\": " << jsonString(unrealString(product.GetAsciiLabel()))
         << ",\n";
    file << "      \"long_label\": " << jsonString(unrealString(product.GetLongLabel()))
         << ",\n";
    file << "      \"sort_label\": " << jsonString(unrealString(product.GetSortLabel()))
         << ",\n";
    file << "      \"display_label_slot\": "
         << jsonString(unrealString(product.GetDisplayLabelSlot())) << ",\n";
    file << "      \"slot_index\": " << productSlotIndex(slot) << ",\n";
    file << "      \"slot_label\": " << jsonString(productSlotLabel(slot)) << ",\n";
    file << "      \"slot_plural_label\": " << jsonString(productSlotPluralLabel(slot)) << ",\n";
    file << "      \"quality\": " << static_cast<int>(product.GetQuality()) << ",\n";
    file << "      \"paintable\": " << (product.IsPaintable() ? "true" : "false") << ",\n";
    file << "      \"licensed\": " << (product.IsLicensed() ? "true" : "false") << ",\n";
    file << "      \"can_equip\": " << (product.CanEquip() ? "true" : "false") << ",\n";
    file << "      \"asset_package_name\": " << jsonString(product.GetAssetPackageName())
         << ",\n";
    file << "      \"asset_path\": " << jsonString(unrealString(product.GetAssetPath()))
         << ",\n";
    file << "      \"thumbnail_package_name\": "
         << jsonString(product.GetThumbnailPackageName()) << ",\n";
    file << "      \"thumbnail_asset_name\": " << jsonString(product.GetThumbnailAssetName())
         << "\n";
    file << "    }";
  }

  file << "\n";
  file << "  ],\n";
  file << "  \"total_products_seen\": " << totalCount << ",\n";
  file << "  \"products_written\": " << writtenCount << "\n";
  file << "}\n";

  if (!file) {
    cvarManager->log("subtr-actor: failed while writing product dump JSON");
    return;
  }

  cvarManager->log(std::format(
      "subtr-actor: wrote {} product records to {}",
      writtenCount,
      outputPath.string()));
}
