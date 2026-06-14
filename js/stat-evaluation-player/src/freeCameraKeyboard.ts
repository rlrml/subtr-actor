import * as THREE from "three";
import type { StatsReplayPlayer } from "./statsReplayPlayer.ts";

/** Fly speed of the free camera, in Unreal units per second. */
const FREE_CAMERA_SPEED_UU_PER_SEC = 3500;

/** Movement axes the WASD/Space/Shift keys drive. */
interface FreeCameraInput {
  forward: boolean;
  backward: boolean;
  left: boolean;
  right: boolean;
  up: boolean;
  down: boolean;
}

export interface FreeCameraKeyboardOptions {
  getReplayPlayer(): StatsReplayPlayer | null;
  signal: AbortSignal;
}

/**
 * True when keyboard movement should be ignored because the user is typing into
 * a form field (input/textarea/select/contenteditable). WASD only flies the
 * camera when "nothing else has the focus".
 */
function isTextEntryFocused(): boolean {
  const active = document.activeElement;
  if (!(active instanceof HTMLElement)) {
    return false;
  }
  if (active.isContentEditable) {
    return true;
  }
  const tag = active.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT";
}

/**
 * Drive the free (unattached) camera with WASD — forward/left/back/right along
 * the current view, Space/Shift for world up/down — whenever no text field has
 * focus. Movement only applies in "free" camera mode; in follow mode the camera
 * plugin owns the camera each frame. Listeners and the animation loop are torn
 * down when `signal` aborts.
 */
export function installFreeCameraKeyboard(options: FreeCameraKeyboardOptions): void {
  const { getReplayPlayer, signal } = options;
  const input: FreeCameraInput = {
    forward: false,
    backward: false,
    left: false,
    right: false,
    up: false,
    down: false,
  };

  const clearInput = (): void => {
    input.forward = false;
    input.backward = false;
    input.left = false;
    input.right = false;
    input.up = false;
    input.down = false;
  };

  // Returns true when the event mapped to a movement axis (so we can swallow it).
  const applyKey = (code: string, pressed: boolean): boolean => {
    switch (code) {
      case "KeyW":
        input.forward = pressed;
        return true;
      case "KeyS":
        input.backward = pressed;
        return true;
      case "KeyA":
        input.left = pressed;
        return true;
      case "KeyD":
        input.right = pressed;
        return true;
      case "Space":
        input.up = pressed;
        return true;
      case "ShiftLeft":
      case "ShiftRight":
        input.down = pressed;
        return true;
      default:
        return false;
    }
  };

  const onKeyDown = (event: KeyboardEvent): void => {
    // Leave browser/OS chords (Ctrl/Cmd/Alt) and text entry alone.
    if (event.ctrlKey || event.metaKey || event.altKey || isTextEntryFocused()) {
      return;
    }
    if (applyKey(event.code, true)) {
      // Stop Space from scrolling / toggling buttons while flying.
      event.preventDefault();
    }
  };

  const onKeyUp = (event: KeyboardEvent): void => {
    // Always release on keyup (even over a focused field) so keys never stick.
    applyKey(event.code, false);
  };

  window.addEventListener("keydown", onKeyDown, { signal });
  window.addEventListener("keyup", onKeyUp, { signal });
  window.addEventListener("blur", clearInput, { signal });

  const forward = new THREE.Vector3();
  const right = new THREE.Vector3();
  const move = new THREE.Vector3();
  const worldUp = new THREE.Vector3(0, 1, 0);

  let lastNow: number | null = null;
  let frameId = 0;

  const tick = (now: number): void => {
    frameId = requestAnimationFrame(tick);
    const dt = lastNow === null ? 0 : Math.min(0.1, (now - lastNow) / 1000);
    lastNow = now;

    const forwardAxis = (input.forward ? 1 : 0) - (input.backward ? 1 : 0);
    const rightAxis = (input.right ? 1 : 0) - (input.left ? 1 : 0);
    const upAxis = (input.up ? 1 : 0) - (input.down ? 1 : 0);
    if (dt === 0 || (forwardAxis === 0 && rightAxis === 0 && upAxis === 0)) {
      return;
    }

    const player = getReplayPlayer();
    if (!player || player.getState().cameraViewMode !== "free") {
      return;
    }

    const camera = player.camera;
    const controls = player.controls;
    camera.getWorldDirection(forward);
    right.set(1, 0, 0).applyQuaternion(camera.quaternion);

    move
      .set(0, 0, 0)
      .addScaledVector(forward, forwardAxis)
      .addScaledVector(right, rightAxis)
      .addScaledVector(worldUp, upAxis);
    if (move.lengthSq() === 0) {
      return;
    }
    move.normalize().multiplyScalar(FREE_CAMERA_SPEED_UU_PER_SEC * dt);

    // Translate the camera and its orbit target together so OrbitControls keeps
    // the current view direction (a fly/pan) instead of snapping back.
    camera.position.add(move);
    controls.target.add(move);
  };

  frameId = requestAnimationFrame(tick);
  signal.addEventListener("abort", () => {
    cancelAnimationFrame(frameId);
  });
}
