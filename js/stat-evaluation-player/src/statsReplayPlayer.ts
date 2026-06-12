import type { ReplayModel } from "@rlrml/player";
import type { ViewerPlayer } from "@rlrml/viewer";

/**
 * The player this app drives: @rlrml/viewer's `ViewerPlayer`, which implements
 * @rlrml/player's full `ReplayPlayer` control / timeline / scene / plugin
 * surface (js/viewer/docs/PLAYER_PARITY.md).
 *
 * The intersection pins `replay` non-null: we always construct through
 * `createViewerFromParsed` with the worker-parsed `ReplayModel`, so consumers
 * keep @rlrml/player's `player.replay` (non-null) reads unchanged.
 */
export type StatsReplayPlayer = ViewerPlayer & { readonly replay: ReplayModel };
