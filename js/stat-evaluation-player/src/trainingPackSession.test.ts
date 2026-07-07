import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

import type { TrainingPackBindings } from "@rlrml/player";
import { TrainingPackSession } from "./trainingPackSession.ts";

const here = path.dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

const bindings = require("../../pkg-node/rl_replay_subtr_actor.js") as TrainingPackBindings;

const FIXTURE_PATH = path.join(
  here,
  "../../../crates/subtr-actor-training/assets/synthetic-pack.tem",
);

const CAPTURE = {
  ball: {
    position: { x: 100, y: 200, z: 300 },
    linearVelocity: { x: 0, y: 1000, z: 0 },
  },
  shooter: {
    position: { x: 0, y: -1000, z: 17 },
    rotation: { x: 0, y: 0, z: 0, w: 1 },
  },
} as const;

test("a new session starts empty with capture defaults", async () => {
  const session = await TrainingPackSession.createNew({ bindings });
  assert.equal(session.shotCount, 0);
  assert.equal(session.hasUnsavedShots, false);
  assert.equal(session.file.trainingType, "Training_Striker");
  assert.equal(session.file.difficulty, "D_Medium");
  assert.notDeepEqual(session.file.guid, { a: 0, b: 0, c: 0, d: 0 });
  assert.match(session.downloadFileName(), /^[0-9A-F]{32}\.Tem$/);
});

test("capturing appends shots with source-time annotations", async () => {
  const session = await TrainingPackSession.createNew({ bindings });
  const first = session.captureShot({ ...CAPTURE, timeLimit: 10 }, 63.3);
  const second = session.captureShot(CAPTURE);
  assert.equal(first, 0);
  assert.equal(second, 1);
  assert.equal(session.hasUnsavedShots, true);
  assert.deepEqual(session.shots(), [
    { index: 0, timeLimit: 10, sourceReplayTime: 63.3 },
    { index: 1, timeLimit: 8, sourceReplayTime: null },
  ]);
});

test("loading a pack seeds the session and captures append non-destructively", async () => {
  const original = readFileSync(FIXTURE_PATH);
  const session = await TrainingPackSession.loadFromBytes(original, { bindings });
  const loadedCount = session.shotCount;
  const loadedRounds = session.file.rounds;
  assert.ok(loadedCount > 0);
  assert.equal(session.hasUnsavedShots, false);
  assert.ok(session.shots().every((shot) => shot.sourceReplayTime === null));

  session.captureShot(CAPTURE, 12.5);
  assert.equal(session.shotCount, loadedCount + 1);
  // Pre-existing rounds survive byte-for-byte at the typed level.
  assert.deepEqual(session.file.rounds.slice(0, loadedCount), loadedRounds);
  assert.deepEqual(session.shots()[loadedCount], {
    index: loadedCount,
    timeLimit: 8,
    sourceReplayTime: 12.5,
  });
});

test("removeShot keeps annotations aligned and marks the session dirty", async () => {
  const session = await TrainingPackSession.createNew({ bindings });
  session.captureShot(CAPTURE, 1);
  session.captureShot(CAPTURE, 2);
  session.captureShot(CAPTURE, 3);
  session.toBytes(1_700_000_000);
  assert.equal(session.hasUnsavedShots, false);

  session.removeShot(1);
  assert.equal(session.hasUnsavedShots, true);
  assert.deepEqual(
    session.shots().map((shot) => shot.sourceReplayTime),
    [1, 3],
  );
});

test("toBytes refreshes UpdatedAt, clears the dirty flag, and round-trips", async () => {
  const session = await TrainingPackSession.createNew({ bindings });
  session.captureShot(CAPTURE, 5);
  const bytes = session.toBytes(1_700_000_123);
  assert.equal(session.hasUnsavedShots, false);
  assert.equal(session.file.pack.updated_at, 1_700_000_123n);

  const reloaded = await TrainingPackSession.loadFromBytes(bytes, { bindings });
  assert.equal(reloaded.shotCount, 1);
  assert.equal(reloaded.file.pack.updated_at, 1_700_000_123n);
});

test("setShotTimeLimit edits the round in place", async () => {
  const session = await TrainingPackSession.createNew({ bindings });
  session.captureShot(CAPTURE, 5);
  session.toBytes(1_700_000_000);
  session.setShotTimeLimit(0, 12.5);
  assert.equal(session.shots()[0]!.timeLimit, 12.5);
  assert.equal(session.hasUnsavedShots, true);
});
