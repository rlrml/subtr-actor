import { readFile } from "node:fs/promises";
import path from "node:path";

import init, {
  get_legacy_stats_timeline_json,
  get_replay_frames_data_json_with_progress,
  get_stats_timeline_json_parts,
} from "@rlrml/subtr-actor";
import { normalizeReplayDataAsync } from "@rlrml/player";
import type { RawReplayFramesData } from "@rlrml/player";
import {
  createStatsFrameLookup,
  type CompactStatsTimeline,
  type MaterializedStatsTimeline,
  type StatsTimeline,
} from "./statsTimeline.ts";

const PLAYER_IDENTITY_FIELDS = new Set(["is_team_0", "name", "player_id"]);
const FLOAT_EXACTNESS_TOLERANCE = 0;

function envFlag(name: string): boolean {
  return process.env[name] === "1";
}

function logProgress(message: string): void {
  if (envFlag("SUBTR_ACTOR_REPLAY_FIXTURE_PROGRESS")) {
    process.stderr.write(`${message}\n`);
  }
}

function elapsedMs(startedAt: bigint): number {
  return Number((process.hrtime.bigint() - startedAt) / 1_000_000n);
}

function parseJsonBuffer<T>(decoder: TextDecoder, buffer: Uint8Array): T {
  return JSON.parse(decoder.decode(buffer)) as T;
}

function parseRawStatsTimelineFrames(
  decoder: TextDecoder,
  parts: ReturnType<typeof get_stats_timeline_json_parts>,
): Array<Record<string, unknown>> {
  return parts.frameChunks.flatMap((chunk) =>
    parseJsonBuffer<Array<Record<string, unknown>>>(decoder, chunk),
  );
}

function parseStatsTimelineParts(
  decoder: TextDecoder,
  parts: ReturnType<typeof get_stats_timeline_json_parts>,
): { statsTimeline: StatsTimeline; frames: MaterializedStatsTimeline["frames"] } {
  const statsTimeline = {
    config: parseJsonBuffer<StatsTimeline["config"]>(decoder, parts.config),
    replay_meta: parseJsonBuffer<StatsTimeline["replay_meta"]>(decoder, parts.replayMeta),
    events: parseJsonBuffer<StatsTimeline["events"]>(decoder, parts.events),
    positioning_summary: parseJsonBuffer<CompactStatsTimeline["positioning_summary"]>(
      decoder,
      parts.positioningSummary,
    ),
    frames: parts.frameChunks.flatMap((chunk) =>
      parseJsonBuffer<StatsTimeline["frames"]>(decoder, chunk),
    ),
    accumulation_tracks: parseJsonBuffer<CompactStatsTimeline["accumulation_tracks"]>(
      decoder,
      parts.accumulationTracks,
    ),
  } satisfies StatsTimeline;
  const statsFrameLookup = createStatsFrameLookup(statsTimeline, undefined, {
    materializationChunkSize: Math.max(1, statsTimeline.frames.length),
  });
  return {
    statsTimeline,
    frames: statsTimeline.frames.map((frame) => {
      const hydratedFrame = statsFrameLookup.get(frame.frame_number);
      if (!hydratedFrame) {
        throw new Error(`missing hydrated stats frame ${frame.frame_number}`);
      }
      return hydratedFrame;
    }),
  };
}

function assertSkeletalStatsFrame(frame: Record<string, unknown>): void {
  const teamZero = frame.team_zero;
  const teamOne = frame.team_one;
  if (
    !teamZero ||
    typeof teamZero !== "object" ||
    Array.isArray(teamZero) ||
    Object.keys(teamZero).length !== 0 ||
    !teamOne ||
    typeof teamOne !== "object" ||
    Array.isArray(teamOne) ||
    Object.keys(teamOne).length !== 0
  ) {
    throw new Error("expected compacted stats timeline frame to omit team stat modules");
  }

  const players = frame.players;
  if (!Array.isArray(players)) {
    throw new Error("expected compacted stats timeline frame to expose player identities");
  }
  const player = players.find(
    (entry): entry is Record<string, unknown> =>
      !!entry && typeof entry === "object" && !Array.isArray(entry),
  );
  if (!player) {
    throw new Error("expected compacted stats timeline frame to contain at least one player");
  }

  const playerFields = Object.keys(player);
  if (
    playerFields.length !== PLAYER_IDENTITY_FIELDS.size ||
    playerFields.some((field) => !PLAYER_IDENTITY_FIELDS.has(field))
  ) {
    throw new Error(
      `expected compacted stats timeline player to only serialize identity fields, found ${playerFields.join(",")}`,
    );
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === "object" && !Array.isArray(value);
}

function previewValue(value: unknown): string {
  const serialized = JSON.stringify(value);
  if (serialized == null) {
    return String(value);
  }
  return serialized.length > 240 ? `${serialized.slice(0, 237)}...` : serialized;
}

function numbersMatchLegacyStats(left: number, right: number): boolean {
  return (
    Object.is(left, right) ||
    Math.abs(left - right) <= FLOAT_EXACTNESS_TOLERANCE ||
    (Number.isFinite(left) && Number.isFinite(right) && Math.fround(left) === Math.fround(right))
  );
}

function findFirstMismatch(left: unknown, right: unknown, pathLabel = "$"): string | null {
  if (Object.is(left, right)) {
    return null;
  }

  if (typeof left === "number" && typeof right === "number") {
    if (numbersMatchLegacyStats(left, right)) {
      return null;
    }
    return `${pathLabel}: expected ${left}, got ${right}`;
  }

  if (Array.isArray(left) || Array.isArray(right)) {
    if (!Array.isArray(left) || !Array.isArray(right)) {
      return `${pathLabel}: expected ${previewValue(left)}, got ${previewValue(right)}`;
    }
    if (left.length !== right.length) {
      const missing = left.filter((entry) => !right.includes(entry));
      const extra = right.filter((entry) => !left.includes(entry));
      return `${pathLabel}: expected array length ${left.length}, got ${right.length}; missing=${previewValue(missing)}, extra=${previewValue(extra)}`;
    }
    for (let index = 0; index < left.length; index += 1) {
      const mismatch = findFirstMismatch(left[index], right[index], `${pathLabel}[${index}]`);
      if (mismatch) {
        return mismatch;
      }
    }
    return null;
  }

  if (isRecord(left) || isRecord(right)) {
    if (!isRecord(left) || !isRecord(right)) {
      return `${pathLabel}: expected ${previewValue(left)}, got ${previewValue(right)}`;
    }
    const leftKeys = Object.keys(left).sort();
    const rightKeys = Object.keys(right).sort();
    const keyMismatch = findFirstMismatch(leftKeys, rightKeys, `${pathLabel}{keys}`);
    if (keyMismatch) {
      return keyMismatch;
    }
    for (const key of leftKeys) {
      const mismatch = findFirstMismatch(left[key], right[key], `${pathLabel}.${key}`);
      if (mismatch) {
        return mismatch;
      }
    }
    return null;
  }

  return `${pathLabel}: expected ${previewValue(left)}, got ${previewValue(right)}`;
}

function assertHydratedStatsTimelineMatchesLegacy(
  decoder: TextDecoder,
  bytes: Uint8Array,
  frames: MaterializedStatsTimeline["frames"],
): void {
  const legacyTimeline = parseJsonBuffer<MaterializedStatsTimeline>(
    decoder,
    get_legacy_stats_timeline_json(bytes),
  );
  const mismatch = findFirstMismatch(legacyTimeline.frames, frames, "$.frames");
  if (mismatch) {
    throw new Error(
      `event-derived stats timeline did not match legacy serialized frames: ${mismatch}`,
    );
  }
}

async function main(): Promise<void> {
  const fixture = process.argv[2];
  if (!fixture) {
    throw new Error("missing replay fixture argument");
  }
  const compareLegacyStatsTimeline = envFlag("SUBTR_ACTOR_COMPARE_LEGACY_STATS_TIMELINE");
  const statsTimelineOnly = envFlag("SUBTR_ACTOR_REPLAY_FIXTURE_STATS_TIMELINE_ONLY");

  const jsRoot = path.resolve(import.meta.dirname, "../..");
  const repoRoot = path.resolve(jsRoot, "..");
  await init({
    module_or_path: await readFile(path.join(jsRoot, "pkg/rl_replay_subtr_actor_bg.wasm")),
  });

  const bytes = new Uint8Array(await readFile(path.join(repoRoot, "assets", fixture)));
  const progressStages: string[] = [];
  let rawReplayData: RawReplayFramesData | null = null;
  let replayFrameCount: number | null = null;
  let replayPlayerCount: number | null = null;
  const decoder = new TextDecoder();
  if (!statsTimelineOnly) {
    const replayDataStartedAt = process.hrtime.bigint();
    logProgress(`${fixture}: loading replay frames`);
    const rawReplayDataBuffer = get_replay_frames_data_json_with_progress(
      bytes,
      (progress: unknown) => {
        const stage =
          progress instanceof Map
            ? progress.get("stage")
            : progress && typeof progress === "object" && "stage" in progress
              ? progress.stage
              : null;
        if (typeof stage === "string") {
          progressStages.push(stage);
        }
      },
      500,
    );
    logProgress(`${fixture}: replay frames loaded in ${elapsedMs(replayDataStartedAt)}ms`);
    rawReplayData = JSON.parse(decoder.decode(rawReplayDataBuffer)) as RawReplayFramesData;
  }
  progressStages.push("stats-timeline");
  const statsTimelineStartedAt = process.hrtime.bigint();
  logProgress(`${fixture}: loading compact stats timeline`);
  const statsTimelineParts = get_stats_timeline_json_parts(bytes, 32 * 1024 * 1024);
  logProgress(
    `${fixture}: compact stats timeline loaded in ${elapsedMs(statsTimelineStartedAt)}ms`,
  );

  const rawStatsFrames = parseRawStatsTimelineFrames(decoder, statsTimelineParts);
  const rawStatsFrameWithPlayer = rawStatsFrames.find((frame) => {
    const players = frame.players;
    return Array.isArray(players) && players.length > 0;
  });
  if (!rawStatsFrameWithPlayer) {
    throw new Error("expected compacted stats timeline parts to contain at least one player frame");
  }
  assertSkeletalStatsFrame(rawStatsFrameWithPlayer);
  const { statsTimeline, frames: hydratedStatsFrames } = parseStatsTimelineParts(
    decoder,
    statsTimelineParts,
  );
  if (compareLegacyStatsTimeline) {
    const legacyStartedAt = process.hrtime.bigint();
    logProgress(`${fixture}: comparing hydrated compact stats timeline with legacy full timeline`);
    assertHydratedStatsTimelineMatchesLegacy(decoder, bytes, hydratedStatsFrames);
    logProgress(`${fixture}: legacy stats comparison passed in ${elapsedMs(legacyStartedAt)}ms`);
  }
  if (rawReplayData) {
    const normalizeStartedAt = process.hrtime.bigint();
    logProgress(`${fixture}: normalizing replay data`);
    const replay = await normalizeReplayDataAsync(rawReplayData);
    logProgress(`${fixture}: replay data normalized in ${elapsedMs(normalizeStartedAt)}ms`);
    replayFrameCount = replay.frameCount;
    replayPlayerCount = replay.players.length;
  }
  const statsFrameWithPlayer = hydratedStatsFrames.find((frame) => frame.players.length > 0);
  const statsPlayer = statsFrameWithPlayer?.players[0];
  if (!statsFrameWithPlayer || !statsPlayer) {
    throw new Error("expected hydrated stats timeline to contain at least one player frame");
  }
  if (
    statsFrameWithPlayer.team_zero.core.goals == null ||
    statsFrameWithPlayer.team_zero.possession.tracked_time == null ||
    statsPlayer.core.goals == null ||
    statsPlayer.speed_flip.count == null ||
    statsPlayer.boost.tracked_time == null ||
    statsPlayer.boost.amount_used == null
  ) {
    throw new Error("expected compacted stats timeline parts to be hydrated before use");
  }

  process.stdout.write(
    `${JSON.stringify({
      fixture,
      frameCount: replayFrameCount ?? 0,
      players: replayPlayerCount ?? 0,
      statsFrames: statsTimeline.frames.length,
      progressStages: [...new Set(progressStages)],
    })}\n`,
  );
}

main().catch((error: unknown) => {
  const message =
    error instanceof Error
      ? `${error.name}: ${error.message}\n${error.stack ?? ""}`
      : String(error);
  process.stderr.write(message);
  process.exit(1);
});
