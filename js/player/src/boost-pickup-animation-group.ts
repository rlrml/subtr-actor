import * as THREE from "three";
import {
  BOOST_PICKUP_RING_BASE_RADIUS,
  BOOST_PICKUP_RING_Z,
  BOOST_PICKUP_SPRITE_BASE_HEIGHT,
  BOOST_PICKUP_SPRITE_BASE_WIDTH,
} from "./boost-pickup-animation-constants";
import type { BoostPickupAnimationEvent } from "./boost-pickup-animation-types";
import {
  createBoostPickupCountTexture,
  disposeBoostPickupTexture,
} from "./boost-pickup-animation-texture";

export function createBoostPickupGroup(color: string): {
  group: THREE.Group;
  textMaterial: THREE.SpriteMaterial;
  ringMaterial: THREE.MeshBasicMaterial;
} {
  const group = new THREE.Group();
  group.visible = false;
  group.renderOrder = 60;
  group.frustumCulled = false;

  const texture = createBoostPickupCountTexture(1, color);
  const textMaterial = new THREE.SpriteMaterial({
    map: texture,
    transparent: true,
    depthTest: false,
    depthWrite: false,
  });
  const sprite = new THREE.Sprite(textMaterial);
  sprite.scale.set(BOOST_PICKUP_SPRITE_BASE_WIDTH, BOOST_PICKUP_SPRITE_BASE_HEIGHT, 1);
  sprite.renderOrder = 62;
  sprite.frustumCulled = false;
  group.add(sprite);

  const ringMaterial = new THREE.MeshBasicMaterial({
    color,
    transparent: true,
    opacity: 0,
    side: THREE.DoubleSide,
    depthTest: false,
    depthWrite: false,
    blending: THREE.AdditiveBlending,
  });
  const ring = new THREE.Mesh(
    new THREE.RingGeometry(BOOST_PICKUP_RING_BASE_RADIUS * 0.72, BOOST_PICKUP_RING_BASE_RADIUS, 36),
    ringMaterial,
  );
  ring.position.z = BOOST_PICKUP_RING_Z;
  ring.renderOrder = 61;
  ring.frustumCulled = false;
  group.add(ring);

  return { group, textMaterial, ringMaterial };
}

export function syncBoostPickupCountTexture(
  event: BoostPickupAnimationEvent,
  count: number,
): void {
  if (event.currentCount === count) {
    return;
  }

  disposeBoostPickupTexture(event.textMaterial.map);
  event.textMaterial.map = createBoostPickupCountTexture(count, event.color);
  event.textMaterial.needsUpdate = true;
  event.currentCount = count;
}

export function disposeBoostPickupAnimationEvent(event: BoostPickupAnimationEvent): void {
  event.group.removeFromParent();
  event.group.traverse((node) => {
    if (node instanceof THREE.Mesh || node instanceof THREE.Sprite) {
      node.geometry?.dispose();
    }
  });
  event.textMaterial.map?.dispose();
  event.textMaterial.dispose();
  event.ringMaterial.dispose();
}
