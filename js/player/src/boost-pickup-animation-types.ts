import type * as THREE from "three";
import type {
  ReplayBoostPad,
  ReplayBoostPadEvent,
  ReplayPlayerPluginContext,
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

export interface BoostPickupAnimationEvent {
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

export type BoostPickupAnimationContext = ReplayPlayerPluginContext;
