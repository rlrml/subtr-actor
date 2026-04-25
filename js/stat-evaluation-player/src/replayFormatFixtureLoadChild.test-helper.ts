import { readFile } from "node:fs/promises";
import path from "node:path";

import init, {
  get_replay_frames_data_json_with_progress,
  get_stats_timeline_json_parts,
} from "@colonelpanic8/subtr-actor";
import { normalizeReplayDataAsync } from "subtr-actor-player";
import type { RawReplayFramesData } from "subtr-actor-player";
import type { StatsTimeline } from "./statsTimeline.ts";

function parseJsonBuffer<T>(decoder: TextDecoder, buffer: Uint8Array): T {
  return JSON.parse(decoder.decode(buffer)) as T;
}

function parseStatsTimelineParts(
  decoder: TextDecoder,
  parts: ReturnType<typeof get_stats_timeline_json_parts>,
): StatsTimeline {
  return {
    config: parseJsonBuffer(decoder, parts.config),
    replay_meta: parseJsonBuffer(decoder, parts.replayMeta),
    events: parseJsonBuffer(decoder, parts.events),
    frames: parts.frameChunks.flatMap((chunk) =>
      parseJsonBuffer<StatsTimeline["frames"]>(decoder, chunk)
    ),
  };
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
      const stage = progress instanceof Map
        ? progress.get("stage")
        : progress &&
          typeof progress === "object" &&
          "stage" in progress
          ? progress.stage
          : null;
      if (typeof stage === "string") {
        progressStages.push(stage);
      }
    },
    500,
  );
  progressStages.push("stats-timeline");
  const statsTimelineParts = get_stats_timeline_json_parts(
    bytes,
    32 * 1024 * 1024,
  );

  const decoder = new TextDecoder();
  const rawReplayData = JSON.parse(
    decoder.decode(rawReplayDataBuffer),
  ) as RawReplayFramesData;
  const statsTimeline = parseStatsTimelineParts(decoder, statsTimelineParts);
  const replay = await normalizeReplayDataAsync(rawReplayData);

  process.stdout.write(`${JSON.stringify({
    fixture,
    frameCount: replay.frameCount,
    players: replay.players.length,
    statsFrames: statsTimeline.frames.length,
    progressStages: [...new Set(progressStages)],
  })}\n`);
}

main().catch((error: unknown) => {
  const message = error instanceof Error
    ? `${error.name}: ${error.message}\n${error.stack ?? ""}`
    : String(error);
  process.stderr.write(message);
  process.exit(1);
});
