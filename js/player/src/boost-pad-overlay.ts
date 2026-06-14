import * as THREE from "three";
import { RoomEnvironment } from "three/examples/jsm/environments/RoomEnvironment.js";
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
  base: THREE.Group;
  ring: THREE.Mesh<THREE.RingGeometry, THREE.MeshBasicMaterial>;
  core: THREE.Mesh<THREE.CircleGeometry, THREE.MeshBasicMaterial>;
  cooldown: THREE.Mesh<THREE.CircleGeometry, THREE.MeshBasicMaterial>;
  orb: THREE.Mesh<
    THREE.BufferGeometry,
    THREE.MeshBasicMaterial | THREE.MeshPhongMaterial | THREE.MeshPhysicalMaterial
  >;
  lensColumn: THREE.Mesh<THREE.BufferGeometry, THREE.MeshBasicMaterial>;
  lensRim: THREE.Mesh<THREE.BufferGeometry, THREE.MeshBasicMaterial>;
  sheen: THREE.Mesh<THREE.CircleGeometry, THREE.MeshBasicMaterial>;
  glow: THREE.Mesh<THREE.BufferGeometry, THREE.MeshBasicMaterial>;
}

interface BoostPadVisualEnvironment {
  reflectionMap: THREE.Texture | null;
  dispose(): void;
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

const PAD_SURFACE_Z_OFFSET = 6;
const PAD_VISUAL_SCALE = 0.6;

function scaledPadDimension(value: number): number {
  return value * PAD_VISUAL_SCALE;
}

function padRadius(pad: ReplayBoostPad): number {
  return scaledPadDimension(pad.size === "big" ? 150 : 92);
}

function padOrbRadius(pad: ReplayBoostPad): number {
  return scaledPadDimension(pad.size === "big" ? 104 : 46);
}

function padOrbBottomClearance(pad: ReplayBoostPad): number {
  return scaledPadDimension(pad.size === "big" ? 34 : 14);
}

function padOrbCenterZ(pad: ReplayBoostPad): number {
  return PAD_SURFACE_Z_OFFSET + padOrbBottomClearance(pad) + padOrbRadius(pad);
}

function padLightCenterZ(pad: ReplayBoostPad): number {
  if (pad.size === "big") {
    return padOrbCenterZ(pad);
  }
  return PAD_SURFACE_Z_OFFSET + scaledPadDimension(1.2);
}

function padGlowCenterZ(pad: ReplayBoostPad): number {
  if (pad.size === "big") {
    return padOrbCenterZ(pad);
  }
  return PAD_SURFACE_Z_OFFSET + scaledPadDimension(0.8);
}

function padColor(pad: ReplayBoostPad): number {
  return pad.size === "big" ? 0xc98500 : 0xffd119;
}

function setMeshBasicOpacity(group: THREE.Group, opacityScale: number): void {
  group.traverse((child) => {
    const mesh = child as THREE.Mesh<THREE.BufferGeometry, THREE.MeshBasicMaterial>;
    if (!mesh.isMesh || !(mesh.material instanceof THREE.MeshBasicMaterial)) {
      return;
    }
    const baseOpacity = mesh.userData.baseOpacity as number | undefined;
    mesh.material.opacity = (baseOpacity ?? mesh.material.opacity) * opacityScale;
  });
}

function createBigBoostBase(radius: number): THREE.Group {
  const group = new THREE.Group();
  group.renderOrder = 18;
  group.frustumCulled = false;

  const darkMaterial = new THREE.MeshBasicMaterial({
    color: 0x11110d,
    transparent: true,
    opacity: 0.86,
    side: THREE.DoubleSide,
    depthWrite: false,
  });
  configureOverlayMaterial(darkMaterial);

  const panelMaterial = new THREE.MeshBasicMaterial({
    color: 0xffa000,
    transparent: true,
    opacity: 0.86,
    side: THREE.DoubleSide,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
  });
  configureOverlayMaterial(panelMaterial);

  const center = new THREE.Mesh(new THREE.CircleGeometry(radius * 0.55, 48), darkMaterial.clone());
  center.userData.baseOpacity = darkMaterial.opacity;
  center.position.z = PAD_SURFACE_Z_OFFSET - 2.2;
  center.renderOrder = 18;
  center.frustumCulled = false;
  group.add(center);

  const centerRing = new THREE.Mesh(
    new THREE.RingGeometry(radius * 0.45, radius * 0.62, 48),
    new THREE.MeshBasicMaterial({
      color: 0xffd13a,
      transparent: true,
      opacity: 0.78,
      side: THREE.DoubleSide,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
    }),
  );
  configureOverlayMaterial(centerRing.material);
  centerRing.userData.baseOpacity = centerRing.material.opacity;
  centerRing.position.z = PAD_SURFACE_Z_OFFSET - 1.1;
  centerRing.renderOrder = 20;
  centerRing.frustumCulled = false;
  group.add(centerRing);

  function createArmShape(
    innerRadius: number,
    outerRadius: number,
    halfAngle: number,
  ): THREE.Shape {
    const shape = new THREE.Shape();
    const points: Array<[number, number]> = [
      [innerRadius * Math.cos(-halfAngle * 0.72), innerRadius * Math.sin(-halfAngle * 0.72)],
      [outerRadius * Math.cos(-halfAngle), outerRadius * Math.sin(-halfAngle)],
      [outerRadius * Math.cos(halfAngle), outerRadius * Math.sin(halfAngle)],
      [innerRadius * Math.cos(halfAngle * 0.72), innerRadius * Math.sin(halfAngle * 0.72)],
    ];
    points.forEach(([x, y], index) => {
      if (index === 0) {
        shape.moveTo(x, y);
      } else {
        shape.lineTo(x, y);
      }
    });
    shape.closePath();
    return shape;
  }

  for (let index = 0; index < 3; index += 1) {
    const rotation = (index * (Math.PI * 2)) / 3 + Math.PI / 2;
    const arm = new THREE.Mesh(
      new THREE.ShapeGeometry(createArmShape(radius * 0.52, radius * 1.42, 0.33)),
      darkMaterial.clone(),
    );
    arm.userData.baseOpacity = darkMaterial.opacity;
    arm.position.z = PAD_SURFACE_Z_OFFSET - 2;
    arm.rotation.z = rotation;
    arm.renderOrder = 18;
    arm.frustumCulled = false;
    group.add(arm);

    const panel = new THREE.Mesh(
      new THREE.ShapeGeometry(createArmShape(radius * 0.66, radius * 1.2, 0.21)),
      panelMaterial.clone(),
    );
    panel.userData.baseOpacity = panelMaterial.opacity;
    panel.position.z = PAD_SURFACE_Z_OFFSET - 0.8;
    panel.rotation.z = rotation;
    panel.renderOrder = 19;
    panel.frustumCulled = false;
    group.add(panel);
  }

  return group;
}

function createPadMeshes(
  pad: ReplayBoostPad,
  visualEnvironment: BoostPadVisualEnvironment,
): BoostPadMeshes {
  const radius = padRadius(pad);
  const color = padColor(pad);
  const orbRadius = padOrbRadius(pad);
  const isBigPad = pad.size === "big";
  const group = new THREE.Group();
  group.position.set(pad.position.x, pad.position.y, pad.position.z);
  group.renderOrder = 20;
  group.frustumCulled = false;

  const base = isBigPad ? createBigBoostBase(radius) : new THREE.Group();
  if (isBigPad) {
    group.add(base);
  }

  const ring = new THREE.Mesh(
    new THREE.RingGeometry(radius * (isBigPad ? 0.62 : 0.72), radius * (isBigPad ? 0.82 : 1), 32),
    new THREE.MeshBasicMaterial({
      color: isBigPad ? 0xffd137 : color,
      transparent: true,
      opacity: isBigPad ? 0.82 : 0.92,
      side: THREE.DoubleSide,
      blending: isBigPad ? THREE.AdditiveBlending : THREE.NormalBlending,
      depthWrite: false,
    }),
  );
  configureOverlayMaterial(ring.material);
  ring.position.z = PAD_SURFACE_Z_OFFSET;
  ring.renderOrder = 20;
  ring.frustumCulled = false;
  group.add(ring);

  const core = new THREE.Mesh(
    new THREE.CircleGeometry(radius * (isBigPad ? 0.54 : 0.58), 32),
    new THREE.MeshBasicMaterial({
      color: isBigPad ? 0xffa600 : color,
      transparent: true,
      opacity: isBigPad ? 0.46 : 0.3,
      side: THREE.DoubleSide,
      blending: isBigPad ? THREE.AdditiveBlending : THREE.NormalBlending,
      depthWrite: false,
    }),
  );
  configureOverlayMaterial(core.material);
  core.position.z = PAD_SURFACE_Z_OFFSET + 0.5;
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
    }),
  );
  configureOverlayMaterial(cooldown.material);
  cooldown.position.z = PAD_SURFACE_Z_OFFSET + 1;
  cooldown.renderOrder = 22;
  cooldown.frustumCulled = false;
  group.add(cooldown);

  const orb = new THREE.Mesh(
    isBigPad
      ? new THREE.SphereGeometry(orbRadius, 32, 18)
      : new THREE.SphereGeometry(orbRadius * 1.22, 32, 12),
    isBigPad
      ? new THREE.MeshPhysicalMaterial({
          color: 0xffb21a,
          emissive: new THREE.Color(0xff8a00),
          emissiveIntensity: 0.42,
          metalness: 0.04,
          roughness: 0.08,
          clearcoat: 1,
          clearcoatRoughness: 0.025,
          transmission: 0.18,
          thickness: scaledPadDimension(48),
          ior: 1.42,
          envMap: visualEnvironment.reflectionMap,
          envMapIntensity: 1.9,
          transparent: true,
          opacity: 0.68,
          depthWrite: false,
          blending: THREE.AdditiveBlending,
        })
      : new THREE.MeshPhysicalMaterial({
          color,
          emissive: new THREE.Color(0xff9d00),
          emissiveIntensity: 0.72,
          metalness: 0.88,
          roughness: 0.14,
          clearcoat: 1,
          clearcoatRoughness: 0.05,
          envMap: visualEnvironment.reflectionMap,
          envMapIntensity: 2.0,
          transparent: true,
          opacity: 0.96,
          depthWrite: false,
        }),
  );
  orb.position.z = padLightCenterZ(pad);
  if (!isBigPad) {
    orb.scale.z = 0.18;
  }
  orb.renderOrder = 23;
  orb.frustumCulled = false;
  group.add(orb);

  const lensColumn = new THREE.Mesh(
    isBigPad
      ? new THREE.CylinderGeometry(radius * 0.07, radius * 0.11, orbRadius * 2.04, 24, 1, true)
      : new THREE.CylinderGeometry(
          orbRadius * 0.72,
          orbRadius * 1.12,
          orbRadius * 0.42,
          24,
          1,
          true,
        ),
    new THREE.MeshBasicMaterial({
      color: isBigPad ? 0xffc340 : 0xffd64a,
      transparent: true,
      opacity: isBigPad ? 0.28 : 0.12,
      side: THREE.DoubleSide,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
    }),
  );
  lensColumn.rotation.x = Math.PI / 2;
  lensColumn.position.z = isBigPad
    ? PAD_SURFACE_Z_OFFSET + scaledPadDimension(58)
    : PAD_SURFACE_Z_OFFSET + scaledPadDimension(7);
  lensColumn.renderOrder = 23;
  lensColumn.frustumCulled = false;
  group.add(lensColumn);

  const lensRim = new THREE.Mesh(
    new THREE.SphereGeometry(isBigPad ? orbRadius * 1.01 : orbRadius * 1.24, 32, 14),
    new THREE.MeshBasicMaterial({
      color: isBigPad ? 0xffdf7a : color,
      transparent: true,
      opacity: isBigPad ? 0.32 : 0.12,
      side: THREE.BackSide,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
    }),
  );
  lensRim.position.z = padLightCenterZ(pad);
  if (!isBigPad) {
    lensRim.scale.z = 0.18;
  }
  lensRim.renderOrder = 24;
  lensRim.frustumCulled = false;
  group.add(lensRim);

  const sheen = new THREE.Mesh(
    new THREE.CircleGeometry(isBigPad ? orbRadius * 0.72 : orbRadius * 0.82, 24),
    new THREE.MeshBasicMaterial({
      color: isBigPad ? 0xffdf82 : 0xfff7cf,
      transparent: true,
      opacity: isBigPad ? 0.52 : 0.34,
      side: THREE.DoubleSide,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
    }),
  );
  configureOverlayMaterial(sheen.material);
  sheen.scale.y = isBigPad ? 0.36 : 0.44;
  sheen.position.set(
    isBigPad ? -orbRadius * 0.16 : -orbRadius * 0.22,
    isBigPad ? orbRadius * 0.1 : orbRadius * 0.12,
    padLightCenterZ(pad) + (isBigPad ? orbRadius * 0.5 : orbRadius * 0.16),
  );
  sheen.renderOrder = 25;
  sheen.frustumCulled = false;
  group.add(sheen);

  const glow = new THREE.Mesh(
    isBigPad
      ? new THREE.SphereGeometry(orbRadius * 1.28, 32, 14)
      : new THREE.CircleGeometry(orbRadius * 1.82, 32),
    new THREE.MeshBasicMaterial({
      color,
      transparent: true,
      opacity: isBigPad ? 0.16 : 0.28,
      side: THREE.DoubleSide,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
    }),
  );
  glow.position.z = padGlowCenterZ(pad);
  glow.renderOrder = 24;
  glow.frustumCulled = false;
  group.add(glow);

  return { group, base, ring, core, cooldown, orb, lensColumn, lensRim, sheen, glow };
}

function createVisualEnvironment(context: ReplayPlayerPluginContext): BoostPadVisualEnvironment {
  const sceneEnvironment = context.scene.scene.environment;
  if (sceneEnvironment instanceof THREE.Texture) {
    return {
      reflectionMap: sceneEnvironment,
      dispose() {},
    };
  }

  const pmrem = new THREE.PMREMGenerator(context.scene.renderer);
  const reflectionMap = pmrem.fromScene(new RoomEnvironment(), 0.04).texture;
  pmrem.dispose();
  return {
    reflectionMap,
    dispose() {
      reflectionMap.dispose();
    },
  };
}

function boostPadAvailableState(
  pad: ReplayBoostPad,
  currentTime: number,
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

  const nextAvailable = pad.events.slice(lastEventIndex + 1).find((event) => event.available);
  if (!nextAvailable || nextAvailable.time <= lastEvent.time) {
    return { available: false, progress: 0 };
  }

  return {
    available: false,
    progress: THREE.MathUtils.clamp(
      (currentTime - lastEvent.time) / (nextAvailable.time - lastEvent.time),
      0,
      1,
    ),
  };
}

function updatePadMeshes(
  meshes: BoostPadMeshes,
  pad: ReplayBoostPad,
  currentTime: number,
  showCooldownProgress: boolean,
): void {
  const { available, progress } = boostPadAvailableState(pad, currentTime);
  const isBigPad = pad.size === "big";
  const pulse = 0.92 + 0.08 * Math.sin(currentTime * 6 + pad.index * 0.45);
  const orbPulse =
    (isBigPad ? 0.98 : 0.96) +
    (isBigPad ? 0.025 : 0.04) * Math.sin(currentTime * (isBigPad ? 4.8 : 7.2) + pad.index * 0.37);
  const hover = isBigPad ? Math.sin(currentTime * 2.2 + pad.index * 0.61) * 10 : 0;
  const lightZ = padLightCenterZ(pad) + hover;
  const glowZ = padGlowCenterZ(pad) + hover;
  const orbRadius = padOrbRadius(pad);

  meshes.orb.position.z = lightZ;
  meshes.lensRim.position.z = lightZ;
  meshes.sheen.position.z = lightZ + (isBigPad ? orbRadius * 0.5 : orbRadius * 0.16);
  meshes.glow.position.z = glowZ;
  meshes.orb.rotation.z = currentTime * (isBigPad ? 0.9 : 1.25);
  meshes.lensRim.rotation.z = -currentTime * (isBigPad ? 0.7 : 1.1);
  meshes.lensColumn.rotation.z = currentTime * 0.18;
  meshes.sheen.rotation.z = currentTime * (isBigPad ? -0.35 : -0.8);
  meshes.glow.rotation.z = -currentTime * 0.45;

  if (available) {
    meshes.group.visible = true;
    setMeshBasicOpacity(meshes.base, 1);
    meshes.ring.material.opacity = isBigPad ? 0.82 : 0.95;
    meshes.core.material.opacity = isBigPad ? 0.48 : 0.5;
    meshes.cooldown.visible = false;
    meshes.base.visible = isBigPad;
    meshes.base.scale.setScalar(1 + (pulse - 0.92) * 0.18);
    meshes.ring.scale.setScalar(isBigPad ? 1 + (pulse - 0.92) * 0.32 : pulse);
    meshes.core.scale.setScalar(1);
    meshes.orb.visible = true;
    meshes.lensColumn.visible = true;
    meshes.lensRim.visible = true;
    meshes.sheen.visible = true;
    meshes.glow.visible = true;
    meshes.orb.material.opacity = isBigPad ? 0.68 : 0.98;
    meshes.lensColumn.material.opacity =
      (isBigPad ? 0.28 : 0.12) + (orbPulse - (isBigPad ? 0.98 : 0.96)) * 0.34;
    meshes.lensRim.material.opacity =
      (isBigPad ? 0.32 : 0.12) + (orbPulse - (isBigPad ? 0.98 : 0.96)) * 0.55;
    meshes.sheen.material.opacity =
      (isBigPad ? 0.52 : 0.34) + (orbPulse - (isBigPad ? 0.98 : 0.96)) * 0.7;
    meshes.glow.material.opacity = (isBigPad ? 0.16 : 0.28) + (orbPulse - (isBigPad ? 0.98 : 0.96));
    meshes.orb.scale.setScalar(orbPulse);
    meshes.lensRim.scale.setScalar(1.01 + (orbPulse - (isBigPad ? 0.98 : 0.96)) * 1.7);
    meshes.lensColumn.scale.setScalar(1 + (orbPulse - (isBigPad ? 0.98 : 0.96)) * 1.2);
    if (!isBigPad) {
      meshes.orb.scale.z = 0.18 * orbPulse;
      meshes.lensRim.scale.z = 0.18 * orbPulse;
    }
    meshes.sheen.scale.setScalar(isBigPad ? 1.02 + (orbPulse - 0.96) : 1.06 + (orbPulse - 0.96));
    meshes.sheen.scale.y *= isBigPad ? 0.36 : 0.44;
    meshes.glow.scale.setScalar(isBigPad ? 1.02 + (orbPulse - 0.96) * 2 : 1);
    return;
  }

  meshes.group.visible = true;
  setMeshBasicOpacity(meshes.base, 0.26);
  meshes.ring.material.opacity = 0.18;
  meshes.core.material.opacity = 0.07;
  meshes.base.scale.setScalar(1);
  meshes.ring.scale.setScalar(1);
  meshes.core.scale.setScalar(1);
  meshes.orb.visible = false;
  meshes.lensColumn.visible = false;
  meshes.lensRim.visible = false;
  meshes.sheen.visible = false;
  meshes.glow.visible = false;

  meshes.cooldown.visible = showCooldownProgress;
  if (showCooldownProgress) {
    const cooldownScale = 0.3 + progress * 0.7;
    meshes.cooldown.scale.setScalar(cooldownScale);
    meshes.cooldown.material.opacity = 0.16 + progress * 0.2;
  }
}

export function createBoostPadOverlayPlugin(
  options: BoostPadOverlayPluginOptions = {},
): ReplayPlayerPlugin {
  const showCooldownProgress = options.showCooldownProgress ?? true;

  let padRoot: THREE.Group | null = null;
  let visualEnvironment: BoostPadVisualEnvironment | null = null;
  const padMeshes = new Map<number, BoostPadMeshes>();

  function buildPads(context: ReplayPlayerPluginContext): void {
    padRoot = new THREE.Group();
    padRoot.name = "boost-pad-overlay";
    padRoot.renderOrder = 20;
    padRoot.frustumCulled = false;
    visualEnvironment = createVisualEnvironment(context);

    for (const pad of context.replay.boostPads) {
      const meshes = createPadMeshes(pad, visualEnvironment);
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
      updatePadMeshes(meshes, pad, context.state.currentTime, showCooldownProgress);
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
      visualEnvironment?.dispose();
      visualEnvironment = null;
      padRoot = null;
      padMeshes.clear();
    },
  };
}
