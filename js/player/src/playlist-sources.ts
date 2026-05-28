import { loadReplayFromBytes } from "./wasm";
import type {
  LoadedReplay,
  PlaybackBound,
  PlaylistItem,
  PlaylistSourceLoadContext,
  ReplaySource,
} from "./types";

type ReplayPathLoader = (
  path: string,
  context?: PlaylistSourceLoadContext,
) => Promise<LoadedReplay>;
type ReplaySourceLoader = (context?: PlaylistSourceLoadContext) => Promise<LoadedReplay>;

export interface FullReplayPlaylistItemOptions {
  label?: string;
  meta?: Record<string, unknown>;
}

export function frameBound(value: number): PlaybackBound {
  return { kind: "frame", value };
}

export function timeBound(value: number): PlaybackBound {
  return { kind: "time", value };
}

export function createReplaySource(id: string, load: ReplaySourceLoader): ReplaySource {
  return { id, load };
}

export function createStaticReplaySource(id: string, replay: LoadedReplay): ReplaySource {
  return createReplaySource(id, async () => replay);
}

export function createReplayBytesSource(id: string, data: Uint8Array): ReplaySource {
  return createReplaySource(id, async () => loadReplayFromBytes(data, { useWorker: true }));
}

export function createReplayFileSource(
  file: File,
  id = file.webkitRelativePath || file.name,
): ReplaySource {
  return createReplaySource(id, async () => {
    const bytes = new Uint8Array(await file.arrayBuffer());
    return loadReplayFromBytes(bytes, { useWorker: true });
  });
}

export function createReplayPathSource(
  path: string,
  loadReplay: ReplayPathLoader,
  id = path,
): ReplaySource {
  return createReplaySource(id, async (context) => loadReplay(path, context));
}

export function createFullReplayPlaylistItem(
  replay: ReplaySource,
  options: FullReplayPlaylistItemOptions = {},
): PlaylistItem {
  return {
    replay,
    start: timeBound(0),
    end: timeBound(Number.POSITIVE_INFINITY),
    label: options.label,
    meta: options.meta,
  };
}
