export const PLAYBACK_RATE_NOTCHES = [0.25, 0.5, 1, 1.5, 2] as const;

const PLAYBACK_RATE_SNAP_DISTANCE = 0.06;

export function snapPlaybackRate(value: number): number {
  if (!Number.isFinite(value)) {
    return 1;
  }

  for (const notch of PLAYBACK_RATE_NOTCHES) {
    if (Math.abs(value - notch) <= PLAYBACK_RATE_SNAP_DISTANCE) {
      return notch;
    }
  }
  return Math.min(2, Math.max(0.25, value));
}

export function formatPlaybackRate(value: number): string {
  return `${value.toFixed(2).replace(/\.?0+$/, "")}x`;
}
