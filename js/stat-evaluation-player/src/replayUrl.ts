import { deflateSync, inflateSync, strFromU8, strToU8 } from "fflate";

const REPLAY_URL_QUERY_PARAMS = ["replayUrl", "replay_url", "replay"] as const;
const COMPRESSED_REPLAY_URL_QUERY_PARAMS = [
  "r",
  "replayUrlZ",
  "replay_url_z",
] as const;

function bytesToBase64Url(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }

  return btoa(binary)
    .replaceAll("+", "-")
    .replaceAll("/", "_")
    .replace(/=+$/, "");
}

function base64UrlToBytes(value: string): Uint8Array {
  const normalized = value.replaceAll("-", "+").replaceAll("_", "/");
  const padded = normalized.padEnd(Math.ceil(normalized.length / 4) * 4, "=");
  const binary = atob(padded);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes;
}

export function encodeCompressedReplayUrl(url: string | URL): string {
  const value = url instanceof URL ? url.href : url;
  return bytesToBase64Url(deflateSync(strToU8(value), { level: 9 }));
}

export function decodeCompressedReplayUrl(value: string): string {
  try {
    return strFromU8(inflateSync(base64UrlToBytes(value)));
  } catch (error) {
    throw new Error(
      `Invalid compressed replay URL: ${
        error instanceof Error ? error.message : String(error)
      }`,
    );
  }
}

export function getReplayUrlFromSearch(
  search: string,
  baseUrl: string,
): URL | null {
  const params = new URLSearchParams(search);

  for (const name of REPLAY_URL_QUERY_PARAMS) {
    const rawValue = params.get(name)?.trim();
    if (!rawValue) {
      continue;
    }

    const url = new URL(rawValue, baseUrl);
    if (url.protocol !== "http:" && url.protocol !== "https:") {
      throw new Error(`Unsupported replay URL protocol: ${url.protocol}`);
    }
    return url;
  }

  for (const name of COMPRESSED_REPLAY_URL_QUERY_PARAMS) {
    const rawValue = params.get(name)?.trim();
    if (!rawValue) {
      continue;
    }

    const url = new URL(decodeCompressedReplayUrl(rawValue), baseUrl);
    if (url.protocol !== "http:" && url.protocol !== "https:") {
      throw new Error(`Unsupported replay URL protocol: ${url.protocol}`);
    }
    return url;
  }

  return null;
}

export function getReplayFileNameFromUrl(url: URL): string {
  const pathname = url.pathname.replace(/\/+$/, "");
  const pathName = pathname.split("/").pop();
  if (!pathName) {
    return url.hostname || "remote replay";
  }

  try {
    return decodeURIComponent(pathName);
  } catch {
    return pathName;
  }
}
