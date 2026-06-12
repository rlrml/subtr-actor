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
import type { CameraPlugin } from "./plugins/camera.js";
import type {
  BallRenderState,
  BeforeRenderCallback,
  CameraSettings,
  CarRenderState,
  FrameRenderInfo,
  ViewerCameraViewMode,
  ViewerFreeCameraPreset,
  ViewerOptions,
  ViewerPlugin,
  ViewerPluginContext,
  ViewerPluginDefinition,
  ViewerPluginStateContext,
  ViewerRenderContext,
  ViewerSnapshot,
  ViewerState,
  ViewerStatePatch,
} from "./types.js";

type ViewerListener = (state: ViewerState) => void;
type InstalledPlugin = { definition: ViewerPluginDefinition; plugin: ViewerPlugin };

// With `effects: false`, every EffectsManager call from ActorManager is a no-op.
const effectsStub = new Proxy({}, { get: () => () => {} }) as EffectsManager;

/**
 * Drop non-finite fields, matching @rlrml/player's normalizeCustomCameraSettings.
 * (`pitch` passes through — the camera plugin maps it onto `angle` on entry.)
 */
function normalizeCustomCameraSettings(
  settings: CameraSettings | null | undefined,
): CameraSettings | null {
  if (!settings) return null;
  const normalized: CameraSettings = {};
  for (const key of Object.keys(settings) as Array<keyof CameraSettings>) {
    const value = settings[key];
    if (typeof value === "number" && Number.isFinite(value)) {
      normalized[key] = value;
    }
  }
  return normalized;
}

/**
 * Free-camera preset poses — @rlrml/player's exact constants
 * (player-internals/spatial.ts), converted from its Z-up world to this
 * package's THREE Y-up space (x→x, z→y, y→z; see adapter/coords.ts).
 */
const FREE_CAMERA_PRESETS: Record<
  ViewerFreeCameraPreset,
  { position: [number, number, number]; target: [number, number, number]; up: [number, number, number] }
> = {
  overhead: { position: [0, 18800, 0], target: [0, 700, 0], up: [-1, 0, 0] },
  side: { position: [-9600, 6400, -12600], target: [0, 900, 0], up: [0, 1, 0] },
};

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
  private readonly beforeRenderCallbacks: BeforeRenderCallback[] = [];
  private resizeObserver: ResizeObserver | null = null;
  private animationFrameId: number | null = null;
  private disposed = false;
  private playing = false;
  private speed: number;
  private loop: boolean;
  private currentTime = 0;
  private lastTickAt: number | null = null;

  // ── @rlrml/player-parity state (docs/PLAYER_PARITY.md). Camera fields are
  //    delegated to an installed camera plugin (id "camera") when present; the
  //    display toggles are tracked-but-inert until their rendering lands.
  private cameraDistanceScaleValue: number;
  private customCameraSettingsValue: CameraSettings | null;
  private cameraViewModeValue: ViewerCameraViewMode;
  private attachedPlayerIdValue: string | null;
  /** null = never set: follow the camera plugin's recorded-state behavior. */
  private ballCamEnabledValue: boolean | null;
  private boostMeterEnabledValue: boolean;
  private boostPickupAnimationEnabledValue: boolean;
  private hitboxWireframesEnabledValue: boolean;
  private hitboxOnlyModeEnabledValue: boolean;
  private skipPostGoalTransitionsEnabledValue: boolean;
  private skipKickoffsEnabledValue: boolean;
  /** True once view-mode/attachment was set through the parity surface. */
  private attachmentTouched = false;

  constructor(container: HTMLElement, adapter: SubtrActorPlayer, options: ViewerOptions = {}) {
    super();
    this.container = container;
    this.adapter = adapter;
    this.options = options;
    this.speed = Math.max(0.1, options.initialPlaybackRate ?? options.speed ?? 1);
    this.loop = options.loop ?? false;
    this.cameraDistanceScaleValue = Math.max(0.25, options.initialCameraDistanceScale ?? 1);
    this.customCameraSettingsValue = normalizeCustomCameraSettings(
      options.initialCustomCameraSettings,
    );
    this.attachedPlayerIdValue = options.initialAttachedPlayerId ?? null;
    this.cameraViewModeValue =
      options.initialCameraViewMode ?? (this.attachedPlayerIdValue ? "follow" : "free");
    this.ballCamEnabledValue = options.initialBallCamEnabled ?? null;
    this.boostMeterEnabledValue = options.initialBoostMeterEnabled ?? false;
    this.boostPickupAnimationEnabledValue = options.initialBoostPickupAnimationEnabled ?? true;
    this.hitboxWireframesEnabledValue = options.initialHitboxWireframesEnabled ?? false;
    this.hitboxOnlyModeEnabledValue = options.initialHitboxOnlyModeEnabled ?? false;
    this.skipPostGoalTransitionsEnabledValue =
      options.initialSkipPostGoalTransitionsEnabled ?? true;
    this.skipKickoffsEnabledValue = options.initialSkipKickoffsEnabled ?? false;

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
    this.applyInitialCameraOptions();
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
    this.setPlayingInternal(true);
    this.emitChange();
  }

  pause(): void {
    if (!this.playing) return;
    this.setPlayingInternal(false);
    this.emitChange();
  }

  togglePlayback(): void {
    this.playing ? this.pause() : this.play();
  }

  seek(time: number): void {
    this.seekInternal(time);
    this.emitChange();
  }

  setPlaybackRate(speed: number): void {
    this.speed = Math.max(0.1, speed);
    this.emitChange();
  }

  setLoop(loop: boolean): void {
    this.loop = loop;
  }

  // ── Frame stepping (@rlrml/player parity, off the adapter's frame timeline) ──
  setFrameIndex(frameIndex: number): void {
    const times = this.adapter.frameTimes;
    if (times.length === 0 || !Number.isFinite(frameIndex)) return;
    const clamped = Math.min(Math.max(Math.trunc(frameIndex), 0), times.length - 1);
    // @rlrml/player semantics: landing on an exact frame implies paused playback.
    if (this.playing) this.setPlayingInternal(false);
    this.seekInternal(times[clamped]);
    this.emitChange();
  }

  stepFrames(delta: number): void {
    if (!Number.isFinite(delta)) return;
    this.setFrameIndex(this.adapter.frameIndexAt(this.currentTime) + Math.trunc(delta));
  }

  stepForwardFrame(): void {
    this.stepFrames(1);
  }

  stepBackwardFrame(): void {
    this.stepFrames(-1);
  }

  // ── Camera controls (@rlrml/player parity) ──────────────────────────────────
  // All delegate to an installed camera plugin (id "camera") when present —
  // state is tracked either way, so a plugin added later picks it up.
  setCameraDistanceScale(scale: number): void {
    this.cameraDistanceScaleValue = Math.max(0.25, scale);
    this.getCameraPlugin()?.setDistanceScale(this.cameraDistanceScaleValue);
    this.emitChange();
  }

  setCustomCameraSettings(settings: CameraSettings | null): void {
    this.applyCustomCameraSettings(settings);
    this.emitChange();
  }

  setAttachedPlayer(playerId: string | null): void {
    this.attachedPlayerIdValue = playerId;
    this.cameraViewModeValue = playerId ? "follow" : "free";
    this.attachmentTouched = true;
    this.syncCameraAttachment();
    this.emitChange();
  }

  setCameraViewMode(mode: ViewerCameraViewMode): void {
    this.cameraViewModeValue = mode;
    this.attachmentTouched = true;
    this.syncCameraAttachment();
    this.emitChange();
  }

  setFreeCameraPreset(preset: ViewerFreeCameraPreset): void {
    this.cameraViewModeValue = "free";
    this.attachmentTouched = true;
    this.syncCameraAttachment(); // leave follow mode if we were in it
    const pose = FREE_CAMERA_PRESETS[preset];
    this.camera.up.set(...pose.up);
    this.camera.position.set(...pose.position);
    this.controls.target.set(...pose.target);
    this.controls.update();
    this.emitChange();
  }

  setBallCamEnabled(enabled: boolean): void {
    this.ballCamEnabledValue = enabled;
    this.getCameraPlugin()?.setBallCam(enabled);
    this.emitChange();
  }

  // ── Display toggles (@rlrml/player parity; tracked-but-inert for now) ───────
  setBoostMeterEnabled(enabled: boolean): void {
    this.boostMeterEnabledValue = enabled;
    this.emitChange();
  }

  setBoostPickupAnimationEnabled(enabled: boolean): void {
    this.boostPickupAnimationEnabledValue = enabled;
    this.emitChange();
  }

  setHitboxWireframesEnabled(enabled: boolean): void {
    this.hitboxWireframesEnabledValue = enabled;
    this.emitChange();
  }

  setHitboxOnlyModeEnabled(enabled: boolean): void {
    this.hitboxOnlyModeEnabledValue = enabled;
    this.emitChange();
  }

  setSkipPostGoalTransitionsEnabled(enabled: boolean): void {
    this.skipPostGoalTransitionsEnabledValue = enabled;
    this.emitChange();
  }

  setSkipKickoffsEnabled(enabled: boolean): void {
    this.skipKickoffsEnabledValue = enabled;
    this.emitChange();
  }

  // ── State surface (@rlrml/player parity) ────────────────────────────────────
  setState(patch: ViewerStatePatch): void {
    if (patch.speed !== undefined) {
      this.speed = Math.max(0.1, patch.speed);
    }
    if (patch.cameraDistanceScale !== undefined) {
      this.cameraDistanceScaleValue = Math.max(0.25, patch.cameraDistanceScale);
      this.getCameraPlugin()?.setDistanceScale(this.cameraDistanceScaleValue);
    }
    if (patch.customCameraSettings !== undefined) {
      this.applyCustomCameraSettings(patch.customCameraSettings);
    }
    if (patch.cameraViewMode !== undefined) {
      this.cameraViewModeValue = patch.cameraViewMode;
      this.attachmentTouched = true;
    }
    if (patch.attachedPlayerId !== undefined) {
      this.attachedPlayerIdValue = patch.attachedPlayerId;
      this.attachmentTouched = true;
      if (patch.cameraViewMode === undefined) {
        this.cameraViewModeValue = patch.attachedPlayerId ? "follow" : "free";
      }
    }
    if (patch.cameraViewMode !== undefined || patch.attachedPlayerId !== undefined) {
      this.syncCameraAttachment();
    }
    if (patch.ballCamEnabled !== undefined) {
      this.ballCamEnabledValue = patch.ballCamEnabled;
      this.getCameraPlugin()?.setBallCam(patch.ballCamEnabled);
    }
    if (patch.boostMeterEnabled !== undefined) {
      this.boostMeterEnabledValue = patch.boostMeterEnabled;
    }
    if (patch.boostPickupAnimationEnabled !== undefined) {
      this.boostPickupAnimationEnabledValue = patch.boostPickupAnimationEnabled;
    }
    if (patch.hitboxWireframesEnabled !== undefined) {
      this.hitboxWireframesEnabledValue = patch.hitboxWireframesEnabled;
    }
    if (patch.hitboxOnlyModeEnabled !== undefined) {
      this.hitboxOnlyModeEnabledValue = patch.hitboxOnlyModeEnabled;
    }
    if (patch.skipPostGoalTransitionsEnabled !== undefined) {
      this.skipPostGoalTransitionsEnabledValue = patch.skipPostGoalTransitionsEnabled;
    }
    if (patch.skipKickoffsEnabled !== undefined) {
      this.skipKickoffsEnabledValue = patch.skipKickoffsEnabled;
    }
    if (patch.currentTime !== undefined) {
      this.seekInternal(patch.currentTime);
    }
    if (patch.playing !== undefined && patch.playing !== this.playing) {
      this.setPlayingInternal(patch.playing);
    }
    this.emitChange();
  }

  getState(): ViewerState {
    const camera = this.getCameraPlugin();
    let cameraViewMode = this.cameraViewModeValue;
    let attachedPlayerId = this.attachedPlayerIdValue;
    if (camera) {
      // Derive the camera fields from the plugin so state stays truthful even
      // when a consumer drives the plugin handle directly (e.g. the dev UI).
      if (camera.getMode() === "follow") {
        cameraViewMode = "follow";
        const targetName = camera.getTarget();
        const info = targetName
          ? this.adapter.playerList.find((p) => p.name === targetName)
          : undefined;
        attachedPlayerId = info?.id ?? attachedPlayerId;
      } else {
        cameraViewMode = "free";
        attachedPlayerId = null;
      }
    }
    return {
      currentTime: this.currentTime,
      duration: this.duration,
      frameIndex: this.adapter.frameIndexAt(this.currentTime),
      activeMetadata: null,
      playing: this.playing,
      speed: this.speed,
      cameraDistanceScale: this.cameraDistanceScaleValue,
      customCameraSettings: this.customCameraSettingsValue,
      cameraViewMode,
      attachedPlayerId,
      ballCamEnabled: camera ? camera.getBallCam() : (this.ballCamEnabledValue ?? false),
      boostMeterEnabled: this.boostMeterEnabledValue,
      boostPickupAnimationEnabled: this.boostPickupAnimationEnabledValue,
      hitboxWireframesEnabled: this.hitboxWireframesEnabledValue,
      hitboxOnlyModeEnabled: this.hitboxOnlyModeEnabledValue,
      skipPostGoalTransitionsEnabled: this.skipPostGoalTransitionsEnabledValue,
      skipKickoffsEnabled: this.skipKickoffsEnabledValue,
    };
  }

  getSnapshot(): ViewerSnapshot {
    return this.getState();
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

  /** Per-render frame-timing callback (@rlrml/player parity). Returns a remover. */
  onBeforeRender(callback: BeforeRenderCallback): () => void {
    this.beforeRenderCallbacks.push(callback);
    return () => {
      const index = this.beforeRenderCallbacks.indexOf(callback);
      if (index >= 0) {
        this.beforeRenderCallbacks.splice(index, 1);
      }
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
    this.beforeRenderCallbacks.length = 0;
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
  private setPlayingInternal(playing: boolean): void {
    this.playing = playing;
    this.lastTickAt = null;
    if (playing) {
      this.actorManager.resumeAnimations();
    } else {
      this.actorManager.pauseAnimations();
    }
  }

  private seekInternal(time: number): void {
    this.currentTime = THREE.MathUtils.clamp(time, 0, this.duration);
    // Sync the THREE animation system (if active) to the new time, and reset
    // trackers that work off frame-to-frame deltas so they don't see the jump:
    // the ball trail would draw a segment connecting old/new positions and the
    // wheels would spin wildly from the position delta.
    this.actorManager.seekAnimations(this.currentTime);
    this.effectsManager.resetBallTrail();
    this.actorManager.resetWheelTracking();
  }

  /** The installed camera plugin, when one is present (duck-typed by id). */
  private getCameraPlugin(): CameraPlugin | null {
    const plugin = this.plugins.find((entry) => entry.plugin.id === "camera")?.plugin;
    return plugin && typeof (plugin as CameraPlugin).follow === "function"
      ? (plugin as CameraPlugin)
      : null;
  }

  private playerNameForId(id: string): string | null {
    return this.adapter.playerList.find((p) => p.id === id)?.name ?? null;
  }

  /** Push the parity view-mode/attachment onto the camera plugin. */
  private syncCameraAttachment(): void {
    const camera = this.getCameraPlugin();
    if (!camera) return;
    if (this.cameraViewModeValue === "follow" && this.attachedPlayerIdValue) {
      const name = this.playerNameForId(this.attachedPlayerIdValue);
      if (!name) {
        console.warn(`[viewer] no player with id ${JSON.stringify(this.attachedPlayerIdValue)}`);
        return;
      }
      // Follow mode owns the camera; make sure a preset's custom up is undone.
      this.camera.up.set(0, 1, 0);
      camera.follow(name);
      return;
    }
    // "free": only leave follow mode — never stomp the viewer-native free-fly /
    // ballOrbit modes a consumer may have set on the plugin handle directly.
    if (camera.getMode() === "follow") {
      camera.release();
    }
  }

  private applyCustomCameraSettings(settings: CameraSettings | null | undefined): void {
    this.customCameraSettingsValue = normalizeCustomCameraSettings(settings);
    const camera = this.getCameraPlugin();
    if (camera) {
      // Replace, not merge: @rlrml/player treats customCameraSettings as a
      // whole object, so clear the plugin's overrides before applying.
      camera.setCameraSettings(null);
      if (this.customCameraSettingsValue) {
        camera.setCameraSettings(this.customCameraSettingsValue);
      }
    }
  }

  /** Push explicitly-set parity camera state onto an (newly) installed plugin. */
  private pushCameraParityState(): void {
    const camera = this.getCameraPlugin();
    if (!camera) return;
    if (this.cameraDistanceScaleValue !== 1) {
      camera.setDistanceScale(this.cameraDistanceScaleValue);
    }
    if (this.customCameraSettingsValue) {
      camera.setCameraSettings(this.customCameraSettingsValue);
    }
    if (this.ballCamEnabledValue !== null) {
      camera.setBallCam(this.ballCamEnabledValue);
    }
    if (this.attachmentTouched) {
      this.syncCameraAttachment();
    }
  }

  private applyInitialCameraOptions(): void {
    const o = this.options;
    if (o.initialAttachedPlayerId !== undefined || o.initialCameraViewMode !== undefined) {
      // Only then may the parity state override the plugin's own follow/mode
      // options (e.g. createCameraPlugin({ follow })).
      this.attachmentTouched = true;
    }
    this.pushCameraParityState();
  }

  private computeFrameRenderInfo(): FrameRenderInfo {
    const times = this.adapter.frameTimes;
    const frameIndex = this.adapter.frameIndexAt(this.currentTime);
    const nextFrameIndex = Math.min(frameIndex + 1, Math.max(times.length - 1, 0));
    const t0 = times[frameIndex] ?? 0;
    const t1 = times[nextFrameIndex] ?? t0;
    const alpha = t1 > t0 ? THREE.MathUtils.clamp((this.currentTime - t0) / (t1 - t0), 0, 1) : 0;
    return { frameIndex, nextFrameIndex, alpha, currentTime: this.currentTime };
  }

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

    if (this.beforeRenderCallbacks.length > 0) {
      const info = this.computeFrameRenderInfo();
      for (const callback of [...this.beforeRenderCallbacks]) {
        callback(info);
      }
    }
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
    if (plugin.id === "camera") {
      // A camera plugin installed after construction picks up any parity
      // camera state already set through the ViewerPlayer surface.
      this.pushCameraParityState();
    }
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
        id: entity.id,
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
