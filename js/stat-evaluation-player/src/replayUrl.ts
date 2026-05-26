import { deflateSync, inflateSync, strFromU8, strToU8 } from "fflate";
import {
  getBallchasingReplayFileName,
  getBallchasingReplayFileUrl,
  normalizeBallchasingReplayId,
} from "@rlrml/subtr-actor-player";

const REPLAY_URL_QUERY_PARAMS = ["replayUrl", "replay_url", "replay"] as const;
const COMPRESSED_REPLAY_URL_QUERY_PARAMS = ["r", "replayUrlZ", "replay_url_z"] as const;
const BALLCHASING_REPLAY_QUERY_PARAMS = [
  "ballchasing",
  "ballchasingId",
  "ballchasingUuid",
  "ballchasingReplay",
] as const;

export type ReplayFetchRequestKind = "url" | "ballchasing";

export interface ReplayFetchRequest {
  readonly kind: ReplayFetchRequestKind;
  readonly url: URL;
  readonly name: string;
  readonly fetchInit?: RequestInit;
}

function bytesToBase64Url(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }

  return btoa(binary).replaceAll("+", "-").replaceAll("/", "_").replace(/=+$/, "");
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
      `Invalid compressed replay URL: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}

export function getReplayUrlFromSearch(search: string, baseUrl: string): URL | null {
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

function getFirstParam(params: URLSearchParams, names: readonly string[]): string | null {
  for (const name of names) {
    const rawValue = params.get(name)?.trim();
    if (rawValue) {
      return rawValue;
    }
  }
  return null;
}

export function getReplayFetchRequestFromSearch(
  search: string,
  baseUrl: string,
): ReplayFetchRequest | null {
  const params = new URLSearchParams(search);
  const ballchasingValue = getFirstParam(params, BALLCHASING_REPLAY_QUERY_PARAMS);
  if (ballchasingValue) {
    const id = normalizeBallchasingReplayId(ballchasingValue);
    return {
      kind: "ballchasing",
      url: getBallchasingReplayFileUrl(id),
      name: getBallchasingReplayFileName(id),
      fetchInit: { method: "POST" },
    };
  }

  const replayUrl = getReplayUrlFromSearch(search, baseUrl);
  if (!replayUrl) {
    return null;
  }

  return {
    kind: "url",
    url: replayUrl,
    name: getReplayFileNameFromUrl(replayUrl),
  };
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
