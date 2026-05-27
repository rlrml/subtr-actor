import * as THREE from "three";
import {
  boostPadColor,
  boostPadGlowCenterZ,
  boostPadLightCenterZ,
  boostPadOrbRadius,
} from "./boost-pad-overlay-geometry";
import type { ReplayBoostPad } from "./types";

export function createBoostPadOrb(
  pad: ReplayBoostPad,
): THREE.Mesh<THREE.BufferGeometry, THREE.MeshBasicMaterial | THREE.MeshPhongMaterial> {
  const color = boostPadColor(pad);
  const orbRadius = boostPadOrbRadius(pad);
  const isBigPad = pad.size === "big";
  const orb = new THREE.Mesh(
    isBigPad
      ? new THREE.SphereGeometry(orbRadius, 32, 18)
      : new THREE.CircleGeometry(orbRadius * 0.9, 24),
    isBigPad
      ? new THREE.MeshPhongMaterial({
          color,
          emissive: new THREE.Color(color),
          emissiveIntensity: 0.6,
          shininess: 88,
          specular: new THREE.Color(0xfff2c2),
          transparent: true,
          opacity: 0.92,
          depthWrite: false,
        })
      : new THREE.MeshBasicMaterial({
          color,
          transparent: true,
          opacity: 0.88,
          side: THREE.DoubleSide,
          blending: THREE.AdditiveBlending,
          depthWrite: false,
        }),
  );
  orb.position.z = boostPadLightCenterZ(pad);
  orb.renderOrder = 23;
  orb.frustumCulled = false;
  return orb;
}

export function createBoostPadGlow(
  pad: ReplayBoostPad,
): THREE.Mesh<THREE.BufferGeometry, THREE.MeshBasicMaterial> {
  const color = boostPadColor(pad);
  const orbRadius = boostPadOrbRadius(pad);
  const isBigPad = pad.size === "big";
  const glow = new THREE.Mesh(
    isBigPad
      ? new THREE.SphereGeometry(orbRadius * 1.36, 32, 14)
      : new THREE.CircleGeometry(orbRadius * 1.35, 28),
    new THREE.MeshBasicMaterial({
      color,
      transparent: true,
      opacity: isBigPad ? 0.2 : 0.16,
      side: THREE.DoubleSide,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
    }),
  );
  glow.position.z = boostPadGlowCenterZ(pad);
  glow.renderOrder = 24;
  glow.frustumCulled = false;
  return glow;
}
