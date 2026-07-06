declare module "*.wasm?url" {
  const url: string;
  export default url;
}

declare module "@rlrml/subtr-actor" {
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
  export function parse_training_pack(data: Uint8Array): unknown;
  export function parse_training_pack_lossless(data: Uint8Array): string;
  export function serialize_training_pack(lossless: string): Uint8Array;
  export function training_pack_from_lossless(lossless: string): unknown;
  export function new_training_pack(typedPack: unknown): string;
  export function update_training_pack_metadata(lossless: string, typedPack: unknown): string;
  export function training_pack_add_round(lossless: string, round: unknown): string;
  export function training_pack_insert_round(
    lossless: string,
    index: number,
    round: unknown,
  ): string;
  export function training_pack_remove_round(lossless: string, index: number): string;
  export function training_pack_move_round(lossless: string, from: number, to: number): string;
  export function training_pack_duplicate_round(lossless: string, index: number): string;
  export function training_pack_append_rounds(lossless: string, otherLossless: string): string;
  export default function init(moduleOrPath?: unknown): Promise<unknown>;
}
