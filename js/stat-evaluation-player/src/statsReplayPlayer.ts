import type { ReplayModel } from "@rlrml/player";
import type { ReplayPlayer } from "@rlrml/player";

/**
 * The player this app drives: @rlrml/player's `ReplayPlayer`, which implements
 * @rlrml/player's full `ReplayPlayer` control / timeline / scene / plugin
 * surface (js/player/docs/player/PLAYER_PARITY.md).
 *
 * The intersection pins `replay` non-null: we always construct through
 * `createPlayerFromParsed` with the worker-parsed `ReplayModel`, so consumers
 * keep @rlrml/player's `player.replay` (non-null) reads unchanged.
 */
export type StatsReplayPlayer = ReplayPlayer & { readonly replay: ReplayModel };
