import { loadReplayFromBytes } from "./wasm";
import type { LoadedReplay, PlaylistSourceLoadContext, ReplaySource } from "./types";
import {
  BALLCHASING_BASE_URL,
  getBallchasingReplayFileUrl,
  normalizeBallchasingReplayId,
} from "./ballchasing-url";

export {
  BALLCHASING_API_BASE_URL,
  BALLCHASING_BASE_URL,
  getBallchasingReplayApiFileUrl,
  getBallchasingReplayFileName,
  getBallchasingReplayFileUrl,
  isBallchasingReplayId,
  normalizeBallchasingReplayId,
} from "./ballchasing-url";

export interface BallchasingReplayDownloadOptions {
  baseUrl?: string | URL;
  fetch?: typeof fetch;
  fetchInit?: RequestInit;
  signal?: AbortSignal;
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
