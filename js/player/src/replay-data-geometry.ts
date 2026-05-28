import type { Quaternion, Vec3 } from "./types";

export function normalizeVector(value: Vec3): Vec3 | null {
  const magnitude = Math.hypot(value.x, value.y, value.z);
  if (magnitude < 0.000001) {
    return null;
  }

  return {
    x: value.x / magnitude,
    y: value.y / magnitude,
    z: value.z / magnitude,
  };
}

export function normalizeQuaternion(raw: Quaternion): Quaternion | null {
  const magnitude = Math.hypot(raw.x, raw.y, raw.z, raw.w);
  if (magnitude < 0.000001) {
    return null;
  }

  return {
    x: raw.x / magnitude,
    y: raw.y / magnitude,
    z: raw.z / magnitude,
    w: raw.w / magnitude,
  };
}

function multiplyQuaternions(left: Quaternion, right: Quaternion): Quaternion {
  return {
    w: left.w * right.w - left.x * right.x - left.y * right.y - left.z * right.z,
    x: left.w * right.x + left.x * right.w + left.y * right.z - left.z * right.y,
    y: left.w * right.y - left.x * right.z + left.y * right.w + left.z * right.x,
    z: left.w * right.z + left.x * right.y - left.y * right.x + left.z * right.w,
  };
}

export function rotateVectorByQuaternion(vector: Vec3, quaternion: Quaternion): Vec3 {
  const rotated = multiplyQuaternions(
    multiplyQuaternions(quaternion, {
      x: vector.x,
      y: vector.y,
      z: vector.z,
      w: 0,
    }),
    {
      x: -quaternion.x,
      y: -quaternion.y,
      z: -quaternion.z,
      w: quaternion.w,
    },
  );

  return {
    x: rotated.x,
    y: rotated.y,
    z: rotated.z,
  };
}
