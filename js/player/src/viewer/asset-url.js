let configuredAssetBase = null;

function documentBaseUrl() {
  if (typeof document !== "undefined" && document.baseURI) {
    return document.baseURI;
  }
  if (typeof window !== "undefined" && window.location?.href) {
    return window.location.href;
  }
  return import.meta.url;
}

function viteBaseUrl() {
  return import.meta.env?.BASE_URL ?? "/";
}

function isAbsoluteUrl(path) {
  return /^[a-z][a-z\d+\-.]*:/i.test(path) || path.startsWith("//");
}

export function setViewerAssetBase(assetBase) {
  configuredAssetBase = assetBase == null ? null : String(assetBase);
}

export function getViewerAssetBase(assetBase = configuredAssetBase) {
  const base = assetBase == null || assetBase === "" ? viteBaseUrl() : String(assetBase);
  return new URL(base, documentBaseUrl()).href;
}

export function resolveViewerAssetUrl(path, assetBase = configuredAssetBase) {
  const rawPath = String(path);
  if (isAbsoluteUrl(rawPath)) {
    return rawPath;
  }
  return new URL(rawPath.replace(/^\/+/, ""), getViewerAssetBase(assetBase)).href;
}
