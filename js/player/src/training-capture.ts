import type { BallSpawn, CarSpawn, PlayerCarSpawn } from "./training-pack";
import type { TrainingPackFile } from "./training-pack";
import type { Guid } from "./generated/Guid";
import type { TrainingPack } from "./generated/TrainingPack";
import type { Quaternion, Vec3 } from "./types";

/**
 * Capture of a replay frame's ball + car states into a custom training
 * (`.tem`) round, mirroring the BakkesMod tem-recorder plugin's output
 * vocabulary (`bakkesmod-tem-recorder/rust/src/archetypes.rs`) so packs
 * captured in-browser and in-game look alike.
 *
 * COORDINATE FRAME: every input here is the *replay model's* rigid-body
 * data (`ReplayModel` ball/player samples, i.e. `RigidBodyTs`): native
 * Rocket League / Unreal convention — Z-up, raw Unreal units, quaternion
 * `{x, y, z, w}` applied as a standard right-handed rotation (the same
 * convention `replay-data.ts` uses to derive each sample's `forward`/`up`
 * vectors). That is exactly the frame the training archetypes store, so
 * positions copy 1:1. Do NOT feed three.js viewer-scene data here — the
 * viewer remaps to Y-up (`player/adapter/coords.ts`) at render time.
 */

/** Default per-shot time limit in seconds, matching the BakkesMod plugin. */
export const DEFAULT_TRAINING_SHOT_TIME_LIMIT_SECONDS = 8;

/**
 * Minimum car spawn height in Unreal units. A grounded car's rest height is
 * ~17uu; replay rigid-body samples can dip fractionally below that, and a
 * below-rest spawn Z clips the car into the floor when the game places it.
 * Airborne transforms (Z above this) pass through untouched.
 */
export const MIN_CAR_SPAWN_Z = 17;

/** Unreal rotator units per radian: 65536 units per full turn. */
const ROTATOR_UNITS_PER_RADIAN = 32768 / Math.PI;

/** An integer UE rotator (65536 units = 360 degrees). */
export interface RotatorUnits {
  pitch: number;
  yaw: number;
  roll: number;
}

/** The ball state of one replay frame, in the replay model's RL frame. */
export interface CapturedBallState {
  /** Ball center, raw Unreal units. */
  position: Vec3;
  /** Ball velocity, raw Unreal units per second. */
  linearVelocity?: Vec3 | null;
}

/** A car state of one replay frame, in the replay model's RL frame. */
export interface CapturedCarState {
  /** Car position, raw Unreal units. */
  position: Vec3;
  /**
   * Car orientation as the replay model's `{x, y, z, w}` quaternion.
   * `null`/absent falls back to the identity rotator (facing +X).
   */
  rotation?: Quaternion | null;
}

/** One replay frame's states to turn into a training round. */
export interface TrainingCaptureOptions {
  ball: CapturedBallState;
  /**
   * The car whose captured transform is written to BOTH the round's
   * `DynamicSpawnPointMesh` spawn point and its `IsPC` player car. Any
   * other cars in the frame are omitted, matching the BakkesMod plugin
   * (the in-game editor corpus never contains a second car, so there is
   * no observed vocabulary for one).
   */
  shooter: CapturedCarState;
  /** Per-shot time limit in seconds; 0 means unlimited. Default 8. */
  timeLimit?: number;
}

/** Converts an angle in radians to integer UE rotator units. */
export function radiansToRotatorUnits(radians: number): number {
  return Math.round(radians * ROTATOR_UNITS_PER_RADIAN);
}

/**
 * Rotates `vector` by the unit quaternion `quaternion` (standard
 * right-handed rotation, the same convention `replay-data.ts` applies to
 * derive player `forward`/`up`).
 */
function rotateVectorByQuaternion(vector: Vec3, quaternion: Quaternion): Vec3 {
  const { x: qx, y: qy, z: qz, w: qw } = quaternion;
  // t = 2 * (q.xyz cross v); v' = v + w * t + (q.xyz cross t)
  const tx = 2 * (qy * vector.z - qz * vector.y);
  const ty = 2 * (qz * vector.x - qx * vector.z);
  const tz = 2 * (qx * vector.y - qy * vector.x);
  return {
    x: vector.x + qw * tx + (qy * tz - qz * ty),
    y: vector.y + qw * ty + (qz * tx - qx * tz),
    z: vector.z + qw * tz + (qx * ty - qy * tx),
  };
}

/**
 * Decomposes a velocity vector into a direction rotator plus speed
 * magnitude, the `VelocityStartRotation{P,Y,R}` + `VelocityStartSpeed`
 * encoding ball archetypes store. Mirrors the BakkesMod plugin: pitch from
 * the vertical component (`atan2(z, hypot(x, y))`), yaw in the ground plane
 * (`atan2(y, x)`), roll 0 (meaningless for a direction), and zero velocity
 * collapsing to speed 0 with the default rotator.
 */
export function velocityToRotatorAndSpeed(velocity: Vec3 | null | undefined): {
  rotator: RotatorUnits;
  speed: number;
} {
  const x = velocity?.x ?? 0;
  const y = velocity?.y ?? 0;
  const z = velocity?.z ?? 0;
  const horizontal = Math.hypot(x, y);
  const speed = Math.hypot(horizontal, z);
  if (speed === 0) {
    return { rotator: { pitch: 0, yaw: 0, roll: 0 }, speed: 0 };
  }
  return {
    rotator: {
      pitch: radiansToRotatorUnits(Math.atan2(z, horizontal)),
      yaw: radiansToRotatorUnits(Math.atan2(y, x)),
      roll: 0,
    },
    speed,
  };
}

/**
 * Converts the replay model's orientation quaternion to an integer UE
 * rotator.
 *
 * Convention (verified against this codebase's established quaternion
 * usage): in the RL Z-up frame the quaternion rotates the car's local axes
 * — x = forward, y = right, z = up (roof) — into world space as a standard
 * right-handed rotation. The rotator is then read off the rotated basis:
 *
 * - yaw   = atan2(forward.y, forward.x)      (0 faces +X, 16384 faces +Y),
 * - pitch = atan2(forward.z, |forward.xy|)   (positive nose-up, straight up
 *   is 16384 — same convention as the ball-velocity pitch),
 * - roll  = atan2(right.z, up.z)             (0 when flat; positive rolls
 *   the right side toward the roof's old +Z, i.e. the inverse of
 *   `Rz(yaw)·Ry(-pitch)·Rx(roll)` applied to the local axes).
 */
export function quaternionToRotator(rotation: Quaternion): RotatorUnits {
  const forward = rotateVectorByQuaternion({ x: 1, y: 0, z: 0 }, rotation);
  const right = rotateVectorByQuaternion({ x: 0, y: 1, z: 0 }, rotation);
  const up = rotateVectorByQuaternion({ x: 0, y: 0, z: 1 }, rotation);
  return {
    pitch: radiansToRotatorUnits(Math.atan2(forward.z, Math.hypot(forward.x, forward.y))),
    yaw: radiansToRotatorUnits(Math.atan2(forward.y, forward.x)),
    roll: radiansToRotatorUnits(Math.atan2(right.z, up.z)),
  };
}

/** Builds the ball archetype of a captured round. */
export function ballSpawnFromReplayState(ball: CapturedBallState): BallSpawn {
  const { rotator, speed } = velocityToRotatorAndSpeed(ball.linearVelocity);
  return {
    start_location_x: ball.position.x,
    start_location_y: ball.position.y,
    start_location_z: ball.position.z,
    velocity_start_rotation_p: rotator.pitch,
    velocity_start_rotation_y: rotator.yaw,
    velocity_start_rotation_r: rotator.roll,
    velocity_start_speed: speed,
    extras: {},
  };
}

/**
 * The clamped spawn transform of a captured car: position 1:1 except Z
 * raised to {@link MIN_CAR_SPAWN_Z} (ground-clip guard; airborne Z passes
 * through), rotation as an integer UE rotator (identity when the sample
 * has none).
 */
function carSpawnTransform(car: CapturedCarState): {
  position: Vec3;
  rotator: RotatorUnits;
} {
  return {
    position: {
      x: car.position.x,
      y: car.position.y,
      z: Math.max(car.position.z, MIN_CAR_SPAWN_Z),
    },
    rotator: car.rotation ? quaternionToRotator(car.rotation) : { pitch: 0, yaw: 0, roll: 0 },
  };
}

/**
 * Builds the `DynamicSpawnPointMesh` spawn-point archetype of a captured
 * round, carrying the captured shooter transform. The game places the
 * training car from THIS entry (not the `IsPC` one), so it must never be a
 * hardcoded default.
 */
export function carSpawnFromReplayState(car: CapturedCarState): CarSpawn {
  const { position, rotator } = carSpawnTransform(car);
  return {
    location_x: position.x,
    location_y: position.y,
    location_z: position.z,
    rotation_p: rotator.pitch,
    rotation_y: rotator.yaw,
    rotation_r: rotator.roll,
    velocity_start_speed: 0,
    extras: {},
  };
}

/** Builds the `IsPC` player-car archetype of a captured round. */
export function playerCarSpawnFromReplayState(car: CapturedCarState): PlayerCarSpawn {
  const { position, rotator } = carSpawnTransform(car);
  return {
    is_pc: true,
    location_x: position.x,
    location_y: position.y,
    location_z: position.z,
    rotation_p: rotator.pitch,
    rotation_y: rotator.yaw,
    rotation_r: rotator.roll,
    extras: {},
  };
}

/**
 * Appends one captured replay frame to `file` as a new round and returns
 * the new round's index.
 *
 * The round's archetypes match the BakkesMod plugin's (and the in-game
 * editor corpus') shape and order exactly: the ball, the
 * `DynamicSpawnPointMesh` spawn point, then the single `IsPC` player car.
 * The game places the training car from the spawn-point entry — NOT the
 * `IsPC` entry — so both carry the captured shooter transform (the `IsPC`
 * duplicate matches user-made packs and the fixed BakkesMod plugin).
 */
export function appendCapturedRound(
  file: TrainingPackFile,
  options: TrainingCaptureOptions,
): number {
  const index = file.roundCount;
  file.addRound({
    time_limit: options.timeLimit ?? DEFAULT_TRAINING_SHOT_TIME_LIMIT_SECONDS,
    serialized_archetypes: [],
  });
  file.setRoundBall(index, ballSpawnFromReplayState(options.ball));
  file.addRoundCar(index, carSpawnFromReplayState(options.shooter));
  file.addRoundArchetype(index, {
    kind: "PlayerCar",
    ...playerCarSpawnFromReplayState(options.shooter),
  });
  return index;
}

/**
 * Generates a pseudo-random pack GUID (uniqueness, not unpredictability,
 * is what matters — the game identifies packs and names `.Tem` files by
 * it).
 */
export function generateTrainingPackGuid(): Guid {
  const words = new Int32Array(4);
  if (typeof crypto !== "undefined" && typeof crypto.getRandomValues === "function") {
    crypto.getRandomValues(words);
  } else {
    for (let index = 0; index < words.length; index += 1) {
      words[index] = Math.floor(Math.random() * 0x1_0000_0000) | 0;
    }
  }
  return { a: words[0]!, b: words[1]!, c: words[2]!, d: words[3]! };
}

/**
 * Pack GUID as 32 uppercase hex characters, matching the game's `.Tem`
 * filename convention.
 */
export function trainingPackGuidHex(guid: Guid): string {
  return [guid.a, guid.b, guid.c, guid.d]
    .map((word) => (word >>> 0).toString(16).padStart(8, "0").toUpperCase())
    .join("");
}

/**
 * Download / save filename for a pack: `<guid-hex>.Tem`. The game lists
 * custom training by scanning for GUID-named `.Tem` files, so using the
 * GUID (rather than a name slug) keeps a downloaded pack drop-in usable
 * and matches the BakkesMod plugin's output naming.
 */
export function trainingPackFileName(guid: Guid): string {
  return `${trainingPackGuidHex(guid)}.Tem`;
}

/**
 * Metadata defaults for a freshly captured pack, matching the BakkesMod
 * plugin's `RecorderPack::new`: a random GUID, current timestamps, and
 * corpus-matching training type / difficulty / map. Spread user overrides
 * on top and pass to `TrainingPackFile.create`.
 */
export function capturedTrainingPackDefaults(
  nowSeconds: number = Math.floor(Date.now() / 1000),
): Partial<TrainingPack> {
  const now = BigInt(nowSeconds);
  return {
    guid: generateTrainingPackGuid(),
    name: "Captured Training Pack",
    training_type: "Training_Striker",
    difficulty: "D_Medium",
    map_name: "Park_P",
    created_at: now,
    updated_at: now,
  };
}
