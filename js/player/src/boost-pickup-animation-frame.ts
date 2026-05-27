import * as THREE from "three";
import {
  BOOST_PICKUP_BIG_PAD_TEXT_Z,
  BOOST_PICKUP_RING_Z,
  BOOST_PICKUP_SMALL_PAD_TEXT_Z,
} from "./boost-pickup-animation-constants";
import type { BoostPickupAnimationEvent } from "./boost-pickup-animation-types";

export function updateBoostPickupAnimationEvent(
  event: BoostPickupAnimationEvent,
  elapsed: number,
  durationSeconds: number,
): void {
  const progress = THREE.MathUtils.clamp(elapsed / durationSeconds, 0, 1);
  const easedOut = 1 - Math.pow(1 - progress, 3);
  const easedIn = progress * progress;
  const textBaseZ = event.size === "big" ? BOOST_PICKUP_BIG_PAD_TEXT_Z : BOOST_PICKUP_SMALL_PAD_TEXT_Z;
  const textRise = event.size === "big" ? 360 : 280;
  const padPulse = 1 + Math.sin(progress * Math.PI) * 0.22;

  event.group.visible = true;
  event.group.position.set(
    event.position.x,
    event.position.y,
    event.position.z + textBaseZ + easedOut * textRise,
  );
  event.group.scale.setScalar(padPulse);
  event.textMaterial.opacity = Math.max(0, 1 - easedIn);
  event.ringMaterial.opacity = Math.max(0, 0.48 * (1 - progress));

  const ring = event.group.children[1];
  if (ring) {
    const ringScale = 0.75 + easedOut * (event.size === "big" ? 2.8 : 1.85);
    ring.scale.setScalar(ringScale);
    ring.position.z = BOOST_PICKUP_RING_Z - textBaseZ - easedOut * textRise;
  }
}
