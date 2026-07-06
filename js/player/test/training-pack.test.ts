import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  TrainingPackFile,
  defaultTrainingPack,
  type TrainingPackBindings,
} from "../src/training-pack";

const here = path.dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

const bindings = require("../../pkg-node/rl_replay_subtr_actor.js") as TrainingPackBindings;

const FIXTURE_PATH = path.join(
  here,
  "../../../crates/subtr-actor-training/assets/synthetic-pack.tem",
);

function loadFixture(): TrainingPackFile {
  return TrainingPackFile.fromBytes(readFileSync(FIXTURE_PATH), bindings);
}

test("loads the synthetic pack and reads metadata and rounds", () => {
  const pack = loadFixture();

  assert.equal(typeof pack.name, "string");
  assert.ok(pack.roundCount > 0);
  assert.equal(pack.rounds.length, pack.roundCount);
  for (const round of pack.rounds) {
    assert.equal(typeof round.time_limit, "number");
    assert.ok(Array.isArray(round.serialized_archetypes));
  }

  const typed = pack.toJSON();
  assert.equal(typed.name, pack.name);
  assert.equal(typed.difficulty, pack.difficulty);
  assert.equal(typed.training_type, pack.trainingType);
  assert.equal(typeof typed.created_at, "bigint");
  assert.equal(typeof typed.creator_player_id.uid, "bigint");
});

test("an untouched pack round-trips to byte-identical output", () => {
  const original = readFileSync(FIXTURE_PATH);
  const pack = TrainingPackFile.fromBytes(original, bindings);
  assert.deepEqual(Buffer.from(pack.toBytes()), Buffer.from(original));
});

test("metadata edits persist through serialize and re-parse", () => {
  const pack = loadFixture();
  pack.setName("Edited Name");
  pack.setDescription("Edited description");
  pack.setCode("ABCD-EFGH-IJKL-MNOP");
  pack.setDifficulty("D_Hard");
  pack.setTrainingType("Training_Striker");
  pack.setMapName("Park_P");
  pack.setTags([3, 7]);

  const reparsed = TrainingPackFile.fromBytes(pack.toBytes(), bindings);
  assert.equal(reparsed.name, "Edited Name");
  assert.equal(reparsed.description, "Edited description");
  assert.equal(reparsed.code, "ABCD-EFGH-IJKL-MNOP");
  assert.equal(reparsed.difficulty, "D_Hard");
  assert.equal(reparsed.trainingType, "Training_Striker");
  assert.equal(reparsed.mapName, "Park_P");
  assert.deepEqual(reparsed.tags, [3, 7]);
});

test("round operations edit and persist", () => {
  const pack = loadFixture();
  const initialRounds = pack.rounds;
  const initialCount = initialRounds.length;

  pack.duplicateRound(0);
  assert.equal(pack.roundCount, initialCount + 1);
  assert.deepEqual(pack.rounds[1], initialRounds[0]);

  pack.addRound({ time_limit: 12.5, serialized_archetypes: ["Archetype:Test"] });
  assert.equal(pack.roundCount, initialCount + 2);
  const added = pack.rounds[pack.roundCount - 1];
  assert.ok(Math.abs(added.time_limit - 12.5) < 1e-6);
  assert.deepEqual(added.serialized_archetypes, ["Archetype:Test"]);

  pack.insertRound(0, { time_limit: 3, serialized_archetypes: [] });
  assert.equal(pack.roundCount, initialCount + 3);
  assert.equal(pack.rounds[0].time_limit, 3);

  pack.moveRound(0, pack.roundCount - 1);
  assert.equal(pack.rounds[pack.roundCount - 1].time_limit, 3);

  const removed = pack.removeRound(pack.roundCount - 1);
  assert.equal(removed.time_limit, 3);
  assert.equal(pack.roundCount, initialCount + 2);

  const reparsed = TrainingPackFile.fromBytes(pack.toBytes(), bindings);
  assert.deepEqual(reparsed.rounds, pack.rounds);
});

test("appendRoundsFrom copies every round of the other pack", () => {
  const target = loadFixture();
  const source = loadFixture();
  const before = target.roundCount;

  const appended = target.appendRoundsFrom(source);
  assert.equal(appended, source.roundCount);
  assert.equal(target.roundCount, before + source.roundCount);
  assert.deepEqual(target.rounds.slice(before), source.rounds);

  const reparsed = TrainingPackFile.fromBytes(target.toBytes(), bindings);
  assert.equal(reparsed.roundCount, before + source.roundCount);
});

test("creates a pack from scratch that survives a byte round trip", () => {
  const pack = TrainingPackFile.createWithBindings(bindings, {
    name: "Fresh Pack",
    creator_name: "tester",
    difficulty: "D_Medium",
    training_type: "Training_Aerial",
    map_name: "Park_P",
    rounds: [{ time_limit: 8, serialized_archetypes: ["Archetype:Fresh"] }],
  });

  assert.equal(pack.name, "Fresh Pack");
  assert.equal(pack.roundCount, 1);

  const reparsed = TrainingPackFile.fromBytes(pack.toBytes(), bindings);
  assert.equal(reparsed.name, "Fresh Pack");
  assert.equal(reparsed.creatorName, "tester");
  assert.equal(reparsed.difficulty, "D_Medium");
  assert.equal(reparsed.trainingType, "Training_Aerial");
  assert.equal(reparsed.mapName, "Park_P");
  assert.deepEqual(reparsed.rounds, [
    { time_limit: 8, serialized_archetypes: ["Archetype:Fresh"] },
  ]);
});

test("defaultTrainingPack returns independent copies", () => {
  const a = defaultTrainingPack();
  const b = defaultTrainingPack();
  a.tags.push(1);
  a.rounds.push({ time_limit: 1, serialized_archetypes: [] });
  assert.deepEqual(b.tags, []);
  assert.deepEqual(b.rounds, []);
});

test("losslessJson restores through fromLosslessJson", () => {
  const pack = loadFixture();
  pack.setName("Snapshot Name");
  const restored = TrainingPackFile.fromLosslessJson(pack.losslessJson, bindings);
  assert.equal(restored.name, "Snapshot Name");
  assert.deepEqual(Buffer.from(restored.toBytes()), Buffer.from(pack.toBytes()));
});

test("invalid input surfaces as an Error", () => {
  assert.throws(
    () => TrainingPackFile.fromBytes(new Uint8Array([1, 2, 3]), bindings),
    (error: unknown) => error instanceof Error && /training pack/i.test(error.message),
  );
});
