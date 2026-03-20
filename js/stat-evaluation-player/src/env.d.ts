/// <reference types="vite/client" />

declare module "*.wasm?url" {
  const url: string;
  export default url;
}

declare module "@colonelpanic8/subtr-actor" {
  export function get_replay_frames_data(data: Uint8Array): unknown;
  export function get_replay_frames_data_with_progress(
    data: Uint8Array,
    callback: (progress: unknown) => void,
    reportEveryNFrames?: number,
  ): unknown;
  export function get_replay_frames_data_json_with_progress(
    data: Uint8Array,
    callback: (progress: unknown) => void,
    reportEveryNFrames?: number,
  ): Uint8Array;
  export function get_replay_bundle_json_with_progress(
    data: Uint8Array,
    callback: (progress: unknown) => void,
    reportEveryNFrames?: number,
  ): {
    rawReplayData: Uint8Array;
    statsTimeline: Uint8Array;
  };
  export function validate_replay(data: Uint8Array): unknown;
  export function get_stats_timeline(data: Uint8Array): unknown;
  export function get_stats_timeline_json(data: Uint8Array): Uint8Array;
  export function get_dynamic_stats_timeline(data: Uint8Array): unknown;
  export function get_dynamic_stats_timeline_json(data: Uint8Array): Uint8Array;
}

declare module "../scripts/ensure-wasm-package.mjs" {
  export function ensureWasmPackageFresh(options: {
    force?: boolean;
    log?: (message: string) => void;
  }): Promise<void>;
  export function getWasmWatchTargets(): string[];
  export function isWasmSourcePath(filePath: string): boolean;
}
