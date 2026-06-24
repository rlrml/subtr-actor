import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import type { ReplayModel } from "./types";
import { boostAmountToPercent } from "./boost-units";
import { getReplayHitboxOverlayTransform, type ReplayHitboxSpec } from "./hitboxes";

const HITBOX_OVERLAY_FILL_OPACITY = 0.08;
const HITBOX_ONLY_FILL_OPACITY = 0.22;
const HITBOX_OVERLAY_DARKEN = 0.94;

/** A steerable/spinnable wheel on the example car mesh. */
export interface CarWheel {
  /** Steering pivot — rotated about local Z (car up) for front-wheel steer. */
  pivot: THREE.Group;
  /** Cylinder mesh — spun about its axle for roll. */
  wheel: THREE.Mesh;
  isFront: boolean;
}

/** Effective wheel radius in model (uu) space: geometry radius × inner scale. */
export const EXAMPLE_CAR_WHEEL_RADIUS_UU = 70 * 0.35;

/** Returns the steerable wheels attached to an example car mesh, if any. */
export function getCarWheels(mesh: THREE.Object3D): CarWheel[] | undefined {
  const wheels = (mesh.userData as { wheels?: CarWheel[] }).wheels;
  return Array.isArray(wheels) ? wheels : undefined;
}

export interface ReplayScene {
  scene: THREE.Scene;
  replayRoot: THREE.Group;
  camera: THREE.PerspectiveCamera;
  renderer: THREE.WebGLRenderer;
  controls: OrbitControls;
  resize: () => void;
  dispose: () => void;
  ballMesh: THREE.Mesh;
  playerMeshes: Map<string, THREE.Object3D>;
  playerBodyMeshes: Map<string, THREE.Object3D>;
  playerHitboxes: Map<string, THREE.Object3D>;
  playerBoostTrails: Map<string, THREE.Group>;
  playerBoostMeters: Map<string, BoostMeter>;
  playerDemoIndicators: Map<string, DemoIndicator>;
  updateWallVisibility: () => void;
}

interface WallPanel {
  mesh: THREE.Mesh;
  material: THREE.Material;
  outwardLocal: THREE.Vector3;
  fixedOpacity: number | null;
}

const OPAQUE_WALL_OPACITY = 1;
const OUTSIDE_WALL_OPACITY = 0.32;
const BALL_TEXTURE_SIZE = 1024;
const KEYBOARD_PAN_SPEED = 16;
const MAX_RENDERER_PIXEL_RATIO = 1.5;

function createWallMaterial(color: number): THREE.MeshBasicMaterial {
  const material = new THREE.MeshBasicMaterial({
    color,
    transparent: true,
    opacity: OPAQUE_WALL_OPACITY,
    side: THREE.DoubleSide,
  });
  material.forceSinglePass = true;
  return material;
}

function createFloorMaterial(color: number): THREE.MeshLambertMaterial {
  return new THREE.MeshLambertMaterial({
    color,
    side: THREE.DoubleSide,
    transparent: true,
    opacity: OPAQUE_WALL_OPACITY,
  });
}

function createVerticalWallBox(
  length: number,
  height: number,
  thickness: number,
  material: THREE.Material,
): THREE.Mesh {
  return new THREE.Mesh(new THREE.BoxGeometry(length, thickness, height, 6, 1, 6), material);
}

export function getHitboxOverlayColor(lineColor: string): THREE.Color {
  return new THREE.Color(lineColor).lerp(new THREE.Color(0x000000), HITBOX_OVERLAY_DARKEN);
}

function drawBallLatitudeBand(
  context: CanvasRenderingContext2D,
  width: number,
  height: number,
  normalizedY: number,
  amplitude: number,
  phase: number,
  lineWidth: number,
  color: string,
): void {
  context.beginPath();
  for (let x = 0; x <= width; x += 8) {
    const t = x / width;
    const y =
      normalizedY * height +
      Math.sin(t * Math.PI * 2 + phase) * amplitude +
      Math.sin(t * Math.PI * 4 + phase * 0.5) * amplitude * 0.35;
    if (x === 0) {
      context.moveTo(x, y);
    } else {
      context.lineTo(x, y);
    }
  }
  context.lineWidth = lineWidth;
  context.strokeStyle = color;
  context.stroke();
}

function drawBallLongitudeBand(
  context: CanvasRenderingContext2D,
  width: number,
  height: number,
  normalizedX: number,
  amplitude: number,
  phase: number,
  lineWidth: number,
  color: string,
): void {
  context.beginPath();
  for (let y = 0; y <= height; y += 8) {
    const t = y / height;
    const x =
      normalizedX * width +
      Math.sin(t * Math.PI * 2 + phase) * amplitude +
      Math.sin(t * Math.PI * 6 + phase * 0.3) * amplitude * 0.18;
    if (y === 0) {
      context.moveTo(x, y);
    } else {
      context.lineTo(x, y);
    }
  }
  context.lineWidth = lineWidth;
  context.strokeStyle = color;
  context.stroke();
}

function drawBallMarker(
  context: CanvasRenderingContext2D,
  centerX: number,
  centerY: number,
  radius: number,
  fillStyle: string,
  strokeStyle: string,
): void {
  context.beginPath();
  context.arc(centerX, centerY, radius, 0, Math.PI * 2);
  context.fillStyle = fillStyle;
  context.fill();
  context.lineWidth = Math.max(6, radius * 0.15);
  context.strokeStyle = strokeStyle;
  context.stroke();
}

function createBallTexture(renderer: THREE.WebGLRenderer): THREE.CanvasTexture {
  const canvas = document.createElement("canvas");
  canvas.width = BALL_TEXTURE_SIZE;
  canvas.height = BALL_TEXTURE_SIZE;
  const context = canvas.getContext("2d");
  if (!context) {
    throw new Error("Unable to create ball texture canvas");
  }

  const { width, height } = canvas;
  const background = context.createLinearGradient(0, 0, width, height);
  background.addColorStop(0, "#faf7ee");
  background.addColorStop(0.55, "#e7e1d0");
  background.addColorStop(1, "#d5cfbe");
  context.fillStyle = background;
  context.fillRect(0, 0, width, height);

  context.globalAlpha = 0.22;
  for (let row = 0; row < 28; row += 1) {
    const y = (row / 27) * height;
    context.fillStyle = row % 2 === 0 ? "#ffffff" : "#d3cbb6";
    context.fillRect(0, y, width, height / 54);
  }
  context.globalAlpha = 1;

  const seamColor = "#2d313b";
  context.lineCap = "round";
  drawBallLatitudeBand(context, width, height, 0.24, 22, 0.35, 18, seamColor);
  drawBallLatitudeBand(context, width, height, 0.5, 14, 1.1, 20, seamColor);
  drawBallLatitudeBand(context, width, height, 0.77, 20, 2.35, 18, seamColor);
  drawBallLongitudeBand(context, width, height, 0.2, 24, 0.2, 18, seamColor);
  drawBallLongitudeBand(context, width, height, 0.48, 18, 1.6, 18, seamColor);
  drawBallLongitudeBand(context, width, height, 0.76, 26, 2.7, 18, seamColor);

  context.globalAlpha = 0.92;
  drawBallMarker(context, width * 0.28, height * 0.32, 88, "#f1a63a", "#fff4d7");
  drawBallMarker(context, width * 0.68, height * 0.6, 72, "#4db0ff", "#eef8ff");
  drawBallMarker(context, width * 0.76, height * 0.2, 54, "#1f232c", "#f0ece1");
  context.globalAlpha = 1;

  context.beginPath();
  context.moveTo(width * 0.08, height * 0.86);
  context.quadraticCurveTo(width * 0.28, height * 0.72, width * 0.42, height * 0.8);
  context.quadraticCurveTo(width * 0.58, height * 0.9, width * 0.82, height * 0.78);
  context.lineWidth = 24;
  context.strokeStyle = "rgba(255, 246, 220, 0.9)";
  context.stroke();

  const texture = new THREE.CanvasTexture(canvas);
  texture.colorSpace = THREE.SRGBColorSpace;
  texture.anisotropy = Math.min(8, renderer.capabilities.getMaxAnisotropy());
  return texture;
}

function createHorizontalWallBox(
  width: number,
  depth: number,
  thickness: number,
  material: THREE.Material,
): THREE.Mesh {
  return new THREE.Mesh(new THREE.BoxGeometry(width, depth, thickness, 6, 6, 1), material);
}

function createExampleSoccarField(scale: number): {
  stadium: THREE.Group;
  wallPanels: WallPanel[];
} {
  type MirrorSign = 1 | -1;

  const SOCCAR_YSIZE = 10280 * scale;
  const SOCCAR_XSIZE = 8240 * scale;
  const SOCCAR_DEPTH = 1960 * scale;
  const STADIUM_CORNER = 1000 * scale;
  const GOAL_WIDTH = 1900 * scale;
  const GOAL_HEIGHT = 800 * scale;
  const GOAL_DEPTH = 900 * scale;
  const WALL_THICKNESS = Math.max(1, scale);
  const wallPanels: WallPanel[] = [];
  const mirroredSigns: MirrorSign[] = [1, -1];

  function registerWall(
    mesh: THREE.Mesh,
    outwardLocal: THREE.Vector3,
    fixedOpacity: number | null = null,
  ): THREE.Mesh {
    const material = (mesh.material as THREE.Material).clone();
    mesh.material = material;
    wallPanels.push({
      mesh,
      material,
      outwardLocal: outwardLocal.clone().normalize(),
      fixedOpacity,
    });
    return mesh;
  }

  function createBackWall(color: number): THREE.Group {
    const backWall = new THREE.Group();
    const wallMaterial = createWallMaterial(color);
    const sideWallWidth = SOCCAR_XSIZE / 2 - STADIUM_CORNER - GOAL_WIDTH / 2;

    const cornerWidth = Math.sqrt(2 * Math.pow(STADIUM_CORNER, 2));

    for (const xSign of mirroredSigns) {
      const sideWall = registerWall(
        createVerticalWallBox(sideWallWidth, SOCCAR_DEPTH, WALL_THICKNESS, wallMaterial),
        new THREE.Vector3(0, 1, 0),
      );
      sideWall.position.set(xSign * (sideWallWidth / 2 + GOAL_WIDTH / 2), 0, SOCCAR_DEPTH / 2);
      backWall.add(sideWall);

      const corner = registerWall(
        createVerticalWallBox(cornerWidth, SOCCAR_DEPTH, WALL_THICKNESS, wallMaterial),
        new THREE.Vector3(0, 1, 0),
      );
      corner.position.set(
        xSign * (SOCCAR_XSIZE / 2 - STADIUM_CORNER / 2),
        -STADIUM_CORNER / 2,
        SOCCAR_DEPTH / 2,
      );
      corner.rotateZ((-xSign * Math.PI) / 4);
      backWall.add(corner);
    }

    const top = registerWall(
      createVerticalWallBox(GOAL_WIDTH, SOCCAR_DEPTH - GOAL_HEIGHT, WALL_THICKNESS, wallMaterial),
      new THREE.Vector3(0, 1, 0),
    );
    top.position.set(0, 0, SOCCAR_DEPTH / 2 + GOAL_HEIGHT / 2);
    backWall.add(top);

    return backWall;
  }

  function createHalf(color: number, mirrored: boolean): THREE.Group {
    const res = new THREE.Group();
    const floorOutline: Array<[number, number]> = [
      [SOCCAR_XSIZE / 2, 0],
      [-SOCCAR_XSIZE / 2, 0],
      [-SOCCAR_XSIZE / 2, SOCCAR_YSIZE / 2 - STADIUM_CORNER],
      [-SOCCAR_XSIZE / 2 + STADIUM_CORNER, SOCCAR_YSIZE / 2],
      [-GOAL_WIDTH / 2, SOCCAR_YSIZE / 2],
      [-GOAL_WIDTH / 2, SOCCAR_YSIZE / 2 + GOAL_DEPTH],
      [GOAL_WIDTH / 2, SOCCAR_YSIZE / 2 + GOAL_DEPTH],
      [GOAL_WIDTH / 2, SOCCAR_YSIZE / 2],
      [SOCCAR_XSIZE / 2 - STADIUM_CORNER, SOCCAR_YSIZE / 2],
      [SOCCAR_XSIZE / 2, SOCCAR_YSIZE / 2 - STADIUM_CORNER],
      [SOCCAR_XSIZE / 2, 0],
    ];
    const floor = new THREE.Shape();
    floorOutline.forEach(([x, y], index) => {
      if (index === 0) {
        floor.moveTo(x, y);
      } else {
        floor.lineTo(x, y);
      }
    });

    const floorMaterial = createFloorMaterial(color);
    const wallMaterial = createWallMaterial(color);

    const floorMesh = registerWall(
      new THREE.Mesh(new THREE.ShapeGeometry(floor), floorMaterial),
      new THREE.Vector3(0, 0, -1),
    );
    floorMesh.receiveShadow = true;
    res.add(floorMesh);

    for (const xSign of mirroredSigns) {
      const goalPost = registerWall(
        createVerticalWallBox(GOAL_DEPTH, GOAL_HEIGHT, WALL_THICKNESS, wallMaterial),
        new THREE.Vector3(0, -xSign, 0),
        OUTSIDE_WALL_OPACITY,
      );
      goalPost.position.set(
        (xSign * GOAL_WIDTH) / 2,
        SOCCAR_YSIZE / 2 + GOAL_DEPTH / 2,
        GOAL_HEIGHT / 2,
      );
      goalPost.rotateZ(Math.PI / 2);
      res.add(goalPost);
    }

    const goalRoof = registerWall(
      createHorizontalWallBox(GOAL_WIDTH, GOAL_DEPTH, WALL_THICKNESS, wallMaterial),
      new THREE.Vector3(0, 0, 1),
      OUTSIDE_WALL_OPACITY,
    );
    goalRoof.position.set(0, SOCCAR_YSIZE / 2 + GOAL_DEPTH / 2, GOAL_HEIGHT);
    res.add(goalRoof);

    const goalBack = registerWall(
      createVerticalWallBox(GOAL_WIDTH, GOAL_HEIGHT, WALL_THICKNESS, wallMaterial),
      new THREE.Vector3(0, 1, 0),
      OUTSIDE_WALL_OPACITY,
    );
    goalBack.position.set(0, SOCCAR_YSIZE / 2 + GOAL_DEPTH, GOAL_HEIGHT / 2);
    res.add(goalBack);

    const backWall = createBackWall(color);
    backWall.position.y = SOCCAR_YSIZE / 2;
    res.add(backWall);

    for (const xSign of mirroredSigns) {
      const sideWall = registerWall(
        createVerticalWallBox(
          SOCCAR_YSIZE / 2 - STADIUM_CORNER,
          SOCCAR_DEPTH,
          WALL_THICKNESS,
          wallMaterial,
        ),
        new THREE.Vector3(0, -xSign, 0),
      );
      sideWall.position.set(
        (xSign * SOCCAR_XSIZE) / 2,
        (SOCCAR_YSIZE / 2 - STADIUM_CORNER) / 2,
        SOCCAR_DEPTH / 2,
      );
      sideWall.rotateZ(Math.PI / 2);
      res.add(sideWall);
    }

    if (mirrored) {
      res.rotateZ(Math.PI);
    }

    return res;
  }

  const stadium = new THREE.Group();
  stadium.add(createHalf(0xffe8b3, false));
  stadium.add(createHalf(0x7fe3ff, true));
  return { stadium, wallPanels };
}

export function createExampleSoccarStadium(scale: number): THREE.Group {
  return createExampleSoccarField(scale).stadium;
}

function createHitboxOverlay(hitbox: ReplayHitboxSpec, lineColor: string): THREE.Group {
  const transform = getReplayHitboxOverlayTransform(hitbox);
  const overlayColor = getHitboxOverlayColor(lineColor);
  const group = new THREE.Group();
  group.name = `${hitbox.kind}-hitbox-overlay`;
  group.visible = false;
  group.position.set(...transform.position);
  group.rotateY(THREE.MathUtils.degToRad(transform.rotationYDegrees));

  const geometry = new THREE.BoxGeometry(...transform.dimensions);
  const fillMaterial = new THREE.MeshBasicMaterial({
    color: overlayColor,
    transparent: true,
    opacity: HITBOX_OVERLAY_FILL_OPACITY,
    depthTest: false,
    depthWrite: false,
    side: THREE.DoubleSide,
  });
  const fill = new THREE.Mesh(geometry, fillMaterial);
  fill.name = "hitbox-overlay-fill";
  fill.renderOrder = 9;
  group.add(fill);

  const edges = new THREE.EdgesGeometry(geometry);
  const lineMaterial = new THREE.LineBasicMaterial({
    color: overlayColor,
    transparent: true,
    opacity: 1,
    depthTest: false,
    depthWrite: false,
  });
  const lines = new THREE.LineSegments(edges, lineMaterial);
  lines.name = "hitbox-overlay-lines";
  lines.renderOrder = 10;
  group.add(lines);
  return group;
}

export function setHitboxOverlayOnlyMode(hitbox: THREE.Object3D, enabled: boolean): void {
  const fill = hitbox.getObjectByName("hitbox-overlay-fill") as
    | THREE.Mesh<THREE.BufferGeometry, THREE.MeshBasicMaterial>
    | undefined;
  if (fill) {
    fill.material.opacity = enabled ? HITBOX_ONLY_FILL_OPACITY : HITBOX_OVERLAY_FILL_OPACITY;
  }
}

function createExampleCarMesh(color: string): THREE.Group {
  const vertices = [
    [100, -100, 100],
    [100, 100, 100],
    [-100, 100, 100],
    [-100, -100, 100],
    [150, -220, 20],
    [-150, -220, 20],
    [130, -400, -20],
    [-130, -400, -20],
    [140, 170, 25],
    [-140, 170, 25],
    [130, 240, 25],
    [-130, 240, 25],
    [130, -400, -80],
    [-130, -400, -80],
    [150, -220, -80],
    [-150, -220, -80],
    [140, 170, -80],
    [-140, 170, -80],
    [130, 240, -80],
    [-130, 240, -80],
  ];
  const faces = [
    [0, 1, 2],
    [0, 2, 3],
    [4, 0, 5],
    [0, 3, 5],
    [6, 4, 5],
    [6, 5, 7],
    [1, 8, 9],
    [1, 9, 2],
    [4, 8, 1],
    [4, 1, 0],
    [3, 2, 9],
    [3, 9, 5],
    [8, 10, 11],
    [8, 11, 9],
    [12, 6, 7],
    [12, 7, 13],
    [7, 5, 15],
    [7, 15, 13],
    [6, 14, 4],
    [12, 14, 6],
    [14, 16, 4],
    [4, 16, 8],
    [5, 9, 15],
    [15, 9, 17],
    [16, 18, 8],
    [8, 18, 10],
    [9, 11, 17],
    [17, 11, 19],
    [10, 18, 11],
    [11, 18, 19],
    [14, 12, 13],
    [14, 13, 15],
    [16, 14, 15],
    [16, 15, 17],
    [18, 16, 17],
    [18, 17, 19],
  ];

  const bodyGeometry = new THREE.BufferGeometry();
  bodyGeometry.setAttribute("position", new THREE.Float32BufferAttribute(vertices.flat(), 3));
  bodyGeometry.setIndex(faces.flat());
  bodyGeometry.computeVertexNormals();

  const outer = new THREE.Group();
  const inner = new THREE.Group();
  const body = new THREE.Mesh(bodyGeometry, new THREE.MeshLambertMaterial({ color }));
  body.castShadow = true;
  inner.add(body);

  const windowMaterial = new THREE.MeshPhongMaterial({
    color: 0x1a1b2e,
    shininess: 120,
    transparent: true,
    opacity: 0.82,
  });
  const windowVertices = [
    [100, -100, 100],
    [-100, -100, 100],
    [150, -220, 20],
    [-150, -220, 20],
    [100, 100, 100],
    [-100, 100, 100],
    [140, 170, 25],
    [-140, 170, 25],
    [100, -100, 100],
    [100, 100, 100],
    [150, -220, 20],
    [140, 170, 25],
    [-100, -100, 100],
    [-100, 100, 100],
    [-150, -220, 20],
    [-140, 170, 25],
  ];
  const windowFaces = [
    [0, 2, 3],
    [0, 3, 1],
    [4, 6, 7],
    [4, 7, 5],
    [8, 10, 11],
    [8, 11, 9],
    [12, 14, 15],
    [12, 15, 13],
  ];
  const windowGeometry = new THREE.BufferGeometry();
  windowGeometry.setAttribute(
    "position",
    new THREE.Float32BufferAttribute(windowVertices.flat(), 3),
  );
  windowGeometry.setIndex(windowFaces.flat());
  windowGeometry.computeVertexNormals();
  const windowMesh = new THREE.Mesh(windowGeometry, windowMaterial);
  windowMesh.position.z = 1;
  inner.add(windowMesh);

  const windshieldMaterial = new THREE.MeshBasicMaterial({
    color: 0x88d7ff,
    transparent: true,
    opacity: 0.34,
    side: THREE.DoubleSide,
  });
  const windshieldGeometry = new THREE.BufferGeometry();
  windshieldGeometry.setAttribute(
    "position",
    new THREE.Float32BufferAttribute(
      [90, -110, 95, -90, -110, 95, 140, -210, 25, -140, -210, 25],
      3,
    ),
  );
  windshieldGeometry.setIndex([0, 2, 3, 0, 3, 1]);
  windshieldGeometry.computeVertexNormals();
  const windshieldMesh = new THREE.Mesh(windshieldGeometry, windshieldMaterial);
  windshieldMesh.position.z = 2;
  inner.add(windshieldMesh);

  const wheelMaterial = new THREE.MeshPhongMaterial({
    color: 0x222222,
    shininess: 48,
  });
  // Each wheel is a cylinder inside a steering pivot. The pivot rotates about
  // its local Z (the car's up axis here) for steering; the cylinder spins about
  // its own axle. Both are driven per-frame from replay steer + motion in
  // player.ts. The collected pivots are exposed on the car group's userData.
  const wheels: CarWheel[] = [];
  const makeWheel = (
    x: number,
    y: number,
    z: number,
    width: number,
    isFront: boolean,
  ): THREE.Group => {
    const pivot = new THREE.Group();
    pivot.position.set(x, y, z);
    const wheel = new THREE.Mesh(new THREE.CylinderGeometry(70, 70, width, 10), wheelMaterial);
    // Lay the cylinder on its side so the axle runs left-right; spin is layered
    // on top of this in player.ts.
    wheel.quaternion.setFromAxisAngle(new THREE.Vector3(0, 0, 1), Math.PI / 2);
    wheel.castShadow = true;
    pivot.add(wheel);
    wheels.push({ pivot, wheel, isFront });
    return pivot;
  };

  inner.add(makeWheel(120, -300, -60, 50, false));
  inner.add(makeWheel(-120, -300, -60, 50, false));
  inner.add(makeWheel(120, 150, -60, 70, true));
  inner.add(makeWheel(-120, 150, -60, 70, true));
  inner.position.set(0, 0, 50);
  inner.rotateZ(Math.PI / 2);
  inner.scale.set(0.35, 0.35, 0.35);
  outer.add(inner);
  outer.userData.wheels = wheels;
  return outer;
}

function createBoostTrail(): THREE.Group {
  const trail = new THREE.Group();
  trail.visible = false;
  trail.position.set(-124, 0, 8);

  const outerGeometry = new THREE.ConeGeometry(30, 220, 14, 1, true);
  outerGeometry.rotateZ(Math.PI / 2);
  outerGeometry.translate(-110, 0, 0);

  const innerGeometry = new THREE.ConeGeometry(17, 150, 12, 1, true);
  innerGeometry.rotateZ(Math.PI / 2);
  innerGeometry.translate(-75, 0, 0);

  const glowGeometry = new THREE.SphereGeometry(21, 12, 12);
  const nozzleOffsets = [-38, 38];

  for (const lateralOffset of nozzleOffsets) {
    const plume = new THREE.Group();
    plume.position.set(0, lateralOffset, 0);

    const outerMaterial = new THREE.MeshBasicMaterial({
      color: "#ff9b2f",
      transparent: true,
      opacity: 0.42,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
      side: THREE.DoubleSide,
    });
    outerMaterial.forceSinglePass = true;
    const outerFlame = new THREE.Mesh(outerGeometry, outerMaterial);
    outerFlame.name = "outer-flame";
    plume.add(outerFlame);

    const innerMaterial = new THREE.MeshBasicMaterial({
      color: "#fff2ba",
      transparent: true,
      opacity: 0.9,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
      side: THREE.DoubleSide,
    });
    innerMaterial.forceSinglePass = true;
    const innerFlame = new THREE.Mesh(innerGeometry, innerMaterial);
    innerFlame.name = "inner-flame";
    plume.add(innerFlame);

    const glowMaterial = new THREE.MeshBasicMaterial({
      color: "#fff8db",
      transparent: true,
      opacity: 0.62,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
    });
    glowMaterial.forceSinglePass = true;
    const glow = new THREE.Mesh(glowGeometry, glowMaterial);
    glow.name = "glow";
    glow.position.x = -10;
    plume.add(glow);

    trail.add(plume);
  }

  return trail;
}

export interface BoostMeter {
  group: THREE.Group;
  fillMesh: THREE.Mesh;
  fillMaterial: THREE.MeshBasicMaterial;
  labelTexture: THREE.CanvasTexture;
  labelContext: CanvasRenderingContext2D;
  labelCanvas: HTMLCanvasElement;
  lastPercent: number | null;
}

export interface DemoIndicator {
  group: THREE.Group;
  ring: THREE.Mesh;
  label: THREE.Mesh;
}

function createBoostMeter(): BoostMeter {
  const group = new THREE.Group();
  group.visible = false;

  // Position above the car, sized to remain readable from the default camera.
  group.position.set(0, 0, 235);

  const panelWidth = 240;
  const panelHeight = 82;
  const barWidth = 188;
  const barHeight = 20;

  const panelGeometry = new THREE.PlaneGeometry(panelWidth, panelHeight);
  const panelMaterial = new THREE.MeshBasicMaterial({
    color: 0x07131d,
    transparent: true,
    opacity: 0.78,
    side: THREE.DoubleSide,
    depthWrite: false,
  });
  const panelMesh = new THREE.Mesh(panelGeometry, panelMaterial);
  panelMesh.position.z = -1;
  group.add(panelMesh);

  // Background (dark)
  const bgGeometry = new THREE.PlaneGeometry(barWidth, barHeight);
  const bgMaterial = new THREE.MeshBasicMaterial({
    color: 0x152431,
    transparent: true,
    opacity: 0.92,
    side: THREE.DoubleSide,
    depthWrite: false,
  });
  const bgMesh = new THREE.Mesh(bgGeometry, bgMaterial);
  bgMesh.position.y = -18;
  group.add(bgMesh);

  // Fill (yellow-gold, scales with boost amount)
  const fillGeometry = new THREE.PlaneGeometry(barWidth, barHeight);
  const fillMaterial = new THREE.MeshBasicMaterial({
    color: 0xffc247,
    transparent: true,
    opacity: 0.98,
    side: THREE.DoubleSide,
    depthWrite: false,
  });
  const fillMesh = new THREE.Mesh(fillGeometry, fillMaterial);
  fillMesh.position.y = -18;
  group.add(fillMesh);

  const labelCanvas = document.createElement("canvas");
  labelCanvas.width = 512;
  labelCanvas.height = 160;
  const labelContext = labelCanvas.getContext("2d");
  if (!labelContext) {
    throw new Error("Unable to create boost meter label context");
  }

  const labelTexture = new THREE.CanvasTexture(labelCanvas);
  labelTexture.colorSpace = THREE.SRGBColorSpace;
  labelTexture.needsUpdate = true;

  const labelGeometry = new THREE.PlaneGeometry(190, 48);
  const labelMaterial = new THREE.MeshBasicMaterial({
    map: labelTexture,
    transparent: true,
    depthWrite: false,
    side: THREE.DoubleSide,
  });
  const labelMesh = new THREE.Mesh(labelGeometry, labelMaterial);
  labelMesh.position.set(0, 15, 0);
  group.add(labelMesh);

  return {
    group,
    fillMesh,
    fillMaterial,
    labelTexture,
    labelContext,
    labelCanvas,
    lastPercent: null,
  };
}

function createDemoIndicator(): DemoIndicator {
  const group = new THREE.Group();
  group.visible = false;

  const ringMaterial = new THREE.MeshBasicMaterial({
    color: 0xffd15c,
    transparent: true,
    opacity: 0.86,
    depthWrite: false,
  });
  const ring = new THREE.Mesh(new THREE.TorusGeometry(170, 8, 8, 48), ringMaterial);
  ring.position.z = 16;
  group.add(ring);

  const canvas = document.createElement("canvas");
  canvas.width = 512;
  canvas.height = 192;
  const context = canvas.getContext("2d");
  if (!context) {
    throw new Error("Unable to create demo indicator label context");
  }

  context.textAlign = "center";
  context.textBaseline = "middle";
  context.lineJoin = "round";
  context.font = "800 86px sans-serif";
  context.lineWidth = 20;
  context.strokeStyle = "rgba(7, 19, 29, 0.94)";
  context.strokeText("DEMO", canvas.width / 2, 88);
  context.fillStyle = "#fff0b8";
  context.fillText("DEMO", canvas.width / 2, 88);
  context.font = "700 34px sans-serif";
  context.lineWidth = 10;
  context.strokeText("RESPAWNING", canvas.width / 2, 150);
  context.fillStyle = "#ffbd4a";
  context.fillText("RESPAWNING", canvas.width / 2, 150);

  const texture = new THREE.CanvasTexture(canvas);
  texture.colorSpace = THREE.SRGBColorSpace;
  const labelMaterial = new THREE.MeshBasicMaterial({
    map: texture,
    transparent: true,
    depthWrite: false,
    side: THREE.DoubleSide,
  });
  const label = new THREE.Mesh(new THREE.PlaneGeometry(310, 116), labelMaterial);
  label.position.z = 300;
  group.add(label);

  return { group, ring, label };
}

export function updateBoostMeter(
  meter: BoostMeter,
  fraction: number,
  amount: number,
  camera: THREE.Camera,
): void {
  // Scale fill bar horizontally by fraction
  meter.fillMesh.scale.x = Math.max(0.001, fraction);
  // Shift fill so it stays left-aligned: at scale=1 it's centered,
  // so offset = (1 - scale) * halfWidth
  const halfWidth = 94; // barWidth / 2
  meter.fillMesh.position.x = -(1 - fraction) * halfWidth;
  meter.fillMesh.position.y = -18;

  const percent = Math.max(0, Math.min(100, Math.round(boostAmountToPercent(amount))));
  if (meter.lastPercent !== percent) {
    const { labelContext, labelCanvas, labelTexture } = meter;
    labelContext.clearRect(0, 0, labelCanvas.width, labelCanvas.height);
    labelContext.textAlign = "center";
    labelContext.textBaseline = "middle";
    labelContext.lineJoin = "round";

    labelContext.font = "700 84px sans-serif";
    labelContext.lineWidth = 18;
    labelContext.strokeStyle = "rgba(7, 19, 29, 0.92)";
    labelContext.strokeText(`${percent}`, labelCanvas.width / 2, 78);
    labelContext.fillStyle = "#fff8e1";
    labelContext.fillText(`${percent}`, labelCanvas.width / 2, 78);

    labelContext.font = "600 30px sans-serif";
    labelContext.lineWidth = 10;
    labelContext.strokeText("BOOST", labelCanvas.width / 2, 130);
    labelContext.fillStyle = "#ffcf70";
    labelContext.fillText("BOOST", labelCanvas.width / 2, 130);

    labelTexture.needsUpdate = true;
    meter.lastPercent = percent;
  }

  // Billboard: face the camera
  meter.group.quaternion.copy(camera.quaternion);
}

function makeLights(scene: THREE.Scene): void {
  scene.add(new THREE.AmbientLight("#d8ecff", 1.6));

  const keyLight = new THREE.DirectionalLight("#fff6df", 2.4);
  keyLight.position.set(4000, -6000, 5000);
  scene.add(keyLight);

  const fillLight = new THREE.DirectionalLight("#97d7ff", 1.2);
  fillLight.position.set(-5000, 4000, 3000);
  scene.add(fillLight);
}

function makeBallMesh(renderer: THREE.WebGLRenderer): {
  mesh: THREE.Mesh;
  texture: THREE.CanvasTexture;
} {
  const texture = createBallTexture(renderer);
  const material = new THREE.MeshPhongMaterial({
    color: 0xffffff,
    map: texture,
    shininess: 42,
    specular: new THREE.Color("#f7f2e3"),
  });

  return {
    mesh: new THREE.Mesh(new THREE.SphereGeometry(93, 24, 24), material),
    texture,
  };
}

export function createReplayScene(
  container: HTMLElement,
  replay: ReplayModel,
  fieldScale: number,
): ReplayScene {
  const scene = new THREE.Scene();
  scene.background = new THREE.Color("#081119");

  const camera = new THREE.PerspectiveCamera(48, 1, 10 * fieldScale, 500000 * fieldScale);
  camera.up.set(0, 0, 1);
  camera.position.set(0, -9000 * fieldScale, 5000 * fieldScale);
  camera.lookAt(0, 0, 0);

  const renderer = new THREE.WebGLRenderer({
    antialias: false,
    powerPreference: "high-performance",
  });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, MAX_RENDERER_PIXEL_RATIO));
  renderer.domElement.style.display = "block";
  renderer.domElement.style.width = "100%";
  renderer.domElement.style.height = "100%";
  renderer.domElement.tabIndex = 0;
  renderer.domElement.setAttribute("aria-label", "Replay player viewport");
  container.replaceChildren(renderer.domElement);

  const controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;
  controls.maxDistance = 160000 * fieldScale;
  controls.keyPanSpeed = KEYBOARD_PAN_SPEED;
  controls.target.set(0, 0, 600 * fieldScale);
  controls.listenToKeyEvents(renderer.domElement);
  controls.update();

  const focusViewport = (): void => {
    renderer.domElement.focus();
  };
  renderer.domElement.addEventListener("pointerdown", focusViewport);

  const { stadium, wallPanels } = createExampleSoccarField(fieldScale);
  scene.add(stadium);
  makeLights(scene);

  const replayRoot = new THREE.Group();
  replayRoot.scale.set(-fieldScale, fieldScale, fieldScale);
  scene.add(replayRoot);

  const { mesh: ballMesh, texture: ballTexture } = makeBallMesh(renderer);
  replayRoot.add(ballMesh);

  const playerMeshes = new Map<string, THREE.Object3D>();
  const playerBodyMeshes = new Map<string, THREE.Object3D>();
  const playerHitboxes = new Map<string, THREE.Object3D>();
  const playerBoostTrails = new Map<string, THREE.Group>();
  const playerBoostMeters = new Map<string, BoostMeter>();
  const playerDemoIndicators = new Map<string, DemoIndicator>();
  for (const player of replay.players) {
    const mesh = new THREE.Group();
    const teamColor = player.isTeamZero ? "#57a8ff" : "#ff9c40";
    const bodyMesh = createExampleCarMesh(teamColor);
    const hitboxWireframe = createHitboxOverlay(player.hitbox, teamColor);
    mesh.add(bodyMesh);
    mesh.add(hitboxWireframe);
    const boostTrail = createBoostTrail();
    mesh.add(boostTrail);
    const boostMeter = createBoostMeter();
    mesh.add(boostMeter.group);
    const demoIndicator = createDemoIndicator();
    replayRoot.add(mesh);
    replayRoot.add(demoIndicator.group);
    playerMeshes.set(player.id, mesh);
    playerBodyMeshes.set(player.id, bodyMesh);
    playerHitboxes.set(player.id, hitboxWireframe);
    playerBoostTrails.set(player.id, boostTrail);
    playerBoostMeters.set(player.id, boostMeter);
    playerDemoIndicators.set(player.id, demoIndicator);
  }

  const resize = (): void => {
    const width = container.clientWidth || 1;
    const height = container.clientHeight || 1;
    camera.aspect = width / height;
    camera.updateProjectionMatrix();
    renderer.setSize(width, height, false);
  };

  resize();

  const wallCenter = new THREE.Vector3();
  const wallNormal = new THREE.Vector3();
  const wallQuaternion = new THREE.Quaternion();
  const wallToCamera = new THREE.Vector3();
  const updateWallVisibility = (): void => {
    scene.updateMatrixWorld(true);
    for (const panel of wallPanels) {
      if (panel.fixedOpacity !== null) {
        panel.material.transparent = true;
        panel.material.opacity = panel.fixedOpacity;
        panel.material.depthWrite = false;
        continue;
      }

      panel.mesh.getWorldPosition(wallCenter);
      panel.mesh.getWorldQuaternion(wallQuaternion);
      wallNormal.copy(panel.outwardLocal).applyQuaternion(wallQuaternion).normalize();
      wallToCamera.copy(camera.position).sub(wallCenter);
      const isOutside = wallNormal.dot(wallToCamera) > 0;
      panel.material.transparent = true;
      panel.material.opacity = isOutside ? OUTSIDE_WALL_OPACITY : OPAQUE_WALL_OPACITY;
      panel.material.depthWrite = !isOutside;
    }
  };

  const dispose = (): void => {
    renderer.domElement.removeEventListener("pointerdown", focusViewport);
    controls.stopListenToKeyEvents();
    controls.dispose();
    ballTexture.dispose();
    renderer.dispose();
    container.replaceChildren();
  };

  return {
    scene,
    replayRoot,
    camera,
    renderer,
    controls,
    resize,
    dispose,
    ballMesh,
    playerMeshes,
    playerBodyMeshes,
    playerHitboxes,
    playerBoostTrails,
    playerBoostMeters,
    playerDemoIndicators,
    updateWallVisibility,
  };
}
