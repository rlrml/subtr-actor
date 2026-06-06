import type {
  BallSample,
  CameraSettings,
  PlaybackFrame,
  RawDemolishInfo,
  RawGoalEvent,
  RawPlayerStatEvent,
  PlayerSample,
  RawBallFrame,
  RawPlayerFrame,
  RawPlayerInfo,
  RawReplayFramesData,
  ReplayModel,
  ReplayPlayerTrack,
  ReplayTickMark,
  ReplayTimelineEvent,
  Vec3,
  Quaternion,
} from "./types";
import { getReplayHitboxSpec, inferReplayHitboxKind } from "./hitboxes";
import {
  buildBoostPads,
  buildBoostPadsAsync,
  STANDARD_SOCCAR_BOOST_PAD_COUNT,
} from "./replay-boost-pads";
import { buildPlayerLookup, normalizeReplayTime, playerIdToString } from "./replay-data-helpers";
import { buildReplayTickMarks, replayTickMarkTimelineEvent } from "./replay-tick-marks";

export interface NormalizeReplayDataOptions {
  onProgress?: (progress: number, details: NormalizeReplayProgress) => void;
  progressReportMinDelta?: number;
  progressReportFrameInterval?: number;
}

export interface NormalizeReplayDataAsyncOptions extends NormalizeReplayDataOptions {
  yieldEveryMs?: number;
  yieldToMainThread?: () => Promise<void>;
}

export interface NormalizeReplayProgress {
  progress: number;
  processedFrames: number;
  totalFrames: number;
  processedUnits: number;
  totalUnits: number;
}

interface NormalizeReplayProgressTracker {
  advance(units?: number): boolean;
  advanceFrame(units?: number): boolean;
  finish(): void;
}

interface AsyncNormalizeReplayProgressTracker extends NormalizeReplayProgressTracker {
  yieldToMainThread(): Promise<void>;
}

const DEFAULT_CAMERA_SETTINGS: CameraSettings = {
  distance: 270,
  height: 100,
  pitch: -4,
  fov: 110,
};

const NORMALIZATION_PROGRESS_REPORT_MIN_DELTA = 0.005;
const NORMALIZATION_PROGRESS_REPORT_FRAME_INTERVAL = Number.POSITIVE_INFINITY;
const NORMALIZATION_ASYNC_YIELD_INTERVAL_MS = 16;

function normalizeVector(value: Vec3): Vec3 | null {
  const magnitude = Math.hypot(value.x, value.y, value.z);
  if (magnitude < 0.000001) {
    return null;
  }

  return {
    x: value.x / magnitude,
    y: value.y / magnitude,
    z: value.z / magnitude,
  };
}

function normalizeQuaternion(raw: Quaternion): Quaternion | null {
  const magnitude = Math.hypot(raw.x, raw.y, raw.z, raw.w);
  if (magnitude < 0.000001) {
    return null;
  }

  return {
    x: raw.x / magnitude,
    y: raw.y / magnitude,
    z: raw.z / magnitude,
    w: raw.w / magnitude,
  };
}

function multiplyQuaternions(left: Quaternion, right: Quaternion): Quaternion {
  return {
    w: left.w * right.w - left.x * right.x - left.y * right.y - left.z * right.z,
    x: left.w * right.x + left.x * right.w + left.y * right.z - left.z * right.y,
    y: left.w * right.y - left.x * right.z + left.y * right.w + left.z * right.x,
    z: left.w * right.z + left.x * right.y - left.y * right.x + left.z * right.w,
  };
}

function rotateVectorByQuaternion(vector: Vec3, quaternion: Quaternion): Vec3 {
  const rotated = multiplyQuaternions(
    multiplyQuaternions(quaternion, {
      x: vector.x,
      y: vector.y,
      z: vector.z,
      w: 0,
    }),
    {
      x: -quaternion.x,
      y: -quaternion.y,
      z: -quaternion.z,
      w: quaternion.w,
    },
  );

  return {
    x: rotated.x,
    y: rotated.y,
    z: rotated.z,
  };
}

function parseBallFrame(frame: RawBallFrame): BallSample {
  if (frame === "Empty") {
    return {
      position: null,
      linearVelocity: null,
      angularVelocity: null,
      rotation: null,
    };
  }

  const rigidBody = frame.Data.rigid_body;
  return {
    position: rigidBody.location,
    linearVelocity: rigidBody.linear_velocity ?? null,
    angularVelocity: rigidBody.angular_velocity ?? null,
    rotation: normalizeQuaternion(rigidBody.rotation),
  };
}

function parsePlayerFrame(frame: RawPlayerFrame): PlayerSample {
  if (frame === "Empty") {
    return {
      isPresent: false,
      position: null,
      linearVelocity: null,
      angularVelocity: null,
      rotation: null,
      forward: null,
      up: null,
      boostAmount: 0,
      boostFraction: 0,
      boostActive: false,
      powerslideActive: false,
      jumpActive: false,
      doubleJumpActive: false,
      dodgeActive: false,
    };
  }

  const rigidBody = frame.Data.rigid_body;
  const rotation = normalizeQuaternion(rigidBody.rotation);
  const forward = rotation
    ? normalizeVector(rotateVectorByQuaternion({ x: 1, y: 0, z: 0 }, rotation))
    : null;
  const up = rotation
    ? normalizeVector(rotateVectorByQuaternion({ x: 0, y: 0, z: 1 }, rotation))
    : null;

  return {
    isPresent: true,
    position: rigidBody.location,
    linearVelocity: rigidBody.linear_velocity ?? null,
    angularVelocity: rigidBody.angular_velocity ?? null,
    rotation,
    forward,
    up,
    boostAmount: frame.Data.boost_amount,
    boostFraction: Math.max(0, Math.min(1, frame.Data.boost_amount / 255)),
    boostActive: frame.Data.boost_active,
    powerslideActive: frame.Data.powerslide_active,
    jumpActive: frame.Data.jump_active,
    doubleJumpActive: frame.Data.double_jump_active,
    dodgeActive: frame.Data.dodge_active,
  };
}

function hasPlayerPosition(sample: PlayerSample): boolean {
  return sample.position !== null;
}

function carriedPlayerSample(sample: PlayerSample): PlayerSample {
  return {
    ...sample,
    isPresent: false,
    linearVelocity: null,
    angularVelocity: null,
    boostActive: false,
    powerslideActive: false,
    jumpActive: false,
    doubleJumpActive: false,
    dodgeActive: false,
  };
}

function fillBoundedPlayerSampleGaps(frames: PlayerSample[]): void {
  let lastPositionedFrame: PlayerSample | null = null;
  let gapStart: number | null = null;

  for (let index = 0; index < frames.length; index += 1) {
    const frame = frames[index]!;
    if (hasPlayerPosition(frame)) {
      if (gapStart !== null && lastPositionedFrame) {
        const carriedFrame = carriedPlayerSample(lastPositionedFrame);
        for (let gapIndex = gapStart; gapIndex < index; gapIndex += 1) {
          frames[gapIndex] = carriedFrame;
        }
      }
      lastPositionedFrame = frame;
      gapStart = null;
    } else if (lastPositionedFrame && gapStart === null) {
      gapStart = index;
    }
  }
}

function currentTimeMs(): number {
  return typeof performance === "undefined" ? Date.now() : performance.now();
}

function defaultYieldToMainThread(): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

function getNormalizationTotalUnits(raw: RawReplayFramesData): number {
  const playerInfoCount = raw.meta.team_zero.length + raw.meta.team_one.length;
  const playerFrameCount = raw.frame_data.players.reduce(
    (count, [, playerData]) => count + playerData.frames.length,
    0,
  );
  const boostPadCount = raw.boost_pads?.length ?? STANDARD_SOCCAR_BOOST_PAD_COUNT;
  const boostPadEventCount = raw.boost_pad_events?.length ?? 0;
  const timelineEventCount =
    (raw.goal_events?.length ?? 0) +
    (raw.player_stat_events?.length ?? 0) +
    (raw.demolish_infos?.length ?? 0) +
    (raw.replay_tick_marks?.length ?? 0);

  return [
    Math.max(1, raw.frame_data.metadata_frames.length),
    Math.max(1, playerInfoCount),
    Math.max(1, playerFrameCount),
    Math.max(1, raw.frame_data.ball_data.frames.length),
    Math.max(1, boostPadCount + boostPadEventCount),
    Math.max(1, timelineEventCount),
  ].reduce((sum, count) => sum + count, 0);
}

function getNormalizationTotalFrameUnits(raw: RawReplayFramesData): number {
  const playerFrameCount = raw.frame_data.players.reduce(
    (count, [, playerData]) => count + playerData.frames.length,
    0,
  );

  return [
    Math.max(1, raw.frame_data.metadata_frames.length),
    Math.max(1, playerFrameCount),
    Math.max(1, raw.frame_data.ball_data.frames.length),
  ].reduce((sum, count) => sum + count, 0);
}

function createNormalizationProgressTracker(
  raw: RawReplayFramesData,
  onProgress?: (progress: number, details: NormalizeReplayProgress) => void,
  options: {
    progressReportMinDelta?: number;
    progressReportFrameInterval?: number;
    yieldEveryMs?: number;
  } = {},
): NormalizeReplayProgressTracker {
  const totalUnits = getNormalizationTotalUnits(raw);
  const totalFrameUnits = getNormalizationTotalFrameUnits(raw);
  let completedUnits = 0;
  let completedFrameUnits = 0;
  let lastReportedProgress = -1;
  let lastReportedFrameUnits = -1;
  let lastYieldedAt = currentTimeMs();
  const yieldEveryMs = options.yieldEveryMs ?? Number.POSITIVE_INFINITY;
  const progressReportMinDelta =
    options.progressReportMinDelta ?? NORMALIZATION_PROGRESS_REPORT_MIN_DELTA;
  const progressReportFrameInterval = Math.max(
    1,
    options.progressReportFrameInterval ?? NORMALIZATION_PROGRESS_REPORT_FRAME_INTERVAL,
  );

  const maybeReport = () => {
    if (!onProgress) {
      return false;
    }

    const progress = Math.max(0, Math.min(1, completedUnits / totalUnits));
    if (progress <= lastReportedProgress) {
      return false;
    }

    const frameDelta = completedFrameUnits - lastReportedFrameUnits;
    const reachedFrameInterval = frameDelta >= progressReportFrameInterval;
    if (
      progress >= 1 ||
      progress - lastReportedProgress >= progressReportMinDelta ||
      reachedFrameInterval
    ) {
      lastReportedProgress = progress;
      lastReportedFrameUnits = completedFrameUnits;
      onProgress(progress, {
        progress,
        processedFrames: Math.min(completedFrameUnits, totalFrameUnits),
        totalFrames: totalFrameUnits,
        processedUnits: completedUnits,
        totalUnits,
      });
      return true;
    }

    return false;
  };

  const shouldYield = (force = false) => {
    const now = currentTimeMs();
    if (!force && now - lastYieldedAt < yieldEveryMs) {
      return false;
    }
    lastYieldedAt = now;
    return true;
  };

  maybeReport();

  return {
    advance(units = 1) {
      if (units <= 0) {
        return false;
      }
      completedUnits = Math.min(totalUnits, completedUnits + units);
      const reported = maybeReport();
      return shouldYield(reported);
    },
    advanceFrame(units = 1) {
      if (units <= 0) {
        return false;
      }
      completedFrameUnits = Math.min(totalFrameUnits, completedFrameUnits + units);
      completedUnits = Math.min(totalUnits, completedUnits + units);
      const reported = maybeReport();
      return shouldYield(reported);
    },
    finish() {
      completedUnits = totalUnits;
      completedFrameUnits = totalFrameUnits;
      maybeReport();
    },
  };
}

function createAsyncNormalizationProgressTracker(
  raw: RawReplayFramesData,
  options: NormalizeReplayDataAsyncOptions,
): AsyncNormalizeReplayProgressTracker {
  const progressTracker = createNormalizationProgressTracker(raw, options.onProgress, {
    progressReportMinDelta: options.progressReportMinDelta,
    progressReportFrameInterval: options.progressReportFrameInterval,
    yieldEveryMs: options.yieldEveryMs ?? NORMALIZATION_ASYNC_YIELD_INTERVAL_MS,
  });

  return {
    ...progressTracker,
    yieldToMainThread: options.yieldToMainThread ?? defaultYieldToMainThread,
  };
}

function buildPlaybackFrames(
  raw: RawReplayFramesData,
  progressTracker?: NormalizeReplayProgressTracker,
): PlaybackFrame[] {
  const metadataFrames = raw.frame_data.metadata_frames;
  if (metadataFrames.length === 0) {
    progressTracker?.advanceFrame();
    return [];
  }

  const startTime = metadataFrames[0]?.time ?? 0;
  const frames = new Array<PlaybackFrame>(metadataFrames.length);

  for (let index = 0; index < metadataFrames.length; index += 1) {
    const frame = metadataFrames[index]!;
    frames[index] = {
      time: frame.time - startTime,
      secondsRemaining: frame.seconds_remaining,
      gameState: frame.replicated_game_state_name,
      kickoffCountdown: frame.replicated_game_state_time_remaining,
    };
    progressTracker?.advanceFrame();
  }

  return frames;
}

async function buildPlaybackFramesAsync(
  raw: RawReplayFramesData,
  progressTracker: AsyncNormalizeReplayProgressTracker,
): Promise<PlaybackFrame[]> {
  const metadataFrames = raw.frame_data.metadata_frames;
  if (metadataFrames.length === 0) {
    if (progressTracker.advanceFrame()) {
      await progressTracker.yieldToMainThread();
    }
    return [];
  }

  const startTime = metadataFrames[0]?.time ?? 0;
  const frames = new Array<PlaybackFrame>(metadataFrames.length);

  for (let index = 0; index < metadataFrames.length; index += 1) {
    const frame = metadataFrames[index]!;
    frames[index] = {
      time: frame.time - startTime,
      secondsRemaining: frame.seconds_remaining,
      gameState: frame.replicated_game_state_name,
      kickoffCountdown: frame.replicated_game_state_time_remaining,
    };
    if (progressTracker.advanceFrame()) {
      await progressTracker.yieldToMainThread();
    }
  }

  return frames;
}

function inferTeamSide(
  name: string,
  teamZeroNames: Set<string>,
  teamOneNames: Set<string>,
  firstFrame: RawPlayerFrame | undefined,
): boolean {
  if (teamZeroNames.has(name)) {
    return true;
  }

  if (teamOneNames.has(name)) {
    return false;
  }

  if (firstFrame && firstFrame !== "Empty" && typeof firstFrame.Data.is_team_0 === "boolean") {
    return firstFrame.Data.is_team_0;
  }

  return true;
}

function getStatEntries(stats: RawPlayerInfo["stats"] | undefined): Array<[string, unknown]> {
  if (!stats) {
    return [];
  }

  return Object.entries(stats);
}

function extractNumericSetting(entries: Array<[string, unknown]>, key: string): number | undefined {
  const value = entries.find(([entryKey]) => entryKey === key)?.[1];
  return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

function extractCameraSettings(playerInfo?: RawPlayerInfo): CameraSettings {
  const entries = getStatEntries(playerInfo?.stats);
  return {
    fov: extractNumericSetting(entries, "CameraFOV") ?? DEFAULT_CAMERA_SETTINGS.fov,
    height: extractNumericSetting(entries, "CameraHeight") ?? DEFAULT_CAMERA_SETTINGS.height,
    pitch: extractNumericSetting(entries, "CameraPitch") ?? DEFAULT_CAMERA_SETTINGS.pitch,
    distance: extractNumericSetting(entries, "CameraDistance") ?? DEFAULT_CAMERA_SETTINGS.distance,
    stiffness:
      extractNumericSetting(entries, "CameraStiffness") ?? DEFAULT_CAMERA_SETTINGS.stiffness,
    swivelSpeed:
      extractNumericSetting(entries, "CameraSwivelSpeed") ?? DEFAULT_CAMERA_SETTINGS.swivelSpeed,
    transitionSpeed:
      extractNumericSetting(entries, "CameraTransitionSpeed") ??
      DEFAULT_CAMERA_SETTINGS.transitionSpeed,
  };
}

function indexReplayPlayers(
  raw: RawReplayFramesData,
  progressTracker?: NormalizeReplayProgressTracker,
): {
  byId: Map<string, RawPlayerInfo>;
  byName: Map<string, RawPlayerInfo>;
} {
  const byId = new Map<string, RawPlayerInfo>();
  const byName = new Map<string, RawPlayerInfo>();

  const playerInfos = [...raw.meta.team_zero, ...raw.meta.team_one];
  if (playerInfos.length === 0) {
    progressTracker?.advance();
    return { byId, byName };
  }

  for (const playerInfo of playerInfos) {
    byName.set(playerInfo.name, playerInfo);
    if (playerInfo.remote_id) {
      byId.set(playerIdToString(playerInfo.remote_id), playerInfo);
    }
    progressTracker?.advance();
  }

  return { byId, byName };
}

async function indexReplayPlayersAsync(
  raw: RawReplayFramesData,
  progressTracker: AsyncNormalizeReplayProgressTracker,
): Promise<{
  byId: Map<string, RawPlayerInfo>;
  byName: Map<string, RawPlayerInfo>;
}> {
  const byId = new Map<string, RawPlayerInfo>();
  const byName = new Map<string, RawPlayerInfo>();

  const playerInfos = [...raw.meta.team_zero, ...raw.meta.team_one];
  if (playerInfos.length === 0) {
    if (progressTracker.advance()) {
      await progressTracker.yieldToMainThread();
    }
    return { byId, byName };
  }

  for (const playerInfo of playerInfos) {
    byName.set(playerInfo.name, playerInfo);
    if (playerInfo.remote_id) {
      byId.set(playerIdToString(playerInfo.remote_id), playerInfo);
    }
    if (progressTracker.advance()) {
      await progressTracker.yieldToMainThread();
    }
  }

  return { byId, byName };
}

function buildPlayerTracks(
  raw: RawReplayFramesData,
  progressTracker?: NormalizeReplayProgressTracker,
): ReplayPlayerTrack[] {
  const teamZeroNames = new Set(raw.meta.team_zero.map((player) => player.name));
  const teamOneNames = new Set(raw.meta.team_one.map((player) => player.name));
  const replayPlayers = indexReplayPlayers(raw, progressTracker);
  const players: ReplayPlayerTrack[] = [];
  let processedPlayerFrames = 0;

  for (const [playerId, playerData] of raw.frame_data.players) {
    const frames = new Array<PlayerSample>(playerData.frames.length);
    let firstFrame: Exclude<RawPlayerFrame, "Empty"> | undefined;

    for (let index = 0; index < playerData.frames.length; index += 1) {
      const frame = playerData.frames[index]!;
      if (firstFrame === undefined && frame !== "Empty") {
        firstFrame = frame;
      }
      frames[index] = parsePlayerFrame(frame);
      processedPlayerFrames += 1;
      progressTracker?.advanceFrame();
    }
    fillBoundedPlayerSampleGaps(frames);

    const playerIdString = playerIdToString(playerId);
    const name =
      firstFrame !== undefined && firstFrame.Data.player_name
        ? firstFrame.Data.player_name
        : (replayPlayers.byId.get(playerIdString)?.name ?? playerIdString);
    const replayPlayerInfo =
      replayPlayers.byId.get(playerIdString) ?? replayPlayers.byName.get(name);

    players.push({
      id: playerIdString,
      name,
      isTeamZero: inferTeamSide(name, teamZeroNames, teamOneNames, firstFrame),
      cameraSettings: extractCameraSettings(replayPlayerInfo),
      hitbox: getReplayHitboxSpec(inferReplayHitboxKind(replayPlayerInfo)),
      frames,
    });
  }

  if (processedPlayerFrames === 0) {
    progressTracker?.advanceFrame();
  }

  return players;
}

async function buildPlayerTracksAsync(
  raw: RawReplayFramesData,
  progressTracker: AsyncNormalizeReplayProgressTracker,
): Promise<ReplayPlayerTrack[]> {
  const teamZeroNames = new Set(raw.meta.team_zero.map((player) => player.name));
  const teamOneNames = new Set(raw.meta.team_one.map((player) => player.name));
  const replayPlayers = await indexReplayPlayersAsync(raw, progressTracker);
  const players: ReplayPlayerTrack[] = [];
  let processedPlayerFrames = 0;

  for (const [playerId, playerData] of raw.frame_data.players) {
    const frames = new Array<PlayerSample>(playerData.frames.length);
    let firstFrame: Exclude<RawPlayerFrame, "Empty"> | undefined;

    for (let index = 0; index < playerData.frames.length; index += 1) {
      const frame = playerData.frames[index]!;
      if (firstFrame === undefined && frame !== "Empty") {
        firstFrame = frame;
      }
      frames[index] = parsePlayerFrame(frame);
      processedPlayerFrames += 1;
      if (progressTracker.advanceFrame()) {
        await progressTracker.yieldToMainThread();
      }
    }
    fillBoundedPlayerSampleGaps(frames);

    const playerIdString = playerIdToString(playerId);
    const name =
      firstFrame !== undefined && firstFrame.Data.player_name
        ? firstFrame.Data.player_name
        : (replayPlayers.byId.get(playerIdString)?.name ?? playerIdString);
    const replayPlayerInfo =
      replayPlayers.byId.get(playerIdString) ?? replayPlayers.byName.get(name);

    players.push({
      id: playerIdString,
      name,
      isTeamZero: inferTeamSide(name, teamZeroNames, teamOneNames, firstFrame),
      cameraSettings: extractCameraSettings(replayPlayerInfo),
      hitbox: getReplayHitboxSpec(inferReplayHitboxKind(replayPlayerInfo)),
      frames,
    });
  }

  if (processedPlayerFrames === 0 && progressTracker.advanceFrame()) {
    await progressTracker.yieldToMainThread();
  }

  return players;
}

function buildBallFrames(
  raw: RawReplayFramesData,
  progressTracker?: NormalizeReplayProgressTracker,
): BallSample[] {
  const rawBallFrames = raw.frame_data.ball_data.frames;
  if (rawBallFrames.length === 0) {
    progressTracker?.advanceFrame();
    return [];
  }

  const ballFrames = new Array<BallSample>(rawBallFrames.length);
  for (let index = 0; index < rawBallFrames.length; index += 1) {
    ballFrames[index] = parseBallFrame(rawBallFrames[index]!);
    progressTracker?.advanceFrame();
  }

  return ballFrames;
}

async function buildBallFramesAsync(
  raw: RawReplayFramesData,
  progressTracker: AsyncNormalizeReplayProgressTracker,
): Promise<BallSample[]> {
  const rawBallFrames = raw.frame_data.ball_data.frames;
  if (rawBallFrames.length === 0) {
    if (progressTracker.advanceFrame()) {
      await progressTracker.yieldToMainThread();
    }
    return [];
  }

  const ballFrames = new Array<BallSample>(rawBallFrames.length);
  for (let index = 0; index < rawBallFrames.length; index += 1) {
    ballFrames[index] = parseBallFrame(rawBallFrames[index]!);
    if (progressTracker.advanceFrame()) {
      await progressTracker.yieldToMainThread();
    }
  }

  return ballFrames;
}

function createTimelineEventId(prefix: string, frame: number, suffix: string): string {
  return `${prefix}:${frame}:${suffix}`;
}

function sortTimelineEvents(events: ReplayTimelineEvent[]): ReplayTimelineEvent[] {
  return events.sort((left, right) => {
    if (left.time !== right.time) {
      return left.time - right.time;
    }
    return (left.frame ?? 0) - (right.frame ?? 0);
  });
}

function goalTimelineEvent(
  event: RawGoalEvent,
  playersById: Map<string, ReplayPlayerTrack>,
  startTime: number,
): ReplayTimelineEvent {
  const playerId = event.player ? playerIdToString(event.player) : null;
  const playerName = playerId ? (playersById.get(playerId)?.name ?? playerId) : null;
  const label = playerName ? `${playerName} scored` : "Goal";
  return {
    id: createTimelineEventId("goal", event.frame, playerId ?? "team"),
    time: normalizeReplayTime(event.time, startTime),
    frame: event.frame,
    kind: "goal",
    label,
    shortLabel: "G",
    playerId,
    playerName,
    isTeamZero: event.scoring_team_is_team_0,
  };
}

function playerStatTimelineEvent(
  event: RawPlayerStatEvent,
  playersById: Map<string, ReplayPlayerTrack>,
  startTime: number,
): ReplayTimelineEvent {
  const playerId = playerIdToString(event.player);
  const playerName = playersById.get(playerId)?.name ?? playerId;
  const kind = event.kind.toLowerCase() as ReplayTimelineEvent["kind"];
  const verb = event.kind === "Shot" ? "shot" : event.kind === "Save" ? "save" : "assist";
  const shortLabel = event.kind === "Shot" ? "SH" : event.kind === "Save" ? "SV" : "A";
  return {
    id: createTimelineEventId(kind, event.frame, playerId),
    time: normalizeReplayTime(event.time, startTime),
    frame: event.frame,
    kind,
    label: `${playerName} ${verb}`,
    shortLabel,
    playerId,
    playerName,
    location: event.shot?.shot_touch_position ?? event.shot?.ball_position ?? null,
    shot: event.shot ?? null,
    isTeamZero: event.is_team_0,
  };
}

function demoTimelineEvent(
  event: RawDemolishInfo,
  playersById: Map<string, ReplayPlayerTrack>,
  startTime: number,
): ReplayTimelineEvent {
  const attackerId = playerIdToString(event.attacker);
  const victimId = playerIdToString(event.victim);
  const attacker = playersById.get(attackerId);
  const victim = playersById.get(victimId);
  return {
    id: createTimelineEventId("demo", event.frame, `${attackerId}:${victimId}`),
    time: normalizeReplayTime(event.time, startTime),
    frame: event.frame,
    kind: "demo",
    label: `${attacker?.name ?? attackerId} demoed ${victim?.name ?? victimId}`,
    shortLabel: "D",
    playerId: attackerId,
    playerName: attacker?.name ?? attackerId,
    secondaryPlayerId: victimId,
    secondaryPlayerName: victim?.name ?? victimId,
    location: event.victim_location,
    isTeamZero: attacker?.isTeamZero ?? null,
  };
}

function buildTimelineEvents(
  raw: RawReplayFramesData,
  players: ReplayPlayerTrack[],
  tickMarks: ReplayTickMark[],
  startTime: number,
  progressTracker?: NormalizeReplayProgressTracker,
): ReplayTimelineEvent[] {
  const playersById = buildPlayerLookup(players);
  const timelineEvents: ReplayTimelineEvent[] = [];

  for (const event of raw.goal_events ?? []) {
    timelineEvents.push(goalTimelineEvent(event, playersById, startTime));
    progressTracker?.advance();
  }

  for (const event of raw.player_stat_events ?? []) {
    timelineEvents.push(playerStatTimelineEvent(event, playersById, startTime));
    progressTracker?.advance();
  }

  for (const event of raw.demolish_infos ?? []) {
    timelineEvents.push(demoTimelineEvent(event, playersById, startTime));
    progressTracker?.advance();
  }

  for (const tickMark of tickMarks) {
    timelineEvents.push(replayTickMarkTimelineEvent(tickMark));
  }

  if (timelineEvents.length === 0) {
    progressTracker?.advance();
  }

  return sortTimelineEvents(timelineEvents);
}

async function buildTimelineEventsAsync(
  raw: RawReplayFramesData,
  players: ReplayPlayerTrack[],
  tickMarks: ReplayTickMark[],
  startTime: number,
  progressTracker: AsyncNormalizeReplayProgressTracker,
): Promise<ReplayTimelineEvent[]> {
  const playersById = buildPlayerLookup(players);
  const timelineEvents: ReplayTimelineEvent[] = [];

  for (const event of raw.goal_events ?? []) {
    timelineEvents.push(goalTimelineEvent(event, playersById, startTime));
    if (progressTracker.advance()) {
      await progressTracker.yieldToMainThread();
    }
  }

  for (const event of raw.player_stat_events ?? []) {
    timelineEvents.push(playerStatTimelineEvent(event, playersById, startTime));
    if (progressTracker.advance()) {
      await progressTracker.yieldToMainThread();
    }
  }

  for (const event of raw.demolish_infos ?? []) {
    timelineEvents.push(demoTimelineEvent(event, playersById, startTime));
    if (progressTracker.advance()) {
      await progressTracker.yieldToMainThread();
    }
  }

  for (const tickMark of tickMarks) {
    timelineEvents.push(replayTickMarkTimelineEvent(tickMark));
  }

  if (timelineEvents.length === 0 && progressTracker.advance()) {
    await progressTracker.yieldToMainThread();
  }

  return sortTimelineEvents(timelineEvents);
}

export function normalizeReplayData(
  raw: RawReplayFramesData,
  options: NormalizeReplayDataOptions = {},
): ReplayModel {
  const progressTracker = createNormalizationProgressTracker(raw, options.onProgress, {
    progressReportMinDelta: options.progressReportMinDelta,
    progressReportFrameInterval: options.progressReportFrameInterval,
  });
  const startTime = raw.frame_data.metadata_frames[0]?.time ?? 0;
  const frames = buildPlaybackFrames(raw, progressTracker);
  const players = buildPlayerTracks(raw, progressTracker);
  const ballFrames = buildBallFrames(raw, progressTracker);
  const boostPads = buildBoostPads(raw, players, startTime, progressTracker);
  const tickMarks = buildReplayTickMarks(raw, startTime, progressTracker);
  const timelineEvents = buildTimelineEvents(raw, players, tickMarks, startTime, progressTracker);
  progressTracker.finish();

  return {
    frameCount: frames.length,
    duration: frames.at(-1)?.time ?? 0,
    rawStartTime: startTime,
    frames,
    ballFrames,
    boostPads,
    players,
    tickMarks,
    timelineEvents,
    teamZeroNames: raw.meta.team_zero.map((player) => player.name),
    teamOneNames: raw.meta.team_one.map((player) => player.name),
  };
}

export async function normalizeReplayDataAsync(
  raw: RawReplayFramesData,
  options: NormalizeReplayDataAsyncOptions = {},
): Promise<ReplayModel> {
  const progressTracker = createAsyncNormalizationProgressTracker(raw, options);
  const startTime = raw.frame_data.metadata_frames[0]?.time ?? 0;
  const frames = await buildPlaybackFramesAsync(raw, progressTracker);
  const players = await buildPlayerTracksAsync(raw, progressTracker);
  const ballFrames = await buildBallFramesAsync(raw, progressTracker);
  const boostPads = await buildBoostPadsAsync(raw, players, startTime, progressTracker);
  const tickMarks = buildReplayTickMarks(raw, startTime, progressTracker);
  const timelineEvents = await buildTimelineEventsAsync(
    raw,
    players,
    tickMarks,
    startTime,
    progressTracker,
  );
  progressTracker.finish();

  return {
    frameCount: frames.length,
    duration: frames.at(-1)?.time ?? 0,
    rawStartTime: startTime,
    frames,
    ballFrames,
    boostPads,
    players,
    tickMarks,
    timelineEvents,
    teamZeroNames: raw.meta.team_zero.map((player) => player.name),
    teamOneNames: raw.meta.team_one.map((player) => player.name),
  };
}

export function findFrameIndexAtTime(replay: ReplayModel, time: number): number {
  if (replay.frames.length === 0) {
    return 0;
  }

  let low = 0;
  let high = replay.frames.length - 1;

  while (low <= high) {
    const middle = Math.floor((low + high) / 2);
    const middleTime = replay.frames[middle]?.time ?? 0;

    if (middleTime < time) {
      low = middle + 1;
    } else if (middleTime > time) {
      high = middle - 1;
    } else {
      return middle;
    }
  }

  return Math.max(0, low - 1);
}
