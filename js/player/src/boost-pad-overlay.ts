import * as THREE from "three";
import type {
  ReplayBoostPad,
  ReplayPlayerPlugin,
  ReplayPlayerPluginContext,
  ReplayPlayerPluginStateContext,
} from "./types";

export interface BoostPadOverlayPluginOptions {
  showCooldownProgress?: boolean;
}

interface BoostPadMeshes {
  group: THREE.Group;
  ring: THREE.Mesh<THREE.RingGeometry, THREE.MeshBasicMaterial>;
  core: THREE.Mesh<THREE.CircleGeometry, THREE.MeshBasicMaterial>;
  cooldown: THREE.Mesh<THREE.CircleGeometry, THREE.MeshBasicMaterial>;
}

function configureOverlayMaterial(material: THREE.MeshBasicMaterial): void {
  material.depthTest = false;
  material.depthWrite = false;
  material.transparent = true;
  material.polygonOffset = true;
  material.polygonOffsetFactor = -2;
  material.polygonOffsetUnits = -2;
  material.forceSinglePass = true;
}

function padRadius(pad: ReplayBoostPad): number {
  return pad.size === "big" ? 150 : 92;
}

function padColor(pad: ReplayBoostPad): number {
  return pad.size === "big" ? 0xf59e0b : 0xfacc15;
}

function createPadMeshes(pad: ReplayBoostPad): BoostPadMeshes {
  const radius = padRadius(pad);
  const color = padColor(pad);
  const group = new THREE.Group();
  group.position.set(pad.position.x, pad.position.y, pad.position.z + 6);
  group.renderOrder = 20;
  group.frustumCulled = false;

  const ring = new THREE.Mesh(
    new THREE.RingGeometry(radius * 0.72, radius, 24),
    new THREE.MeshBasicMaterial({
      color,
      transparent: true,
      opacity: 0.92,
      side: THREE.DoubleSide,
      depthWrite: false,
    })
  );
  configureOverlayMaterial(ring.material);
  ring.renderOrder = 20;
  ring.frustumCulled = false;
  group.add(ring);

  const core = new THREE.Mesh(
    new THREE.CircleGeometry(radius * 0.58, 24),
    new THREE.MeshBasicMaterial({
      color,
      transparent: true,
      opacity: 0.3,
      side: THREE.DoubleSide,
      depthWrite: false,
    })
  );
  configureOverlayMaterial(core.material);
  core.position.z = 0.5;
  core.renderOrder = 21;
  core.frustumCulled = false;
  group.add(core);

  const cooldown = new THREE.Mesh(
    new THREE.CircleGeometry(radius * 0.42, 20),
    new THREE.MeshBasicMaterial({
      color: 0xffffff,
      transparent: true,
      opacity: 0.22,
      side: THREE.DoubleSide,
      depthWrite: false,
    })
  );
  configureOverlayMaterial(cooldown.material);
  cooldown.position.z = 1;
  cooldown.renderOrder = 22;
  cooldown.frustumCulled = false;
  group.add(cooldown);

  return { group, ring, core, cooldown };
}

function boostPadAvailableState(
  pad: ReplayBoostPad,
  currentTime: number
): {
  available: boolean;
  progress: number;
} {
  let lastEventIndex = -1;
  for (let index = 0; index < pad.events.length; index += 1) {
    if (pad.events[index].time > currentTime) {
      break;
    }
    lastEventIndex = index;
  }

  if (lastEventIndex < 0) {
    return { available: true, progress: 1 };
  }

  const lastEvent = pad.events[lastEventIndex];
  if (lastEvent.available) {
    return { available: true, progress: 1 };
  }

  const nextAvailable = pad.events
    .slice(lastEventIndex + 1)
    .find((event) => event.available);
  if (!nextAvailable || nextAvailable.time <= lastEvent.time) {
    return { available: false, progress: 0 };
  }

  return {
    available: false,
    progress: THREE.MathUtils.clamp(
      (currentTime - lastEvent.time) / (nextAvailable.time - lastEvent.time),
      0,
      1
    ),
  };
}

function updatePadMeshes(
  meshes: BoostPadMeshes,
  pad: ReplayBoostPad,
  currentTime: number,
  showCooldownProgress: boolean
): void {
  const { available, progress } = boostPadAvailableState(pad, currentTime);
  const pulse = 0.92 + 0.08 * Math.sin(currentTime * 6 + pad.index * 0.45);

  if (available) {
    meshes.group.visible = true;
    meshes.ring.material.opacity = 0.95;
    meshes.core.material.opacity = pad.size === "big" ? 0.56 : 0.44;
    meshes.cooldown.visible = false;
    meshes.ring.scale.setScalar(pulse);
    meshes.core.scale.setScalar(1);
    return;
  }

  meshes.group.visible = true;
  meshes.ring.material.opacity = 0.18;
  meshes.core.material.opacity = 0.07;
  meshes.ring.scale.setScalar(1);
  meshes.core.scale.setScalar(1);

  meshes.cooldown.visible = showCooldownProgress;
  if (showCooldownProgress) {
    const cooldownScale = 0.3 + progress * 0.7;
    meshes.cooldown.scale.setScalar(cooldownScale);
    meshes.cooldown.material.opacity = 0.16 + progress * 0.2;
  }
}

export function createBoostPadOverlayPlugin(
  options: BoostPadOverlayPluginOptions = {}
): ReplayPlayerPlugin {
  const showCooldownProgress = options.showCooldownProgress ?? true;

  let padRoot: THREE.Group | null = null;
  const padMeshes = new Map<number, BoostPadMeshes>();

  function buildPads(context: ReplayPlayerPluginContext): void {
    padRoot = new THREE.Group();
    padRoot.name = "boost-pad-overlay";
    padRoot.renderOrder = 20;
    padRoot.frustumCulled = false;

    for (const pad of context.replay.boostPads) {
      const meshes = createPadMeshes(pad);
      padRoot.add(meshes.group);
      padMeshes.set(pad.index, meshes);
    }

    context.scene.replayRoot.add(padRoot);
  }

  function syncPads(context: ReplayPlayerPluginStateContext): void {
    for (const pad of context.replay.boostPads) {
      const meshes = padMeshes.get(pad.index);
      if (!meshes) {
        continue;
      }
      updatePadMeshes(
        meshes,
        pad,
        context.state.currentTime,
        showCooldownProgress
      );
    }
  }

  return {
    id: "boost-pad-overlay",
    setup(context): void {
      buildPads(context);
      syncPads({
        ...context,
        state: context.player.getState(),
      });
    },
    onStateChange(context): void {
      syncPads(context);
    },
    teardown(): void {
      padRoot?.removeFromParent();
      padRoot = null;
      padMeshes.clear();
    },
  };
}
