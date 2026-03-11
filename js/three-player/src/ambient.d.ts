declare module "@subtr-actor-wasm" {
  export default function init(): Promise<void>;
  export function get_replay_frames_data(data: Uint8Array): unknown;
  export function validate_replay(
    data: Uint8Array
  ): { valid: boolean; message?: string; error?: string };
}
