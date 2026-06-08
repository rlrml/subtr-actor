// Included by SubtrActorPlugin.cpp; shares the plugin translation unit.
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
    file << "      \"ascii_label\": " << jsonString(unrealString(product.GetAsciiLabel())) << ",\n";
    file << "      \"long_label\": " << jsonString(unrealString(product.GetLongLabel())) << ",\n";
    file << "      \"sort_label\": " << jsonString(unrealString(product.GetSortLabel())) << ",\n";
    file << "      \"display_label_slot\": "
         << jsonString(unrealString(product.GetDisplayLabelSlot())) << ",\n";
    file << "      \"slot_index\": " << productSlotIndex(slot) << ",\n";
    file << "      \"slot_label\": " << jsonString(productSlotLabel(slot)) << ",\n";
    file << "      \"slot_plural_label\": " << jsonString(productSlotPluralLabel(slot)) << ",\n";
    file << "      \"quality\": " << static_cast<int>(product.GetQuality()) << ",\n";
    file << "      \"paintable\": " << (product.IsPaintable() ? "true" : "false") << ",\n";
    file << "      \"licensed\": " << (product.IsLicensed() ? "true" : "false") << ",\n";
    file << "      \"can_equip\": " << (product.CanEquip() ? "true" : "false") << ",\n";
    file << "      \"asset_package_name\": "
         << jsonString(product.GetAssetPackageName()) << ",\n";
    file << "      \"asset_path\": " << jsonString(unrealString(product.GetAssetPath())) << ",\n";
    file << "      \"thumbnail_package_name\": "
         << jsonString(product.GetThumbnailPackageName()) << ",\n";
    file << "      \"thumbnail_asset_name\": "
         << jsonString(product.GetThumbnailAssetName()) << "\n";
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
