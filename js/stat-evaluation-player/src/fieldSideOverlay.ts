import * as THREE from "three";
import {
  BLUE_TEAM_ACCENT_COLOR,
  FIELD_HALF_X,
  FIELD_ZONE_BOUNDARY_Y,
  ORANGE_TEAM_ACCENT_COLOR,
} from "./overlayConstants.ts";

const HALF_FIELD_BASE_OPACITY = 0.12;
const HALF_FIELD_ACTIVE_OPACITY = 0.28;
const HALF_FIELD_Z = 2;

const ZONE_BOUNDARY_Z = 6;
const ZONE_BOUNDARY_COLOR = 0x0d1117;
const ZONE_BOUNDARY_OPACITY = 0.42;
const ZONE_BOUNDARY_THICKNESS = 18;
const ZONE_MIDLINE_OPACITY = 0.24;
const ZONE_MIDLINE_THICKNESS = 10;

interface HalfFieldSide {
  mesh: THREE.Mesh;
  material: THREE.MeshBasicMaterial;
}

export function getBallSideFromY(
  ballY: number | null | undefined,
): "team-zero" | "team-one" | null {
  if (ballY === null || ballY === undefined || Number.isNaN(ballY)) {
    return null;
  }

  return ballY < 0 ? "team-zero" : "team-one";
}

export class HalfFieldOverlay {
  private group: THREE.Group;
  private teamZeroSide: HalfFieldSide;
  private teamOneSide: HalfFieldSide;

  constructor(scene: THREE.Scene, fieldScale: number) {
    this.group = new THREE.Group();
    this.teamZeroSide = this.createHalfFieldSide(BLUE_TEAM_ACCENT_COLOR);
    this.teamOneSide = this.createHalfFieldSide(ORANGE_TEAM_ACCENT_COLOR);

    const halfWidth = FIELD_HALF_X * fieldScale;
    const halfDepth = 5120 * fieldScale;

    this.teamZeroSide.mesh.position.set(0, -halfDepth / 2, HALF_FIELD_Z);
    this.teamZeroSide.mesh.scale.set(halfWidth * 2, halfDepth, 1);
    this.teamOneSide.mesh.position.set(0, halfDepth / 2, HALF_FIELD_Z);
    this.teamOneSide.mesh.scale.set(halfWidth * 2, halfDepth, 1);

    this.group.add(this.teamZeroSide.mesh);
    this.group.add(this.teamOneSide.mesh);
    scene.add(this.group);
  }

  update(ballY: number | null | undefined): void {
    const ballSide = getBallSideFromY(ballY);
    this.teamZeroSide.material.opacity =
      ballSide === "team-zero" ? HALF_FIELD_ACTIVE_OPACITY : HALF_FIELD_BASE_OPACITY;
    this.teamOneSide.material.opacity =
      ballSide === "team-one" ? HALF_FIELD_ACTIVE_OPACITY : HALF_FIELD_BASE_OPACITY;
  }

  dispose(): void {
    this.group.removeFromParent();
    this.teamZeroSide.mesh.geometry.dispose();
    this.teamZeroSide.material.dispose();
    this.teamOneSide.mesh.geometry.dispose();
    this.teamOneSide.material.dispose();
  }

  private createHalfFieldSide(color: number): HalfFieldSide {
    const geometry = new THREE.PlaneGeometry(1, 1);
    const material = new THREE.MeshBasicMaterial({
      color,
      transparent: true,
      opacity: HALF_FIELD_BASE_OPACITY,
      side: THREE.DoubleSide,
      depthWrite: false,
      depthTest: false,
    });
    const mesh = new THREE.Mesh(geometry, material);
    mesh.renderOrder = 18;
    return { mesh, material };
  }
}

export function createZoneBoundaryLines(scene: THREE.Scene, fieldScale: number): THREE.Group {
  const group = new THREE.Group();
  const fieldWidth = FIELD_HALF_X * 2 * fieldScale;

  const makeStrip = (y: number, thickness: number, opacity: number): THREE.Mesh => {
    const geometry = new THREE.PlaneGeometry(fieldWidth, thickness * fieldScale);
    const material = new THREE.MeshBasicMaterial({
      color: ZONE_BOUNDARY_COLOR,
      transparent: true,
      opacity,
      side: THREE.DoubleSide,
      depthWrite: false,
      depthTest: false,
    });
    const mesh = new THREE.Mesh(geometry, material);
    mesh.position.set(0, y, ZONE_BOUNDARY_Z);
    mesh.renderOrder = 24;
    return mesh;
  };

  for (const ySign of [-1, 1]) {
    const y = ySign * FIELD_ZONE_BOUNDARY_Y * fieldScale;
    group.add(makeStrip(y, ZONE_BOUNDARY_THICKNESS, ZONE_BOUNDARY_OPACITY));
  }

  group.add(makeStrip(0, ZONE_MIDLINE_THICKNESS, ZONE_MIDLINE_OPACITY));

  scene.add(group);
  return group;
}
