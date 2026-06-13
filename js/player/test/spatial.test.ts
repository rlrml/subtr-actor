import test from "node:test";
import assert from "node:assert/strict";
import * as THREE from "three";

import { getReplayHitboxOverlayTransform, getReplayHitboxSpec } from "../src/hitboxes";
import {
  getFreeCameraPreset,
  interpolatePositionHermite,
  interpolateQuaternion,
  updateAttachedCamera,
} from "../src/player-internals/spatial";
import type { ReplayModel } from "../src/types";
import { getHitboxOverlayColor, setHitboxOverlayOnlyMode, type ReplayScene } from "../src/scene";

function buildScene(): ReplayScene {
  const camera = new THREE.PerspectiveCamera(110, 16 / 9, 0.1, 10000);
  camera.up.set(0, 0, 1);

  return {
    camera,
    controls: {
      enabled: true,
      target: new THREE.Vector3(),
    },
  } as ReplayScene;
}

function buildReplay(): ReplayModel {
  return {
    frameCount: 1,
    duration: 0,
    frames: [
      {
        time: 0,
        secondsRemaining: 300,
        gameState: 0,
        kickoffCountdown: 0,
      },
    ],
    ballFrames: [],
    boostPads: [],
    players: [
      {
        id: "player-1",
        name: "Player 1",
        isTeamZero: true,
        cameraSettings: {
          distance: 270,
          height: 100,
          pitch: -4,
          fov: 110,
        },
        hitbox: getReplayHitboxSpec("octane"),
        frames: [
          {
            position: { x: 0, y: 0, z: 17 },
            linearVelocity: { x: -1000, y: 0, z: 0 },
            angularVelocity: null,
            rotation: null,
            forward: { x: -1, y: 0, z: 0 },
            up: { x: 0, y: 0, z: 1 },
            boostAmount: 0,
            boostFraction: 0,
            boostActive: false,
            powerslideActive: false,
            jumpActive: false,
            doubleJumpActive: false,
            dodgeActive: false,
          },
        ],
      },
    ],
    tickMarks: [],
    timelineEvents: [],
    teamZeroNames: [],
    teamOneNames: [],
  };
}

test("ball cam stays behind the attached player when the ball is ahead", () => {
  const desiredCameraPosition = new THREE.Vector3();
  const desiredLookTarget = new THREE.Vector3();

  updateAttachedCamera({
    sceneState: buildScene(),
    replay: buildReplay(),
    fieldScale: 1,
    cameraViewMode: "follow",
    attachedPlayerId: "player-1",
    ballCamEnabled: true,
    cameraDistanceScale: 2.25,
    customCameraSettings: null,
    frameIndex: 0,
    nextFrameIndex: 0,
    alpha: 0,
    dt: 0,
    ballPosition: new THREE.Vector3(3000, 0, 93),
    desiredCameraPosition,
    desiredLookTarget,
  });

  assert.ok(
    desiredCameraPosition.x < 0,
    "expected ball cam to keep the camera behind the player, not between the car and ball",
  );
  assert.ok(desiredLookTarget.x > 0, "expected ball cam to keep looking toward the ball");
});

test("free camera presets frame the arena closer than the old fixed defaults", () => {
  const overhead = getFreeCameraPreset("overhead", 1, 16 / 9);
  assert.equal(overhead.position.x, 0);
  assert.equal(overhead.position.y, 0);
  assert.ok(
    overhead.position.z < 14000,
    `expected overhead preset to sit closer than the old 18800uu default, got ${overhead.position.z}`,
  );

  const diagonal = getFreeCameraPreset("side", 1, 16 / 9);
  assert.ok(
    diagonal.position.distanceTo(diagonal.target) < 15000,
    "expected diagonal preset to sit closer than the old fixed diagonal camera",
  );
});

test("interpolateQuaternion blends rotation samples instead of holding the previous frame", () => {
  const current = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(0, 0, 1), 0);
  const next = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(0, 0, 1), Math.PI);

  const halfway = interpolateQuaternion(
    { x: current.x, y: current.y, z: current.z, w: current.w },
    { x: next.x, y: next.y, z: next.z, w: next.w },
    0.5,
  );
  assert.ok(halfway);

  const rotated = new THREE.Vector3(1, 0, 0).applyQuaternion(halfway);
  assert.ok(Math.abs(rotated.x) < 1e-10);
  assert.ok(rotated.y > 0.999);
});

test("interpolatePositionHermite matches a lerp when motion is straight and constant", () => {
  // Velocity exactly equals the secant (position moves +10/s over a 1s gap), so
  // the Hermite curve has no curvature and should coincide with linear interp.
  const result = interpolatePositionHermite(
    { x: 0, y: 0, z: 0 },
    { x: 10, y: 0, z: 0 },
    { x: 10, y: 0, z: 0 },
    { x: 10, y: 0, z: 0 },
    1,
    0.5,
  );
  assert.ok(result);
  assert.ok(Math.abs(result.x - 5) < 1e-9);
  assert.ok(Math.abs(result.y) < 1e-9);
});

test("interpolatePositionHermite curves toward the sample velocities", () => {
  // Endpoints sit on the x-axis, but the samples carry +y velocity at the start
  // and -y velocity at the end (a projectile-style arc). Linear interp would
  // hold y at 0; Hermite should bow the midpoint above the straight line.
  const result = interpolatePositionHermite(
    { x: 0, y: 0, z: 0 },
    { x: 10, y: 0, z: 0 },
    { x: 10, y: 10, z: 0 },
    { x: 10, y: -10, z: 0 },
    1,
    0.5,
  );
  assert.ok(result);
  assert.ok(result.y > 0.5, "expected the curve to bow toward the velocity tangents");
});

test("interpolatePositionHermite falls back to linear when velocity is missing", () => {
  const result = interpolatePositionHermite(
    { x: 0, y: 0, z: 0 },
    { x: 10, y: 4, z: 0 },
    null,
    null,
    1,
    0.25,
  );
  assert.ok(result);
  assert.ok(Math.abs(result.x - 2.5) < 1e-9);
  assert.ok(Math.abs(result.y - 1) < 1e-9);
});

test("interpolatePositionHermite rejects implausible tangents and falls back to linear", () => {
  // A 100x-too-large velocity (e.g. a units mismatch) would swing the curve far
  // past the segment; the deviation guard should discard it and use the lerp.
  const result = interpolatePositionHermite(
    { x: 0, y: 0, z: 0 },
    { x: 10, y: 0, z: 0 },
    { x: 1000, y: 0, z: 0 },
    { x: 1000, y: 0, z: 0 },
    1,
    0.5,
  );
  assert.ok(result);
  assert.ok(
    Math.abs(result.x - 5) < 1e-9,
    "expected the pathological tangent to fall back to lerp",
  );
});

test("hitbox overlay transform uses Rocket League local axes", () => {
  const hitbox = getReplayHitboxSpec("octane");
  const transform = getReplayHitboxOverlayTransform(hitbox);

  assert.deepEqual(transform.position, [hitbox.offset, 0, hitbox.elevation]);
  assert.equal(transform.rotationYDegrees, hitbox.slopeDegrees);
  assert.deepEqual(transform.dimensions, [hitbox.length, hitbox.width, hitbox.height]);
});

test("hitbox-only mode increases overlay fill without changing line visibility", () => {
  const group = new THREE.Group();
  const fill = new THREE.Mesh(
    new THREE.BoxGeometry(1, 1, 1),
    new THREE.MeshBasicMaterial({ opacity: 0.08, transparent: true }),
  );
  fill.name = "hitbox-overlay-fill";
  const lines = new THREE.LineSegments(
    new THREE.EdgesGeometry(new THREE.BoxGeometry(1, 1, 1)),
    new THREE.LineBasicMaterial({ opacity: 1, transparent: true }),
  );
  lines.name = "hitbox-overlay-lines";
  group.add(fill);
  group.add(lines);

  setHitboxOverlayOnlyMode(group, true);

  assert.equal(fill.material.opacity, 0.22);
  assert.equal(lines.visible, true);

  setHitboxOverlayOnlyMode(group, false);

  assert.equal(fill.material.opacity, 0.08);
  assert.equal(lines.visible, true);
});

test("hitbox overlay color keeps a dark team tint", () => {
  const overlayColor = getHitboxOverlayColor("#57a8ff");

  assert.equal(overlayColor.getHexString(), "112a45");
});
