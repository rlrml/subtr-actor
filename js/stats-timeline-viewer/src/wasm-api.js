import init, {
  get_replay_frames_data,
  get_replay_info,
  get_stats_timeline,
  validate_replay,
} from "../../pkg/rl_replay_subtr_actor";
import wasmUrl from "../../pkg/rl_replay_subtr_actor_bg.wasm?url";

let wasmReady = false;

export async function initializeWasm() {
  if (wasmReady) {
    return;
  }
  await init({ module_or_path: wasmUrl });
  wasmReady = true;
}

export async function loadReplayArtifacts(replayBytes) {
  const validation = normalizeWasmValue(validate_replay(replayBytes));
  if (!validation.valid) {
    throw new Error(validation.error ?? "Replay is not valid");
  }

  const info = normalizeWasmValue(get_replay_info(replayBytes));
  const frameData = normalizeWasmValue(get_replay_frames_data(replayBytes));
  const statsTimeline = normalizeWasmValue(get_stats_timeline(replayBytes));

  return { info, frameData, statsTimeline };
}

export function normalizeWasmValue(value) {
  if (value == null) {
    return value;
  }
  if (typeof value.get === "function" && typeof value.entries === "function") {
    return Object.fromEntries(
      Array.from(value.entries(), ([key, nestedValue]) => [
        key,
        normalizeWasmValue(nestedValue),
      ]),
    );
  }
  if (Array.isArray(value)) {
    return value.map(normalizeWasmValue);
  }
  if (typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value).map(([key, nestedValue]) => [
        key,
        normalizeWasmValue(nestedValue),
      ]),
    );
  }
  return value;
}
