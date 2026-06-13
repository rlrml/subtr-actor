/**
 * Name-tag plugin — floating player name + boost labels above each car.
 *
 * The first real `ViewerPlugin`: it proves the contract from
 * docs/EXTENSIBILITY.md by wrapping the existing NameTagManager entirely
 * through plugin hooks (no core wiring). Per the convention from
 * `@rlrml/player`, consumers opt in via the factory:
 *
 *   createViewer(container, bytes, { plugins: [createNameTagPlugin()] })
 */
import { NameTagManager } from "../managers/NameTagManager.js";
import type { ViewerPlugin } from "../types.js";

export function createNameTagPlugin(): ViewerPlugin {
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
      manager.update(actors, boosts, nameToActorId, null);
    },
    teardown() {
      manager?.dispose();
      manager = null;
    },
  };
}
