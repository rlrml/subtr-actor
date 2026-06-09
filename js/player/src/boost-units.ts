// Canonical boost-unit conversion for JavaScript/TypeScript consumers of
// subtr-actor.
//
// Rocket League replays store boost on a raw `0..=255` scale (a full tank is
// 255 and a standard kickoff starts at 85). subtr-actor keeps boost in these
// raw units all the way through processing and the bindings; the rescale to the
// `0..=100` display scale must happen at display time. This module is the one
// place the JS bindings perform that conversion, so every consumer renders
// boost the same way.
//
// Mirrors `boost_amount_to_percent` / `boost_percent_to_amount` in
// `src/domain/boost_units.rs`.

/** The maximum raw boost value stored in replay data. */
export const BOOST_RAW_MAX = 255;

/** Converts a raw replay boost amount (`0..=255`) to a percentage (`0..=100`). */
export function boostAmountToPercent(amount: number): number;
export function boostAmountToPercent(amount: number | null | undefined): number | null;
export function boostAmountToPercent(amount: number | null | undefined): number | null {
  if (amount == null) return null;
  return (amount * 100) / BOOST_RAW_MAX;
}

/** Converts a boost percentage (`0..=100`) to a raw replay boost amount (`0..=255`). */
export function boostPercentToAmount(percent: number): number;
export function boostPercentToAmount(percent: number | null | undefined): number | null;
export function boostPercentToAmount(percent: number | null | undefined): number | null {
  if (percent == null) return null;
  return (percent * BOOST_RAW_MAX) / 100;
}
