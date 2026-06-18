declare module "@rlrml/subtr-actor" {
  export default function init(input?: unknown): Promise<unknown>;
  export function get_replay_frames_data(data: Uint8Array): unknown;
}
