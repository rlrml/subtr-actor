import type * as THREE from "three";

export interface BoostPadOverlayPluginOptions {
  showCooldownProgress?: boolean;
}

export interface BoostPadMeshes {
  group: THREE.Group;
  ring: THREE.Mesh<THREE.RingGeometry, THREE.MeshBasicMaterial>;
  core: THREE.Mesh<THREE.CircleGeometry, THREE.MeshBasicMaterial>;
  cooldown: THREE.Mesh<THREE.CircleGeometry, THREE.MeshBasicMaterial>;
  orb: THREE.Mesh<THREE.BufferGeometry, THREE.MeshBasicMaterial | THREE.MeshPhongMaterial>;
  glow: THREE.Mesh<THREE.BufferGeometry, THREE.MeshBasicMaterial>;
}
