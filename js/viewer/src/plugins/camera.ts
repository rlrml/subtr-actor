/**
 * Camera plugin — all of the original ballcam camera system, wrapped behind
 * plugin hooks. Wraps the original CameraManager (state-blended car cam ⇄ ball
 * cam with SLERP transitions, FPS free cam, ball-orbit cam) and ports the
 * GameEngine.updateCamera() glue that drove it (per-frame targets, follow
 * settings, RL horizontal-FOV → three.js vertical-FOV conversion).
 *
 * Modes:
 * - `"orbit"`    — the core's plain OrbitControls (default; the manager idles).
 * - `"free"`     — FPS-style fly cam: WASD/arrows + Space/Shift, right-click
 *                  drag to look (pointer lock). Owned by CameraManager.
 * - `"ballOrbit"`— orbit the ball; the camera tracks ball movement while
 *                  preserving the user's orbit angle and scroll zoom.
 * - `"follow"`   — RL-style follow camera for a chosen player; ball cam ⇄ car
 *                  cam via `setBallCam` (default follows the replay's recorded
 *                  ball-cam state for that player). Follow settings default to
 *                  the player's RECORDED camera preset (replays replicate each
 *                  player's distance/height/angle/stiffness/swivel/fov);
 *                  `setCameraSettings` overrides win per field.
 *
 *   const cam = createCameraPlugin();
 *   createViewer(container, bytes, { plugins: [cam] });
 *   cam.follow("SomePlayer");     // follow mode, attached to a player
 *   cam.setBallCam(false);        // force car cam (null = recorded state)
 *   cam.setMode("free");          // fly around
 *   cam.release();                // back to orbit
 */
import * as THREE from "three";
import { CameraManager } from "../managers/CameraManager.js";
import type { ViewerPlugin, ViewerPluginContext, ViewerRenderContext } from "../types.js";

export type CameraPluginMode = "orbit" | "free" | "ballOrbit" | "follow";

/** RL-style camera settings (original GameEngine defaults). */
export interface CameraSettings {
  /** Distance behind car in UU (RL range 100-400). */
  distance?: number;
  /** Height above car in UU (RL range 40-200). */
  height?: number;
  /** Pitch angle in degrees, negative = look down (RL range -15 to 0). */
  angle?: number;
  /** Camera stiffness (RL range 0.0-1.0; higher = more responsive). */
  stiffness?: number;
  /** Swivel speed around the car (RL range 1.0-10.0). */
  swivelSpeed?: number;
  /** Ball cam transition speed (RL range 1.0-2.0). */
  transitionSpeed?: number;
  /** HORIZONTAL field of view in degrees (RL range 60-110). */
  fov?: number;
  /** Free-cam fly speed in UU/s. */
  freeCamSpeed?: number;
}

export interface CameraPluginOptions {
  /** Initial mode. Default "orbit" (or "follow" when `follow` is set). */
  mode?: CameraPluginMode;
  /** Player name to start following as soon as the plugin installs. */
  follow?: string;
  /**
   * Ball cam override: true/false forces it; null/undefined follows the
   * replay's recorded per-player ball-cam state (the original behavior).
   */
  ballCam?: boolean | null;
  /**
   * Explicit RL camera settings. Per-field precedence (highest first):
   * these explicit settings → the followed player's recorded preset (when the
   * replay carries one and `useRecordedSettings` isn't false) → RL defaults.
   */
  settings?: CameraSettings;
  /**
   * Seed follow-mode settings from the followed player's recorded camera
   * preset (replays replicate each player's distance/height/angle/stiffness/
   * swivel/transition/fov). Default true.
   */
  useRecordedSettings?: boolean;
}

export interface CameraPlugin extends ViewerPlugin {
  setMode(mode: CameraPluginMode): void;
  getMode(): CameraPluginMode;
  /** Attach the camera to this player (by adapter player name) — mode "follow". */
  follow(playerName: string): void;
  /** Detach and return control to the core's orbit camera — mode "orbit". */
  release(): void;
  getTarget(): string | null;
  /** true/false = force ball/car cam; null = use the replay's recorded state. */
  setBallCam(enabled: boolean | null): void;
  /** The ball-cam state currently applied to the camera. */
  getBallCam(): boolean;
  /** Merge explicit settings (they win over the recorded preset + defaults). */
  setCameraSettings(settings: CameraSettings): void;
  /** The effective settings currently applied (defaults ⊕ recorded ⊕ explicit). */
  getCameraSettings(): CameraSettings;
  /** The follow target's recorded camera preset, when the replay carries one. */
  getRecordedSettings(): CameraSettings | null;
}

// Original GameEngine.cameraSettings defaults (RL camera options).
const DEFAULT_SETTINGS: Required<Omit<CameraSettings, "freeCamSpeed">> = {
  distance: 260,
  height: 90,
  angle: -4,
  stiffness: 0.45,
  swivelSpeed: 4.3,
  transitionSpeed: 1.3,
  fov: 110,
};

export function createCameraPlugin(options: CameraPluginOptions = {}): CameraPlugin {
  let manager: CameraManager | null = null;
  let pluginCtx: ViewerPluginContext | null = null;
  let mode: CameraPluginMode = options.mode ?? (options.follow ? "follow" : "orbit");
  let target: string | null = options.follow ?? null;
  let ballCamOverride: boolean | null = options.ballCam ?? null;
  let effectiveBallCam = ballCamOverride ?? true;
  // Explicit settings only — the effective set is recomputed on read so the
  // recorded preset switches with the follow target (see settings()).
  let overrides: CameraSettings = { ...options.settings };
  const useRecorded = options.useRecordedSettings !== false;
  let lastNow: number | null = null;
  const lastCarPos = new THREE.Vector3();
  let hasLastCarPos = false;

  // Ported from GameEngine.setCameraMode(): map the plugin mode onto the
  // manager ('car' doubles as "idle" in orbit mode — its internal controls and
  // free-cam keys stay disabled, and we simply don't call update()).
  function applyMode(): void {
    if (!pluginCtx || !manager) return;
    // The core's orbit controls own the camera only in "orbit" mode.
    pluginCtx.player.controls.enabled = mode === "orbit";
    if (mode === "free") {
      manager.setMode("free");
    } else if (mode === "ballOrbit") {
      // Original order: aim at the ball first so setMode's setLookAt frames it.
      const ball = getBallMesh();
      if (ball) manager.setTargetBall(ball);
      manager.setMode("ballOrbit");
    } else {
      manager.setMode("car");
    }
    lastNow = null;
  }

  /** The follow target's recorded RL camera preset, when available. */
  function recordedSettings(): CameraSettings | null {
    if (!useRecorded || !pluginCtx || !target) return null;
    return pluginCtx.player.adapter.getPlayer(target)?.cameraSettings ?? null;
  }

  /** Effective settings: defaults ⊕ recorded preset ⊕ explicit overrides. */
  function settings(): CameraSettings {
    return { ...DEFAULT_SETTINGS, ...recordedSettings(), ...overrides };
  }

  function getBallMesh(): THREE.Object3D | null {
    if (!pluginCtx) return null;
    const am = pluginCtx.player.actorManager as unknown as {
      ballActorId: string | number | null;
      actors: Record<string | number, THREE.Object3D | undefined>;
    };
    return am.ballActorId != null ? (am.actors[am.ballActorId] ?? null) : null;
  }

  /**
   * Ported from GameEngine.updateCamera(): Rocket League FOV settings are
   * HORIZONTAL, three.js is VERTICAL. Convert at the current aspect ratio, but
   * never drop below the 16:9 baseline so ultra-wide screens don't cut off the
   * car. Applies in every mode, like the original.
   */
  function applyFov(camera: THREE.PerspectiveCamera): void {
    const fov = settings().fov;
    if (!fov) return;
    const horizontalFovRad = (fov * Math.PI) / 180;
    const baselineAspect = 16 / 9;
    const baselineVerticalFovRad = 2 * Math.atan(Math.tan(horizontalFovRad / 2) / baselineAspect);
    const calculatedVerticalFovRad = 2 * Math.atan(Math.tan(horizontalFovRad / 2) / camera.aspect);
    const verticalFovDeg =
      (Math.max(baselineVerticalFovRad, calculatedVerticalFovRad) * 180) / Math.PI;
    if (Math.abs(camera.fov - verticalFovDeg) > 0.1) {
      camera.fov = verticalFovDeg;
      camera.updateProjectionMatrix();
    }
  }

  // Ported from GameEngine.updateCamera()'s per-mode branches.
  function updateCamera(ctx: ViewerRenderContext, dt: number): void {
    if (!manager) return;
    if (mode === "free") {
      if (overrides.freeCamSpeed) {
        (manager as unknown as { freeCamSpeed: number }).freeCamSpeed = overrides.freeCamSpeed;
      }
      manager.update(dt);
      return;
    }
    if (mode === "ballOrbit") {
      if (ctx.ball.object3d) manager.setTargetBall(ctx.ball.object3d);
      manager.update(dt);
      return;
    }
    // follow
    const car = target ? ctx.cars.find((c) => c.name === target) : undefined;
    const carMesh = car?.object3d ?? null;
    if (!carMesh) {
      // Demolished / not spawned: keep the camera's smoothing alive in place.
      manager.update(dt);
      return;
    }
    manager.setTargetCar(carMesh);
    if (ctx.ball.object3d) manager.setTargetBall(ctx.ball.object3d);
    // setFollowSettings guards each field with `!== undefined`, so a partial
    // settings object is fine despite the JSDoc-inferred required params.
    manager.setFollowSettings(settings() as Parameters<CameraManager["setFollowSettings"]>[0]);
    // Ball cam: explicit override wins, else the replay's recorded state.
    const entity = target ? ctx.player.adapter.getPlayer(target) : undefined;
    effectiveBallCam = ballCamOverride ?? entity?.isBallCam ?? true;
    lastCarPos.copy(carMesh.position);
    hasLastCarPos = true;
    manager.update(dt, effectiveBallCam);
  }

  return {
    id: "camera",

    setup(ctx) {
      pluginCtx = ctx;
      manager = new CameraManager(ctx.camera, ctx.renderer.domElement);
      applyMode();
    },

    beforeRender(ctx) {
      if (!manager) return;
      applyFov(ctx.camera as THREE.PerspectiveCamera);
      if (mode === "orbit") return; // the core's OrbitControls own the camera

      // CameraManager smooths in real time, independent of playback speed.
      const now = performance.now();
      const dt = lastNow === null ? 1 / 60 : Math.min(0.1, (now - lastNow) / 1000);
      lastNow = now;
      updateCamera(ctx, dt);
    },

    teardown() {
      mode = "orbit";
      if (pluginCtx) pluginCtx.player.controls.enabled = true;
      manager?.dispose();
      manager = null;
      pluginCtx = null;
    },

    setMode(next: CameraPluginMode) {
      if (next === mode) return;
      mode = next;
      applyMode();
    },

    getMode(): CameraPluginMode {
      return mode;
    },

    follow(playerName: string) {
      target = playerName;
      mode = "follow";
      applyMode();
    },

    release() {
      mode = "orbit";
      // Re-aim the orbit controls at the car we were watching so the handoff
      // keeps the current camera pose instead of snapping to the old target.
      if (pluginCtx && hasLastCarPos) {
        pluginCtx.player.controls.target.copy(lastCarPos);
      }
      applyMode();
    },

    getTarget(): string | null {
      return target;
    },

    setBallCam(enabled: boolean | null) {
      ballCamOverride = enabled;
    },

    getBallCam(): boolean {
      return effectiveBallCam;
    },

    setCameraSettings(next: CameraSettings) {
      overrides = { ...overrides, ...next };
    },

    getCameraSettings(): CameraSettings {
      return settings();
    },

    getRecordedSettings(): CameraSettings | null {
      const recorded = recordedSettings();
      return recorded ? { ...recorded } : null;
    },
  };
}
