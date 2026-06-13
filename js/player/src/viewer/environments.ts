/**
 * Skybox environments for the viewer.
 *
 * An "environment" drives both the visible skybox (`scene.background`) and the
 * image-based lighting (`scene.environment` → reflections/ambient on every PBR
 * material — cars, arena, ball). This is the polish layer the original ballcam
 * viewer got from its HDR skyboxes; the neutral `RoomEnvironment` fallback in
 * `SceneManager.initDefaultEnvironment()` keeps the scene lit before/without one.
 *
 * Environments are static, client-side descriptors (no backend). The built-in
 * "space" mirrors ballcam's Space environment (PlanetaryEarth4k HDR). Register
 * more with `registerEnvironment`, or pass a full descriptor inline.
 */

export interface ViewerEnvironment {
  /** Stable id (also the key used to look it up). */
  id: string;
  /**
   * URL of the equirectangular HDR skybox, resolved against the web root.
   * Bundled assets live under `public/skyboxes/` and ship with the package.
   */
  skyboxUrl: string;
  /** `renderer.toneMappingExposure` while this environment is active (default 1.0). */
  exposure?: number;
  /** Static skybox tilt in degrees, applied to background + environment maps. */
  rotation?: { x?: number; y?: number; z?: number };
  /** Optional slow drift about the Y axis (degrees/second). Disabled by default. */
  animation?: { enabled: boolean; speed: number };
}

/**
 * An environment spec accepted by the viewer:
 * - a built-in id (e.g. `"space"`),
 * - a full {@link ViewerEnvironment} descriptor, or
 * - `false` to use only the neutral default lighting (no skybox).
 */
export type ViewerEnvironmentSpec = string | ViewerEnvironment | false;

/** The viewer's default environment when none is specified. */
export const DEFAULT_ENVIRONMENT_ID = "space";

const BUILTIN_ENVIRONMENTS: Record<string, ViewerEnvironment> = {
  // Mirrors ballcam's "Space" environment, including its slow skybox drift.
  // Disable with `animation: { enabled: false }` on a custom descriptor.
  space: {
    id: "space",
    skyboxUrl: "/skyboxes/PlanetaryEarth4k.hdr",
    exposure: 1.45,
    rotation: { x: 8, y: 0, z: 28 },
    animation: { enabled: true, speed: 2 },
  },
};

/**
 * Register (or override) a built-in environment so it can be referenced by id
 * via the `environment` option or {@link ViewerPlayer.setEnvironment}.
 */
export function registerEnvironment(env: ViewerEnvironment): void {
  BUILTIN_ENVIRONMENTS[env.id] = env;
}

/** List the currently-registered built-in environment ids. */
export function listEnvironments(): string[] {
  return Object.keys(BUILTIN_ENVIRONMENTS);
}

/**
 * Resolve a spec to a concrete {@link ViewerEnvironment}, or `null` for the
 * neutral default (when `spec` is `false` or an unknown id).
 */
export function resolveEnvironment(spec: ViewerEnvironmentSpec): ViewerEnvironment | null {
  if (spec === false) return null;
  if (typeof spec === "string") {
    const env = BUILTIN_ENVIRONMENTS[spec];
    if (!env) {
      console.warn(`[viewer] unknown environment "${spec}"; using neutral default`);
      return null;
    }
    return env;
  }
  return spec;
}
