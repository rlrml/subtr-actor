/**
 * Thin WASM entry: .replay bytes -> subtr-actor raw ReplayData + the shared
 * normalized ReplayModel.
 *
 * We reuse @rlrml/player's tested WASM plumbing (validation + toPlainData, which
 * flattens serde_wasm_bindgen Maps a direct get_replay_frames_data call would
 * leave as Map proxies). It hands back BOTH layers:
 *
 *  - `raw`: the untouched subtr-actor output. The adapter owns its own
 *    coordinate transform end-to-end over this — see adapter/coords.ts.
 *  - `replay`: @rlrml/player's `ReplayModel` (normalizeReplayData over the same
 *    raw output). This is the data surface @rlrml/player consumers — most
 *    importantly js/stat-evaluation-player — read (docs/PLAYER_PARITY.md
 *    Phase 2). It costs nothing extra: loadReplayFromBytes computes it anyway.
 *
 * useWorker:false keeps it dependency-light for bring-up.
 */
import { loadReplayFromBytes } from "@rlrml/player";
import type { ReplayLoadResult } from "@rlrml/player";

/** .replay bytes -> { replay: ReplayModel, raw } (plain JS, Maps flattened). */
export async function loadReplay(bytes: Uint8Array): Promise<ReplayLoadResult> {
  return loadReplayFromBytes(bytes, { useWorker: false });
}

/** .replay bytes -> raw subtr-actor ReplayData only (legacy entry). */
export async function parseReplay(bytes: Uint8Array): Promise<unknown> {
  const { raw } = await loadReplay(bytes);
  return raw;
}
