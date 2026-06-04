import test from "node:test";
import assert from "node:assert/strict";
import * as THREE from "three";

import { getReplayHitboxOverlayTransform, getReplayHitboxSpec } from "../src/hitboxes";
import { interpolateQuaternion, updateAttachedCamera } from "../src/player-internals/spatial";
import type { ReplayModel } from "../src/types";
import type { ReplayScene } from "../src/scene";

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

test("hitbox overlay transform uses Rocket League local axes", () => {
  const hitbox = getReplayHitboxSpec("octane");
  const transform = getReplayHitboxOverlayTransform(hitbox);

  assert.deepEqual(transform.position, [hitbox.offset, 0, hitbox.elevation]);
  assert.equal(transform.rotationYDegrees, hitbox.slopeDegrees);
  assert.deepEqual(transform.dimensions, [hitbox.length, hitbox.width, hitbox.height]);
});
