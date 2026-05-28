import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import type { ReplayModel } from "./types";
import {
  createExampleSoccarField,
  OPAQUE_WALL_OPACITY,
  OUTSIDE_WALL_OPACITY,
} from "./scene-field";
import { createBallTexture } from "./scene-ball-texture";
import { createExampleCarMesh } from "./scene-car-mesh";

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
  playerBoostTrails: Map<string, THREE.Group>;
  playerBoostMeters: Map<string, BoostMeter>;
  playerDemoIndicators: Map<string, DemoIndicator>;
  updateWallVisibility: () => void;
}

const KEYBOARD_PAN_SPEED = 16;
const MAX_RENDERER_PIXEL_RATIO = 1.5;

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
  const playerBoostTrails = new Map<string, THREE.Group>();
  const playerBoostMeters = new Map<string, BoostMeter>();
  const playerDemoIndicators = new Map<string, DemoIndicator>();
  for (const player of replay.players) {
    const mesh = createExampleCarMesh(player.isTeamZero ? "#57a8ff" : "#ff9c40");
    const boostTrail = createBoostTrail();
    mesh.add(boostTrail);
    const boostMeter = createBoostMeter();
    mesh.add(boostMeter.group);
    const demoIndicator = createDemoIndicator();
    replayRoot.add(mesh);
    replayRoot.add(demoIndicator.group);
    playerMeshes.set(player.id, mesh);
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
    playerBoostTrails,
    playerBoostMeters,
    playerDemoIndicators,
    updateWallVisibility,
  };
}
