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
  export function validate_replay(data: Uint8Array): unknown;
  export default function init(moduleOrPath?: unknown): Promise<unknown>;
}
