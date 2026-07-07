import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  DEFAULT_TRAINING_SHOT_TIME_LIMIT_SECONDS,
  MIN_CAR_SPAWN_Z,
  appendCapturedRound,
  ballSpawnFromReplayState,
  capturedTrainingPackDefaults,
  carSpawnFromReplayState,
  generateTrainingPackGuid,
  momentumLossWarning,
  playerCarSpawnFromReplayState,
  quaternionToRotator,
  radiansToRotatorUnits,
  trainingPackFileName,
  trainingPackGuidHex,
  velocityToRotatorAndSpeed,
} from "../src/training-capture";
import { TrainingPackFile, type TrainingPackBindings } from "../src/training-pack";
import type { Quaternion } from "../src/types";

const here = path.dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

const bindings = require("../../pkg-node/rl_replay_subtr_actor.js") as TrainingPackBindings;

const FIXTURE_PATH = path.join(
  here,
  "../../../crates/subtr-actor-training/assets/synthetic-pack.tem",
);

/** Quaternion for a right-handed rotation of `radians` about `axis`. */
function axisAngle(axis: { x: number; y: number; z: number }, radians: number): Quaternion {
  const half = radians / 2;
  const sin = Math.sin(half);
  return { x: axis.x * sin, y: axis.y * sin, z: axis.z * sin, w: Math.cos(half) };
}

test("radiansToRotatorUnits maps quarter and half turns", () => {
  assert.equal(radiansToRotatorUnits(0), 0);
  assert.equal(radiansToRotatorUnits(Math.PI / 2), 16384);
  assert.equal(radiansToRotatorUnits(-Math.PI / 2), -16384);
  assert.equal(radiansToRotatorUnits(Math.PI), 32768);
});

test("velocity straight +X is yaw 0, pitch 0", () => {
  const { rotator, speed } = velocityToRotatorAndSpeed({ x: 1000, y: 0, z: 0 });
  assert.deepEqual(rotator, { pitch: 0, yaw: 0, roll: 0 });
  assert.equal(speed, 1000);
});

test("velocity straight up is pitch 16384", () => {
  const { rotator, speed } = velocityToRotatorAndSpeed({ x: 0, y: 0, z: 500 });
  assert.equal(rotator.pitch, 16384);
  assert.equal(rotator.roll, 0);
  assert.equal(speed, 500);
});

test("velocity straight +Y is yaw 16384", () => {
  const { rotator, speed } = velocityToRotatorAndSpeed({ x: 0, y: 1200, z: 0 });
  assert.deepEqual(rotator, { pitch: 0, yaw: 16384, roll: 0 });
  assert.equal(speed, 1200);
});

test("velocity magnitude combines components", () => {
  const { rotator, speed } = velocityToRotatorAndSpeed({ x: 300, y: 400, z: 0 });
  assert.equal(speed, 500);
  assert.equal(rotator.yaw, radiansToRotatorUnits(Math.atan2(400, 300)));
});

test("zero velocity collapses to speed 0 with the default rotator", () => {
  assert.deepEqual(velocityToRotatorAndSpeed({ x: 0, y: 0, z: 0 }), {
    rotator: { pitch: 0, yaw: 0, roll: 0 },
    speed: 0,
  });
  assert.deepEqual(velocityToRotatorAndSpeed(null), {
    rotator: { pitch: 0, yaw: 0, roll: 0 },
    speed: 0,
  });
});

test("identity quaternion is the zero rotator", () => {
  assert.deepEqual(quaternionToRotator({ x: 0, y: 0, z: 0, w: 1 }), {
    pitch: 0,
    yaw: 0,
    roll: 0,
  });
});

test("90-degree yaw quaternion (about +Z) is yaw 16384", () => {
  const rotator = quaternionToRotator(axisAngle({ x: 0, y: 0, z: 1 }, Math.PI / 2));
  assert.equal(rotator.yaw, 16384);
  assert.equal(rotator.pitch, 0);
  assert.equal(rotator.roll, 0);
});

test("nose-up quaternion (forward to +Z) is pitch 16384", () => {
  // Right-handed rotation about +Y by -90 degrees points local +X at world +Z.
  const rotator = quaternionToRotator(axisAngle({ x: 0, y: 1, z: 0 }, -Math.PI / 2));
  assert.equal(rotator.pitch, 16384);
  assert.equal(rotator.yaw, 0);
});

test("90-degree roll quaternion (about forward) is roll 16384", () => {
  const rotator = quaternionToRotator(axisAngle({ x: 1, y: 0, z: 0 }, Math.PI / 2));
  assert.equal(rotator.roll, 16384);
  assert.equal(rotator.pitch, 0);
  assert.equal(rotator.yaw, 0);
});

test("yaw + pitch compose: yaw 90 then nose up 45", () => {
  // Intrinsic composition: world yaw about +Z, then pitch about the yawed
  // right axis (world +X after a 90-degree yaw... the car's right is -X).
  const yawQuat = axisAngle({ x: 0, y: 0, z: 1 }, Math.PI / 2);
  const pitchQuat = axisAngle({ x: 0, y: 1, z: 0 }, -Math.PI / 4);
  // q = yaw * pitch applies pitch in the car frame first.
  const q = {
    w:
      yawQuat.w * pitchQuat.w -
      yawQuat.x * pitchQuat.x -
      yawQuat.y * pitchQuat.y -
      yawQuat.z * pitchQuat.z,
    x:
      yawQuat.w * pitchQuat.x +
      yawQuat.x * pitchQuat.w +
      yawQuat.y * pitchQuat.z -
      yawQuat.z * pitchQuat.y,
    y:
      yawQuat.w * pitchQuat.y -
      yawQuat.x * pitchQuat.z +
      yawQuat.y * pitchQuat.w +
      yawQuat.z * pitchQuat.x,
    z:
      yawQuat.w * pitchQuat.z +
      yawQuat.x * pitchQuat.y -
      yawQuat.y * pitchQuat.x +
      yawQuat.z * pitchQuat.w,
  };
  const rotator = quaternionToRotator(q);
  assert.equal(rotator.yaw, 16384);
  assert.equal(rotator.pitch, 8192);
  assert.equal(rotator.roll, 0);
});

test("ballSpawnFromReplayState maps position 1:1 and encodes velocity", () => {
  const spawn = ballSpawnFromReplayState({
    position: { x: 62.16, y: 4502.21, z: 776.38 },
    linearVelocity: { x: 0, y: 0, z: 1500 },
  });
  assert.equal(spawn.start_location_x, 62.16);
  assert.equal(spawn.start_location_y, 4502.21);
  assert.equal(spawn.start_location_z, 776.38);
  assert.equal(spawn.velocity_start_rotation_p, 16384);
  assert.equal(spawn.velocity_start_rotation_r, 0);
  assert.equal(spawn.velocity_start_speed, 1500);
});

test("playerCarSpawnFromReplayState maps position and quaternion rotation", () => {
  const spawn = playerCarSpawnFromReplayState({
    position: { x: -599.9999, y: 100, z: 17.01 },
    rotation: axisAngle({ x: 0, y: 0, z: 1 }, Math.PI / 2),
  });
  assert.equal(spawn.is_pc, true);
  assert.equal(spawn.location_x, -599.9999);
  assert.equal(spawn.location_y, 100);
  assert.equal(spawn.location_z, 17.01);
  assert.equal(spawn.rotation_y, 16384);
  assert.equal(spawn.rotation_p, 0);
  assert.equal(spawn.rotation_r, 0);
});

test("missing car rotation falls back to the identity rotator", () => {
  const spawn = playerCarSpawnFromReplayState({ position: { x: 0, y: 0, z: 17 } });
  assert.equal(spawn.rotation_p, 0);
  assert.equal(spawn.rotation_y, 0);
  assert.equal(spawn.rotation_r, 0);
});

test("guid hex and filename follow the game's .Tem convention", () => {
  const guid = { a: 0x0012abcd, b: -1, c: 0, d: 0x7fffffff };
  assert.equal(trainingPackGuidHex(guid), "0012ABCDFFFFFFFF000000007FFFFFFF");
  assert.equal(trainingPackFileName(guid), "0012ABCDFFFFFFFF000000007FFFFFFF.Tem");
});

test("generated pack guids are non-zero and distinct", () => {
  const first = generateTrainingPackGuid();
  const second = generateTrainingPackGuid();
  assert.notDeepEqual(first, { a: 0, b: 0, c: 0, d: 0 });
  assert.notDeepEqual(first, second);
});

test("capture -> round -> toBytes -> re-parse round-trips through the typed API", () => {
  const defaults = capturedTrainingPackDefaults(1_700_000_000);
  const file = TrainingPackFile.createWithBindings(bindings, {
    ...defaults,
    name: "Browser Capture Test",
  });

  const index = appendCapturedRound(file, {
    ball: {
      position: { x: 62.16, y: 4502.21, z: 776.38 },
      linearVelocity: { x: 100, y: 2600, z: -180 },
    },
    shooter: {
      position: { x: 139.64, y: 1767.91, z: 17.01 },
      rotation: axisAngle({ x: 0, y: 0, z: 1 }, Math.PI / 2),
    },
    timeLimit: 10,
  });
  assert.equal(index, 0);
  assert.equal(file.roundCount, 1);

  const reparsed = TrainingPackFile.fromBytes(file.toBytes(), bindings);
  assert.equal(reparsed.name, "Browser Capture Test");
  assert.equal(reparsed.trainingType, "Training_Striker");
  assert.equal(reparsed.difficulty, "D_Medium");
  assert.equal(reparsed.mapName, "Park_P");
  assert.equal(reparsed.pack.created_at, 1_700_000_000n);
  assert.equal(reparsed.roundCount, 1);

  const round = reparsed.rounds[0]!;
  assert.equal(round.time_limit, 10);
  assert.equal(round.serialized_archetypes.length, 3);

  const archetypes = reparsed.getRoundArchetypes(0);
  assert.deepEqual(
    archetypes.map((archetype) => archetype.kind),
    ["Ball", "CarSpawnPoint", "PlayerCar"],
  );

  const ball = archetypes[0]!;
  assert.equal(ball.kind, "Ball");
  if (ball.kind !== "Ball") return;
  // Serialized floats round to four decimals.
  assert.ok(Math.abs(ball.start_location_x - 62.16) < 1e-3);
  assert.ok(Math.abs(ball.start_location_y - 4502.21) < 1e-3);
  assert.ok(Math.abs(ball.velocity_start_speed - Math.hypot(100, 2600, -180)) < 1e-3);
  assert.equal(ball.velocity_start_rotation_y, radiansToRotatorUnits(Math.atan2(2600, 100)));

  // The game places the training car from the spawn-point entry, so it must
  // carry the CAPTURED shooter transform, not a hardcoded default.
  const spawnPoint = archetypes[1]!;
  assert.equal(spawnPoint.kind, "CarSpawnPoint");
  if (spawnPoint.kind !== "CarSpawnPoint") return;
  assert.ok(Math.abs(spawnPoint.location_x - 139.64) < 1e-3);
  assert.ok(Math.abs(spawnPoint.location_y - 1767.91) < 1e-3);
  assert.ok(Math.abs(spawnPoint.location_z - 17.01) < 1e-3);
  assert.equal(spawnPoint.rotation_y, 16384);
  assert.equal(spawnPoint.rotation_p, 0);
  assert.equal(spawnPoint.velocity_start_speed, 0);

  const car = archetypes[2]!;
  assert.equal(car.kind, "PlayerCar");
  if (car.kind !== "PlayerCar") return;
  assert.equal(car.is_pc, true);
  // The IsPC entry duplicates the spawn point's captured transform.
  assert.equal(car.location_x, spawnPoint.location_x);
  assert.equal(car.location_y, spawnPoint.location_y);
  assert.equal(car.location_z, spawnPoint.location_z);
  assert.equal(car.rotation_p, spawnPoint.rotation_p);
  assert.equal(car.rotation_y, spawnPoint.rotation_y);
  assert.equal(car.rotation_r, spawnPoint.rotation_r);
});

test("car spawn Z clamps to 17uu for ground-clipped samples", () => {
  const grounded = { position: { x: 10, y: 20, z: 16.3 } };
  assert.equal(carSpawnFromReplayState(grounded).location_z, MIN_CAR_SPAWN_Z);
  assert.equal(playerCarSpawnFromReplayState(grounded).location_z, MIN_CAR_SPAWN_Z);
  // X/Y are untouched by the clamp.
  assert.equal(carSpawnFromReplayState(grounded).location_x, 10);
  assert.equal(carSpawnFromReplayState(grounded).location_y, 20);
});

test("airborne car spawn Z passes through unclamped", () => {
  const airborne = { position: { x: 0, y: 0, z: 512.75 } };
  assert.equal(carSpawnFromReplayState(airborne).location_z, 512.75);
  assert.equal(playerCarSpawnFromReplayState(airborne).location_z, 512.75);
});

test("every built round contains ball + spawn-mesh + IsPC with the captured car transform", () => {
  const file = TrainingPackFile.createWithBindings(bindings, capturedTrainingPackDefaults());
  const shooters = [
    {
      position: { x: -1500.5, y: 2200.25, z: 17.05 },
      rotation: axisAngle({ x: 0, y: 0, z: 1 }, Math.PI / 2),
    },
    {
      position: { x: 800, y: -3000, z: 450 },
      rotation: axisAngle({ x: 0, y: 1, z: 0 }, -Math.PI / 4),
    },
  ];
  for (const shooter of shooters) {
    appendCapturedRound(file, {
      ball: { position: { x: 0, y: 0, z: 93 }, linearVelocity: { x: 0, y: 1000, z: 0 } },
      shooter,
    });
  }
  for (const [index, shooter] of shooters.entries()) {
    const archetypes = file.getRoundArchetypes(index);
    assert.deepEqual(
      archetypes.map((archetype) => archetype.kind),
      ["Ball", "CarSpawnPoint", "PlayerCar"],
    );
    const expected = quaternionToRotator(shooter.rotation);
    const spawnPoint = archetypes[1]!;
    if (spawnPoint.kind !== "CarSpawnPoint") throw new Error("expected CarSpawnPoint");
    const car = archetypes[2]!;
    if (car.kind !== "PlayerCar") throw new Error("expected PlayerCar");
    for (const entry of [spawnPoint, car]) {
      // Captured (non-default) transform on BOTH car entries.
      assert.ok(Math.abs((entry.location_x ?? NaN) - shooter.position.x) < 1e-3);
      assert.ok(Math.abs((entry.location_y ?? NaN) - shooter.position.y) < 1e-3);
      assert.ok(Math.abs((entry.location_z ?? NaN) - shooter.position.z) < 1e-3);
      assert.equal(entry.rotation_p, expected.pitch);
      assert.equal(entry.rotation_y, expected.yaw);
      assert.equal(entry.rotation_r, expected.roll);
    }
    assert.equal(car.is_pc, true);
  }
});

test("default time limit is 8 seconds", () => {
  const file = TrainingPackFile.createWithBindings(bindings, capturedTrainingPackDefaults());
  appendCapturedRound(file, {
    ball: { position: { x: 0, y: 0, z: 93 } },
    shooter: { position: { x: 0, y: -1000, z: 17 } },
  });
  assert.equal(DEFAULT_TRAINING_SHOT_TIME_LIMIT_SECONDS, 8);
  assert.equal(file.rounds[0]!.time_limit, 8);
});

test("captured archetype key sets match the synthetic fixture's vocabulary", () => {
  const file = TrainingPackFile.createWithBindings(bindings, capturedTrainingPackDefaults());
  appendCapturedRound(file, {
    ball: {
      position: { x: 1, y: 2, z: 3 },
      linearVelocity: { x: 10, y: 20, z: 30 },
    },
    shooter: {
      position: { x: 4, y: 5, z: 6 },
      rotation: { x: 0, y: 0, z: 0, w: 1 },
    },
  });
  const captured = file.rounds[0]!.serialized_archetypes.map(
    (raw) => JSON.parse(raw) as Record<string, unknown>,
  );

  const fixture = TrainingPackFile.fromBytes(readFileSync(FIXTURE_PATH), bindings);
  const fixtureArchetypes = fixture.rounds.flatMap((round) =>
    round.serialized_archetypes.map((raw) => JSON.parse(raw) as Record<string, unknown>),
  );

  const keySet = (record: Record<string, unknown>) => Object.keys(record).sort().join(",");
  const fixtureByArchetype = new Map<string, string[]>();
  for (const record of fixtureArchetypes) {
    const kind = record.IsPC !== undefined ? "IsPC" : String(record.ObjectArchetype);
    const sets = fixtureByArchetype.get(kind) ?? [];
    sets.push(keySet(record));
    fixtureByArchetype.set(kind, sets);
  }

  // Ball and player car match a fixture archetype's key set exactly.
  const [capturedBall, capturedSpawnPoint, capturedCar] = captured;
  assert.ok(
    fixtureByArchetype.get("Archetypes.Ball.Ball_GameEditor")!.includes(keySet(capturedBall!)),
    `ball keys ${keySet(capturedBall!)} not found in fixture`,
  );
  assert.ok(fixtureByArchetype.get("IsPC")!.includes(keySet(capturedCar!)), "player car keys");

  // The spawn-point marker matches the fixture's key set plus
  // VelocityStartSpeed, which the BakkesMod plugin (and this path) writes
  // explicitly while Psyonix-style fixtures omit it when zero.
  const fixtureSpawnKeys = fixtureByArchetype.get(
    "Archetypes.GameEditor.DynamicSpawnPointMesh",
  )![0]!;
  assert.equal(
    keySet(capturedSpawnPoint!),
    [...fixtureSpawnKeys.split(","), "VelocityStartSpeed"].sort().join(","),
  );
});

test("momentumLossWarning is null for along-facing motion and slow cars", () => {
  // Fast, but straight down the facing (identity rotation faces +X).
  assert.equal(
    momentumLossWarning({
      position: { x: 0, y: 0, z: 17 },
      linearVelocity: { x: 1400, y: 0, z: 0 },
    }),
    null,
  );
  // Pure sideways drift, but under the 300 uu/s total-speed floor.
  assert.equal(
    momentumLossWarning({
      position: { x: 0, y: 0, z: 17 },
      linearVelocity: { x: 0, y: 250, z: 0 },
    }),
    null,
  );
  // No velocity sample at all.
  assert.equal(momentumLossWarning({ position: { x: 0, y: 0, z: 17 } }), null);
});

test("momentumLossWarning fires for fast sideways drift with the plugin's wording", () => {
  assert.equal(
    momentumLossWarning({
      position: { x: 0, y: 0, z: 17 },
      linearVelocity: { x: 0, y: 900, z: 0 },
    }),
    "car moving 900 uu/s at 90\u{b0} off facing; only 0 uu/s representable as spawn momentum",
  );
});

test("momentumLossWarning treats reversing as losing everything", () => {
  assert.equal(
    momentumLossWarning({
      position: { x: 0, y: 0, z: 17 },
      linearVelocity: { x: -1000, y: 0, z: 0 },
    }),
    "car moving 1000 uu/s at 180\u{b0} off facing; only 0 uu/s representable as spawn momentum",
  );
});

test("momentumLossWarning angle gate fires under the lost-speed threshold; mild angles stay quiet", () => {
  // Facing +Y (90-degree yaw); moving 45 degrees off it at 500 uu/s:
  // lost = 500*sin(45) ~ 354 <= 400, but 45 > 30 degrees trips the gate.
  const facingPlusY = axisAngle({ x: 0, y: 0, z: 1 }, Math.PI / 2);
  assert.equal(
    momentumLossWarning({
      position: { x: 0, y: 0, z: 17 },
      rotation: facingPlusY,
      linearVelocity: { x: -353.5534, y: 353.5534, z: 0 },
    }),
    "car moving 500 uu/s at 45\u{b0} off facing; only 354 uu/s representable as spawn momentum",
  );

  // 20 degrees off at 800 uu/s: lost ~274 and angle under 30 -> quiet.
  const angle = (20 * Math.PI) / 180;
  assert.equal(
    momentumLossWarning({
      position: { x: 0, y: 0, z: 17 },
      linearVelocity: { x: 800 * Math.cos(angle), y: 800 * Math.sin(angle), z: 0 },
    }),
    null,
  );
});
