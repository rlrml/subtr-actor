import type { Object3D } from "three";
import { createZoneBoundaryLines } from "../overlays.ts";
import type { StatModuleContext } from "./types.ts";

function disposeIfPossible(value: unknown): void {
  if (
    value &&
    typeof value === "object" &&
    "dispose" in value &&
    typeof value.dispose === "function"
  ) {
    value.dispose();
  }
}

function disposeZoneBoundaryLines(
  zoneBoundaryLines: ReturnType<typeof createZoneBoundaryLines> | null,
): void {
  if (!zoneBoundaryLines) {
    return;
  }

  zoneBoundaryLines.removeFromParent();
  zoneBoundaryLines.traverse((node: Object3D) => {
    const geometry = "geometry" in node ? node.geometry : null;
    disposeIfPossible(geometry);

    const material = "material" in node ? node.material : null;
    if (Array.isArray(material)) {
      for (const entry of material) {
        disposeIfPossible(entry);
      }
    } else {
      disposeIfPossible(material);
    }
  });
}

function createSharedZoneBoundaryOverlayManager() {
  let refCount = 0;
  let zoneBoundaryLines: ReturnType<typeof createZoneBoundaryLines> | null = null;

  return {
    acquire(ctx: StatModuleContext): void {
      if (!zoneBoundaryLines) {
        zoneBoundaryLines = createZoneBoundaryLines(ctx.player.sceneState.scene, ctx.fieldScale);
      }
      refCount += 1;
    },

    release(): void {
      if (refCount <= 0) {
        return;
      }

      refCount -= 1;
      if (refCount === 0) {
        disposeZoneBoundaryLines(zoneBoundaryLines);
        zoneBoundaryLines = null;
      }
    },
  };
}

export const zoneBoundaryOverlayManager = createSharedZoneBoundaryOverlayManager();
