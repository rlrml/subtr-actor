import * as THREE from "three";
import type { ReplayScene } from "../scene";
import type {
  CameraSettings,
  Quaternion,
  ReplayCameraViewMode,
  ReplayFreeCameraPreset,
  ReplayModel,
  Vec3,
} from "../types";

const CHASE_CAMERA_HEIGHT_MULTIPLIER = 1.4;
const CAMERA_SMOOTHING = 0.18;
const FREE_CAMERA_TRANSITION_SMOOTHING = 0.14;
const GROUND_HEIGHT_THRESHOLD_UU = 120;
const MIN_CAMERA_HEIGHT_UU = 90;
const PLAYER_FOCUS_HEIGHT_UU = 40;
const BALL_CAM_AERIAL_HEIGHT_ADJUSTMENT_UU = 100;
const BALL_CAM_HEIGHT_BIAS_UU = 45;
const BALL_CAM_LOOK_BLEND = 0.58;
const BALL_CAM_DIRECTION_BLEND = 0.82;
const BALL_CAM_MAX_FOV = 132;
const BALL_CAM_TRANSITION_BASE_DURATION_SECONDS = 0.5;
const DEFAULT_FORWARD = new THREE.Vector3(-1, 0, 0);
const DEFAULT_UP = new THREE.Vector3(0, 0, 1);
const OVERHEAD_UP = new THREE.Vector3(-1, 0, 0);
const OVERHEAD_CAMERA_POSITION_UU = new THREE.Vector3(0, 0, 18800);
const OVERHEAD_LOOK_TARGET_UU = new THREE.Vector3(0, 0, 700);
const SIDE_CAMERA_POSITION_UU = new THREE.Vector3(-9600, -12600, 6400);
const SIDE_LOOK_TARGET_UU = new THREE.Vector3(0, 0, 900);
const FREE_CAMERA_FOV = 48;
const CAMERA_POSITION_EPSILON_SQ = 16;
const CAMERA_TARGET_EPSILON_SQ = 16;
const CAMERA_UP_EPSILON_RAD = 0.003;
const CAMERA_FOV_EPSILON = 0.05;

export interface AttachedCameraBlendState {
  currentBlend: number;
  targetBlend: number;
  lastIsBallCam: boolean | null;
}

export function interpolatePosition(
  current: Vec3 | null,
  next: Vec3 | null,
  alpha: number,
): Vec3 | null {
  if (!current) {
    return next;
  }

  if (!next || alpha <= 0) {
    return current;
  }

  return {
    x: THREE.MathUtils.lerp(current.x, next.x, alpha),
    y: THREE.MathUtils.lerp(current.y, next.y, alpha),
    z: THREE.MathUtils.lerp(current.z, next.z, alpha),
  };
}

// Cubic Hermite interpolation using the replay's per-sample velocities as
// tangents. Linear interpolation (interpolatePosition) makes the car travel in
// straight segments and change direction abruptly at every ~30Hz sample, which
// reads as jitter when rendered at 60Hz+. Hermite is C1-continuous (smooth
// velocity through the sample points) so the motion looks fluid.
//
// `velocity` is in position-units per second and `dt` is the segment duration in
// seconds, so `velocity * dt` is the tangent in position units. When velocity is
// unavailable, or the velocity-implied curve deviates implausibly far from the
// straight-line path (e.g. a units mismatch), we fall back to a plain lerp so we
// never look worse than the original linear behavior.
export function interpolatePositionHermite(
  current: Vec3 | null,
  next: Vec3 | null,
  currentVelocity: Vec3 | null,
  nextVelocity: Vec3 | null,
  dt: number,
  alpha: number,
): Vec3 | null {
  const linear = interpolatePosition(current, next, alpha);
  if (
    !current ||
    !next ||
    !currentVelocity ||
    !nextVelocity ||
    dt <= 0 ||
    alpha <= 0 ||
    alpha >= 1
  ) {
    return linear;
  }

  const s = alpha;
  const s2 = s * s;
  const s3 = s2 * s;
  const h00 = 2 * s3 - 3 * s2 + 1;
  const h10 = s3 - 2 * s2 + s;
  const h01 = -2 * s3 + 3 * s2;
  const h11 = s3 - s2;

  const hermiteAxis = (p0: number, p1: number, v0: number, v1: number): number =>
    h00 * p0 + h10 * v0 * dt + h01 * p1 + h11 * v1 * dt;

  const result = {
    x: hermiteAxis(current.x, next.x, currentVelocity.x, nextVelocity.x),
    y: hermiteAxis(current.y, next.y, currentVelocity.y, nextVelocity.y),
    z: hermiteAxis(current.z, next.z, currentVelocity.z, nextVelocity.z),
  };

  // Guard against pathological tangents (e.g. a units mismatch between position
  // and velocity): if the curve swings far past the straight-line segment, the
  // tangents are untrustworthy, so prefer the linear result.
  if (linear) {
    const dx = result.x - linear.x;
    const dy = result.y - linear.y;
    const dz = result.z - linear.z;
    const deviationSq = dx * dx + dy * dy + dz * dz;
    const sx = next.x - current.x;
    const sy = next.y - current.y;
    const sz = next.z - current.z;
    const segmentSq = sx * sx + sy * sy + sz * sz;
    if (deviationSq > segmentSq) {
      return linear;
    }
  }

  return result;
}

export function interpolateQuaternion(
  current: Quaternion | null,
  next: Quaternion | null,
  alpha: number,
): THREE.Quaternion | null {
  const source = current ?? next;
  if (!source) {
    return null;
  }

  const result = new THREE.Quaternion(source.x, source.y, source.z, source.w);
  if (!next || alpha <= 0 || current === null) {
    return result;
  }

  return result.slerp(new THREE.Quaternion(next.x, next.y, next.z, next.w), alpha);
}

export function rootPosition(position: Vec3): THREE.Vector3 {
  return new THREE.Vector3(position.x, position.y, position.z);
}

export function worldPosition(position: Vec3, fieldScale: number): THREE.Vector3 {
  return new THREE.Vector3(
    -position.x * fieldScale,
    position.y * fieldScale,
    position.z * fieldScale,
  );
}

function worldDirection(direction: Vec3): THREE.Vector3 {
  return new THREE.Vector3(-direction.x, direction.y, direction.z).normalize();
}

export function getFreeCameraPreset(
  preset: ReplayFreeCameraPreset,
  fieldScale: number,
): {
  position: THREE.Vector3;
  target: THREE.Vector3;
  up: THREE.Vector3;
  fov: number;
} {
  switch (preset) {
    case "overhead":
      return {
        position: OVERHEAD_CAMERA_POSITION_UU.clone().multiplyScalar(fieldScale),
        target: OVERHEAD_LOOK_TARGET_UU.clone().multiplyScalar(fieldScale),
        up: OVERHEAD_UP.clone(),
        fov: FREE_CAMERA_FOV,
      };
    case "side":
      return {
        position: SIDE_CAMERA_POSITION_UU.clone().multiplyScalar(fieldScale),
        target: SIDE_LOOK_TARGET_UU.clone().multiplyScalar(fieldScale),
        up: DEFAULT_UP.clone(),
        fov: FREE_CAMERA_FOV,
      };
  }
}

export function updateFreeCameraTransition(options: {
  sceneState: ReplayScene;
  position: THREE.Vector3;
  target: THREE.Vector3;
  up: THREE.Vector3;
  fov: number;
}): boolean {
  const { fov, position, sceneState, target, up } = options;
  const { camera, controls } = sceneState;

  controls.enabled = false;
  camera.position.lerp(position, FREE_CAMERA_TRANSITION_SMOOTHING);
  controls.target.lerp(target, FREE_CAMERA_TRANSITION_SMOOTHING);
  camera.up.lerp(up, FREE_CAMERA_TRANSITION_SMOOTHING).normalize();
  camera.fov = THREE.MathUtils.lerp(camera.fov, fov, FREE_CAMERA_TRANSITION_SMOOTHING);
  camera.updateProjectionMatrix();
  camera.lookAt(controls.target);

  const reachedPosition = camera.position.distanceToSquared(position) <= CAMERA_POSITION_EPSILON_SQ;
  const reachedTarget = controls.target.distanceToSquared(target) <= CAMERA_TARGET_EPSILON_SQ;
  const reachedUp = camera.up.angleTo(up) <= CAMERA_UP_EPSILON_RAD;
  const reachedFov = Math.abs(camera.fov - fov) <= CAMERA_FOV_EPSILON;
  if (!reachedPosition || !reachedTarget || !reachedUp || !reachedFov) {
    return false;
  }

  camera.position.copy(position);
  controls.target.copy(target);
  camera.up.copy(up).normalize();
  camera.fov = fov;
  camera.updateProjectionMatrix();
  camera.lookAt(target);
  controls.enabled = true;
  return true;
}

type PlayerFrame = ReplayModel["players"][number]["frames"][number];

// Build a frame whose geometric fields (position/orientation/velocity) are
// interpolated between two replay samples, mirroring how the car mesh itself is
// rendered. Without this the camera chases a discrete, stair-stepped target
// while the car glides smoothly, which reads as jitter in the viewport.
function interpolateCameraFrame(
  frame: PlayerFrame,
  nextFrame: PlayerFrame | null | undefined,
  dt: number,
  alpha: number,
): PlayerFrame {
  if (!nextFrame || alpha <= 0 || nextFrame.isPresent === false || !nextFrame.position) {
    return frame;
  }

  return {
    ...frame,
    position:
      interpolatePositionHermite(
        frame.position ?? null,
        nextFrame.position,
        frame.linearVelocity ?? null,
        nextFrame.linearVelocity ?? null,
        dt,
        alpha,
      ) ?? frame.position,
    forward:
      interpolatePosition(frame.forward ?? null, nextFrame.forward ?? null, alpha) ?? frame.forward,
    up: interpolatePosition(frame.up ?? null, nextFrame.up ?? null, alpha) ?? frame.up,
    linearVelocity:
      interpolatePosition(frame.linearVelocity ?? null, nextFrame.linearVelocity ?? null, alpha) ??
      frame.linearVelocity,
  };
}

function getOrientationVectors(frame: ReplayModel["players"][number]["frames"][number]): {
  forward: THREE.Vector3;
  up: THREE.Vector3;
  right: THREE.Vector3;
} | null {
  const velocity = frame.linearVelocity ? worldDirection(frame.linearVelocity) : null;
  const rawForward = frame.forward ? worldDirection(frame.forward) : null;
  const rawUp = frame.up ? worldDirection(frame.up) : null;
  const grounded = (frame.position?.z ?? Infinity) < GROUND_HEIGHT_THRESHOLD_UU;

  if (grounded) {
    const forward = (rawForward ?? velocity ?? DEFAULT_FORWARD.clone()).clone().setZ(0);

    if (forward.lengthSq() < 0.0001) {
      return null;
    }

    forward.normalize();
    if (velocity && velocity.lengthSq() > 0.0001 && forward.dot(velocity) < 0) {
      forward.negate();
    }
    const right = new THREE.Vector3().crossVectors(DEFAULT_UP, forward).normalize();
    const up = new THREE.Vector3().crossVectors(forward, right).normalize();
    return { forward, up, right };
  }

  if (!rawForward || !rawUp) {
    return null;
  }

  const forward = rawForward.clone().normalize();
  const right = new THREE.Vector3().crossVectors(rawUp, forward).normalize();
  const up = new THREE.Vector3().crossVectors(forward, right).normalize();

  return { forward, up, right };
}

export function updateAttachedCamera(options: {
  sceneState: ReplayScene;
  replay: ReplayModel;
  fieldScale: number;
  cameraViewMode: ReplayCameraViewMode;
  attachedPlayerId: string | null;
  ballCamEnabled: boolean;
  cameraDistanceScale: number;
  customCameraSettings: CameraSettings | null;
  frameIndex: number;
  nextFrameIndex: number;
  alpha: number;
  dt: number;
  renderDelta: number;
  attachedPlayerUnavailable?: boolean;
  ballPosition: THREE.Vector3 | null;
  desiredCameraPosition: THREE.Vector3;
  desiredLookTarget: THREE.Vector3;
  blendState?: AttachedCameraBlendState;
}): void {
  const {
    cameraViewMode,
    attachedPlayerId,
    ballCamEnabled,
    ballPosition,
    cameraDistanceScale,
    customCameraSettings,
    desiredCameraPosition,
    desiredLookTarget,
    attachedPlayerUnavailable = false,
    fieldScale,
    frameIndex,
    nextFrameIndex,
    alpha,
    dt,
    renderDelta,
    replay,
    sceneState,
    blendState: providedBlendState,
  } = options;
  const controls = sceneState.controls;
  const blendState =
    providedBlendState ??
    ({
      currentBlend: ballCamEnabled ? 1 : 0,
      targetBlend: ballCamEnabled ? 1 : 0,
      lastIsBallCam: ballCamEnabled,
    } satisfies AttachedCameraBlendState);

  if (cameraViewMode === "free") {
    controls.enabled = true;
    sceneState.camera.fov = THREE.MathUtils.lerp(
      sceneState.camera.fov,
      FREE_CAMERA_FOV,
      CAMERA_SMOOTHING,
    );
    sceneState.camera.updateProjectionMatrix();
    return;
  }

  if (!attachedPlayerId) {
    controls.enabled = true;
    sceneState.camera.fov = THREE.MathUtils.lerp(
      sceneState.camera.fov,
      FREE_CAMERA_FOV,
      CAMERA_SMOOTHING,
    );
    sceneState.camera.updateProjectionMatrix();
    return;
  }

  const attachedPlayer = replay.players.find((player) => player.id === attachedPlayerId);
  const frame = attachedPlayer?.frames[frameIndex];

  if (
    !attachedPlayer ||
    attachedPlayerUnavailable ||
    !frame?.position ||
    frame.isPresent === false
  ) {
    controls.enabled = true;
    return;
  }

  controls.enabled = false;

  const nextFrame = attachedPlayer.frames[nextFrameIndex] ?? frame;
  const renderFrame = interpolateCameraFrame(frame, nextFrame, dt, alpha);

  const basePosition = worldPosition(renderFrame.position ?? frame.position, fieldScale);
  const orientation = getOrientationVectors(renderFrame);
  const forward = orientation?.forward ?? DEFAULT_FORWARD.clone();
  const right = orientation?.right ?? new THREE.Vector3(0, 1, 0);

  const cameraSettings = {
    ...attachedPlayer.cameraSettings,
    ...(customCameraSettings ?? {}),
  };
  const distance = (cameraSettings.distance ?? 270) * fieldScale * cameraDistanceScale;
  const height = (cameraSettings.height ?? 100) * fieldScale * CHASE_CAMERA_HEIGHT_MULTIPLIER;
  const pitch = THREE.MathUtils.degToRad(cameraSettings.pitch ?? -4);
  const lookDirection = forward.clone().applyAxisAngle(right, pitch).normalize();
  const chaseAnchor = basePosition.clone().addScaledVector(DEFAULT_UP, height);
  const followOffset = forward
    .clone()
    .multiplyScalar(-distance)
    .addScaledVector(DEFAULT_UP, height)
    .applyAxisAngle(right, pitch);
  const playerFocusPoint = basePosition
    .clone()
    .addScaledVector(DEFAULT_UP, PLAYER_FOCUS_HEIGHT_UU * fieldScale);
  const carTargetFov = cameraSettings.fov ?? 110;
  let ballTargetFov = carTargetFov;

  const carCameraPosition = playerFocusPoint.clone().add(followOffset);
  carCameraPosition.z = Math.max(MIN_CAMERA_HEIGHT_UU * fieldScale, carCameraPosition.z);
  const carLookTarget = playerFocusPoint.clone();
  let ballCameraPosition = carCameraPosition;
  let ballLookTarget = carLookTarget;

  if (ballPosition) {
    const ballFocusPoint = ballPosition
      .clone()
      .addScaledVector(DEFAULT_UP, BALL_CAM_HEIGHT_BIAS_UU * fieldScale);
    const playerToBall = ballFocusPoint.clone().sub(playerFocusPoint);
    const ballCamDirection = (
      playerToBall.lengthSq() > 0.0001 ? playerToBall.normalize() : lookDirection.clone()
    )
      .multiplyScalar(BALL_CAM_DIRECTION_BLEND)
      .addScaledVector(lookDirection, 1 - BALL_CAM_DIRECTION_BLEND)
      .normalize();

    ballLookTarget = playerFocusPoint.clone().lerp(ballFocusPoint, BALL_CAM_LOOK_BLEND);
    ballCameraPosition = chaseAnchor.clone().addScaledVector(ballCamDirection, -distance);
    const heightBlend = Math.min(
      1,
      Math.max(0, (ballFocusPoint.z - playerFocusPoint.z) / (800 * fieldScale)),
    );
    ballCameraPosition.z -= heightBlend * BALL_CAM_AERIAL_HEIGHT_ADJUSTMENT_UU * fieldScale;
    ballCameraPosition.z = Math.max(MIN_CAMERA_HEIGHT_UU * fieldScale, ballCameraPosition.z);
    const cameraToPlayer = playerFocusPoint.clone().sub(ballCameraPosition);
    const cameraToBall = ballFocusPoint.clone().sub(ballCameraPosition);
    if (cameraToPlayer.lengthSq() > 0.0001 && cameraToBall.lengthSq() > 0.0001) {
      const separationAngle = cameraToPlayer.angleTo(cameraToBall);
      ballTargetFov = Math.min(
        BALL_CAM_MAX_FOV,
        Math.max(ballTargetFov, THREE.MathUtils.radToDeg(separationAngle) * 1.7),
      );
    }
  }

  if (blendState.lastIsBallCam !== null && blendState.lastIsBallCam !== ballCamEnabled) {
    blendState.targetBlend = ballCamEnabled ? 1 : 0;
  } else {
    blendState.targetBlend = ballCamEnabled ? 1 : 0;
    if (blendState.lastIsBallCam === null) {
      blendState.currentBlend = blendState.targetBlend;
    }
  }
  blendState.lastIsBallCam = ballCamEnabled;

  const transitionSpeed = cameraSettings.transitionSpeed ?? 1.3;
  const transitionDuration = Math.max(
    0.15,
    Math.min(0.6, BALL_CAM_TRANSITION_BASE_DURATION_SECONDS / transitionSpeed),
  );
  const transitionStep = Math.max(0, renderDelta) / transitionDuration;
  if (blendState.currentBlend < blendState.targetBlend) {
    blendState.currentBlend = Math.min(
      blendState.currentBlend + transitionStep,
      blendState.targetBlend,
    );
  } else if (blendState.currentBlend > blendState.targetBlend) {
    blendState.currentBlend = Math.max(
      blendState.currentBlend - transitionStep,
      blendState.targetBlend,
    );
  }
  const blend =
    blendState.currentBlend * blendState.currentBlend * (3 - 2 * blendState.currentBlend);

  desiredCameraPosition.lerpVectors(carCameraPosition, ballCameraPosition, blend);
  desiredLookTarget.lerpVectors(carLookTarget, ballLookTarget, blend);

  sceneState.camera.position.copy(desiredCameraPosition);
  sceneState.camera.up.copy(DEFAULT_UP);
  controls.target.copy(desiredLookTarget);
  sceneState.camera.fov = THREE.MathUtils.lerp(carTargetFov, ballTargetFov, blend);
  sceneState.camera.updateProjectionMatrix();
  sceneState.camera.lookAt(controls.target);
}
