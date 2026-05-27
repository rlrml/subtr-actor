import * as THREE from "three";
import type { Quaternion, Vec3 } from "../types";

export function interpolatePosition(
  current: Vec3 | null,
  next: Vec3 | null,
  alpha: number,
): Vec3 | null {
  if (!current) {
    return next;
  }

  if (!next || alpha <= 0) {
    return current;
  }

  return {
    x: THREE.MathUtils.lerp(current.x, next.x, alpha),
    y: THREE.MathUtils.lerp(current.y, next.y, alpha),
    z: THREE.MathUtils.lerp(current.z, next.z, alpha),
  };
}

export function interpolateQuaternion(
  current: Quaternion | null,
  next: Quaternion | null,
  alpha: number,
): THREE.Quaternion | null {
  const source = current ?? next;
  if (!source) {
    return null;
  }

  const result = new THREE.Quaternion(source.x, source.y, source.z, source.w);
  if (!next || alpha <= 0 || current === null) {
    return result;
  }

  return result.slerp(new THREE.Quaternion(next.x, next.y, next.z, next.w), alpha);
}

export function rootPosition(position: Vec3): THREE.Vector3 {
  return new THREE.Vector3(position.x, position.y, position.z);
}

export function worldPosition(position: Vec3, fieldScale: number): THREE.Vector3 {
  return new THREE.Vector3(
    -position.x * fieldScale,
    position.y * fieldScale,
    position.z * fieldScale,
  );
}

export function worldDirection(direction: Vec3): THREE.Vector3 {
  return new THREE.Vector3(-direction.x, direction.y, direction.z).normalize();
}
