/**
 * Thin WASM entry: .replay bytes -> subtr-actor raw ReplayData.
 *
 * We call subtr-actor's `get_replay_frames_data` directly (rather than going
 * through @rlrml/player's normalizeReplayDataAsync) so the adapter owns the
 * coordinate transform end-to-end — see adapter/coords.ts.
 */
import { loadReplayFromBytes } from "@rlrml/player";

/**
 * .replay bytes -> subtr-actor raw ReplayData (plain JS, Maps flattened).
 *
 * Reuses @rlrml/player's tested WASM plumbing (validation + toPlainData, which
 * flattens serde_wasm_bindgen Maps a direct get_replay_frames_data call would
 * leave as Map proxies). We take only `.raw` and do our own normalization in the
 * adapter. useWorker:false keeps it dependency-light for bring-up.
 */
export async function parseReplay(bytes: Uint8Array): Promise<unknown> {
  const { raw } = await loadReplayFromBytes(bytes, { useWorker: false });
  return raw;
}
