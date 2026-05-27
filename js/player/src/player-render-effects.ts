import * as THREE from "three";
import type { DemoIndicator } from "./scene";
import type { ReplayModel, ReplayTimelineEvent, Vec3 } from "./types";
import { rootPosition } from "./player-internals/spatial";

const DEMO_INDICATOR_DURATION_SECONDS = 3.2;

export function getActiveDemoEvent(
  replay: ReplayModel,
  victimPlayerId: string,
  currentTime: number,
): ReplayTimelineEvent | null {
  for (let index = replay.timelineEvents.length - 1; index >= 0; index -= 1) {
    const event = replay.timelineEvents[index]!;
    const age = currentTime - event.time;
    if (age < 0) {
      continue;
    }
    if (age > DEMO_INDICATOR_DURATION_SECONDS) {
      break;
    }
    if (event.kind === "demo" && event.secondaryPlayerId === victimPlayerId) {
      return event;
    }
  }
  return null;
}

export function updateDemoIndicator(options: {
  indicator: DemoIndicator | null;
  fallbackPosition: Vec3 | null;
  demoEvent: ReplayTimelineEvent | null;
  currentTime: number;
  camera: THREE.Camera;
}): void {
  const { indicator, fallbackPosition, demoEvent, currentTime, camera } = options;
  if (!indicator) {
    return;
  }

  const position = demoEvent?.location ?? fallbackPosition;
  if (!demoEvent || !position) {
    indicator.group.visible = false;
    return;
  }

  const age = Math.max(0, currentTime - demoEvent.time);
  const phase = currentTime * 8;
  const pulse = 1 + 0.08 * Math.sin(phase);
  indicator.group.visible = true;
  indicator.group.position.copy(rootPosition(position));
  indicator.ring.rotation.z = phase * 0.15;
  indicator.ring.scale.setScalar(pulse);
  indicator.label.quaternion.copy(camera.quaternion);
  indicator.label.scale.setScalar(1 + 0.04 * Math.sin(phase + 1.3));

  const opacity = THREE.MathUtils.clamp(1 - age / DEMO_INDICATOR_DURATION_SECONDS, 0.28, 1);
  for (const node of [indicator.ring, indicator.label]) {
    const material = node.material;
    if (material instanceof THREE.Material) {
      material.opacity = opacity;
    }
  }
}

export function updateBoostTrail(
  boostTrail: THREE.Group,
  boostActive: boolean,
  boostFraction: number,
  time: number,
  playerIndex: number,
): void {
  if (!boostActive) {
    boostTrail.visible = false;
    return;
  }

  boostTrail.visible = true;

  const phase = time * 36 + playerIndex * 1.7;
  const pulse = 0.86 + 0.14 * Math.sin(phase);
  const intensity = THREE.MathUtils.clamp(0.62 + boostFraction * 0.88, 0.62, 1.5);
  const lengthScale = intensity * (1.02 + pulse * 0.52);
  const widthScale = 1.02 + intensity * 0.28;
  boostTrail.scale.set(lengthScale, widthScale, widthScale);

  for (const [index, child] of boostTrail.children.entries()) {
    const plume = child as THREE.Group;
    const plumePulse = 0.92 + 0.14 * Math.sin(phase + index * 0.85);
    plume.scale.setScalar(plumePulse);

    plume.traverse((node: THREE.Object3D) => {
      if (!(node instanceof THREE.Mesh)) {
        return;
      }

      const material = node.material;
      if (!(material instanceof THREE.MeshBasicMaterial)) {
        return;
      }

      switch (node.name) {
        case "outer-flame":
          material.opacity = 0.24 + intensity * 0.24;
          break;
        case "inner-flame":
          material.opacity = 0.58 + intensity * 0.3;
          break;
        case "glow":
          material.opacity = 0.4 + intensity * 0.26;
          break;
        default:
          break;
      }
    });
  }
}
