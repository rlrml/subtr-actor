import test from "node:test";
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";

import { ensureWasmPackageFresh } from "../../scripts/ensure-wasm-package.mjs";

const FIXTURE_LOAD_TIMEOUT_MS = 180_000;
const SMOKE_FIXTURES = new Set([
  "old-ballchasing-midfield-car.replay",
  "replay-format-2018-03-15-v868-20-net5-modern-vectors-legacy-rotation.replay",
  "replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay",
]);

interface FixtureLoadResult {
  fixture: string;
  frameCount: number;
  players: number;
  statsFrames: number;
  progressStages: string[];
}

interface ChildResult {
  code: number | null;
  signal: NodeJS.Signals | null;
  stdout: string;
  stderr: string;
  timedOut: boolean;
}

async function replayFormatFixtureNames(): Promise<string[]> {
  const repoRoot = path.resolve(import.meta.dirname, "../../..");
  const docs = await readFile(path.join(repoRoot, "docs/replay-format-evolution.md"), "utf8");
  const fixtures = [...docs.matchAll(/\| `([^`]+\.replay)` \|/g)].map((match) => match[1]!);
  const uniqueFixtures = [...new Set(fixtures)];
  const mode = process.env.SUBTR_ACTOR_REPLAY_FIXTURE_MODE ?? "full";
  if (mode === "full") {
    return uniqueFixtures;
  }
  if (mode !== "smoke") {
    throw new Error(`unknown SUBTR_ACTOR_REPLAY_FIXTURE_MODE: ${mode}`);
  }

  const smokeFixtures = uniqueFixtures.filter((fixture) =>
    SMOKE_FIXTURES.has(fixture)
  );
  assert.equal(
    smokeFixtures.length,
    SMOKE_FIXTURES.size,
    "expected smoke fixtures to be present in replay-format docs table",
  );
  return smokeFixtures;
}

function fixtureConcurrency(): number {
  const requested = Number.parseInt(
    process.env.SUBTR_ACTOR_REPLAY_FIXTURE_CONCURRENCY ?? "1",
    10,
  );
  return Number.isFinite(requested) && requested > 0 ? requested : 1;
}

async function loadFixtures(
  fixtures: string[],
): Promise<{ failures: string[]; loaded: FixtureLoadResult[] }> {
  const failures: string[] = [];
  const loaded: FixtureLoadResult[] = [];
  let nextFixtureIndex = 0;

  const workerCount = Math.min(fixtureConcurrency(), fixtures.length);
  await Promise.all(
    Array.from({ length: workerCount }, async () => {
      for (;;) {
        const fixture = fixtures[nextFixtureIndex];
        nextFixtureIndex += 1;
        if (!fixture) {
          return;
        }

        const result = await runFixtureLoadChild(fixture);
        if (result.timedOut) {
          failures.push(`${fixture}: timed out after ${FIXTURE_LOAD_TIMEOUT_MS}ms`);
          continue;
        }
        if (result.code !== 0) {
          failures.push(
            `${fixture}: exited ${result.code}${result.signal ? ` (${result.signal})` : ""}\n${result.stderr.trim()}`,
          );
          continue;
        }

        const jsonLine = result.stdout.trim().split(/\r?\n/).at(-1) ?? "";
        const parsed = JSON.parse(jsonLine) as FixtureLoadResult;
        assert.equal(parsed.fixture, fixture);
        assert.ok(parsed.frameCount > 0, `${fixture} should expose replay frames`);
        assert.ok(parsed.players > 0, `${fixture} should expose players`);
        assert.ok(parsed.statsFrames > 0, `${fixture} should expose stats frames`);
        assert.ok(
          parsed.progressStages.includes("processing"),
          `${fixture} should report processing progress`,
        );
        loaded.push(parsed);
      }
    }),
  );

  return { failures, loaded };
}

function runFixtureLoadChild(fixture: string): Promise<ChildResult> {
  return new Promise((resolve, reject) => {
    const child = spawn(
      process.execPath,
      [
        "--import",
        "tsx",
        path.join(import.meta.dirname, "replayFormatFixtureLoadChild.test-helper.ts"),
        fixture,
      ],
      {
        cwd: path.resolve(import.meta.dirname, ".."),
        env: {
          ...process.env,
          TSX_TSCONFIG_PATH: path.resolve(import.meta.dirname, "../tsconfig.test.json"),
        },
        stdio: ["ignore", "pipe", "pipe"],
      },
    );

    let stdout = "";
    let stderr = "";
    let settled = false;
    const timeout = setTimeout(() => {
      settled = true;
      child.kill("SIGTERM");
      resolve({
        code: null,
        signal: "SIGTERM",
        stdout,
        stderr,
        timedOut: true,
      });
    }, FIXTURE_LOAD_TIMEOUT_MS);

    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });
    child.on("error", (error) => {
      clearTimeout(timeout);
      if (!settled) {
        reject(error);
      }
    });
    child.on("exit", (code, signal) => {
      clearTimeout(timeout);
      if (settled) {
        return;
      }
      settled = true;
      resolve({
        code,
        signal,
        stdout,
        stderr,
        timedOut: false,
      });
    });
  });
}

test(
  "all replay-format table fixtures load through the TypeScript stats-player path",
  { timeout: 1_500_000 },
  async () => {
    await ensureWasmPackageFresh({ log: () => {} });
    const fixtures = await replayFormatFixtureNames();
    assert.ok(fixtures.length > 0, "expected replay fixtures in docs table");

    const { failures, loaded } = await loadFixtures(fixtures);

    assert.deepEqual(
      failures,
      [],
      `expected all replay-format fixtures to load; loaded ${loaded
        .map((result) => result.fixture)
        .join(", ")}`,
    );
  },
);
