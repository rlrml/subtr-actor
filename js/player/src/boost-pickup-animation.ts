import * as THREE from "three";
import type {
  ReplayBoostPad,
  ReplayBoostPadEvent,
  ReplayPlayerPlugin,
  ReplayPlayerPluginContext,
  ReplayPlayerRenderContext,
  ReplayPlayerTrack,
} from "./types";

export interface BoostPickupAnimationPluginOptions {
  durationSeconds?: number;
  includePickup?: BoostPickupAnimationFilter;
}

export interface BoostPickupAnimationPickup {
  pad: ReplayBoostPad;
  event: ReplayBoostPadEvent;
  player: ReplayPlayerTrack;
}

export type BoostPickupAnimationFilter = (pickup: BoostPickupAnimationPickup) => boolean;

interface BoostPickupAnimationEvent {
  time: number;
  pad: ReplayBoostPad;
  event: ReplayBoostPadEvent;
  player: ReplayPlayerTrack;
  color: string;
  currentCount: number | null;
  position: THREE.Vector3;
  size: ReplayBoostPad["size"];
  group: THREE.Group;
  textMaterial: THREE.SpriteMaterial;
  ringMaterial: THREE.MeshBasicMaterial;
}

const DEFAULT_DURATION_SECONDS = 1.35;
const TEAM_ZERO_COLOR = "#57a8ff";
const TEAM_ONE_COLOR = "#ff9c40";
const TEXT_CANVAS_WIDTH = 256;
const TEXT_CANVAS_HEIGHT = 160;
const SPRITE_BASE_WIDTH = 360;
const SPRITE_BASE_HEIGHT = 225;
const SMALL_PAD_TEXT_Z = 260;
const BIG_PAD_TEXT_Z = 430;
const RING_Z = 18;
const RING_BASE_RADIUS = 120;

function teamColor(isTeamZero: boolean): string {
  return isTeamZero ? TEAM_ZERO_COLOR : TEAM_ONE_COLOR;
}

function padPickupEvents(pad: ReplayBoostPad): ReplayBoostPadEvent[] {
  return pad.events.filter((event) => !event.available && event.playerId);
}

function createCountTexture(count: number, color: string): THREE.CanvasTexture {
  const canvas = document.createElement("canvas");
  canvas.width = TEXT_CANVAS_WIDTH;
  canvas.height = TEXT_CANVAS_HEIGHT;
  const context = canvas.getContext("2d");
  if (!context) {
    throw new Error("Unable to create boost pickup count canvas");
  }

  context.clearRect(0, 0, canvas.width, canvas.height);
  context.textAlign = "center";
  context.textBaseline = "middle";
  context.lineJoin = "round";
  context.font = "800 124px sans-serif";
  context.lineWidth = 18;
  context.strokeStyle = "rgba(4, 10, 18, 0.88)";
  context.strokeText(`${count}`, canvas.width / 2, canvas.height / 2);
  context.fillStyle = color;
  context.fillText(`${count}`, canvas.width / 2, canvas.height / 2);

  const texture = new THREE.CanvasTexture(canvas);
  texture.colorSpace = THREE.SRGBColorSpace;
  texture.needsUpdate = true;
  return texture;
}

function disposeTexture(texture: THREE.Texture | null): void {
  texture?.dispose();
}

function createPickupGroup(color: string): {
  group: THREE.Group;
  textMaterial: THREE.SpriteMaterial;
  ringMaterial: THREE.MeshBasicMaterial;
} {
  const group = new THREE.Group();
  group.visible = false;
  group.renderOrder = 60;
  group.frustumCulled = false;

  const texture = createCountTexture(1, color);
  const textMaterial = new THREE.SpriteMaterial({
    map: texture,
    transparent: true,
    depthTest: false,
    depthWrite: false,
  });
  const sprite = new THREE.Sprite(textMaterial);
  sprite.scale.set(SPRITE_BASE_WIDTH, SPRITE_BASE_HEIGHT, 1);
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
    new THREE.RingGeometry(RING_BASE_RADIUS * 0.72, RING_BASE_RADIUS, 36),
    ringMaterial,
  );
  ring.position.z = RING_Z;
  ring.renderOrder = 61;
  ring.frustumCulled = false;
  group.add(ring);

  return { group, textMaterial, ringMaterial };
}

function syncPickupCountTexture(event: BoostPickupAnimationEvent, count: number): void {
  if (event.currentCount === count) {
    return;
  }

  disposeTexture(event.textMaterial.map);
  event.textMaterial.map = createCountTexture(count, event.color);
  event.textMaterial.needsUpdate = true;
  event.currentCount = count;
}

function buildAnimationEvents(context: ReplayPlayerPluginContext): BoostPickupAnimationEvent[] {
  const playersById = new Map<string, ReplayPlayerTrack>();
  for (const player of context.replay.players) {
    playersById.set(player.id, player);
  }

  const rawEvents: Array<{
    pad: ReplayBoostPad;
    event: ReplayBoostPadEvent;
  }> = [];
  for (const pad of context.replay.boostPads) {
    for (const event of padPickupEvents(pad)) {
      rawEvents.push({ pad, event });
    }
  }
  rawEvents.sort((left, right) => {
    if (left.event.time !== right.event.time) {
      return left.event.time - right.event.time;
    }
    if (left.event.frame !== right.event.frame) {
      return left.event.frame - right.event.frame;
    }
    return left.pad.index - right.pad.index;
  });

  const animationEvents: BoostPickupAnimationEvent[] = [];
  for (const { pad, event } of rawEvents) {
    if (!event.playerId) {
      continue;
    }
    const player = playersById.get(event.playerId);
    if (!player) {
      continue;
    }

    const color = teamColor(player.isTeamZero);
    const { group, textMaterial, ringMaterial } = createPickupGroup(color);
    group.position.copy(pad.position);
    context.scene.replayRoot.add(group);
    animationEvents.push({
      time: event.time,
      pad,
      event,
      player,
      color,
      currentCount: 1,
      position: new THREE.Vector3(pad.position.x, pad.position.y, pad.position.z),
      size: pad.size,
      group,
      textMaterial,
      ringMaterial,
    });
  }

  return animationEvents;
}

function updateAnimationEvent(
  event: BoostPickupAnimationEvent,
  elapsed: number,
  durationSeconds: number,
): void {
  const progress = THREE.MathUtils.clamp(elapsed / durationSeconds, 0, 1);
  const easedOut = 1 - Math.pow(1 - progress, 3);
  const easedIn = progress * progress;
  const textBaseZ = event.size === "big" ? BIG_PAD_TEXT_Z : SMALL_PAD_TEXT_Z;
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
    ring.position.z = RING_Z - textBaseZ - easedOut * textRise;
  }
}

export function createBoostPickupAnimationPlugin(
  options: BoostPickupAnimationPluginOptions = {},
): ReplayPlayerPlugin {
  const durationSeconds = Math.max(0.1, options.durationSeconds ?? DEFAULT_DURATION_SECONDS);
  let events: BoostPickupAnimationEvent[] = [];

  function includeEvent(event: BoostPickupAnimationEvent): boolean {
    return (
      options.includePickup?.({
        pad: event.pad,
        event: event.event,
        player: event.player,
      }) ?? true
    );
  }

  function hideAll(): void {
    for (const event of events) {
      event.group.visible = false;
    }
  }

  return {
    id: "boost-pickup-animation",
    setup(context): void {
      events = buildAnimationEvents(context);
    },
    beforeRender(context: ReplayPlayerRenderContext): void {
      if (!context.state.boostPickupAnimationEnabled) {
        hideAll();
        return;
      }

      const startTime = context.currentTime - durationSeconds;
      const countsByPlayer = new Map<string, number>();
      for (const event of events) {
        if (event.time > context.currentTime) {
          event.group.visible = false;
          continue;
        }
        if (!includeEvent(event)) {
          event.group.visible = false;
          continue;
        }

        const pickupCount = (countsByPlayer.get(event.player.id) ?? 0) + 1;
        countsByPlayer.set(event.player.id, pickupCount);
        if (event.time < startTime) {
          event.group.visible = false;
          continue;
        }

        syncPickupCountTexture(event, pickupCount);
        updateAnimationEvent(event, context.currentTime - event.time, durationSeconds);
      }
    },
    teardown(): void {
      for (const event of events) {
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
      events = [];
    },
  };
}
