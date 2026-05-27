import type { PlaylistItem, PlaylistManifest, PlaylistManifestReplay, ReplaySource } from "./types";

export function resolvePlaylistItemsFromManifest(
  manifest: PlaylistManifest,
  resolveReplaySource: (context: {
    replayId: string;
    replay?: PlaylistManifestReplay;
  }) => ReplaySource,
): PlaylistItem[] {
  const replaysById = new Map<string, PlaylistManifestReplay>(
    (manifest.replays ?? []).map((replay) => [replay.id, replay]),
  );

  return manifest.items.map((item) => {
    const replay = replaysById.get(item.replay);
    return {
      replay: resolveReplaySource({
        replayId: item.replay,
        replay,
      }),
      start: item.start,
      end: item.end,
      label: item.label || replay?.label,
      meta: item.meta,
    };
  });
}
