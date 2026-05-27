import type * as THREE from "three";

export function configureBoostPadOverlayMaterial(material: THREE.MeshBasicMaterial): void {
  material.depthTest = false;
  material.depthWrite = false;
  material.transparent = true;
  material.polygonOffset = true;
  material.polygonOffsetFactor = -2;
  material.polygonOffsetUnits = -2;
  material.forceSinglePass = true;
}
