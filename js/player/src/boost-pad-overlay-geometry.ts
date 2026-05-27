import type { ReplayBoostPad } from "./types";

export const BOOST_PAD_SURFACE_Z_OFFSET = 6;
const BOOST_PAD_VISUAL_SCALE = 0.6;

function scaledPadDimension(value: number): number {
  return value * BOOST_PAD_VISUAL_SCALE;
}

export function boostPadRadius(pad: ReplayBoostPad): number {
  return scaledPadDimension(pad.size === "big" ? 150 : 92);
}

export function boostPadOrbRadius(pad: ReplayBoostPad): number {
  return scaledPadDimension(pad.size === "big" ? 155 : 46);
}

function padOrbBottomClearance(pad: ReplayBoostPad): number {
  return scaledPadDimension(pad.size === "big" ? 34 : 14);
}

function padOrbCenterZ(pad: ReplayBoostPad): number {
  return BOOST_PAD_SURFACE_Z_OFFSET + padOrbBottomClearance(pad) + boostPadOrbRadius(pad);
}

export function boostPadLightCenterZ(pad: ReplayBoostPad): number {
  if (pad.size === "big") {
    return padOrbCenterZ(pad);
  }
  return BOOST_PAD_SURFACE_Z_OFFSET + scaledPadDimension(1.2);
}

export function boostPadGlowCenterZ(pad: ReplayBoostPad): number {
  if (pad.size === "big") {
    return padOrbCenterZ(pad);
  }
  return BOOST_PAD_SURFACE_Z_OFFSET + scaledPadDimension(0.8);
}

export function boostPadColor(pad: ReplayBoostPad): number {
  return pad.size === "big" ? 0xf59e0b : 0xfacc15;
}
