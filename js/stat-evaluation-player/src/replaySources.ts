import {
  loadReplayBundleInWorker,
  type ReplayLoadBundle,
  type ReplayLoadProgress,
} from "./replayLoader.ts";
import type { ReplayFetchRequest } from "./replayUrl.ts";

export interface ReplayInputSource {
  name: string;
  preparingStatus: string;
  readBytes(): Promise<Uint8Array>;
}

export function createFileReplaySource(file: File): ReplayInputSource {
  return {
    name: file.name,
    preparingStatus: "Preparing replay...",
    async readBytes() {
      return new Uint8Array(await file.arrayBuffer());
    },
  };
}

export function createRemoteReplaySource(
  request: ReplayFetchRequest,
  signal: AbortSignal,
): ReplayInputSource {
  return {
    name: request.name,
    preparingStatus: "Fetching replay...",
    async readBytes() {
      const response = await fetch(request.url, {
        ...request.fetchInit,
        signal,
      });
      if (!response.ok) {
        const statusText = response.statusText ? ` ${response.statusText}` : "";
        const authHint =
          request.kind === "ballchasing" && [401, 403, 404].includes(response.status)
            ? ". The replay may be private, unavailable, or not downloadable without a Ballchasing session"
            : "";
        throw new Error(
          `Failed to fetch replay from ${request.url.href} (${response.status}${statusText})${authHint}`,
        );
      }
      return new Uint8Array(await response.arrayBuffer());
    },
  };
}

export async function loadReplayBundleFromSource(
  source: ReplayInputSource,
  onProgress?: (progress: ReplayLoadProgress) => void,
): Promise<ReplayLoadBundle> {
  const bytes = await source.readBytes();
  return loadReplayBundleInWorker(bytes, {
    reportEveryNFrames: 100,
    onProgress,
  });
}
