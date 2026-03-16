import * as THREE from "three";
import type { ReplayScene } from "../../player/src/scene.ts";
import type { PlayerStatsSnapshot, StatsFrame } from "./statsTimeline.ts";

const BLUE_TOUCH_COLOR = 0x59c3ff;
const ORANGE_TOUCH_COLOR = 0xffc15c;
const TOUCH_RING_INNER_RADIUS = 135;
const TOUCH_RING_OUTER_RADIUS = 210;
const TOUCH_RING_HEIGHT = 32;

export function getLastTouchPlayer(statsFrame: StatsFrame): PlayerStatsSnapshot | null {
  return statsFrame.players.find((player) => player.touch?.is_last_touch) ?? null;
}

export class LastTouchOverlay {
  private readonly ring: THREE.Mesh;
  private readonly material: THREE.MeshBasicMaterial;
  private readonly fieldScale: number;
  private readonly scene: ReplayScene;

  constructor(scene: ReplayScene, fieldScale: number) {
    this.scene = scene;
    this.fieldScale = fieldScale;
    this.material = new THREE.MeshBasicMaterial({
      color: BLUE_TOUCH_COLOR,
      transparent: true,
      opacity: 0.85,
      side: THREE.DoubleSide,
      depthWrite: false,
      depthTest: false,
    });
    this.ring = new THREE.Mesh(
      new THREE.RingGeometry(
        TOUCH_RING_INNER_RADIUS * fieldScale,
        TOUCH_RING_OUTER_RADIUS * fieldScale,
        48,
      ),
      this.material,
    );
    this.ring.rotation.x = -Math.PI / 2;
    this.ring.renderOrder = 40;
    this.ring.visible = false;
    this.scene.scene.add(this.ring);
  }

  update(
    playerId: string | null,
    isTeamZero: boolean | null,
    timeSinceTouch: number | null,
  ): void {
    if (!playerId || isTeamZero === null) {
      this.ring.visible = false;
      return;
    }

    const playerMesh = this.scene.playerMeshes.get(playerId);
    if (!playerMesh) {
      this.ring.visible = false;
      return;
    }

    this.material.color.setHex(isTeamZero ? BLUE_TOUCH_COLOR : ORANGE_TOUCH_COLOR);
    const age = timeSinceTouch ?? 0;
    const fade = Math.max(0.3, 1 - Math.min(age / 5, 0.7));
    const pulse = 1 + 0.08 * Math.sin(age * 10);
    this.material.opacity = 0.35 + 0.5 * fade;
    this.ring.scale.setScalar(pulse);
    this.ring.position.copy(playerMesh.position);
    this.ring.position.z += TOUCH_RING_HEIGHT * this.fieldScale;
    this.ring.visible = true;
  }

  dispose(): void {
    this.ring.removeFromParent();
    this.ring.geometry.dispose();
    this.material.dispose();
  }
}
