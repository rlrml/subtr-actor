import * as THREE from "three";
import type { ReplayBoostPad } from "./types";

export function boostPadAvailableState(
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
