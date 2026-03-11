import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import type { ReplayModel } from "./types";

export interface ReplayScene {
  scene: THREE.Scene;
  camera: THREE.PerspectiveCamera;
  renderer: THREE.WebGLRenderer;
  controls: OrbitControls;
  resize: () => void;
  dispose: () => void;
  ballMesh: THREE.Mesh;
  playerMeshes: Map<string, THREE.Mesh>;
}

function makeFieldGroup(): THREE.Group {
  const group = new THREE.Group();

  const field = new THREE.Mesh(
    new THREE.BoxGeometry(82, 0.5, 102),
    new THREE.MeshStandardMaterial({
      color: "#1f4f44",
      roughness: 0.9,
      metalness: 0.05,
    })
  );
  field.position.y = -0.25;
  group.add(field);

  const trim = new THREE.LineSegments(
    new THREE.EdgesGeometry(new THREE.BoxGeometry(82.5, 0.5, 102.5)),
    new THREE.LineBasicMaterial({ color: "#d8e6d4" })
  );
  trim.position.y = 0.01;
  group.add(trim);

  const midfield = new THREE.Mesh(
    new THREE.BoxGeometry(0.3, 0.02, 102),
    new THREE.MeshBasicMaterial({ color: "#d8e6d4" })
  );
  midfield.position.y = 0.02;
  group.add(midfield);

  const blueGoal = new THREE.Mesh(
    new THREE.BoxGeometry(18, 4, 2),
    new THREE.MeshStandardMaterial({ color: "#4a90e2", transparent: true, opacity: 0.65 })
  );
  blueGoal.position.set(0, 2, -52);
  group.add(blueGoal);

  const orangeGoal = blueGoal.clone();
  (orangeGoal.material as THREE.MeshStandardMaterial).color = new THREE.Color("#f39b32");
  orangeGoal.position.z = 52;
  group.add(orangeGoal);

  return group;
}

function makeLights(scene: THREE.Scene): void {
  scene.add(new THREE.AmbientLight("#d8ecff", 1.6));

  const keyLight = new THREE.DirectionalLight("#fff6df", 2.4);
  keyLight.position.set(40, 70, 20);
  scene.add(keyLight);

  const fillLight = new THREE.DirectionalLight("#97d7ff", 1.2);
  fillLight.position.set(-30, 25, -25);
  scene.add(fillLight);
}

function makeBallMesh(): THREE.Mesh {
  return new THREE.Mesh(
    new THREE.SphereGeometry(1.82, 32, 32),
    new THREE.MeshStandardMaterial({
      color: "#f6f7fb",
      emissive: "#9ec7ff",
      emissiveIntensity: 0.14,
      roughness: 0.2,
      metalness: 0.15,
    })
  );
}

function makePlayerMesh(color: string): THREE.Mesh {
  const mesh = new THREE.Mesh(
    new THREE.BoxGeometry(0.84, 0.36, 1.18),
    new THREE.MeshStandardMaterial({
      color,
      roughness: 0.45,
      metalness: 0.18,
    })
  );
  mesh.castShadow = false;
  mesh.receiveShadow = false;
  return mesh;
}

export function createReplayScene(
  container: HTMLElement,
  replay: ReplayModel
): ReplayScene {
  const scene = new THREE.Scene();
  scene.background = new THREE.Color("#081119");
  scene.fog = new THREE.Fog("#081119", 90, 180);

  const camera = new THREE.PerspectiveCamera(48, 1, 0.1, 400);
  camera.position.set(0, 58, 76);
  camera.lookAt(0, 0, 0);

  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setPixelRatio(window.devicePixelRatio);
  container.replaceChildren(renderer.domElement);

  const controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;
  controls.target.set(0, 4, 0);
  controls.update();

  scene.add(makeFieldGroup());
  makeLights(scene);

  const ballMesh = makeBallMesh();
  ballMesh.position.set(0, 1.82, 0);
  scene.add(ballMesh);

  const playerMeshes = new Map<string, THREE.Mesh>();
  for (const player of replay.players) {
    const mesh = makePlayerMesh(player.isTeamZero ? "#57a8ff" : "#ff9c40");
    scene.add(mesh);
    playerMeshes.set(player.id, mesh);
  }

  const resize = (): void => {
    const width = container.clientWidth || 1;
    const height = container.clientHeight || 1;
    camera.aspect = width / height;
    camera.updateProjectionMatrix();
    renderer.setSize(width, height, false);
  };

  resize();

  const dispose = (): void => {
    controls.dispose();
    renderer.dispose();
    container.replaceChildren();
  };

  return {
    scene,
    camera,
    renderer,
    controls,
    resize,
    dispose,
    ballMesh,
    playerMeshes,
  };
}
