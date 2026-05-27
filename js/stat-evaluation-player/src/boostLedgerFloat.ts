export function f32(value: number): number {
  return Math.fround(value);
}

export function addF32(left: number, right: number): number {
  return f32(f32(left) + f32(right));
}

export function subF32(left: number, right: number): number {
  return f32(f32(left) - f32(right));
}

export function mulF32(left: number, right: number): number {
  return f32(f32(left) * f32(right));
}

export function divF32(left: number, right: number): number {
  return f32(f32(left) / f32(right));
}
