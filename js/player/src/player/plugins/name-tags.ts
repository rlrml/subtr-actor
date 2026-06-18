/**
 * Name-tag plugin — floating player name + boost labels above each car.
 *
 * The first real `PlayerPlugin`: it proves the contract from
 * docs/player/EXTENSIBILITY.md by wrapping the existing NameTagManager entirely
 * through plugin hooks (no core wiring). Per the convention from
 * `@rlrml/player`, consumers opt in via the factory:
 *
 *   createPlayer(container, bytes, { plugins: [createNameTagPlugin()] })
 */
import { NameTagManager } from "../managers/NameTagManager.js";
import type { PlayerPlugin } from "../types.js";

export function createNameTagPlugin(): PlayerPlugin {
  let manager: NameTagManager | null = null;

  return {
    id: "name-tags",
    setup(ctx) {
      manager = new NameTagManager(ctx.scene, ctx.camera);
      manager.setPlayerTeams(ctx.player.adapter.getPlayerTeams());
    },
    beforeRender(ctx) {
      if (!manager) return;
      // NameTagManager.update reads (actors, boosts, name->actorId). We key
      // everything by player name and hand it the rendered car meshes so tags
      // pin to the post-interpolation transform.
      const actors: Record<string, unknown> = {};
      const boosts: Record<string, number> = {};
      const nameToActorId: Record<string, string> = {};
      for (const car of ctx.cars) {
        if (!car.object3d) continue;
        actors[car.name] = car.object3d;
        boosts[car.name] = car.boost;
        nameToActorId[car.name] = car.name;
      }
      const followedPlayer =
        ctx.state.cameraViewMode === "follow" && ctx.state.attachedPlayerId
          ? (ctx.player.adapter.playerList.find(
              (player) =>
                player.id === ctx.state.attachedPlayerId ||
                player.name === ctx.state.attachedPlayerId,
            )?.name ?? null)
          : null;
      manager.update(actors, boosts, nameToActorId, followedPlayer ?? null);
    },
    teardown() {
      manager?.dispose();
      manager = null;
    },
  };
}
