import {
  boostPadGlowCenterZ,
  boostPadLightCenterZ,
} from "./boost-pad-overlay-geometry";
import { boostPadAvailableState } from "./boost-pad-overlay-state";
import type { BoostPadMeshes } from "./boost-pad-overlay-types";
import type { ReplayBoostPad } from "./types";

export function updateBoostPadMeshes(
  meshes: BoostPadMeshes,
  pad: ReplayBoostPad,
  currentTime: number,
  showCooldownProgress: boolean,
): void {
  const { available, progress } = boostPadAvailableState(pad, currentTime);
  const isBigPad = pad.size === "big";
  const pulse = 0.92 + 0.08 * Math.sin(currentTime * 6 + pad.index * 0.45);
  const orbPulse = 0.96 + 0.04 * Math.sin(currentTime * (isBigPad ? 4.8 : 7.2) + pad.index * 0.37);
  const hover = isBigPad ? Math.sin(currentTime * 2.2 + pad.index * 0.61) * 18 : 0;
  const lightZ = boostPadLightCenterZ(pad) + hover;
  const glowZ = boostPadGlowCenterZ(pad) + hover;

  meshes.orb.position.z = lightZ;
  meshes.glow.position.z = glowZ;
  meshes.orb.rotation.z = currentTime * (isBigPad ? 0.9 : 1.25);
  meshes.glow.rotation.z = -currentTime * 0.45;

  if (available) {
    meshes.group.visible = true;
    meshes.ring.material.opacity = 0.95;
    meshes.core.material.opacity = isBigPad ? 0.56 : 0.5;
    meshes.cooldown.visible = false;
    meshes.ring.scale.setScalar(pulse);
    meshes.core.scale.setScalar(1);
    meshes.orb.visible = true;
    meshes.glow.visible = true;
    meshes.orb.material.opacity = isBigPad ? 0.96 : 0.9;
    meshes.glow.material.opacity = (isBigPad ? 0.2 : 0.16) + (orbPulse - 0.96);
    meshes.orb.scale.setScalar(orbPulse);
    meshes.glow.scale.setScalar(isBigPad ? 1.02 + (orbPulse - 0.96) * 2 : 1);
    return;
  }

  meshes.group.visible = true;
  meshes.ring.material.opacity = 0.18;
  meshes.core.material.opacity = 0.07;
  meshes.ring.scale.setScalar(1);
  meshes.core.scale.setScalar(1);
  meshes.orb.visible = false;
  meshes.glow.visible = false;

  meshes.cooldown.visible = showCooldownProgress;
  if (showCooldownProgress) {
    const cooldownScale = 0.3 + progress * 0.7;
    meshes.cooldown.scale.setScalar(cooldownScale);
    meshes.cooldown.material.opacity = 0.16 + progress * 0.2;
  }
}
