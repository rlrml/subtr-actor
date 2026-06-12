/**
 * Hand-written declarations for the vendored TrailRenderer (TrailRendererJS
 * port). The .js source predates this package and trips TS9005 under
 * declaration-emit inference, so we pin a minimal surface here instead —
 * only what EffectsManager / BallTrailRenderer actually touch.
 */
import * as THREE from "three";

export class TrailRenderer extends THREE.Object3D {
  constructor(scene: THREE.Object3D, orientToMovement?: boolean);
  static createBaseMaterial(customUniforms?: Record<string, unknown>): THREE.ShaderMaterial;
  static createTexturedMaterial(customUniforms?: Record<string, unknown>): THREE.ShaderMaterial;
  initialize(
    material: THREE.Material,
    length: number,
    dragTexture: boolean,
    localHeadWidth: number,
    localHeadGeometry: THREE.Vector3[] | null | undefined,
    targetObject: THREE.Object3D,
  ): void;
  activate(): void;
  deactivate(): void;
  reset(): void;
  advance(): void;
  updateHead(): void;
  destroyMesh(): void;
  [key: string]: unknown;
}
