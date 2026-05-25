import { loadReplayFromBytes } from "./wasm";
import type { LoadedReplay, PlaylistSourceLoadContext, ReplaySource } from "./types";

export const BALLCHASING_API_BASE_URL = "https://ballchasing.com/api";
export const BALLCHASING_BASE_URL = "https://ballchasing.com";

const BALLCHASING_REPLAY_ID_PATTERN =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export interface BallchasingReplayDownloadOptions {
  baseUrl?: string | URL;
  fetch?: typeof fetch;
  fetchInit?: RequestInit;
  signal?: AbortSignal;
}

function buildApiUrl(path: string, apiBaseUrl: string | URL): URL {
  const baseHref = apiBaseUrl instanceof URL ? apiBaseUrl.href : apiBaseUrl;
  const normalizedBase = baseHref.replace(/\/+$/, "");
  return new URL(`${normalizedBase}/${path.replace(/^\/+/, "")}`);
}

function mergeFetchInit(options: BallchasingReplayDownloadOptions): RequestInit {
  const headers = new Headers(options.fetchInit?.headers);

  return {
    ...options.fetchInit,
    method: options.fetchInit?.method ?? "POST",
    headers,
    signal: options.signal ?? options.fetchInit?.signal,
  };
}

function describeBallchasingFetchError(response: Response, url: URL): string {
  const statusText = response.statusText ? ` ${response.statusText}` : "";
  const authHint =
    response.status === 401 || response.status === 403 || response.status === 404
      ? ". The replay may be private, unavailable, or not downloadable without a Ballchasing session"
      : "";
  return (
    `Failed to fetch Ballchasing replay from ${url.href} ` +
    `(${response.status}${statusText})${authHint}`
  );
}

export function isBallchasingReplayId(value: string): boolean {
  return BALLCHASING_REPLAY_ID_PATTERN.test(value.trim());
}

export function normalizeBallchasingReplayId(value: string): string {
  const trimmed = value.trim();
  if (isBallchasingReplayId(trimmed)) {
    return trimmed.toLowerCase();
  }

  let url: URL;
  try {
    url = new URL(trimmed);
  } catch {
    throw new Error(`Invalid Ballchasing replay id: ${value}`);
  }

  if (!/(^|\.)ballchasing\.com$/i.test(url.hostname)) {
    throw new Error(`Invalid Ballchasing replay URL: ${value}`);
  }

  const pathParts = url.pathname.split("/").filter(Boolean);
  const replayIndex = pathParts.findIndex((part) => part === "replay");
  const apiReplayIndex = pathParts.findIndex((part) => part === "replays");
  const id =
    replayIndex >= 0
      ? pathParts[replayIndex + 1]
      : apiReplayIndex >= 0
        ? pathParts[apiReplayIndex + 1]
        : undefined;

  if (!id || !isBallchasingReplayId(id)) {
    throw new Error(`Invalid Ballchasing replay URL: ${value}`);
  }

  return id.toLowerCase();
}

export function getBallchasingReplayFileName(idOrUrl: string): string {
  return `ballchasing-${normalizeBallchasingReplayId(idOrUrl)}.replay`;
}

export function getBallchasingReplayFileUrl(
  idOrUrl: string,
  baseUrl: string | URL = BALLCHASING_BASE_URL,
): URL {
  const id = normalizeBallchasingReplayId(idOrUrl);
  return buildApiUrl(`dl/replay/${encodeURIComponent(id)}`, baseUrl);
}

export function getBallchasingReplayApiFileUrl(
  idOrUrl: string,
  apiBaseUrl: string | URL = BALLCHASING_API_BASE_URL,
): URL {
  const id = normalizeBallchasingReplayId(idOrUrl);
  return buildApiUrl(`replays/${encodeURIComponent(id)}/file`, apiBaseUrl);
}

export async function fetchBallchasingReplayBytes(
  idOrUrl: string,
  options: BallchasingReplayDownloadOptions = {},
): Promise<Uint8Array> {
  const replayUrl = getBallchasingReplayFileUrl(idOrUrl, options.baseUrl ?? BALLCHASING_BASE_URL);
  const fetcher = options.fetch ?? globalThis.fetch;
  if (!fetcher) {
    throw new Error("No fetch implementation is available");
  }

  const response = await fetcher(replayUrl, mergeFetchInit(options));
  if (!response.ok) {
    throw new Error(describeBallchasingFetchError(response, replayUrl));
  }

  return new Uint8Array(await response.arrayBuffer());
}

export function createBallchasingReplaySource(
  idOrUrl: string,
  options: BallchasingReplayDownloadOptions = {},
): ReplaySource {
  const id = normalizeBallchasingReplayId(idOrUrl);
  return {
    id: `ballchasing:${id}`,
    async load(context?: PlaylistSourceLoadContext): Promise<LoadedReplay> {
      const bytes = await fetchBallchasingReplayBytes(id, options);
      return loadReplayFromBytes(bytes, {
        useWorker: true,
        onProgress(progress) {
          context?.updateProgress({
            stage: progress.stage,
            progress: progress.progress,
            processedFrames: progress.processedFrames,
            totalFrames: progress.totalFrames,
          });
        },
      });
    },
  };
}
