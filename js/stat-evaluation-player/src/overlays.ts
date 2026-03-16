import * as THREE from "three";
import type { ReplayModel } from "../../player/src/types.ts";
import type { ReplayScene } from "../../player/src/scene.ts";
import type { FrameRenderInfo } from "../../player/src/types.ts";

const ROLE_COLORS = {
  back: 0xff3333,
  forward: 0x33ff33,
  even: 0x888888,
  mid: 0xffaa33,
} as const;

// Must match Rust DEFAULT_MOST_BACK_FORWARD_THRESHOLD_Y (approx one car length)
const MOST_BACK_FORWARD_THRESHOLD_Y = 118.0;

type Role = "back" | "forward" | "even" | "mid";

interface PlayerRoleRing {
  ring: THREE.Mesh;
  material: THREE.MeshBasicMaterial;
}

export class RoleOverlay {
  private rings = new Map<string, PlayerRoleRing>();
  private replay: ReplayModel;

  constructor(
    sceneState: ReplayScene,
    replay: ReplayModel,
  ) {
    this.replay = replay;

    for (const player of replay.players) {
      const mesh = sceneState.playerMeshes.get(player.id);
      if (!mesh) continue;

      const material = new THREE.MeshBasicMaterial({
        color: ROLE_COLORS.mid,
        transparent: true,
        opacity: 0.6,
        side: THREE.DoubleSide,
        depthWrite: false,
      });

      const geometry = new THREE.RingGeometry(140, 180, 24);
      geometry.rotateX(Math.PI / 2);
      const ring = new THREE.Mesh(geometry, material);
      ring.position.set(0, 0, -40);
      mesh.add(ring);

      this.rings.set(player.id, { ring, material });
    }
  }

  update(info: FrameRenderInfo): void {
    const { frameIndex } = info;

    // Group players by team and get their Y positions
    const teams = new Map<boolean, Array<{ id: string; y: number }>>();

    for (const player of this.replay.players) {
      const frame = player.frames[frameIndex];
      if (!frame?.position) continue;

      // normalized_y: for team 0, use raw Y; for team 1, negate Y
      const normalizedY = player.isTeamZero
        ? frame.position.y
        : -frame.position.y;

      const team = teams.get(player.isTeamZero) ?? [];
      team.push({ id: player.id, y: normalizedY });
      teams.set(player.isTeamZero, team);
    }

    for (const [isTeamZero, teamPlayers] of teams) {
      const teamRosterCount = this.replay.players.filter(
        (player) => player.isTeamZero === isTeamZero,
      ).length;
      if (teamRosterCount < 2 || teamPlayers.length !== teamRosterCount) {
        for (const p of teamPlayers) {
          const entry = this.rings.get(p.id);
          if (!entry) continue;
          entry.material.color.setHex(ROLE_COLORS.mid);
        }
        continue;
      }

      teamPlayers.sort((a, b) => a.y - b.y);

      const minY = teamPlayers[0]?.y ?? 0;
      const maxY = teamPlayers[teamPlayers.length - 1]?.y ?? 0;
      const spread = maxY - minY;

      const roles = new Map<string, Role>();

      if (spread <= MOST_BACK_FORWARD_THRESHOLD_Y) {
        for (const p of teamPlayers) {
          roles.set(p.id, "even");
        }
      } else {
        for (const p of teamPlayers) {
          const nearBack = (p.y - minY) <= MOST_BACK_FORWARD_THRESHOLD_Y;
          const nearFront = (maxY - p.y) <= MOST_BACK_FORWARD_THRESHOLD_Y;

          if (nearBack && !nearFront) {
            roles.set(p.id, "back");
          } else if (nearFront && !nearBack) {
            roles.set(p.id, "forward");
          } else {
            roles.set(p.id, "mid");
          }
        }
      }

      for (const [playerId, role] of roles) {
        const entry = this.rings.get(playerId);
        if (!entry) continue;
        entry.material.color.setHex(ROLE_COLORS[role]);
      }
    }
  }

  dispose(): void {
    for (const [, { ring, material }] of this.rings) {
      ring.geometry.dispose();
      material.dispose();
      ring.removeFromParent();
    }
    this.rings.clear();
  }
}

// Dynamic threshold zones showing most-back/most-forward classification bands
const FIELD_HALF_X = 4120;
const WALL_HEIGHT = 1960;

interface ThresholdZone {
  group: THREE.Group;
  floorMesh: THREE.Mesh;
  leftWallMesh: THREE.Mesh;
  rightWallMesh: THREE.Mesh;
  floorGeom: THREE.PlaneGeometry;
  leftWallGeom: THREE.PlaneGeometry;
  rightWallGeom: THREE.PlaneGeometry;
  material: THREE.MeshBasicMaterial;
}

function makeThresholdZone(
  fieldScale: number,
  color: number,
  opacity: number,
): ThresholdZone {
  const hw = FIELD_HALF_X * fieldScale;
  const wh = WALL_HEIGHT * fieldScale;
  // Zone depth (Y extent) will be set dynamically via scale, so create at unit size
  const zoneDepth = 1; // will be scaled

  const material = new THREE.MeshBasicMaterial({
    color,
    transparent: true,
    opacity,
    side: THREE.DoubleSide,
    depthWrite: false,
  });

  const group = new THREE.Group();
  group.visible = false;

  // Floor plane: width = full field, depth = 1 (scaled dynamically)
  const floorGeom = new THREE.PlaneGeometry(hw * 2, zoneDepth);
  const floorMesh = new THREE.Mesh(floorGeom, material);
  floorMesh.position.z = 2; // slightly above floor to avoid z-fighting
  group.add(floorMesh);

  // Left wall strip
  const leftWallGeom = new THREE.PlaneGeometry(zoneDepth, wh);
  const leftWallMesh = new THREE.Mesh(leftWallGeom, material);
  leftWallMesh.position.set(-hw, 0, wh / 2);
  leftWallMesh.rotation.y = Math.PI / 2;
  group.add(leftWallMesh);

  // Right wall strip
  const rightWallGeom = new THREE.PlaneGeometry(zoneDepth, wh);
  const rightWallMesh = new THREE.Mesh(rightWallGeom, material);
  rightWallMesh.position.set(hw, 0, wh / 2);
  rightWallMesh.rotation.y = Math.PI / 2;
  group.add(rightWallMesh);

  return { group, floorMesh, leftWallMesh, rightWallMesh, floorGeom, leftWallGeom, rightWallGeom, material };
}

function updateZonePosition(zone: ThresholdZone, centerY: number, halfDepth: number, fieldScale: number): void {
  const depth = halfDepth * 2 * fieldScale;
  const wh = WALL_HEIGHT * fieldScale;
  const hw = FIELD_HALF_X * fieldScale;

  zone.group.position.y = centerY * fieldScale;

  // Floor: scale Y to match zone depth
  zone.floorMesh.scale.y = depth;

  // Wall strips: scale X to match zone depth, reposition
  zone.leftWallMesh.scale.x = depth;
  zone.leftWallMesh.position.set(-hw, 0, wh / 2);

  zone.rightWallMesh.scale.x = depth;
  zone.rightWallMesh.position.set(hw, 0, wh / 2);

  zone.group.visible = true;
}

export class ThresholdZoneOverlay {
  private replay: ReplayModel;
  private blueBack: ThresholdZone;
  private blueFront: ThresholdZone;
  private orangeBack: ThresholdZone;
  private orangeFront: ThresholdZone;

  constructor(scene: THREE.Scene, replay: ReplayModel, fieldScale: number) {
    this.replay = replay;
    // Back zones = red tint, forward zones = green tint
    this.blueBack = makeThresholdZone(fieldScale, 0xff3333, 0.12);
    this.blueFront = makeThresholdZone(fieldScale, 0x33ff33, 0.12);
    this.orangeBack = makeThresholdZone(fieldScale, 0xff3333, 0.12);
    this.orangeFront = makeThresholdZone(fieldScale, 0x33ff33, 0.12);
    scene.add(this.blueBack.group);
    scene.add(this.blueFront.group);
    scene.add(this.orangeBack.group);
    scene.add(this.orangeFront.group);
  }

  update(info: FrameRenderInfo, fieldScale: number): void {
    const { frameIndex } = info;
    const threshold = MOST_BACK_FORWARD_THRESHOLD_Y;

    for (const isTeamZero of [true, false]) {
      const teamRosterCount = this.replay.players.filter(
        (player) => player.isTeamZero === isTeamZero,
      ).length;
      const rawYs: number[] = [];
      for (const player of this.replay.players) {
        if (player.isTeamZero !== isTeamZero) continue;
        const frame = player.frames[frameIndex];
        if (!frame?.position) continue;
        rawYs.push(frame.position.y);
      }

      const backZone = isTeamZero ? this.blueBack : this.orangeBack;
      const frontZone = isTeamZero ? this.blueFront : this.orangeFront;

      if (teamRosterCount < 2 || rawYs.length !== teamRosterCount) {
        backZone.group.visible = false;
        frontZone.group.visible = false;
        continue;
      }

      const rawMin = Math.min(...rawYs);
      const rawMax = Math.max(...rawYs);

      // For team 0: most back player is at rawMin, most forward at rawMax
      // For team 1: most back player is at rawMax, most forward at rawMin
      // Zone is centered on the extreme player, extending ±threshold
      const backPlayerY = isTeamZero ? rawMin : rawMax;
      const frontPlayerY = isTeamZero ? rawMax : rawMin;

      updateZonePosition(backZone, backPlayerY, threshold, fieldScale);
      updateZonePosition(frontZone, frontPlayerY, threshold, fieldScale);
    }
  }

  dispose(): void {
    for (const zone of [this.blueBack, this.blueFront, this.orangeBack, this.orangeFront]) {
      zone.group.removeFromParent();
      zone.floorGeom.dispose();
      zone.leftWallGeom.dispose();
      zone.rightWallGeom.dispose();
      zone.material.dispose();
    }
  }
}

const FIELD_ZONE_BOUNDARY_Y = 2300.0;

export function createZoneBoundaryLines(
  scene: THREE.Scene,
  fieldScale: number,
): THREE.Group {
  const group = new THREE.Group();
  const FIELD_HALF_WIDTH = 4120 * fieldScale;

  const material = new THREE.LineBasicMaterial({
    color: 0xffffff,
    transparent: true,
    opacity: 0.25,
  });

  for (const ySign of [-1, 1]) {
    const y = ySign * FIELD_ZONE_BOUNDARY_Y * fieldScale;
    const points = [
      new THREE.Vector3(-FIELD_HALF_WIDTH, y, 2),
      new THREE.Vector3(FIELD_HALF_WIDTH, y, 2),
    ];
    const geometry = new THREE.BufferGeometry().setFromPoints(points);
    const line = new THREE.Line(geometry, material);
    group.add(line);
  }

  // Midfield line
  const midPoints = [
    new THREE.Vector3(-FIELD_HALF_WIDTH, 0, 2),
    new THREE.Vector3(FIELD_HALF_WIDTH, 0, 2),
  ];
  const midGeometry = new THREE.BufferGeometry().setFromPoints(midPoints);
  const midMaterial = new THREE.LineBasicMaterial({
    color: 0xffffff,
    transparent: true,
    opacity: 0.15,
  });
  group.add(new THREE.Line(midGeometry, midMaterial));

  scene.add(group);
  return group;
}
