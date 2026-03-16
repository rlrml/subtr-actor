import * as THREE from "three";
import type { FrameRenderInfo, ReplayModel } from "../../player/src/types.ts";

// Must match Rust DEFAULT_MOST_BACK_FORWARD_THRESHOLD_Y (approx two car lengths)
const MOST_BACK_FORWARD_THRESHOLD_Y = 236.0;

const FIELD_HALF_X = 4120;
const FIELD_ZONE_BOUNDARY_Y = 2300.0;

const BAND_FILL_COLOR = 0xf6f6f3;
const BAND_FILL_OPACITY = 0.18;
const BAND_ARROW_COLOR = 0x111111;
const BLUE_TEAM_ACCENT_COLOR = 0x59c3ff;
const ORANGE_TEAM_ACCENT_COLOR = 0xffc15c;
const TEAM_STRIPE_OPACITY = 0.55;

const BAND_BASE_Z = 3;
const STRIPE_Z = 4;
const ARROW_Z = 5;

const TEAM_LANE_EDGE_INSET_X = 220;
const TEAM_LANE_GAP_X = 200;
const TEAM_STRIPE_WIDTH = 140;
const MIN_ARROW_LENGTH = 220;
const ARROW_HEAD_LENGTH = 100;
const ARROW_HEAD_WIDTH = 120;

type BandKind = "back" | "forward" | "other";

export interface TeamLaneBounds {
  minX: number;
  maxX: number;
  centerX: number;
  width: number;
}

export interface RelativeBandDescriptor {
  kind: BandKind;
  centerY: number;
  halfDepth: number;
  directions: number[];
}

interface DirectionMarker {
  group: THREE.Group;
  shaftGeom: THREE.PlaneGeometry;
  shaftMesh: THREE.Mesh;
  headGeom: THREE.ShapeGeometry;
  headMesh: THREE.Mesh;
  material: THREE.MeshBasicMaterial;
  headLength: number;
}

interface ThresholdZone {
  group: THREE.Group;
  floorGeom: THREE.PlaneGeometry;
  floorMesh: THREE.Mesh;
  floorMaterial: THREE.MeshBasicMaterial;
  stripeGeom: THREE.PlaneGeometry;
  stripeMesh: THREE.Mesh;
  stripeMaterial: THREE.MeshBasicMaterial;
  primaryMarker: DirectionMarker;
  secondaryMarker: DirectionMarker;
}

export function getTeamLaneBounds(isTeamZero: boolean): TeamLaneBounds {
  const innerEdge = TEAM_LANE_GAP_X / 2;

  if (isTeamZero) {
    const minX = -FIELD_HALF_X + TEAM_LANE_EDGE_INSET_X;
    const maxX = -innerEdge;
    return {
      minX,
      maxX,
      centerX: (minX + maxX) / 2,
      width: maxX - minX,
    };
  }

  const minX = innerEdge;
  const maxX = FIELD_HALF_X - TEAM_LANE_EDGE_INSET_X;
  return {
    minX,
    maxX,
    centerX: (minX + maxX) / 2,
    width: maxX - minX,
  };
}

export function computeTeamBandDescriptors(
  rawYs: number[],
  isTeamZero: boolean,
  threshold: number,
): RelativeBandDescriptor[] {
  if (rawYs.length < 2) return [];

  const rawMin = Math.min(...rawYs);
  const rawMax = Math.max(...rawYs);
  const spread = rawMax - rawMin;
  const backDirection = isTeamZero ? -1 : 1;
  const forwardDirection = -backDirection;

  if (spread <= threshold) {
    return [{
      kind: "other",
      centerY: (rawMin + rawMax) / 2,
      // Use the overlap of the back/front threshold bands for the collapsed other state.
      halfDepth: Math.max(threshold - spread / 2, threshold * 0.35),
      directions: [backDirection, forwardDirection],
    }];
  }

  return [
    {
      kind: "back",
      centerY: isTeamZero ? rawMin : rawMax,
      halfDepth: threshold,
      directions: [backDirection],
    },
    {
      kind: "forward",
      centerY: isTeamZero ? rawMax : rawMin,
      halfDepth: threshold,
      directions: [forwardDirection],
    },
  ];
}

function createArrowHeadGeometry(width: number, length: number): THREE.ShapeGeometry {
  const shape = new THREE.Shape();
  shape.moveTo(0, length / 2);
  shape.lineTo(width / 2, -length / 2);
  shape.lineTo(-width / 2, -length / 2);
  shape.closePath();
  return new THREE.ShapeGeometry(shape);
}

function makeDirectionMarker(fieldScale: number): DirectionMarker {
  const headLength = ARROW_HEAD_LENGTH * fieldScale;
  const material = new THREE.MeshBasicMaterial({
    color: BAND_ARROW_COLOR,
    transparent: true,
    opacity: 0.9,
    side: THREE.DoubleSide,
    depthWrite: false,
    depthTest: false,
  });

  const group = new THREE.Group();
  group.visible = false;

  const shaftGeom = new THREE.PlaneGeometry(TEAM_STRIPE_WIDTH * 0.55 * fieldScale, 1);
  const shaftMesh = new THREE.Mesh(shaftGeom, material);
  shaftMesh.position.z = ARROW_Z;
  shaftMesh.renderOrder = 22;
  group.add(shaftMesh);

  const headGeom = createArrowHeadGeometry(
    ARROW_HEAD_WIDTH * fieldScale,
    headLength,
  );
  const headMesh = new THREE.Mesh(headGeom, material);
  headMesh.position.z = ARROW_Z;
  headMesh.renderOrder = 23;
  group.add(headMesh);

  return {
    group,
    shaftGeom,
    shaftMesh,
    headGeom,
    headMesh,
    material,
    headLength,
  };
}

function setDirectionMarker(
  marker: DirectionMarker,
  centerX: number,
  totalLength: number,
  direction: number,
): void {
  const shaftLength = Math.max(totalLength - marker.headLength, marker.headLength * 0.2);

  marker.group.position.x = centerX;
  marker.group.rotation.z = direction > 0 ? 0 : Math.PI;

  marker.shaftMesh.scale.y = shaftLength;
  marker.shaftMesh.position.y = -marker.headLength / 2;
  marker.headMesh.position.y = totalLength / 2 - marker.headLength / 2;
  marker.group.visible = true;
}

function hideDirectionMarker(marker: DirectionMarker): void {
  marker.group.visible = false;
}

function makeThresholdZone(fieldScale: number, stripeColor: number): ThresholdZone {
  const group = new THREE.Group();
  group.visible = false;

  const floorMaterial = new THREE.MeshBasicMaterial({
    color: BAND_FILL_COLOR,
    transparent: true,
    opacity: BAND_FILL_OPACITY,
    side: THREE.DoubleSide,
    depthWrite: false,
    depthTest: false,
  });

  const floorGeom = new THREE.PlaneGeometry(1, 1);
  const floorMesh = new THREE.Mesh(floorGeom, floorMaterial);
  floorMesh.position.z = BAND_BASE_Z;
  floorMesh.renderOrder = 20;
  group.add(floorMesh);

  const stripeMaterial = new THREE.MeshBasicMaterial({
    color: stripeColor,
    transparent: true,
    opacity: TEAM_STRIPE_OPACITY,
    side: THREE.DoubleSide,
    depthWrite: false,
    depthTest: false,
  });

  const stripeGeom = new THREE.PlaneGeometry(1, 1);
  const stripeMesh = new THREE.Mesh(stripeGeom, stripeMaterial);
  stripeMesh.position.z = STRIPE_Z;
  stripeMesh.renderOrder = 21;
  group.add(stripeMesh);

  const primaryMarker = makeDirectionMarker(fieldScale);
  const secondaryMarker = makeDirectionMarker(fieldScale);
  group.add(primaryMarker.group);
  group.add(secondaryMarker.group);

  return {
    group,
    floorGeom,
    floorMesh,
    floorMaterial,
    stripeGeom,
    stripeMesh,
    stripeMaterial,
    primaryMarker,
    secondaryMarker,
  };
}

function hideZone(zone: ThresholdZone): void {
  zone.group.visible = false;
  hideDirectionMarker(zone.primaryMarker);
  hideDirectionMarker(zone.secondaryMarker);
}

function updateZone(
  zone: ThresholdZone,
  descriptor: RelativeBandDescriptor,
  lane: TeamLaneBounds,
  fieldScale: number,
): void {
  const depth = descriptor.halfDepth * 2 * fieldScale;
  const fieldWidth = FIELD_HALF_X * 2 * fieldScale;
  const laneWidth = lane.width * fieldScale;
  const laneCenterX = lane.centerX * fieldScale;
  const stripeWidth = TEAM_STRIPE_WIDTH * fieldScale;
  const maxArrowLength = Math.max(depth - 32 * fieldScale, zone.primaryMarker.headLength * 1.15);
  const arrowLength = Math.min(
    maxArrowLength,
    Math.max(MIN_ARROW_LENGTH * fieldScale, depth * 0.6),
  );

  zone.group.position.y = descriptor.centerY * fieldScale;

  zone.floorMesh.position.x = 0;
  zone.floorMesh.scale.set(fieldWidth, depth, 1);

  zone.stripeMesh.position.x = laneCenterX;
  zone.stripeMesh.scale.set(stripeWidth, depth, 1);

  hideDirectionMarker(zone.primaryMarker);
  hideDirectionMarker(zone.secondaryMarker);

  if (descriptor.directions.length === 1) {
    setDirectionMarker(zone.primaryMarker, laneCenterX, arrowLength, descriptor.directions[0]!);
  } else {
    const arrowOffsetX = laneWidth * 0.18;
    setDirectionMarker(
      zone.primaryMarker,
      laneCenterX - arrowOffsetX,
      arrowLength,
      descriptor.directions[0]!,
    );
    setDirectionMarker(
      zone.secondaryMarker,
      laneCenterX + arrowOffsetX,
      arrowLength,
      descriptor.directions[1]!,
    );
  }

  zone.group.visible = true;
}

function disposeDirectionMarker(marker: DirectionMarker): void {
  marker.group.removeFromParent();
  marker.shaftGeom.dispose();
  marker.headGeom.dispose();
  marker.material.dispose();
}

export class ThresholdZoneOverlay {
  private replay: ReplayModel;
  private blueBack: ThresholdZone;
  private blueForward: ThresholdZone;
  private blueOther: ThresholdZone;
  private orangeBack: ThresholdZone;
  private orangeForward: ThresholdZone;
  private orangeOther: ThresholdZone;

  constructor(scene: THREE.Scene, replay: ReplayModel, fieldScale: number) {
    this.replay = replay;
    this.blueBack = makeThresholdZone(fieldScale, BLUE_TEAM_ACCENT_COLOR);
    this.blueForward = makeThresholdZone(fieldScale, BLUE_TEAM_ACCENT_COLOR);
    this.blueOther = makeThresholdZone(fieldScale, BLUE_TEAM_ACCENT_COLOR);
    this.orangeBack = makeThresholdZone(fieldScale, ORANGE_TEAM_ACCENT_COLOR);
    this.orangeForward = makeThresholdZone(fieldScale, ORANGE_TEAM_ACCENT_COLOR);
    this.orangeOther = makeThresholdZone(fieldScale, ORANGE_TEAM_ACCENT_COLOR);

    for (const zone of this.getZones()) {
      scene.add(zone.group);
    }
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

      const lane = getTeamLaneBounds(isTeamZero);
      const teamZones = this.getTeamZones(isTeamZero);
      for (const zone of teamZones.values()) {
        hideZone(zone);
      }

      if (teamRosterCount < 2 || rawYs.length !== teamRosterCount) {
        continue;
      }

      const descriptors = computeTeamBandDescriptors(rawYs, isTeamZero, threshold);
      for (const descriptor of descriptors) {
        const zone = teamZones.get(descriptor.kind);
        if (!zone) continue;
        updateZone(zone, descriptor, lane, fieldScale);
      }
    }
  }

  dispose(): void {
    for (const zone of this.getZones()) {
      zone.group.removeFromParent();
      zone.floorGeom.dispose();
      zone.floorMaterial.dispose();
      zone.stripeGeom.dispose();
      zone.stripeMaterial.dispose();
      disposeDirectionMarker(zone.primaryMarker);
      disposeDirectionMarker(zone.secondaryMarker);
    }
  }

  private getTeamZones(isTeamZero: boolean): Map<BandKind, ThresholdZone> {
    if (isTeamZero) {
      return new Map<BandKind, ThresholdZone>([
        ["back", this.blueBack],
        ["forward", this.blueForward],
        ["other", this.blueOther],
      ]);
    }

    return new Map<BandKind, ThresholdZone>([
      ["back", this.orangeBack],
      ["forward", this.orangeForward],
      ["other", this.orangeOther],
    ]);
  }

  private getZones(): ThresholdZone[] {
    return [
      this.blueBack,
      this.blueForward,
      this.blueOther,
      this.orangeBack,
      this.orangeForward,
      this.orangeOther,
    ];
  }
}

export function createZoneBoundaryLines(
  scene: THREE.Scene,
  fieldScale: number,
): THREE.Group {
  const group = new THREE.Group();
  const fieldHalfWidth = FIELD_HALF_X * fieldScale;

  const material = new THREE.LineBasicMaterial({
    color: 0xffffff,
    transparent: true,
    opacity: 0.25,
  });

  for (const ySign of [-1, 1]) {
    const y = ySign * FIELD_ZONE_BOUNDARY_Y * fieldScale;
    const points = [
      new THREE.Vector3(-fieldHalfWidth, y, 2),
      new THREE.Vector3(fieldHalfWidth, y, 2),
    ];
    const geometry = new THREE.BufferGeometry().setFromPoints(points);
    const line = new THREE.Line(geometry, material);
    group.add(line);
  }

  const midPoints = [
    new THREE.Vector3(-fieldHalfWidth, 0, 2),
    new THREE.Vector3(fieldHalfWidth, 0, 2),
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
