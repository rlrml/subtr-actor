/// <reference types="vite/client" />

declare module "*.wasm?url" {
  const url: string;
  export default url;
}

declare module "@rlrml/subtr-actor" {
  export default function init(moduleOrPath?: unknown): Promise<unknown>;
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
    /** Compact event-backed stats timeline JSON bytes. */
    statsTimeline: Uint8Array;
  };
  export function get_replay_bundle_json_parts_with_progress(
    data: Uint8Array,
    callback: (progress: unknown) => void,
    reportEveryNFrames?: number,
    maxFrameChunkBytes?: number,
  ): {
    rawReplayData: Uint8Array;
    statsTimelineParts: {
      config: Uint8Array;
      replayMeta: Uint8Array;
      events: Uint8Array;
      positioningSummary: Uint8Array;
      frameChunks: Uint8Array[];
    };
  };
  export function validate_replay(data: Uint8Array): unknown;
  /** Returns the compact event-backed timeline with scaffold frames. */
  export function get_stats_timeline(
    data: Uint8Array,
  ): import("./generated/ReplayStatsTimelineScaffold.ts").ReplayStatsTimelineScaffold;
  /** Returns the compact event-backed timeline as JSON bytes. */
  export function get_stats_timeline_json(data: Uint8Array): Uint8Array;
  /** Returns the legacy full partial-sum timeline as JSON bytes. */
  export function get_legacy_stats_timeline_json(data: Uint8Array): Uint8Array;
  export function get_stats_timeline_json_parts(
    data: Uint8Array,
    maxFrameChunkBytes?: number,
  ): {
    config: Uint8Array;
    replayMeta: Uint8Array;
    events: Uint8Array;
    positioningSummary: Uint8Array;
    frameChunks: Uint8Array[];
  };
}

declare module "../scripts/ensure-wasm-package.mjs" {
  export function ensureWasmPackageFresh(options: {
    force?: boolean;
    log?: (message: string) => void;
  }): Promise<void>;
  export function getWasmWatchTargets(): string[];
  export function isWasmSourcePath(filePath: string): boolean;
}
