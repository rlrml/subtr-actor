import * as THREE from "three";
import {
  BOOST_PAD_SURFACE_Z_OFFSET,
  boostPadColor,
  boostPadRadius,
} from "./boost-pad-overlay-geometry";
import { configureBoostPadOverlayMaterial } from "./boost-pad-overlay-material";
import type { ReplayBoostPad } from "./types";

export function createBoostPadRing(
  pad: ReplayBoostPad,
): THREE.Mesh<THREE.RingGeometry, THREE.MeshBasicMaterial> {
  const radius = boostPadRadius(pad);
  const ring = new THREE.Mesh(
    new THREE.RingGeometry(radius * 0.72, radius, 24),
    new THREE.MeshBasicMaterial({
      color: boostPadColor(pad),
      transparent: true,
      opacity: 0.92,
      side: THREE.DoubleSide,
      depthWrite: false,
    }),
  );
  configureBoostPadOverlayMaterial(ring.material);
  ring.position.z = BOOST_PAD_SURFACE_Z_OFFSET;
  ring.renderOrder = 20;
  ring.frustumCulled = false;
  return ring;
}

export function createBoostPadCore(
  pad: ReplayBoostPad,
): THREE.Mesh<THREE.CircleGeometry, THREE.MeshBasicMaterial> {
  const core = new THREE.Mesh(
    new THREE.CircleGeometry(boostPadRadius(pad) * 0.58, 24),
    new THREE.MeshBasicMaterial({
      color: boostPadColor(pad),
      transparent: true,
      opacity: 0.3,
      side: THREE.DoubleSide,
      depthWrite: false,
    }),
  );
  configureBoostPadOverlayMaterial(core.material);
  core.position.z = BOOST_PAD_SURFACE_Z_OFFSET + 0.5;
  core.renderOrder = 21;
  core.frustumCulled = false;
  return core;
}

export function createBoostPadCooldown(
  pad: ReplayBoostPad,
): THREE.Mesh<THREE.CircleGeometry, THREE.MeshBasicMaterial> {
  const cooldown = new THREE.Mesh(
    new THREE.CircleGeometry(boostPadRadius(pad) * 0.42, 20),
    new THREE.MeshBasicMaterial({
      color: 0xffffff,
      transparent: true,
      opacity: 0.22,
      side: THREE.DoubleSide,
      depthWrite: false,
    }),
  );
  configureBoostPadOverlayMaterial(cooldown.material);
  cooldown.position.z = BOOST_PAD_SURFACE_Z_OFFSET + 1;
  cooldown.renderOrder = 22;
  cooldown.frustumCulled = false;
  return cooldown;
}
