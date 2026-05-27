import * as THREE from "three";
import { createBoostPadGlow, createBoostPadOrb } from "./boost-pad-overlay-orb";
import {
  createBoostPadCooldown,
  createBoostPadCore,
  createBoostPadRing,
} from "./boost-pad-overlay-surface";
import type { BoostPadMeshes } from "./boost-pad-overlay-types";
import type { ReplayBoostPad } from "./types";

export function createBoostPadMeshes(pad: ReplayBoostPad): BoostPadMeshes {
  const group = new THREE.Group();
  group.position.set(pad.position.x, pad.position.y, pad.position.z);
  group.renderOrder = 20;
  group.frustumCulled = false;

  const ring = createBoostPadRing(pad);
  const core = createBoostPadCore(pad);
  const cooldown = createBoostPadCooldown(pad);
  const orb = createBoostPadOrb(pad);
  const glow = createBoostPadGlow(pad);

  group.add(ring);
  group.add(core);
  group.add(cooldown);
  group.add(orb);
  group.add(glow);

  return { group, ring, core, cooldown, orb, glow };
}
