import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import type { ReplayModel } from "./types";
import { createExampleSoccarField, OPAQUE_WALL_OPACITY, OUTSIDE_WALL_OPACITY } from "./scene-field";
import { createBallTexture } from "./scene-ball-texture";
import { createExampleCarMesh } from "./scene-car-mesh";
import {
  createBoostMeter,
  createBoostTrail,
  createDemoIndicator,
  type BoostMeter,
  type DemoIndicator,
} from "./scene-player-effects";

export { updateBoostMeter } from "./scene-player-effects";
export type { BoostMeter, DemoIndicator } from "./scene-player-effects";

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
