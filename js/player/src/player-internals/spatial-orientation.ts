import * as THREE from "three";
import type { ReplayModel } from "../types";
import {
  DEFAULT_FORWARD,
  DEFAULT_UP,
  GROUND_HEIGHT_THRESHOLD_UU,
} from "./spatial-constants";
import { worldDirection } from "./spatial-vectors";

export function getOrientationVectors(
  frame: ReplayModel["players"][number]["frames"][number],
): {
  forward: THREE.Vector3;
  up: THREE.Vector3;
  right: THREE.Vector3;
} | null {
  const velocity = frame.linearVelocity ? worldDirection(frame.linearVelocity) : null;
  const rawForward = frame.forward ? worldDirection(frame.forward) : null;
  const rawUp = frame.up ? worldDirection(frame.up) : null;
  const grounded = (frame.position?.z ?? Infinity) < GROUND_HEIGHT_THRESHOLD_UU;

  if (grounded) {
    const forward = (rawForward ?? velocity ?? DEFAULT_FORWARD.clone()).clone().setZ(0);

    if (forward.lengthSq() < 0.0001) {
      return null;
    }

    forward.normalize();
    if (velocity && velocity.lengthSq() > 0.0001 && forward.dot(velocity) < 0) {
      forward.negate();
    }
    const right = new THREE.Vector3().crossVectors(DEFAULT_UP, forward).normalize();
    const up = new THREE.Vector3().crossVectors(forward, right).normalize();
    return { forward, up, right };
  }

  if (!rawForward || !rawUp) {
    return null;
  }

  const forward = rawForward.clone().normalize();
  const right = new THREE.Vector3().crossVectors(rawUp, forward).normalize();
  const up = new THREE.Vector3().crossVectors(forward, right).normalize();

  return { forward, up, right };
}
