import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import initSubtr from "@rlrml/subtr-actor";
import { loadReplay } from "../src/adapter/wasm.js";
import { SubtrActorPlayer, type MotionKeyframe } from "../src/adapter/SubtrActorPlayer.js";

type Vec3 = { x: number; y: number; z: number };

interface BinaryTimeline {
  times: Float32Array;
  positions: Float32Array;
  velocities: Float32Array;
  rotations: Float32Array;
  angularVelocities: Float32Array;
  sleeping: Uint8Array;
}

interface DecodedRlrf {
  version: number;
  header: Record<string, unknown>;
  ballTimeline: BinaryTimeline;
  playerTimelines: Record<string, BinaryTimeline>;
}

interface TimeAnchor {
  frame: number;
  ballcamTime: number;
  adapterTime: number;
}

interface ReplayGap {
  beforeFrame: number;
  afterFrame: number;
  beforeTime: number;
  afterTime: number;
  duration: number;
}

interface BallcamCompaction {
  gaps: ReplayGap[];
  prematchEndTime: number | null;
  rawDuration: number;
  compactedDuration: number;
}

const DEFAULT_REPLAY_ID = "6edf95bb-f39c-45e4-ac1a-4bbbb871d574";
const SCRIPT_DIR = new URL(".", import.meta.url);
const VIEWER_DIR = new URL("..", SCRIPT_DIR);
const REPO_ROOT = new URL("../..", VIEWER_DIR);
const DEFAULT_RLRF = fileURLToPath(
  new URL(`.cache/ballcam/${DEFAULT_REPLAY_ID}.rlrf`, REPO_ROOT),
);
const DEFAULT_REPLAY = fileURLToPath(
  new URL(".cache/ballcam/5358ED3A11F16434F5C4C298BCD7229C.replay", REPO_ROOT),
);
const WASM_PATH = fileURLToPath(
  new URL("./node_modules/@rlrml/subtr-actor/rl_replay_subtr_actor_bg.wasm", VIEWER_DIR),
);

const args = parseArgs(process.argv.slice(2));
const rlrfPath = args.rlrf ?? DEFAULT_RLRF;
const replayPath = args.replay ?? DEFAULT_REPLAY;

await initSubtr(readFileSync(WASM_PATH));

const rlrf = decodeRlrf(readFileSync(rlrfPath));
const replayBytes = new Uint8Array(readFileSync(replayPath));
const { raw } = await loadReplay(replayBytes);
const compaction = buildBallcamCompaction(raw as never);

console.log("== inputs ==");
console.log(`  rlrf:   ${rlrfPath}`);
console.log(`  replay: ${replayPath}`);

console.log("\n== ballcam rlrf ==");
const metadata = objectRecord(rlrf.header.metadata);
console.log(`  version: ${rlrf.version}`);
console.log(`  frameworkVersion: ${metadata.frameworkVersion ?? "(missing)"}`);
console.log(`  duration: ${fmtNumber(metadata.duration)}s`);
console.log(`  frameCount: ${metadata.frameCount ?? "(missing)"}`);
console.log(`  players: ${Object.keys(rlrf.playerTimelines).join(", ")}`);
printTimelineSummary("ball", rlrf.ballTimeline);
for (const [name, timeline] of Object.entries(rlrf.playerTimelines)) {
  printTimelineSummary(name, timeline);
}

console.log("\n== inferred ballcam compaction from subtr-actor raw ==");
console.log(`  rawDuration: ${compaction.rawDuration.toFixed(6)}s`);
console.log(`  compactedDuration: ${compaction.compactedDuration.toFixed(6)}s`);
console.log(
  `  prematchEndTime: ${
    compaction.prematchEndTime == null ? "(none)" : `${compaction.prematchEndTime.toFixed(6)}s`
  }`,
);
console.log(
  `  gaps: ${
    compaction.gaps.length === 0
      ? "(none)"
      : compaction.gaps
          .map(
            (gap) =>
              `f${gap.beforeFrame}->f${gap.afterFrame} ${gap.duration.toFixed(6)}s`,
          )
          .join(", ")
  }`,
);

const variants = [
  {
    label: "adapter default",
    options: {},
    compactByAdapter: false,
  },
  {
    label: "adapter timelineCompaction",
    options: { timelineCompaction: true },
    compactByAdapter: true,
  },
  {
    label: "adapter raw/no-filter",
    options: { motionSmoothing: false, disableFrameFiltering: true },
    compactByAdapter: false,
  },
] as const;

for (const variant of variants) {
  const player = new SubtrActorPlayer(raw as never, variant.options);
  const timelines = player.getTimelines();
  const anchors = buildTimeAnchors(rlrf.header, player.frameTimes);
  const timeMapper = createSparseTimeMapper(anchors, player.duration);

  console.log(`\n== ${variant.label} =`);
  console.log(`  duration: ${player.duration.toFixed(6)}s`);
  console.log(`  rawStartTime: ${player.rawStartTime.toFixed(6)}s`);
  console.log(`  frameTimes: ${player.frameTimes.length}`);
  console.log(`  players: ${player.playerList.map((p) => p.name).join(", ")}`);
  printMotionSummary("ball", timelines.ballTimeline);
  for (const [name, timeline] of Object.entries(timelines.playerTimelines)) {
    printMotionSummary(name, timeline);
  }

  console.log("\n  -- sampled position error vs rlrf --");
  printComparison("ball direct", compareTimelines(rlrf.ballTimeline, timelines.ballTimeline));
  if (!variant.compactByAdapter) {
    printComparison(
      "ball compacted",
      compareTimelines(
        rlrf.ballTimeline,
        compactMotionTimeline(timelines.ballTimeline, compaction),
      ),
    );
  }
  printComparison(
    "ball sparse-remap",
    compareTimelines(rlrf.ballTimeline, timelines.ballTimeline, timeMapper),
  );
  for (const [name, ballcamTimeline] of Object.entries(rlrf.playerTimelines)) {
    const adapterTimeline = timelines.playerTimelines[name];
    if (!adapterTimeline) {
      console.log(`    ${name}: missing from adapter`);
      continue;
    }
    printComparison(`${name} direct`, compareTimelines(ballcamTimeline, adapterTimeline));
    if (!variant.compactByAdapter) {
      printComparison(
        `${name} compacted`,
        compareTimelines(ballcamTimeline, compactMotionTimeline(adapterTimeline, compaction)),
      );
    }
    printComparison(
      `${name} sparse-remap`,
      compareTimelines(ballcamTimeline, adapterTimeline, timeMapper),
    );
  }

  console.log("\n  -- gameEventTimeline frame/time anchors --");
  if (anchors.length === 0) {
    console.log("    no comparable anchors found");
  } else {
    console.log(`    anchors: ${anchors.length}`);
    for (const anchor of anchors.slice(0, 14)) {
      const removed = anchor.adapterTime - anchor.ballcamTime;
      console.log(
        `    frame ${anchor.frame.toString().padStart(5)}  ballcam=${anchor.ballcamTime
          .toFixed(6)
          .padStart(11)}  adapter=${anchor.adapterTime
          .toFixed(6)
          .padStart(11)}  removed=${removed.toFixed(6).padStart(10)}`,
      );
    }
    const last = anchors[anchors.length - 1]!;
    console.log(
      `    last frame ${last.frame}: ballcam=${last.ballcamTime.toFixed(
        6,
      )} adapter=${last.adapterTime.toFixed(6)} removed=${(
        last.adapterTime - last.ballcamTime
      ).toFixed(6)}`,
    );
  }
}

function parseArgs(rawArgs: string[]): Record<string, string | undefined> {
  const parsed: Record<string, string | undefined> = {};
  for (let index = 0; index < rawArgs.length; index += 1) {
    const arg = rawArgs[index]!;
    if (arg === "--rlrf") parsed.rlrf = rawArgs[++index];
    else if (arg === "--replay") parsed.replay = rawArgs[++index];
    else if (arg === "--help" || arg === "-h") {
      console.log(
        [
          "Usage: npx tsx scripts/compare-ballcam-rlrf.mts [--rlrf path] [--replay path]",
          "",
          "Defaults compare the downloaded ballcam.tv fixture in .cache/ballcam/.",
        ].join("\n"),
      );
      process.exit(0);
    }
  }
  return parsed;
}

function decodeRlrf(bytes: Buffer): DecodedRlrf {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const u8 = new Uint8Array(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  let offset = 0;

  const magic = view.getUint32(offset, true);
  offset += 4;
  if (magic !== 0x524c5246) {
    throw new Error(`Invalid RLRF magic 0x${magic.toString(16)}`);
  }

  const version = view.getUint32(offset, true);
  offset += 4;

  const headerLength = view.getUint32(offset, true);
  offset += 4;
  const header = JSON.parse(new TextDecoder().decode(u8.subarray(offset, offset + headerLength)));
  offset += headerLength;

  const [ballTimeline, afterBall] = readTimeline(view, u8, offset);
  offset = afterBall;

  const playerCount = view.getUint32(offset, true);
  offset += 4;
  const playerTimelines: Record<string, BinaryTimeline> = {};
  for (let index = 0; index < playerCount; index += 1) {
    const [name, afterName] = readString(view, u8, offset);
    offset = afterName;
    const [timeline, afterTimeline] = readPlayerTimeline(view, u8, offset);
    offset = afterTimeline;
    playerTimelines[name] = timeline;
  }

  return { version, header, ballTimeline, playerTimelines };
}

function readString(
  view: DataView,
  u8: Uint8Array,
  offset: number,
): [value: string, offset: number] {
  const length = view.getUint32(offset, true);
  offset += 4;
  const value = new TextDecoder().decode(u8.subarray(offset, offset + length));
  return [value, offset + length];
}

function readTimeline(
  view: DataView,
  u8: Uint8Array,
  offset: number,
): [timeline: BinaryTimeline, offset: number] {
  const count = view.getUint32(offset, true);
  offset += 4;
  const [times, afterTimes] = readFloat32Array(view, offset, count);
  offset = afterTimes;
  const [positions, afterPositions] = readFloat32Array(view, offset, count * 3);
  offset = afterPositions;
  const [velocities, afterVelocities] = readFloat32Array(view, offset, count * 3);
  offset = afterVelocities;
  const [rotations, afterRotations] = readFloat32Array(view, offset, count * 4);
  offset = afterRotations;
  const [angularVelocities, afterAngularVelocities] = readFloat32Array(view, offset, count * 3);
  offset = afterAngularVelocities;
  const sleeping = new Uint8Array(count);
  sleeping.set(u8.subarray(offset, offset + count));
  offset += count;
  return [{ times, positions, velocities, rotations, angularVelocities, sleeping }, offset];
}

function readPlayerTimeline(
  view: DataView,
  u8: Uint8Array,
  offset: number,
): [timeline: BinaryTimeline, offset: number] {
  const [timeline, afterMotion] = readTimeline(view, u8, offset);
  offset = afterMotion;

  const count = timeline.times.length;
  offset += count * 4; // steer
  offset += count * 4; // throttle
  offset += count; // handbrake
  offset += count; // isDriving
  return [timeline, offset];
}

function readFloat32Array(
  view: DataView,
  offset: number,
  count: number,
): [values: Float32Array, offset: number] {
  const values = new Float32Array(count);
  for (let index = 0; index < count; index += 1) {
    values[index] = view.getFloat32(offset + index * 4, true);
  }
  return [values, offset + count * 4];
}

function printTimelineSummary(label: string, timeline: BinaryTimeline): void {
  const count = timeline.times.length;
  const first = count ? timeline.times[0]! : 0;
  const last = count ? timeline.times[count - 1]! : 0;
  const bounds = boundsBinary(timeline);
  console.log(
    `  ${label}: ${count} samples, t=${first.toFixed(6)}..${last.toFixed(
      6,
    )}, ${formatBounds(bounds)}`,
  );
}

function printMotionSummary(label: string, timeline: MotionKeyframe[]): void {
  const count = timeline.length;
  const first = count ? timeline[0]!.time : 0;
  const last = count ? timeline[count - 1]!.time : 0;
  const bounds = boundsMotion(timeline);
  console.log(
    `  ${label}: ${count} samples, t=${first.toFixed(6)}..${last.toFixed(
      6,
    )}, ${formatBounds(bounds)}`,
  );
}

function printComparison(label: string, stats: ReturnType<typeof compareTimelines>): void {
  console.log(
    `    ${label}: n=${stats.count} rms=${stats.rms.toFixed(2)} mean=${stats.mean.toFixed(
      2,
    )} p50=${stats.p50.toFixed(2)} p95=${stats.p95.toFixed(2)} max=${stats.max.toFixed(2)}`,
  );
}

function compareTimelines(
  expected: BinaryTimeline,
  actual: MotionKeyframe[],
  mapExpectedTimeToActualTime: (time: number) => number | null = (time) => time,
): {
  count: number;
  rms: number;
  mean: number;
  p50: number;
  p95: number;
  max: number;
} {
  const errors: number[] = [];
  for (let index = 0; index < expected.times.length; index += 1) {
    const expectedTime = expected.times[index]!;
    const actualTime = mapExpectedTimeToActualTime(expectedTime);
    if (actualTime === null) continue;

    const expectedPosition = getBinaryPosition(expected, index);
    const actualPosition = sampleMotion(actual, actualTime);
    if (!actualPosition) continue;
    errors.push(distance(expectedPosition, actualPosition));
  }

  if (errors.length === 0) {
    return { count: 0, rms: 0, mean: 0, p50: 0, p95: 0, max: 0 };
  }

  errors.sort((a, b) => a - b);
  const sum = errors.reduce((total, value) => total + value, 0);
  const sumSquares = errors.reduce((total, value) => total + value * value, 0);
  return {
    count: errors.length,
    rms: Math.sqrt(sumSquares / errors.length),
    mean: sum / errors.length,
    p50: percentile(errors, 0.5),
    p95: percentile(errors, 0.95),
    max: errors[errors.length - 1]!,
  };
}

function buildTimeAnchors(
  header: Record<string, unknown>,
  adapterFrameTimes: number[],
): TimeAnchor[] {
  const events = Array.isArray(header.gameEventTimeline) ? header.gameEventTimeline : [];
  const anchors: TimeAnchor[] = [];

  for (const event of events) {
    if (!event || typeof event !== "object") continue;
    const record = event as Record<string, unknown>;
    const frame = Number(record.frame);
    const ballcamTime = Number(record.time);
    if (!Number.isInteger(frame) || !Number.isFinite(ballcamTime)) continue;
    const adapterTime = adapterFrameTimes[frame];
    if (adapterTime == null) continue;
    anchors.push({ frame, ballcamTime, adapterTime });
  }

  anchors.sort((left, right) => left.ballcamTime - right.ballcamTime);
  return anchors.filter((anchor, index) => {
    if (index === 0) return true;
    const previous = anchors[index - 1]!;
    return anchor.ballcamTime !== previous.ballcamTime || anchor.adapterTime !== previous.adapterTime;
  });
}

function createSparseTimeMapper(
  anchors: TimeAnchor[],
  adapterDuration: number,
): (ballcamTime: number) => number | null {
  if (anchors.length < 2) return () => null;
  const mapAnchors = [
    { ballcamTime: 0, adapterTime: 0 },
    ...anchors.map(({ ballcamTime, adapterTime }) => ({ ballcamTime, adapterTime })),
  ];
  if (mapAnchors[mapAnchors.length - 1]!.adapterTime < adapterDuration) {
    const last = mapAnchors[mapAnchors.length - 1]!;
    mapAnchors.push({
      ballcamTime: last.ballcamTime + adapterDuration - last.adapterTime,
      adapterTime: adapterDuration,
    });
  }

  return (ballcamTime: number): number | null => {
    if (ballcamTime < mapAnchors[0]!.ballcamTime) return null;
    if (ballcamTime > mapAnchors[mapAnchors.length - 1]!.ballcamTime) return null;
    let lo = 0;
    let hi = mapAnchors.length - 1;
    while (lo < hi) {
      const mid = (lo + hi + 1) >> 1;
      if (mapAnchors[mid]!.ballcamTime <= ballcamTime) lo = mid;
      else hi = mid - 1;
    }
    const left = mapAnchors[lo]!;
    const right = mapAnchors[Math.min(lo + 1, mapAnchors.length - 1)]!;
    if (right.ballcamTime === left.ballcamTime) return left.adapterTime;
    const alpha = (ballcamTime - left.ballcamTime) / (right.ballcamTime - left.ballcamTime);
    return left.adapterTime + (right.adapterTime - left.adapterTime) * alpha;
  };
}

function buildBallcamCompaction(raw: {
  frame_data: { metadata_frames: Array<Record<string, unknown>> };
  goal_events?: Array<Record<string, unknown>>;
}): BallcamCompaction {
  const metadataFrames = raw.frame_data.metadata_frames;
  const startTime = Number(metadataFrames[0]?.time ?? 0);
  const frameTimes = metadataFrames.map((frame) => Number(frame.time) - startTime);
  const rawDuration = frameTimes[frameTimes.length - 1] ?? 0;
  const gaps = detectPostGoalTimeGaps(frameTimes, raw.goal_events ?? []);
  const prematchRawEndTime = detectFirstKickoffGoTime(frameTimes, metadataFrames);
  const prematchEndTime =
    prematchRawEndTime == null ? null : remapGapTime(prematchRawEndTime, gaps);
  const gapDuration = gaps.reduce((total, gap) => total + gap.duration, 0);
  const compactedDuration = rawDuration - gapDuration - (prematchEndTime ?? 0);
  return { gaps, prematchEndTime, rawDuration, compactedDuration };
}

function detectPostGoalTimeGaps(
  frameTimes: number[],
  goalEvents: Array<Record<string, unknown>>,
): ReplayGap[] {
  const gaps: ReplayGap[] = [];
  for (const goal of goalEvents) {
    const goalFrame = Number(goal.frame);
    if (!Number.isInteger(goalFrame) || goalFrame < 0 || goalFrame >= frameTimes.length) continue;
    const goalTime = frameTimes[goalFrame]!;
    for (let frame = goalFrame + 1; frame < frameTimes.length; frame += 1) {
      const beforeTime = frameTimes[frame - 1]!;
      const afterTime = frameTimes[frame]!;
      if (beforeTime - goalTime > 10) break;
      const duration = afterTime - beforeTime;
      if (duration > 0.3) {
        gaps.push({
          beforeFrame: frame - 1,
          afterFrame: frame,
          beforeTime,
          afterTime,
          duration,
        });
        break;
      }
    }
  }
  return gaps;
}

function detectFirstKickoffGoTime(
  frameTimes: number[],
  metadataFrames: Array<Record<string, unknown>>,
): number | null {
  let sawCountdown = false;
  for (let index = 0; index < metadataFrames.length; index += 1) {
    const remaining = Number(metadataFrames[index]?.replicated_game_state_time_remaining);
    if (remaining > 0) sawCountdown = true;
    if (sawCountdown && remaining === 0) return frameTimes[index] ?? null;
  }

  const firstActiveFrame = metadataFrames.findIndex(
    (frame) => Number(frame.replicated_game_state_name) === 54,
  );
  return firstActiveFrame === -1 ? null : (frameTimes[firstActiveFrame] ?? null);
}

function compactMotionTimeline(
  timeline: MotionKeyframe[],
  compaction: BallcamCompaction,
): MotionKeyframe[] {
  const afterGaps = remapGapTimeline(timeline, compaction.gaps);
  if (compaction.prematchEndTime == null) return afterGaps;
  return remapPrematchTimeline(afterGaps, compaction.prematchEndTime);
}

function remapGapTimeline(timeline: MotionKeyframe[], gaps: ReplayGap[]): MotionKeyframe[] {
  if (gaps.length === 0) return timeline;

  const inserted: MotionKeyframe[] = [];
  for (const [gapIndex, gap] of gaps.entries()) {
    const entry = timeline.find((frame) => frame.time >= gap.afterTime);
    if (!entry) continue;
    inserted.push({
      ...entry,
      time: remapGapTime(gap.afterTime, gaps.slice(0, gapIndex + 1)),
    });
  }

  const remapped = timeline
    .filter((frame) => !isInReplayGap(frame.time, gaps))
    .map((frame) => ({ ...frame, time: remapGapTime(frame.time, gaps) }));

  for (const entry of inserted) {
    if (remapped.some((frame) => Math.abs(frame.time - entry.time) < 1e-3)) continue;
    const index = remapped.findIndex((frame) => frame.time > entry.time);
    remapped.splice(index === -1 ? remapped.length : index, 0, entry);
  }

  return remapped;
}

function remapPrematchTimeline(
  timeline: MotionKeyframe[],
  prematchEndTime: number,
): MotionKeyframe[] {
  let lastPrematchFrame: MotionKeyframe | null = null;
  for (const frame of timeline) {
    if (frame.time < prematchEndTime) lastPrematchFrame = frame;
    else break;
  }

  const remapped = timeline
    .filter((frame) => frame.time >= prematchEndTime)
    .map((frame) => ({ ...frame, time: frame.time - prematchEndTime }));

  if (lastPrematchFrame && (remapped.length === 0 || remapped[0]!.time > 1e-3)) {
    remapped.unshift({ ...lastPrematchFrame, time: 0 });
  }

  return remapped;
}

function remapGapTime(time: number, gaps: ReplayGap[]): number {
  let cumulativeRemoved = 0;
  for (const gap of gaps) {
    if (time < gap.beforeTime) break;
    if (time >= gap.afterTime) {
      cumulativeRemoved += gap.duration;
      continue;
    }
    return gap.beforeTime - cumulativeRemoved;
  }
  return time - cumulativeRemoved;
}

function isInReplayGap(time: number, gaps: ReplayGap[]): boolean {
  return gaps.some((gap) => time > gap.beforeTime && time < gap.afterTime);
}

function sampleMotion(timeline: MotionKeyframe[], time: number): Vec3 | null {
  if (timeline.length === 0) return null;
  if (time < timeline[0]!.time || time > timeline[timeline.length - 1]!.time) return null;

  let lo = 0;
  let hi = timeline.length - 1;
  while (lo < hi) {
    const mid = (lo + hi + 1) >> 1;
    if (timeline[mid]!.time <= time) lo = mid;
    else hi = mid - 1;
  }
  const left = timeline[lo]!;
  const right = timeline[Math.min(lo + 1, timeline.length - 1)]!;
  if (right.time === left.time) return left.position;
  const alpha = (time - left.time) / (right.time - left.time);
  return {
    x: lerp(left.position.x, right.position.x, alpha),
    y: lerp(left.position.y, right.position.y, alpha),
    z: lerp(left.position.z, right.position.z, alpha),
  };
}

function boundsBinary(timeline: BinaryTimeline): ReturnType<typeof emptyBounds> {
  const bounds = emptyBounds();
  for (let index = 0; index < timeline.times.length; index += 1) {
    include(bounds, getBinaryPosition(timeline, index));
  }
  return bounds;
}

function boundsMotion(timeline: MotionKeyframe[]): ReturnType<typeof emptyBounds> {
  const bounds = emptyBounds();
  for (const frame of timeline) include(bounds, frame.position);
  return bounds;
}

function getBinaryPosition(timeline: BinaryTimeline, index: number): Vec3 {
  return {
    x: timeline.positions[index * 3]!,
    y: timeline.positions[index * 3 + 1]!,
    z: timeline.positions[index * 3 + 2]!,
  };
}

function emptyBounds() {
  return {
    minX: Number.POSITIVE_INFINITY,
    maxX: Number.NEGATIVE_INFINITY,
    minY: Number.POSITIVE_INFINITY,
    maxY: Number.NEGATIVE_INFINITY,
    minZ: Number.POSITIVE_INFINITY,
    maxZ: Number.NEGATIVE_INFINITY,
  };
}

function include(bounds: ReturnType<typeof emptyBounds>, value: Vec3): void {
  bounds.minX = Math.min(bounds.minX, value.x);
  bounds.maxX = Math.max(bounds.maxX, value.x);
  bounds.minY = Math.min(bounds.minY, value.y);
  bounds.maxY = Math.max(bounds.maxY, value.y);
  bounds.minZ = Math.min(bounds.minZ, value.z);
  bounds.maxZ = Math.max(bounds.maxZ, value.z);
}

function formatBounds(bounds: ReturnType<typeof emptyBounds>): string {
  return `x[${bounds.minX.toFixed(0)},${bounds.maxX.toFixed(0)}] y[${bounds.minY.toFixed(
    0,
  )},${bounds.maxY.toFixed(0)}] z[${bounds.minZ.toFixed(0)},${bounds.maxZ.toFixed(0)}]`;
}

function objectRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" ? (value as Record<string, unknown>) : {};
}

function fmtNumber(value: unknown): string {
  return typeof value === "number" ? value.toFixed(6) : String(value);
}

function distance(left: Vec3, right: Vec3): number {
  const dx = right.x - left.x;
  const dy = right.y - left.y;
  const dz = right.z - left.z;
  return Math.sqrt(dx * dx + dy * dy + dz * dz);
}

function percentile(sortedValues: number[], fraction: number): number {
  const index = Math.min(sortedValues.length - 1, Math.floor(sortedValues.length * fraction));
  return sortedValues[index]!;
}

function lerp(left: number, right: number, alpha: number): number {
  return left + (right - left) * alpha;
}
