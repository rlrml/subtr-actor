import test from "node:test";
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";

const DEFAULT_LEGACY_PARITY_FIXTURE = "post-eac-ranked-duel-2026-04-28-a.replay";
const LEGACY_PARITY_TIMEOUT_MS_PER_FIXTURE = Number(
  process.env.SUBTR_ACTOR_LEGACY_STATS_PARITY_TIMEOUT_MS ?? 300_000,
);
const WIDE_REPLAY_FORMAT_PARITY = process.env.SUBTR_ACTOR_WIDE_LEGACY_STATS_PARITY === "1";
const EXPLICIT_PARITY_FIXTURES = process.env.SUBTR_ACTOR_LEGACY_STATS_PARITY_FIXTURES?.split(
  /[,\s]+/,
)
  .map((fixture) => fixture.trim())
  .filter(Boolean);

async function replayFormatFixtureNames(): Promise<string[]> {
  const docs = await readFile(
    path.resolve(import.meta.dirname, "../../..", "docs/replay-format-evolution.md"),
    "utf8",
  );
  return docs
    .split(/\r?\n/)
    .map((line) => line.match(/^\| `([^`]+\.replay)` \|/)?.[1])
    .filter((fixture): fixture is string => !!fixture);
}

async function runLegacyParityFixture(
  fixture: string,
): Promise<{ fixture: string; statsFrames: number }> {
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
        SUBTR_ACTOR_COMPARE_LEGACY_STATS_TIMELINE: "1",
        SUBTR_ACTOR_REPLAY_FIXTURE_STATS_TIMELINE_ONLY: "1",
        TSX_TSCONFIG_PATH: path.resolve(import.meta.dirname, "../tsconfig.test.json"),
      },
      stdio: ["ignore", "pipe", "pipe"],
    },
  );

  let stdout = "";
  let stderr = "";
  const result = await new Promise<{
    code: number | null;
    signal: NodeJS.Signals | null;
    timedOut: boolean;
  }>((resolve, reject) => {
    const timeout = setTimeout(() => {
      child.kill("SIGTERM");
      resolve({ code: null, signal: "SIGTERM", timedOut: true });
    }, LEGACY_PARITY_TIMEOUT_MS_PER_FIXTURE);

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
      reject(error);
    });
    child.on("exit", (code, signal) => {
      clearTimeout(timeout);
      resolve({ code, signal, timedOut: false });
    });
  });

  assert.equal(result.timedOut, false, `${fixture} timed out`);
  assert.equal(
    result.code,
    0,
    `${fixture} exited ${result.code}${result.signal ? ` (${result.signal})` : ""}\n${stderr.trim()}`,
  );
  const jsonLine = stdout.trim().split(/\r?\n/).at(-1) ?? "";
  const parsed = JSON.parse(jsonLine) as { fixture: string; statsFrames: number };
  assert.equal(parsed.fixture, fixture);
  return parsed;
}

test(
  WIDE_REPLAY_FORMAT_PARITY
    ? "event-derived stats timeline frames match legacy full timelines across replay-format fixtures"
    : "event-derived stats timeline frames match the legacy serialized full timeline",
  {
    timeout: LEGACY_PARITY_TIMEOUT_MS_PER_FIXTURE * (WIDE_REPLAY_FORMAT_PARITY ? 20 : 1) + 10_000,
  },
  async () => {
    const fixtures =
      EXPLICIT_PARITY_FIXTURES ??
      (WIDE_REPLAY_FORMAT_PARITY
        ? await replayFormatFixtureNames()
        : [DEFAULT_LEGACY_PARITY_FIXTURE]);
    assert.ok(fixtures.length > 0, "expected at least one stats timeline parity fixture");
    for (const fixture of fixtures) {
      process.stderr.write(`checking ${fixture}\n`);
      const parsed = await runLegacyParityFixture(fixture);
      assert.equal(parsed.fixture, fixture);
      assert.ok(parsed.statsFrames > 0);
    }
    if (WIDE_REPLAY_FORMAT_PARITY && !EXPLICIT_PARITY_FIXTURES) {
      assert.ok(fixtures.length >= 10, "expected replay-format docs to list wide fixture coverage");
    }
  },
);
