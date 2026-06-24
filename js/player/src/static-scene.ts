import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { createExampleSoccarStadium } from "./scene";
import { SubtrActorPlayer } from "./player/adapter/SubtrActorPlayer.js";
import { ReplayPlayer } from "./player/ReplayPlayer.js";
import type { PlayerOptions } from "./player/types.js";
import type { ReplayLoadResult } from "./types";

const DEFAULT_FIELD_SCALE = 1 / 105;
const DEFAULT_BACKGROUND = "#081119";
const DEFAULT_POINT_COLOR = "#62d2a2";
const DEFAULT_POINT_RADIUS_UU = 92;
const DEFAULT_HEAT_BIN_UU = 620;
const DEFAULT_HEAT_RADIUS_UU = 360;
const DEFAULT_HEAT_OPACITY = 0.22;
const MAX_RENDERER_PIXEL_RATIO = 1.5;
const DEFAULT_PICK_RADIUS_PX = 28;

export interface StaticSceneVec3 {
  readonly x: number;
  readonly y: number;
  readonly z: number;
}

export interface StaticScenePoint<TMetadata = unknown> {
  readonly id: string;
  readonly position: StaticSceneVec3;
  readonly color?: THREE.ColorRepresentation;
  readonly radiusUu?: number;
  readonly label?: string;
  readonly metadata?: TMetadata;
}

export interface StaticSceneHeatmapOptions {
  readonly enabled?: boolean;
  readonly binSizeUu?: number;
  readonly radiusUu?: number;
  readonly maxOpacity?: number;
}

export interface StaticScenePointLayerOptions {
  readonly defaultColor?: THREE.ColorRepresentation;
  readonly pointRadiusUu?: number;
  readonly heatmap?: boolean | StaticSceneHeatmapOptions;
}

export interface StaticReplaySceneOptions {
  readonly fieldScale?: number;
  readonly background?: THREE.ColorRepresentation | null;
  readonly includeField?: boolean;
  readonly controls?: boolean;
  readonly ariaLabel?: string;
}

export interface StaticReplayScene {
  readonly scene: THREE.Scene;
  readonly dataRoot: THREE.Group;
  readonly fieldRoot: THREE.Group | null;
  readonly camera: THREE.PerspectiveCamera;
  readonly renderer: THREE.WebGLRenderer;
  readonly controls: OrbitControls | null;
  readonly fieldScale: number;
  readonly player?: ReplayPlayer;
  setPoints<TMetadata = unknown>(
    points: readonly StaticScenePoint<TMetadata>[],
    options?: StaticScenePointLayerOptions,
  ): void;
  clearData(): void;
  pickPoint<TMetadata = unknown>(
    clientX: number,
    clientY: number,
  ): StaticScenePoint<TMetadata> | null;
  resize(): void;
  render(): void;
  dispose(): void;
}

export interface StaticReplayPlayerSceneOptions extends PlayerOptions {
  readonly initialCameraPreset?: "side" | "overhead";
  readonly showReplayActors?: boolean;
}

export function createStaticReplayScene(
  container: HTMLElement,
  options: StaticReplaySceneOptions = {},
): StaticReplayScene {
  const fieldScale = options.fieldScale ?? DEFAULT_FIELD_SCALE;
  const scene = new THREE.Scene();
  if (options.background !== null) {
    scene.background = new THREE.Color(options.background ?? DEFAULT_BACKGROUND);
  }

  const camera = new THREE.PerspectiveCamera(46, 1, 10 * fieldScale, 500000 * fieldScale);
  camera.up.set(0, 0, 1);
  camera.position.set(6500 * fieldScale, -9800 * fieldScale, 5200 * fieldScale);
  camera.lookAt(0, 0, 700 * fieldScale);

  const renderer = new THREE.WebGLRenderer({
    antialias: true,
    alpha: options.background === null,
    preserveDrawingBuffer: true,
    powerPreference: "high-performance",
  });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, MAX_RENDERER_PIXEL_RATIO));
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  renderer.domElement.style.display = "block";
  renderer.domElement.style.width = "100%";
  renderer.domElement.style.height = "100%";
  renderer.domElement.tabIndex = 0;
  renderer.domElement.setAttribute("aria-label", options.ariaLabel ?? "Static replay scene");
  container.replaceChildren(renderer.domElement);

  const controls =
    options.controls === false ? null : new OrbitControls(camera, renderer.domElement);
  if (controls) {
    controls.enableDamping = true;
    controls.dampingFactor = 0.08;
    controls.maxDistance = 160000 * fieldScale;
    controls.minDistance = 1800 * fieldScale;
    controls.target.set(0, 0, 700 * fieldScale);
    controls.listenToKeyEvents(renderer.domElement);
    controls.update();
  }

  makeStaticSceneLights(scene);

  const fieldRoot = options.includeField === false ? null : createExampleSoccarStadium(fieldScale);
  if (fieldRoot) {
    fadeFieldMaterials(fieldRoot);
    scene.add(fieldRoot);
  }

  const dataRoot = new THREE.Group();
  dataRoot.scale.set(-fieldScale, fieldScale, fieldScale);
  scene.add(dataRoot);

  const raycaster = new THREE.Raycaster();
  const pointer = new THREE.Vector2();
  let pointMeshes: THREE.Mesh[] = [];
  let animationFrame = 0;
  let disposed = false;

  const resize = (): void => {
    const width = container.clientWidth || 1;
    const height = container.clientHeight || 1;
    camera.aspect = width / height;
    camera.updateProjectionMatrix();
    renderer.setSize(width, height, false);
  };

  const render = (): void => {
    controls?.update();
    renderer.render(scene, camera);
  };

  const resizeObserver = new ResizeObserver(() => {
    resize();
    render();
  });
  resizeObserver.observe(container);

  const animate = (): void => {
    if (disposed) {
      return;
    }
    render();
    animationFrame = requestAnimationFrame(animate);
  };

  resize();
  animate();

  const controller: StaticReplayScene = {
    scene,
    dataRoot,
    fieldRoot,
    camera,
    renderer,
    controls,
    fieldScale,
    setPoints(points, layerOptions = {}) {
      disposeChildren(dataRoot);
      dataRoot.clear();
      const { heatmap, markers } = createPointLayer(points, layerOptions);
      if (heatmap) dataRoot.add(heatmap);
      dataRoot.add(markers);
      pointMeshes = markers.children.filter(
        (child): child is THREE.Mesh => child instanceof THREE.Mesh,
      );
      render();
    },
    clearData() {
      disposeChildren(dataRoot);
      dataRoot.clear();
      pointMeshes = [];
      render();
    },
    pickPoint<TMetadata = unknown>(clientX: number, clientY: number) {
      const rect = renderer.domElement.getBoundingClientRect();
      pointer.x = ((clientX - rect.left) / rect.width) * 2 - 1;
      pointer.y = -(((clientY - rect.top) / rect.height) * 2 - 1);
      raycaster.setFromCamera(pointer, camera);
      const hit = raycaster.intersectObjects(pointMeshes, false)[0];
      return (
        (hit?.object.userData.staticScenePoint as StaticScenePoint<TMetadata> | undefined) ??
        nearestProjectedPoint<TMetadata>(pointMeshes, camera, renderer.domElement, clientX, clientY)
      );
    },
    resize,
    render,
    dispose() {
      disposed = true;
      cancelAnimationFrame(animationFrame);
      resizeObserver.disconnect();
      controls?.stopListenToKeyEvents();
      controls?.dispose();
      disposeChildren(dataRoot);
      if (fieldRoot) disposeObject(fieldRoot);
      renderer.dispose();
      container.replaceChildren();
    },
  };

  return controller;
}

export function createStaticReplaySceneFromParsed(
  container: HTMLElement,
  parsed: ReplayLoadResult,
  options: StaticReplayPlayerSceneOptions = {},
): StaticReplayScene {
  const player = new ReplayPlayer(
    container,
    new SubtrActorPlayer(parsed.raw as never, {
      motionSmoothing: options.motionSmoothing,
      smoothingBlendFactor: options.smoothingBlendFactor,
      smoothingAnchorInterval: options.smoothingAnchorInterval,
      timelineCompaction: options.timelineCompaction,
      disableFrameFiltering: options.disableFrameFiltering,
    }),
    {
      ...options,
      autoplay: false,
      loop: false,
      preserveDrawingBuffer: options.preserveDrawingBuffer ?? true,
      initialSkipKickoffsEnabled: false,
      initialSkipPostGoalTransitionsEnabled: false,
      initialCameraViewMode: "free",
    },
    parsed.replay,
  );
  player.pause();
  player.setFreeCameraPreset(options.initialCameraPreset ?? "side", { instant: true });
  const removeHideReplayActors =
    options.showReplayActors === true
      ? null
      : player.onBeforeRender(() => hideReplayActors(player));

  const dataRoot = new THREE.Group();
  dataRoot.name = "staticReplayDataRoot";
  player.sceneState.replayRoot.add(dataRoot);

  let pointMeshes: THREE.Mesh[] = [];
  const raycaster = new THREE.Raycaster();
  const pointer = new THREE.Vector2();

  const controller: StaticReplayScene = {
    scene: player.sceneState.scene,
    dataRoot,
    fieldRoot: null,
    camera: player.sceneState.camera,
    renderer: player.sceneState.renderer,
    controls: player.sceneState.controls,
    fieldScale: 1,
    player,
    setPoints(points, layerOptions = {}) {
      disposeChildren(dataRoot);
      dataRoot.clear();
      const { heatmap, markers } = createPointLayer(points, layerOptions);
      if (heatmap) dataRoot.add(heatmap);
      dataRoot.add(markers);
      pointMeshes = markers.children.filter(
        (child): child is THREE.Mesh => child instanceof THREE.Mesh,
      );
      player.renderFrame();
    },
    clearData() {
      disposeChildren(dataRoot);
      dataRoot.clear();
      pointMeshes = [];
      player.renderFrame();
    },
    pickPoint<TMetadata = unknown>(clientX: number, clientY: number) {
      const rect = player.renderer.domElement.getBoundingClientRect();
      pointer.x = ((clientX - rect.left) / rect.width) * 2 - 1;
      pointer.y = -(((clientY - rect.top) / rect.height) * 2 - 1);
      raycaster.setFromCamera(pointer, player.camera);
      const hit = raycaster.intersectObjects(pointMeshes, false)[0];
      return (
        (hit?.object.userData.staticScenePoint as StaticScenePoint<TMetadata> | undefined) ??
        nearestProjectedPoint<TMetadata>(
          pointMeshes,
          player.camera,
          player.renderer.domElement,
          clientX,
          clientY,
        )
      );
    },
    resize() {
      player.sceneState.resize();
      player.renderFrame();
    },
    render() {
      player.renderFrame();
    },
    dispose() {
      removeHideReplayActors?.();
      disposeChildren(dataRoot);
      player.destroy();
    },
  };

  void player.ready.then(() => {
    controller.render();
  });

  return controller;
}

function hideReplayActors(player: ReplayPlayer): void {
  player.sceneState.ballMesh.visible = false;
  for (const mesh of player.sceneState.playerMeshes.values()) {
    mesh.visible = false;
  }
}

function nearestProjectedPoint<TMetadata>(
  pointMeshes: readonly THREE.Mesh[],
  camera: THREE.Camera,
  canvas: HTMLCanvasElement,
  clientX: number,
  clientY: number,
): StaticScenePoint<TMetadata> | null {
  const rect = canvas.getBoundingClientRect();
  const world = new THREE.Vector3();
  const projected = new THREE.Vector3();
  let nearest: StaticScenePoint<TMetadata> | null = null;
  let nearestDistanceSq = DEFAULT_PICK_RADIUS_PX * DEFAULT_PICK_RADIUS_PX;

  for (const mesh of pointMeshes) {
    mesh.getWorldPosition(world);
    projected.copy(world).project(camera);
    if (projected.z < -1 || projected.z > 1) continue;
    const x = rect.left + ((projected.x + 1) / 2) * rect.width;
    const y = rect.top + ((1 - projected.y) / 2) * rect.height;
    const dx = x - clientX;
    const dy = y - clientY;
    const distanceSq = dx * dx + dy * dy;
    if (distanceSq <= nearestDistanceSq) {
      nearestDistanceSq = distanceSq;
      nearest = (mesh.userData.staticScenePoint as StaticScenePoint<TMetadata> | undefined) ?? null;
    }
  }

  return nearest;
}

function createPointLayer<TMetadata>(
  points: readonly StaticScenePoint<TMetadata>[],
  options: StaticScenePointLayerOptions,
): { heatmap: THREE.Group | null; markers: THREE.Group } {
  const markers = new THREE.Group();
  const pointRadius = options.pointRadiusUu ?? DEFAULT_POINT_RADIUS_UU;
  const defaultColor = options.defaultColor ?? DEFAULT_POINT_COLOR;
  const markerGeometry = new THREE.SphereGeometry(pointRadius, 20, 16);

  for (const point of points) {
    const color = new THREE.Color(point.color ?? defaultColor);
    const mesh = new THREE.Mesh(
      markerGeometry.clone(),
      new THREE.MeshStandardMaterial({
        color,
        emissive: color.clone().multiplyScalar(0.34),
        roughness: 0.38,
      }),
    );
    const scale = (point.radiusUu ?? pointRadius) / pointRadius;
    mesh.scale.setScalar(scale);
    mesh.position.set(point.position.x, point.position.y, point.position.z);
    mesh.userData.staticScenePoint = point;
    markers.add(mesh);
  }

  return {
    heatmap: shouldRenderHeatmap(options.heatmap) ? createHeatmap(points, options) : null,
    markers,
  };
}

function createHeatmap<TMetadata>(
  points: readonly StaticScenePoint<TMetadata>[],
  options: StaticScenePointLayerOptions,
): THREE.Group {
  const heatmap =
    typeof options.heatmap === "object"
      ? options.heatmap
      : ({} satisfies StaticSceneHeatmapOptions);
  const binSize = heatmap.binSizeUu ?? DEFAULT_HEAT_BIN_UU;
  const radius = heatmap.radiusUu ?? DEFAULT_HEAT_RADIUS_UU;
  const maxOpacity = heatmap.maxOpacity ?? DEFAULT_HEAT_OPACITY;
  const defaultColor = options.defaultColor ?? DEFAULT_POINT_COLOR;
  const bins = new Map<
    string,
    { position: THREE.Vector3; count: number; color: THREE.ColorRepresentation }
  >();

  for (const point of points) {
    const key = [
      Math.round(point.position.x / binSize),
      Math.round(point.position.y / binSize),
      Math.round(point.position.z / binSize),
      String(point.color ?? defaultColor),
    ].join(":");
    const bin = bins.get(key);
    if (bin) {
      bin.position.add(new THREE.Vector3(point.position.x, point.position.y, point.position.z));
      bin.count += 1;
    } else {
      bins.set(key, {
        position: new THREE.Vector3(point.position.x, point.position.y, point.position.z),
        count: 1,
        color: point.color ?? defaultColor,
      });
    }
  }

  const group = new THREE.Group();
  for (const bin of bins.values()) {
    bin.position.divideScalar(bin.count);
    const mesh = new THREE.Mesh(
      new THREE.SphereGeometry(radius * (0.92 + Math.sqrt(bin.count) * 0.24), 24, 16),
      new THREE.MeshBasicMaterial({
        color: new THREE.Color(bin.color),
        transparent: true,
        opacity: Math.min(maxOpacity, 0.08 + bin.count * 0.035),
        blending: THREE.AdditiveBlending,
        depthWrite: false,
      }),
    );
    mesh.position.copy(bin.position);
    group.add(mesh);
  }
  return group;
}

function shouldRenderHeatmap(heatmap: StaticScenePointLayerOptions["heatmap"]): boolean {
  if (heatmap == null) return true;
  if (typeof heatmap === "boolean") return heatmap;
  return heatmap.enabled !== false;
}

function makeStaticSceneLights(scene: THREE.Scene): void {
  scene.add(new THREE.AmbientLight("#d8ecff", 1.45));

  const keyLight = new THREE.DirectionalLight("#fff6df", 2.2);
  keyLight.position.set(4000, -6000, 5000);
  scene.add(keyLight);

  const fillLight = new THREE.DirectionalLight("#97d7ff", 1.35);
  fillLight.position.set(-5000, 4000, 3000);
  scene.add(fillLight);
}

function fadeFieldMaterials(object: THREE.Object3D): void {
  object.traverse((child) => {
    if (!(child instanceof THREE.Mesh)) return;
    const materials = Array.isArray(child.material) ? child.material : [child.material];
    child.material = materials.map((material) => {
      const clone = material.clone();
      clone.transparent = true;
      clone.opacity = Math.min(clone.opacity, 0.62);
      return clone;
    });
  });
}

function disposeChildren(group: THREE.Group): void {
  for (const child of [...group.children]) {
    disposeObject(child);
  }
}

function disposeObject(object: THREE.Object3D): void {
  object.traverse((child) => {
    if (
      !(
        child instanceof THREE.Mesh ||
        child instanceof THREE.Line ||
        child instanceof THREE.LineSegments
      )
    ) {
      return;
    }
    child.geometry.dispose();
    const material = child.material;
    if (Array.isArray(material)) {
      material.forEach((item) => item.dispose());
    } else {
      material.dispose();
    }
  });
}
