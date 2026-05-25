import { readFile } from "node:fs/promises";
import path from "node:path";

import init, {
  get_replay_frames_data_json_with_progress,
  get_stats_timeline_json_parts,
} from "@colonelpanic8/subtr-actor";
import { normalizeReplayDataAsync } from "subtr-actor-player";
import type { RawReplayFramesData } from "subtr-actor-player";
import { applyStatsTimelineEventDerivedStats } from "./replayLoader.ts";
import type { StatsTimeline } from "./statsTimeline.ts";

const PLAYER_IDENTITY_FIELDS = new Set(["is_team_0", "name", "player_id"]);

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
): StatsTimeline {
  return applyStatsTimelineEventDerivedStats({
    config: parseJsonBuffer(decoder, parts.config),
    replay_meta: parseJsonBuffer(decoder, parts.replayMeta),
    events: parseJsonBuffer(decoder, parts.events),
    frames: parts.frameChunks.flatMap((chunk) =>
      parseJsonBuffer<StatsTimeline["frames"]>(decoder, chunk),
    ),
  });
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

async function main(): Promise<void> {
  const fixture = process.argv[2];
  if (!fixture) {
    throw new Error("missing replay fixture argument");
  }

  const jsRoot = path.resolve(import.meta.dirname, "../..");
  const repoRoot = path.resolve(jsRoot, "..");
  await init({
    module_or_path: await readFile(path.join(jsRoot, "pkg/rl_replay_subtr_actor_bg.wasm")),
  });

  const bytes = new Uint8Array(await readFile(path.join(repoRoot, "assets", fixture)));
  const progressStages: string[] = [];
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
  progressStages.push("stats-timeline");
  const statsTimelineParts = get_stats_timeline_json_parts(bytes, 32 * 1024 * 1024);

  const decoder = new TextDecoder();
  const rawReplayData = JSON.parse(decoder.decode(rawReplayDataBuffer)) as RawReplayFramesData;
  const rawStatsFrames = parseRawStatsTimelineFrames(decoder, statsTimelineParts);
  const rawStatsFrameWithPlayer = rawStatsFrames.find((frame) => {
    const players = frame.players;
    return Array.isArray(players) && players.length > 0;
  });
  if (!rawStatsFrameWithPlayer) {
    throw new Error("expected compacted stats timeline parts to contain at least one player frame");
  }
  assertSkeletalStatsFrame(rawStatsFrameWithPlayer);
  const statsTimeline = parseStatsTimelineParts(decoder, statsTimelineParts);
  const replay = await normalizeReplayDataAsync(rawReplayData);
  const statsFrameWithPlayer = statsTimeline.frames.find((frame) => frame.players.length > 0);
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
      frameCount: replay.frameCount,
      players: replay.players.length,
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
