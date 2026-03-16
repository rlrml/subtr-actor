import * as THREE from "three";
import type { ReplayPlayer } from "../../player/src/lib.ts";

type Vec3Like = {
  x: number;
  y: number;
  z: number;
};

export type FlipResetClipMeta = {
  playerId?: string;
  playerName?: string;
  eventFrame?: number;
  eventTime?: number;
  markerPosition?: Vec3Like;
};

function isObject(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function parseNumber(value: unknown): number | undefined {
  return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

function parseString(value: unknown): string | undefined {
  return typeof value === "string" && value.trim() !== "" ? value : undefined;
}

function parseVec3(value: unknown): Vec3Like | undefined {
  if (!isObject(value)) {
    return undefined;
  }

  const x = parseNumber(value.x);
  const y = parseNumber(value.y);
  const z = parseNumber(value.z);
  if (x === undefined || y === undefined || z === undefined) {
    return undefined;
  }

  return { x, y, z };
}

export function parseFlipResetClipMeta(meta: unknown): FlipResetClipMeta | null {
  if (!isObject(meta)) {
    return null;
  }

  return {
    playerId: parseString(meta.player_id),
    playerName: parseString(meta.player_name),
    eventFrame: parseNumber(meta.event_frame),
    eventTime: parseNumber(meta.event_time),
    markerPosition: parseVec3(meta.marker_position),
  };
}

export class FlipResetOverlay {
  private readonly disposableMaterials: THREE.Material[] = [];
  private readonly disposableGeometries: THREE.BufferGeometry[] = [];
  private readonly attachedObjects: THREE.Object3D[] = [];

  constructor(player: ReplayPlayer, meta: FlipResetClipMeta | null) {
    if (!meta) {
      return;
    }

    if (meta.markerPosition) {
      this.attachMarker(player, meta.markerPosition);
    }

    if (meta.playerId) {
      this.attachPlayerRing(player, meta.playerId);
    }
  }

  dispose(): void {
    for (const object of this.attachedObjects) {
      object.removeFromParent();
    }
    for (const geometry of this.disposableGeometries) {
      geometry.dispose();
    }
    for (const material of this.disposableMaterials) {
      material.dispose();
    }
    this.attachedObjects.length = 0;
    this.disposableGeometries.length = 0;
    this.disposableMaterials.length = 0;
  }

  private attachMarker(player: ReplayPlayer, markerPosition: Vec3Like): void {
    const markerGroup = new THREE.Group();

    const sphereGeometry = new THREE.SphereGeometry(18, 18, 18);
    const sphereMaterial = new THREE.MeshBasicMaterial({
      color: 0xfff0a8,
      transparent: true,
      opacity: 0.92,
      depthWrite: false,
    });
    const sphere = new THREE.Mesh(sphereGeometry, sphereMaterial);
    markerGroup.add(sphere);

    const ringMaterial = new THREE.MeshBasicMaterial({
      color: 0xffb347,
      transparent: true,
      opacity: 0.7,
      side: THREE.DoubleSide,
      depthWrite: false,
    });
    const horizontalRingGeometry = new THREE.TorusGeometry(52, 4, 10, 32);
    const horizontalRing = new THREE.Mesh(horizontalRingGeometry, ringMaterial);
    horizontalRing.rotateX(Math.PI / 2);
    markerGroup.add(horizontalRing);

    const verticalRingGeometry = new THREE.TorusGeometry(38, 3, 10, 24);
    const verticalRing = new THREE.Mesh(verticalRingGeometry, ringMaterial.clone());
    verticalRing.rotateY(Math.PI / 2);
    markerGroup.add(verticalRing);

    markerGroup.position.set(
      markerPosition.x,
      markerPosition.y,
      markerPosition.z
    );
    player.sceneState.replayRoot.add(markerGroup);

    this.attachedObjects.push(markerGroup);
    this.disposableGeometries.push(
      sphereGeometry,
      horizontalRingGeometry,
      verticalRingGeometry
    );
    this.disposableMaterials.push(
      sphereMaterial,
      ringMaterial,
      verticalRing.material as THREE.Material
    );
  }

  private attachPlayerRing(player: ReplayPlayer, playerId: string): void {
    const playerMesh = player.sceneState.playerMeshes.get(playerId);
    if (!playerMesh) {
      return;
    }

    const ringGeometry = new THREE.RingGeometry(145, 190, 28);
    ringGeometry.rotateX(Math.PI / 2);
    const ringMaterial = new THREE.MeshBasicMaterial({
      color: 0xffc857,
      transparent: true,
      opacity: 0.82,
      side: THREE.DoubleSide,
      depthWrite: false,
    });
    const ring = new THREE.Mesh(ringGeometry, ringMaterial);
    ring.position.set(0, 0, -38);
    playerMesh.add(ring);

    this.attachedObjects.push(ring);
    this.disposableGeometries.push(ringGeometry);
    this.disposableMaterials.push(ringMaterial);
  }
}
