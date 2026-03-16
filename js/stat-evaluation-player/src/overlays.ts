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
        color: ROLE_COLORS.even,
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

    for (const [, teamPlayers] of teams) {
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

// Dynamic threshold lines showing most-back/most-forward demarcation per team
const FIELD_HALF_X = 4120;
const WALL_HEIGHT = 1960;

interface ThresholdLine {
  group: THREE.Group;
  positionAttr: THREE.BufferAttribute;
}

function makeThresholdLineMesh(
  fieldScale: number,
  color: number,
  opacity: number,
): ThresholdLine {
  const hw = FIELD_HALF_X * fieldScale;
  const wh = WALL_HEIGHT * fieldScale;

  // U-shape: down left wall, across floor, up right wall
  const points = [
    new THREE.Vector3(-hw, 0, wh),
    new THREE.Vector3(-hw, 0, 0),
    new THREE.Vector3(hw, 0, 0),
    new THREE.Vector3(hw, 0, wh),
  ];
  const geometry = new THREE.BufferGeometry().setFromPoints(points);
  const material = new THREE.LineBasicMaterial({
    color,
    transparent: true,
    opacity,
    depthWrite: false,
  });
  const line = new THREE.Line(geometry, material);
  const group = new THREE.Group();
  group.add(line);
  group.visible = false;
  return {
    group,
    positionAttr: geometry.getAttribute("position") as THREE.BufferAttribute,
  };
}

export class ThresholdLineOverlay {
  private replay: ReplayModel;
  // 2 lines per team: back boundary and forward boundary
  private blueBack: ThresholdLine;
  private blueFront: ThresholdLine;
  private orangeBack: ThresholdLine;
  private orangeFront: ThresholdLine;

  constructor(scene: THREE.Scene, replay: ReplayModel, fieldScale: number) {
    this.replay = replay;
    // Back boundaries = red, forward boundaries = green (matches role ring colors)
    this.blueBack = makeThresholdLineMesh(fieldScale, 0xff5577, 0.4);
    this.blueFront = makeThresholdLineMesh(fieldScale, 0x55ff77, 0.4);
    this.orangeBack = makeThresholdLineMesh(fieldScale, 0xff5577, 0.4);
    this.orangeFront = makeThresholdLineMesh(fieldScale, 0x55ff77, 0.4);
    scene.add(this.blueBack.group);
    scene.add(this.blueFront.group);
    scene.add(this.orangeBack.group);
    scene.add(this.orangeFront.group);
  }

  update(info: FrameRenderInfo, fieldScale: number): void {
    const { frameIndex } = info;

    for (const isTeamZero of [true, false]) {
      const teamPlayers: number[] = []; // raw Y positions
      for (const player of this.replay.players) {
        if (player.isTeamZero !== isTeamZero) continue;
        const frame = player.frames[frameIndex];
        if (!frame?.position) continue;
        teamPlayers.push(frame.position.y);
      }

      const backLine = isTeamZero ? this.blueBack : this.orangeBack;
      const frontLine = isTeamZero ? this.blueFront : this.orangeFront;

      if (teamPlayers.length < 2) {
        backLine.group.visible = false;
        frontLine.group.visible = false;
        continue;
      }

      // For team 0: most back = min raw Y, most forward = max raw Y
      // For team 1: most back = max raw Y, most forward = min raw Y
      const rawMin = Math.min(...teamPlayers);
      const rawMax = Math.max(...teamPlayers);

      // Back boundary (the Y beyond which you're no longer "most back"):
      // Team 0: minY + threshold, Team 1: maxY - threshold
      const backBoundaryRawY = isTeamZero
        ? rawMin + MOST_BACK_FORWARD_THRESHOLD_Y
        : rawMax - MOST_BACK_FORWARD_THRESHOLD_Y;
      // Forward boundary (the Y beyond which you're no longer "most forward"):
      // Team 0: maxY - threshold, Team 1: minY + threshold
      const frontBoundaryRawY = isTeamZero
        ? rawMax - MOST_BACK_FORWARD_THRESHOLD_Y
        : rawMin + MOST_BACK_FORWARD_THRESHOLD_Y;

      backLine.group.position.y = backBoundaryRawY * fieldScale;
      backLine.group.visible = true;
      frontLine.group.position.y = frontBoundaryRawY * fieldScale;
      frontLine.group.visible = true;
    }
  }

  dispose(): void {
    for (const tl of [this.blueBack, this.blueFront, this.orangeBack, this.orangeFront]) {
      tl.group.removeFromParent();
      tl.group.traverse((node) => {
        if (node instanceof THREE.Line) {
          node.geometry.dispose();
          (node.material as THREE.Material).dispose();
        }
      });
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
