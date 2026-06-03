import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import type { ReplayModel } from "./types";
import type { ReplayHitboxSpec } from "./hitboxes";

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

function createHitboxWireframe(hitbox: ReplayHitboxSpec, color: string): THREE.Group {
  const group = new THREE.Group();
  group.name = `${hitbox.kind}-hitbox-wireframe`;
  group.visible = false;
  group.rotateY(THREE.MathUtils.degToRad(hitbox.slopeDegrees));

  const geometry = new THREE.BoxGeometry(hitbox.length, hitbox.width, hitbox.height);
  const edges = new THREE.EdgesGeometry(geometry);
  const material = new THREE.LineBasicMaterial({
    color,
    transparent: true,
    opacity: 0.92,
    depthWrite: false,
  });
  const lines = new THREE.LineSegments(edges, material);
  group.add(lines);
  return group;
}

function createHitboxFitCarMesh(color: string, hitbox: ReplayHitboxSpec): THREE.Group {
  const outer = new THREE.Group();
  const inner = new THREE.Group();
  inner.rotateY(THREE.MathUtils.degToRad(hitbox.slopeDegrees));
  outer.add(inner);

  const bodyMaterial = new THREE.MeshLambertMaterial({ color });
  const accentMaterial = new THREE.MeshPhongMaterial({
    color: 0x13212e,
    shininess: 80,
  });
  const glassMaterial = new THREE.MeshPhongMaterial({
    color: 0x88d7ff,
    transparent: true,
    opacity: 0.42,
    shininess: 120,
  });
  const wheelMaterial = new THREE.MeshPhongMaterial({
    color: 0x1f2025,
    shininess: 48,
  });

  const body = new THREE.Mesh(
    new THREE.BoxGeometry(hitbox.length * 0.94, hitbox.width * 0.88, hitbox.height * 0.44),
    bodyMaterial,
  );
  body.position.z = -hitbox.height * 0.19;
  body.castShadow = true;
  inner.add(body);

  const nose = new THREE.Mesh(
    new THREE.BoxGeometry(hitbox.length * 0.28, hitbox.width * 0.82, hitbox.height * 0.32),
    bodyMaterial,
  );
  nose.position.set(hitbox.length * 0.31, 0, hitbox.height * 0.02);
  nose.castShadow = true;
  inner.add(nose);

  const cabin = new THREE.Mesh(
    new THREE.BoxGeometry(hitbox.length * 0.34, hitbox.width * 0.62, hitbox.height * 0.34),
    glassMaterial,
  );
  cabin.position.set(-hitbox.length * 0.06, 0, hitbox.height * 0.16);
  cabin.castShadow = true;
  inner.add(cabin);

  const rearDeck = new THREE.Mesh(
    new THREE.BoxGeometry(hitbox.length * 0.24, hitbox.width * 0.74, hitbox.height * 0.18),
    accentMaterial,
  );
  rearDeck.position.set(-hitbox.length * 0.35, 0, hitbox.height * 0.03);
  rearDeck.castShadow = true;
  inner.add(rearDeck);

  const wheelRadius = Math.max(6, hitbox.height * 0.27);
  const wheelDepth = Math.max(5, hitbox.width * 0.12);
  const wheelGeometry = new THREE.CylinderGeometry(wheelRadius, wheelRadius, wheelDepth, 14);
  const makeWheel = (x: number, y: number): THREE.Mesh => {
    const wheel = new THREE.Mesh(wheelGeometry, wheelMaterial);
    wheel.position.set(x, y, -hitbox.height * 0.39);
    wheel.castShadow = true;
    return wheel;
  };

  const axleX = hitbox.length * 0.32;
  const wheelY = hitbox.width * 0.5;
  inner.add(makeWheel(axleX, wheelY));
  inner.add(makeWheel(axleX, -wheelY));
  inner.add(makeWheel(-axleX, wheelY));
  inner.add(makeWheel(-axleX, -wheelY));

  outer.userData.hitboxKind = hitbox.kind;
  outer.userData.hitboxLabel = hitbox.label;
  return outer;
}

function createBoostTrail(hitbox: ReplayHitboxSpec): THREE.Group {
  const trail = new THREE.Group();
  trail.visible = false;
  trail.position.set(-hitbox.length * 0.55, 0, 0);

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

  const percent = Math.max(0, Math.min(100, Math.round((amount / 255) * 100)));
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
  const playerHitboxes = new Map<string, THREE.Object3D>();
  const playerBoostTrails = new Map<string, THREE.Group>();
  const playerBoostMeters = new Map<string, BoostMeter>();
  const playerDemoIndicators = new Map<string, DemoIndicator>();
  for (const player of replay.players) {
    const mesh = createHitboxFitCarMesh(player.isTeamZero ? "#57a8ff" : "#ff9c40", player.hitbox);
    const hitboxWireframe = createHitboxWireframe(
      player.hitbox,
      player.isTeamZero ? "#b9e0ff" : "#ffd2a3",
    );
    mesh.add(hitboxWireframe);
    const boostTrail = createBoostTrail(player.hitbox);
    mesh.add(boostTrail);
    const boostMeter = createBoostMeter();
    mesh.add(boostMeter.group);
    const demoIndicator = createDemoIndicator();
    replayRoot.add(mesh);
    replayRoot.add(demoIndicator.group);
    playerMeshes.set(player.id, mesh);
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
    playerHitboxes,
    playerBoostTrails,
    playerBoostMeters,
    playerDemoIndicators,
    updateWallVisibility,
  };
}
