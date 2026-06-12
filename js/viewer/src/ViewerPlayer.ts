/**
 * ViewerPlayer — the bare playback core of @rlrml/viewer.
 *
 * Deliberately minimal, like `@rlrml/player`'s ReplayPlayer: it owns the
 * renderer (scene / arena / actors), a playback clock, and the plugin host —
 * nothing else. Scoreboard, name tags, overlays, effects polish, custom
 * cameras: all plugins (docs/EXTENSIBILITY.md).
 *
 * Data flows one way each frame:
 *   adapter.seek(t) → ActorManager interpolates meshes → plugins beforeRender →
 *   renderer.render.
 */
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { SceneManager } from "./managers/SceneManager.js";
import { ArenaManager } from "./managers/ArenaManager.js";
import { ActorManager } from "./managers/ActorManager.js";
import { EffectsManager } from "./managers/EffectsManager.js";
import type { SubtrActorPlayer } from "./adapter/SubtrActorPlayer.js";
import type {
  BallRenderState,
  CarRenderState,
  ViewerOptions,
  ViewerPlugin,
  ViewerPluginContext,
  ViewerPluginDefinition,
  ViewerPluginStateContext,
  ViewerRenderContext,
  ViewerState,
} from "./types.js";

type ViewerListener = (state: ViewerState) => void;
type InstalledPlugin = { definition: ViewerPluginDefinition; plugin: ViewerPlugin };

// With `effects: false`, every EffectsManager call from ActorManager is a no-op.
const effectsStub = new Proxy({}, { get: () => () => {} }) as EffectsManager;

export class ViewerPlayer extends EventTarget {
  readonly container: HTMLElement;
  /** The subtr-actor adapter — the sole data source (timelines + live entities). */
  readonly adapter: SubtrActorPlayer;
  readonly options: ViewerOptions;

  readonly sceneManager: SceneManager;
  readonly arenaManager: ArenaManager;
  readonly actorManager: ActorManager;
  readonly effectsManager: EffectsManager;
  readonly controls: OrbitControls;
  private readonly effectsEnabled: boolean;
  /** Resolves when async assets (arena meshes, ball model) are in the scene. */
  readonly ready: Promise<void>;

  private readonly plugins: InstalledPlugin[] = [];
  private resizeObserver: ResizeObserver | null = null;
  private animationFrameId: number | null = null;
  private disposed = false;
  private playing = false;
  private speed: number;
  private loop: boolean;
  private currentTime = 0;
  private lastTickAt: number | null = null;

  constructor(container: HTMLElement, adapter: SubtrActorPlayer, options: ViewerOptions = {}) {
    super();
    this.container = container;
    this.adapter = adapter;
    this.options = options;
    this.speed = Math.max(0.1, options.speed ?? 1);
    this.loop = options.loop ?? false;

    this.sceneManager = new SceneManager(container);
    this.sceneManager.initDefaultEnvironment();
    this.arenaManager = new ArenaManager(this.scene);
    // Trails (boost / supersonic / ball). Explosions stay dormant until the
    // adapter exposes goal/demo events (its event getters are still stubs).
    this.effectsEnabled = options.effects ?? true;
    this.effectsManager = this.effectsEnabled ? new EffectsManager(this.scene) : effectsStub;
    this.actorManager = new ActorManager(this.scene, this.effectsManager);
    this.actorManager.initFromFramework(adapter);
    this.actorManager.initInterpolants(adapter.getTimelines());
    // NOTE: deliberately NOT calling effectsManager.setRenderContext() yet — it
    // pre-warms the explosion shader pools, which blocks the main thread for
    // seconds, and nothing can trigger explosions until the adapter exposes
    // goal/demo events. Call it when those events land.

    // Default camera: simple orbit. The full follow/ballcam path is a later
    // bring-up; plugins can also drive `camera` directly.
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.camera.position.set(0, 4000, 6000);
    this.controls.target.set(0, 200, 0);
    this.controls.update();

    this.ready = Promise.all([
      this.arenaManager.loadArenaMeshes().catch((e: unknown) => {
        console.warn("[viewer] arena load failed", e);
      }),
      this.actorManager.waitForBallModel().catch(() => false),
    ]).then(() => undefined);

    this.installResizeHandling();
    for (const definition of options.plugins ?? []) {
      this.installPlugin(definition, false);
    }
    this.scheduleAnimationFrame();
    this.emitChange();

    if (options.autoplay) {
      this.play();
    }
  }

  get scene(): THREE.Scene {
    return this.sceneManager.scene as THREE.Scene;
  }
  get camera(): THREE.PerspectiveCamera {
    return this.sceneManager.camera as THREE.PerspectiveCamera;
  }
  get renderer(): THREE.WebGLRenderer {
    return this.sceneManager.renderer as THREE.WebGLRenderer;
  }
  get duration(): number {
    return this.adapter.duration;
  }

  // ── Playback control ────────────────────────────────────────────────────────
  play(): void {
    if (this.playing) return;
    this.playing = true;
    this.lastTickAt = null;
    this.actorManager.resumeAnimations();
    this.emitChange();
  }

  pause(): void {
    if (!this.playing) return;
    this.playing = false;
    this.lastTickAt = null;
    this.actorManager.pauseAnimations();
    this.emitChange();
  }

  togglePlayback(): void {
    this.playing ? this.pause() : this.play();
  }

  seek(time: number): void {
    this.currentTime = THREE.MathUtils.clamp(time, 0, this.duration);
    // Sync the THREE animation system (if active) to the new time, and reset
    // trackers that work off frame-to-frame deltas so they don't see the jump:
    // the ball trail would draw a segment connecting old/new positions and the
    // wheels would spin wildly from the position delta.
    this.actorManager.seekAnimations(this.currentTime);
    this.effectsManager.resetBallTrail();
    this.actorManager.resetWheelTracking();
    this.emitChange();
  }

  setPlaybackRate(speed: number): void {
    this.speed = Math.max(0.1, speed);
    this.emitChange();
  }

  setLoop(loop: boolean): void {
    this.loop = loop;
  }

  getState(): ViewerState {
    return {
      currentTime: this.currentTime,
      duration: this.duration,
      playing: this.playing,
      speed: this.speed,
    };
  }

  subscribe(listener: ViewerListener): () => void {
    const handleChange = (event: Event): void => {
      listener((event as CustomEvent<ViewerState>).detail);
    };
    this.addEventListener("change", handleChange);
    listener(this.getState());
    return () => {
      this.removeEventListener("change", handleChange);
    };
  }

  // ── Plugin host (mirrors @rlrml/player) ────────────────────────────────────
  addPlugin(definition: ViewerPluginDefinition): () => void {
    return this.installPlugin(definition, true);
  }

  removePlugin(id: string): boolean {
    const index = this.plugins.findIndex((entry) => entry.plugin.id === id);
    if (index < 0) return false;
    const [entry] = this.plugins.splice(index, 1);
    entry.plugin.teardown?.(this.createPluginContext());
    return true;
  }

  getPlugins(): ViewerPlugin[] {
    return this.plugins.map((entry) => entry.plugin);
  }

  destroy(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.playing = false;
    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId);
      this.animationFrameId = null;
    }
    this.resizeObserver?.disconnect();
    this.resizeObserver = null;
    while (this.plugins.length > 0) {
      const entry = this.plugins.pop();
      entry?.plugin.teardown?.(this.createPluginContext());
    }
    this.controls.dispose();
    if (this.effectsEnabled) {
      this.effectsManager.reset();
    }
    this.actorManager.reset();
    this.sceneManager.dispose();
  }

  dispose(): void {
    this.destroy();
  }

  // ── Internals ───────────────────────────────────────────────────────────────
  private installResizeHandling(): void {
    if (typeof ResizeObserver === "undefined") return; // SceneManager's window listener covers it
    this.resizeObserver = new ResizeObserver(() => this.sceneManager.onWindowResize());
    this.resizeObserver.observe(this.container);
  }

  private scheduleAnimationFrame(): void {
    if (this.animationFrameId !== null || this.disposed) return;
    this.animationFrameId = requestAnimationFrame(this.tick);
  }

  private tick = (now: number): void => {
    this.animationFrameId = null;
    if (this.disposed) return;

    let timeChanged = false;
    let dt = 0;
    if (this.playing) {
      dt = this.lastTickAt === null ? 0 : Math.min(0.1, (now - this.lastTickAt) / 1000);
      this.lastTickAt = now;
      let next = this.currentTime + dt * this.speed;
      if (next >= this.duration) {
        if (this.loop) {
          next = 0;
          // Wrapping is a seek: clear delta-based trackers (see seek()).
          this.actorManager.seekAnimations(0);
          this.effectsManager.resetBallTrail();
          this.actorManager.resetWheelTracking();
        } else {
          next = this.duration;
          this.playing = false;
        }
      }
      timeChanged = next !== this.currentTime || !this.playing;
      this.currentTime = next;
    }

    this.render(dt);
    if (timeChanged) {
      this.emitChange();
    }
    this.scheduleAnimationFrame();
  };

  private render(dt = 0): void {
    this.adapter.seek(this.currentTime);
    // Original GameEngine frame order: advance the THREE animation system (when
    // active it owns positions) BEFORE updateFromFramework applies entity state.
    if (this.playing) {
      this.actorManager.updateAnimations(dt * this.speed);
    }
    this.actorManager.updateFromFramework(this.adapter, this.currentTime);
    this.updatePlayerStates();
    this.effectsManager.update(dt, this.playing, this.speed);
    if (this.playing) {
      // Wheel spin works off position deltas (not time), steering off userData.steer.
      this.actorManager.updateWheelRotations();
    }
    this.controls.update();

    if (this.plugins.length > 0) {
      const renderContext = this.createRenderContext();
      for (const entry of this.plugins) {
        entry.plugin.beforeRender?.(renderContext);
      }
    }
    this.renderer.render(this.scene, this.camera);
  }

  /**
   * Per-player boost / supersonic effect state, ported from the original
   * GameEngine.updateScene(): only update particle emission while playing (so
   * paused frames don't emit at frozen positions), and pass isKickoffReset so
   * the kickoff boost-reset doesn't fire particles. ActorManager resolves the
   * car mesh and forwards to EffectsManager (the stub when `effects: false`).
   */
  private updatePlayerStates(): void {
    if (!this.playing) return;
    for (const entity of this.adapter.getAllPlayers()) {
      this.actorManager.updateBoostState(entity.name, entity.isBoosting, entity.isKickoffReset);
      this.actorManager.updateSupersonicState(entity.name, entity.isSupersonic, entity.team);
    }
  }

  private installPlugin(definition: ViewerPluginDefinition, renderAfterSetup: boolean): () => void {
    const plugin = typeof definition === "function" ? definition() : definition;
    if (this.plugins.some((entry) => entry.plugin.id === plugin.id)) {
      throw new Error(`Viewer plugin "${plugin.id}" is already installed`);
    }

    const entry = { definition, plugin };
    this.plugins.push(entry);
    plugin.setup?.(this.createPluginContext());
    plugin.onStateChange?.(this.createPluginStateContext(this.getState()));
    if (renderAfterSetup) {
      this.render();
    }

    return () => {
      const index = this.plugins.indexOf(entry);
      if (index < 0) return;
      this.plugins.splice(index, 1);
      plugin.teardown?.(this.createPluginContext());
    };
  }

  private createPluginContext(): ViewerPluginContext {
    return {
      player: this,
      scene: this.scene,
      camera: this.camera,
      renderer: this.renderer,
      container: this.container,
    };
  }

  private createPluginStateContext(state: ViewerState): ViewerPluginStateContext {
    return { ...this.createPluginContext(), state };
  }

  private createRenderContext(): ViewerRenderContext {
    // ActorManager is untyped JS; view the two lookup tables we read with types.
    const am = this.actorManager as unknown as {
      ballActorId: string | number | null;
      actors: Record<string | number, THREE.Object3D | undefined>;
      playerNameToCarActorId: Record<string, string | number | undefined>;
    };
    const ball = this.adapter.ball;
    const ballState: BallRenderState = {
      position: ball.position,
      rotation: ball.rotation,
      velocity: ball.velocity,
      visible: ball.visible,
      object3d: am.ballActorId != null ? (am.actors[am.ballActorId] ?? null) : null,
    };
    const cars: CarRenderState[] = this.adapter.getAllPlayers().map((entity) => {
      const carActorId = am.playerNameToCarActorId[entity.name];
      return {
        name: entity.name,
        team: entity.team,
        carName: entity.carName,
        hitboxType: entity.hitboxType,
        position: entity.position,
        rotation: entity.rotation,
        velocity: entity.velocity,
        boost: entity.boost,
        isBoosting: entity.isBoosting,
        visible: entity.isVisible,
        object3d: carActorId != null ? (am.actors[carActorId] ?? null) : null,
      };
    });
    return {
      ...this.createPluginContext(),
      time: this.currentTime,
      ball: ballState,
      cars,
    };
  }

  private emitChange(): void {
    const state = this.getState();
    const pluginStateContext = this.createPluginStateContext(state);
    for (const entry of this.plugins) {
      entry.plugin.onStateChange?.(pluginStateContext);
    }
    this.dispatchEvent(new CustomEvent<ViewerState>("change", { detail: state }));
  }
}
