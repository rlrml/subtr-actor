import * as THREE from "three";
import type { ReplayScene } from "../scene";
import type {
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
const BALL_CAM_HEIGHT_BIAS_UU = 45;
const BALL_CAM_LOOK_BLEND = 0.58;
const BALL_CAM_DIRECTION_BLEND = 0.82;
const BALL_CAM_MAX_FOV = 132;
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
  camera.fov = THREE.MathUtils.lerp(
    camera.fov,
    fov,
    FREE_CAMERA_TRANSITION_SMOOTHING,
  );
  camera.updateProjectionMatrix();
  camera.lookAt(controls.target);

  const reachedPosition =
    camera.position.distanceToSquared(position) <= CAMERA_POSITION_EPSILON_SQ;
  const reachedTarget =
    controls.target.distanceToSquared(target) <= CAMERA_TARGET_EPSILON_SQ;
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

function getOrientationVectors(
  frame: ReplayModel["players"][number]["frames"][number],
): {
  forward: THREE.Vector3;
  up: THREE.Vector3;
  right: THREE.Vector3;
} | null {
  const velocity = frame.linearVelocity ? worldDirection(frame.linearVelocity) : null;
  const rawForward = frame.forward ? worldDirection(frame.forward) : null;
  const rawUp = frame.up ? worldDirection(frame.up) : null;
  const grounded = (frame.position?.z ?? Infinity) < GROUND_HEIGHT_THRESHOLD_UU;

  if (grounded) {
    const forward = (rawForward ?? velocity ?? DEFAULT_FORWARD.clone())
      .clone()
      .setZ(0);

    if (forward.lengthSq() < 0.0001) {
      return null;
    }

    forward.normalize();
    if (velocity && velocity.lengthSq() > 0.0001 && forward.dot(velocity) < 0) {
      forward.negate();
    }
    const right = new THREE.Vector3()
      .crossVectors(DEFAULT_UP, forward)
      .normalize();
    const up = new THREE.Vector3()
      .crossVectors(forward, right)
      .normalize();
    return { forward, up, right };
  }

  if (!rawForward || !rawUp) {
    return null;
  }

  const forward = rawForward.clone().normalize();
  const right = new THREE.Vector3().crossVectors(rawUp, forward).normalize();
  const up = new THREE.Vector3()
    .crossVectors(forward, right)
    .normalize();

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
  frameIndex: number;
  ballPosition: THREE.Vector3 | null;
  desiredCameraPosition: THREE.Vector3;
  desiredLookTarget: THREE.Vector3;
}): void {
  const {
    cameraViewMode,
    attachedPlayerId,
    ballCamEnabled,
    ballPosition,
    cameraDistanceScale,
    desiredCameraPosition,
    desiredLookTarget,
    fieldScale,
    frameIndex,
    replay,
    sceneState,
  } = options;
  const controls = sceneState.controls;

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

  const attachedPlayer = replay.players.find(
    (player) => player.id === attachedPlayerId,
  );
  const frame = attachedPlayer?.frames[frameIndex];

  if (!attachedPlayer || !frame?.position) {
    controls.enabled = true;
    return;
  }

  controls.enabled = false;

  const basePosition = worldPosition(frame.position, fieldScale);
  const orientation = getOrientationVectors(frame);
  const forward = orientation?.forward ?? DEFAULT_FORWARD.clone();
  const right = orientation?.right ?? new THREE.Vector3(0, 1, 0);

  const cameraSettings = attachedPlayer.cameraSettings;
  const distance =
    (cameraSettings.distance ?? 270) *
    fieldScale *
    cameraDistanceScale;
  const height =
    (cameraSettings.height ?? 100) *
    fieldScale *
    CHASE_CAMERA_HEIGHT_MULTIPLIER;
  const pitch = THREE.MathUtils.degToRad(cameraSettings.pitch ?? -4);
  const lookDirection = forward
    .clone()
    .applyAxisAngle(right, pitch)
    .normalize();
  const chaseAnchor = basePosition
    .clone()
    .addScaledVector(DEFAULT_UP, height);
  const playerFocusPoint = basePosition
    .clone()
    .addScaledVector(DEFAULT_UP, PLAYER_FOCUS_HEIGHT_UU * fieldScale);
  let targetFov = cameraSettings.fov ?? 110;

  if (ballCamEnabled && ballPosition) {
    const ballFocusPoint = ballPosition
      .clone()
      .addScaledVector(DEFAULT_UP, BALL_CAM_HEIGHT_BIAS_UU * fieldScale);
    const playerToBall = ballFocusPoint.clone().sub(playerFocusPoint);
    const ballCamDirection = (
      playerToBall.lengthSq() > 0.0001
        ? playerToBall.normalize()
        : lookDirection.clone()
    )
      .multiplyScalar(BALL_CAM_DIRECTION_BLEND)
      .addScaledVector(lookDirection, 1 - BALL_CAM_DIRECTION_BLEND)
      .normalize();

    desiredCameraPosition
      .copy(chaseAnchor)
      .addScaledVector(ballCamDirection, -distance);
    desiredCameraPosition.z = Math.max(
      MIN_CAMERA_HEIGHT_UU * fieldScale,
      desiredCameraPosition.z,
    );
    desiredLookTarget
      .copy(playerFocusPoint)
      .lerp(ballFocusPoint, BALL_CAM_LOOK_BLEND);
    const cameraToPlayer = playerFocusPoint.clone().sub(desiredCameraPosition);
    const cameraToBall = ballFocusPoint.clone().sub(desiredCameraPosition);
    if (cameraToPlayer.lengthSq() > 0.0001 && cameraToBall.lengthSq() > 0.0001) {
      const separationAngle = cameraToPlayer.angleTo(cameraToBall);
      targetFov = Math.min(
        BALL_CAM_MAX_FOV,
        Math.max(targetFov, THREE.MathUtils.radToDeg(separationAngle) * 1.7),
      );
    }
  } else {
    desiredCameraPosition
      .copy(chaseAnchor)
      .addScaledVector(forward, -distance);
    desiredCameraPosition.z = Math.max(
      MIN_CAMERA_HEIGHT_UU * fieldScale,
      desiredCameraPosition.z,
    );
    desiredLookTarget
      .copy(basePosition)
      .addScaledVector(lookDirection, distance + 8 * fieldScale)
      .addScaledVector(DEFAULT_UP, PLAYER_FOCUS_HEIGHT_UU * fieldScale);
  }

  sceneState.camera.position.lerp(
    desiredCameraPosition,
    CAMERA_SMOOTHING,
  );
  sceneState.camera.up.lerp(DEFAULT_UP, CAMERA_SMOOTHING).normalize();
  controls.target.lerp(desiredLookTarget, CAMERA_SMOOTHING);
  sceneState.camera.fov = THREE.MathUtils.lerp(
    sceneState.camera.fov,
    targetFov,
    CAMERA_SMOOTHING,
  );
  sceneState.camera.updateProjectionMatrix();
  sceneState.camera.lookAt(controls.target);
}
