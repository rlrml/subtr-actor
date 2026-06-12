/**
 * Public types for @rlrml/viewer — most importantly the `ViewerPlugin` contract.
 *
 * The plugin model mirrors `@rlrml/player` (js/player/src/types.ts): a bare core
 * (`ViewerPlayer`) holds an ordered list of installed plugins and dispatches
 * lifecycle hooks. Everything above raw playback (scoreboard, name tags,
 * overlays, …) is a plugin — see docs/EXTENSIBILITY.md.
 */
import type * as THREE from "three";
import type { Vec3, Quat } from "./adapter/coords.js";
import type { ViewerPlayer } from "./ViewerPlayer.js";

export type { Vec3, Quat };

/** Snapshot of playback state, emitted on every "change" event. */
export interface ViewerState {
  currentTime: number;
  duration: number;
  playing: boolean;
  speed: number;
}

/** Per-frame resolved ball state handed to `beforeRender`. */
export interface BallRenderState {
  position: Vec3;
  rotation: Quat;
  velocity: Vec3;
  visible: boolean;
  /** The ball's THREE object (post-interpolation transform), if spawned. */
  object3d: THREE.Object3D | null;
}

/** Per-frame resolved car state handed to `beforeRender`. */
export interface CarRenderState {
  name: string;
  team: number;
  carName: string;
  hitboxType: string;
  position: Vec3;
  rotation: Quat;
  velocity: Vec3;
  /** 0-100 */
  boost: number;
  isBoosting: boolean;
  visible: boolean;
  /** The car's THREE object (post-interpolation transform), if spawned. */
  object3d: THREE.Object3D | null;
}

export interface ViewerPluginContext {
  /** The core; exposes playback control, state, and the subtr-actor adapter. */
  player: ViewerPlayer;
  scene: THREE.Scene;
  camera: THREE.PerspectiveCamera;
  renderer: THREE.WebGLRenderer;
  /** For plugins that add DOM overlays (HUD, scoreboard, indicators). */
  container: HTMLElement;
}

export interface ViewerPluginStateContext extends ViewerPluginContext {
  state: ViewerState;
}

export interface ViewerRenderContext extends ViewerPluginContext {
  /** Current playback time (s). */
  time: number;
  ball: BallRenderState;
  cars: CarRenderState[];
}

export interface ViewerPlugin {
  id: string;
  /** Install: attach meshes/DOM, subscribe to events. */
  setup?(ctx: ViewerPluginContext): void;
  /** Play/pause/seek/speed changes (and once at install). */
  onStateChange?(ctx: ViewerPluginStateContext): void;
  /** Per frame, after ball/car transforms resolve, before renderer.render. */
  beforeRender?(ctx: ViewerRenderContext): void;
  /** Uninstall: dispose everything created in setup. */
  teardown?(ctx: ViewerPluginContext): void;
}

export type ViewerPluginFactory = () => ViewerPlugin;
export type ViewerPluginDefinition = ViewerPlugin | ViewerPluginFactory;

export interface ViewerOptions {
  plugins?: ViewerPluginDefinition[];
  autoplay?: boolean;
  /** Initial playback rate (default 1). */
  speed?: number;
  /** Wrap to t=0 at the end instead of pausing (default false). */
  loop?: boolean;
  /** Boost/supersonic/ball trail effects (default true). */
  effects?: boolean;
}
