import * as THREE from "three";
import { createBoostPadMeshes } from "./boost-pad-overlay-meshes";
import type { BoostPadMeshes, BoostPadOverlayPluginOptions } from "./boost-pad-overlay-types";
import { updateBoostPadMeshes } from "./boost-pad-overlay-update";
import type {
  ReplayPlayerPlugin,
  ReplayPlayerPluginContext,
  ReplayPlayerPluginStateContext,
} from "./types";

export type { BoostPadOverlayPluginOptions } from "./boost-pad-overlay-types";

export function createBoostPadOverlayPlugin(
  options: BoostPadOverlayPluginOptions = {},
): ReplayPlayerPlugin {
  const showCooldownProgress = options.showCooldownProgress ?? true;

  let padRoot: THREE.Group | null = null;
  const padMeshes = new Map<number, BoostPadMeshes>();

  function buildPads(context: ReplayPlayerPluginContext): void {
    padRoot = new THREE.Group();
    padRoot.name = "boost-pad-overlay";
    padRoot.renderOrder = 20;
    padRoot.frustumCulled = false;

    for (const pad of context.replay.boostPads) {
      const meshes = createBoostPadMeshes(pad);
      padRoot.add(meshes.group);
      padMeshes.set(pad.index, meshes);
    }

    context.scene.replayRoot.add(padRoot);
  }

  function syncPads(context: ReplayPlayerPluginStateContext): void {
    for (const pad of context.replay.boostPads) {
      const meshes = padMeshes.get(pad.index);
      if (!meshes) {
        continue;
      }
      updateBoostPadMeshes(meshes, pad, context.state.currentTime, showCooldownProgress);
    }
  }

  return {
    id: "boost-pad-overlay",
    setup(context): void {
      buildPads(context);
      syncPads({
        ...context,
        state: context.player.getState(),
      });
    },
    onStateChange(context): void {
      syncPads(context);
    },
    teardown(): void {
      padRoot?.removeFromParent();
      padRoot = null;
      padMeshes.clear();
    },
  };
}
