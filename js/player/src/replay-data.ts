import type {
  BallSample,
  CameraSettings,
  PlaybackFrame,
  RawBoostPad,
  RawBoostPadEvent,
  RawDemolishInfo,
  RawGoalEvent,
  RawPlayerStatEvent,
  RawPlayerId,
  PlayerSample,
  RawBallFrame,
  RawPlayerFrame,
  RawPlayerInfo,
  RawReplayFramesData,
  ReplayBoostPad,
  ReplayBoostPadEvent,
  ReplayBoostPadSize,
  ReplayModel,
  ReplayPlayerTrack,
  ReplayTimelineEvent,
  Vec3,
  Quaternion,
} from "./types";

export interface NormalizeReplayDataOptions {
  onProgress?: (progress: number) => void;
}

export interface NormalizeReplayDataAsyncOptions extends NormalizeReplayDataOptions {
  yieldEveryMs?: number;
  yieldToMainThread?: () => Promise<void>;
}

interface NormalizeReplayProgressTracker {
  advance(units?: number): boolean;
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
const BOOST_PAD_SMALL_Z = 70;
const BOOST_PAD_BIG_Z = 73;
const BOOST_PAD_BACK_CORNER_X = 3072;
const BOOST_PAD_BACK_CORNER_Y = 4096;
const BOOST_PAD_BACK_LANE_X = 1792;
const BOOST_PAD_BACK_LANE_Y = 4184;
const BOOST_PAD_BACK_MID_X = 940;
const BOOST_PAD_BACK_MID_Y = 3308;
const BOOST_PAD_CENTER_BACK_Y = 2816;
const BOOST_PAD_SIDE_WALL_X = 3584;
const BOOST_PAD_SIDE_WALL_Y = 2484;
const BOOST_PAD_SIDE_LANE_X = 1788;
const BOOST_PAD_SIDE_LANE_Y = 2300;
const BOOST_PAD_FRONT_LANE_X = 2048;
const BOOST_PAD_FRONT_LANE_Y = 1036;
const BOOST_PAD_CENTER_X = 1024;
const BOOST_PAD_CENTER_MID_Y = 1024;
const BOOST_PAD_GOAL_LINE_Y = 4240;
const STANDARD_SOCCAR_BOOST_PAD_COUNT = 34;
const NORMALIZATION_PROGRESS_REPORT_MIN_DELTA = 0.005;
const NORMALIZATION_ASYNC_YIELD_INTERVAL_MS = 16;

function playerIdToString(playerId: RawPlayerId): string {
  const [kind, value] = Object.entries(playerId)[0] ?? ["Unknown", "unknown"];
  if (typeof value === "string" || typeof value === "number") {
    return `${kind}:${value}`;
  }

  if (value && typeof value === "object" && "online_id" in value) {
    const onlineId = value.online_id;
    if (typeof onlineId === "string" || typeof onlineId === "number") {
      return `${kind}:${onlineId}`;
    }
  }

  return `${kind}:${JSON.stringify(value)}`;
}

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
    }
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

function normalizeReplayTime(rawTime: number, startTime: number): number {
  return Math.max(0, rawTime - startTime);
}

function parsePlayerFrame(frame: RawPlayerFrame): PlayerSample {
  if (frame === "Empty") {
    return {
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
    (raw.goal_events?.length ?? 0)
    + (raw.player_stat_events?.length ?? 0)
    + (raw.demolish_infos?.length ?? 0);

  return [
    Math.max(1, raw.frame_data.metadata_frames.length),
    Math.max(1, playerInfoCount),
    Math.max(1, playerFrameCount),
    Math.max(1, raw.frame_data.ball_data.frames.length),
    Math.max(1, boostPadCount + boostPadEventCount),
    Math.max(1, timelineEventCount),
  ].reduce((sum, count) => sum + count, 0);
}

function createNormalizationProgressTracker(
  raw: RawReplayFramesData,
  onProgress?: (progress: number) => void,
  options: { yieldEveryMs?: number } = {},
): NormalizeReplayProgressTracker {
  if (!onProgress) {
    return {
      advance() {
        return false;
      },
      finish() {},
    };
  }

  const totalUnits = getNormalizationTotalUnits(raw);
  let completedUnits = 0;
  let lastReportedProgress = -1;
  let lastYieldedAt = currentTimeMs();
  const yieldEveryMs =
    options.yieldEveryMs ?? Number.POSITIVE_INFINITY;

  const maybeReport = () => {
    const progress = Math.max(0, Math.min(1, completedUnits / totalUnits));
    if (
      progress >= 1
      || progress - lastReportedProgress >= NORMALIZATION_PROGRESS_REPORT_MIN_DELTA
    ) {
      lastReportedProgress = progress;
      onProgress(progress);
    }
  };

  const shouldYield = () => {
    const now = currentTimeMs();
    if (now - lastYieldedAt < yieldEveryMs) {
      return false;
    }
    lastYieldedAt = now;
    return true;
  };

  return {
    advance(units = 1) {
      if (units <= 0) {
        return false;
      }
      completedUnits = Math.min(totalUnits, completedUnits + units);
      maybeReport();
      return shouldYield();
    },
    finish() {
      completedUnits = totalUnits;
      maybeReport();
    },
  };
}

function createAsyncNormalizationProgressTracker(
  raw: RawReplayFramesData,
  options: NormalizeReplayDataAsyncOptions,
): AsyncNormalizeReplayProgressTracker {
  const progressTracker = createNormalizationProgressTracker(
    raw,
    options.onProgress,
    {
      yieldEveryMs:
        options.yieldEveryMs ?? NORMALIZATION_ASYNC_YIELD_INTERVAL_MS,
    },
  );

  return {
    ...progressTracker,
    yieldToMainThread:
      options.yieldToMainThread ?? defaultYieldToMainThread,
  };
}

function buildPlaybackFrames(
  raw: RawReplayFramesData,
  progressTracker?: NormalizeReplayProgressTracker,
): PlaybackFrame[] {
  const metadataFrames = raw.frame_data.metadata_frames;
  if (metadataFrames.length === 0) {
    progressTracker?.advance();
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
    progressTracker?.advance();
  }

  return frames;
}

async function buildPlaybackFramesAsync(
  raw: RawReplayFramesData,
  progressTracker: AsyncNormalizeReplayProgressTracker,
): Promise<PlaybackFrame[]> {
  const metadataFrames = raw.frame_data.metadata_frames;
  if (metadataFrames.length === 0) {
    if (progressTracker.advance()) {
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
    if (progressTracker.advance()) {
      await progressTracker.yieldToMainThread();
    }
  }

  return frames;
}

function inferTeamSide(
  name: string,
  teamZeroNames: Set<string>,
  teamOneNames: Set<string>,
  firstFrame: RawPlayerFrame | undefined
): boolean {
  if (teamZeroNames.has(name)) {
    return true;
  }

  if (teamOneNames.has(name)) {
    return false;
  }

  if (
    firstFrame &&
    firstFrame !== "Empty" &&
    typeof firstFrame.Data.is_team_0 === "boolean"
  ) {
    return firstFrame.Data.is_team_0;
  }

  return true;
}

function getStatEntries(
  stats: RawPlayerInfo["stats"] | undefined
): Array<[string, unknown]> {
  if (!stats) {
    return [];
  }

  return Object.entries(stats);
}

function extractNumericSetting(
  entries: Array<[string, unknown]>,
  key: string
): number | undefined {
  const value = entries.find(([entryKey]) => entryKey === key)?.[1];
  return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

function extractCameraSettings(playerInfo?: RawPlayerInfo): CameraSettings {
  const entries = getStatEntries(playerInfo?.stats);
  return {
    fov: extractNumericSetting(entries, "CameraFOV") ?? DEFAULT_CAMERA_SETTINGS.fov,
    height: extractNumericSetting(entries, "CameraHeight") ?? DEFAULT_CAMERA_SETTINGS.height,
    pitch: extractNumericSetting(entries, "CameraPitch") ?? DEFAULT_CAMERA_SETTINGS.pitch,
    distance:
      extractNumericSetting(entries, "CameraDistance") ?? DEFAULT_CAMERA_SETTINGS.distance,
    stiffness:
      extractNumericSetting(entries, "CameraStiffness") ?? DEFAULT_CAMERA_SETTINGS.stiffness,
    swivelSpeed:
      extractNumericSetting(entries, "CameraSwivelSpeed") ??
      DEFAULT_CAMERA_SETTINGS.swivelSpeed,
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
      progressTracker?.advance();
    }

    const playerIdString = playerIdToString(playerId);
    const name =
      firstFrame !== undefined && firstFrame.Data.player_name
        ? firstFrame.Data.player_name
        : replayPlayers.byId.get(playerIdString)?.name ?? playerIdString;
    const replayPlayerInfo =
      replayPlayers.byId.get(playerIdString) ?? replayPlayers.byName.get(name);

    players.push({
      id: playerIdString,
      name,
      isTeamZero: inferTeamSide(name, teamZeroNames, teamOneNames, firstFrame),
      cameraSettings: extractCameraSettings(replayPlayerInfo),
      frames,
    });
  }

  if (processedPlayerFrames === 0) {
    progressTracker?.advance();
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
      if (progressTracker.advance()) {
        await progressTracker.yieldToMainThread();
      }
    }

    const playerIdString = playerIdToString(playerId);
    const name =
      firstFrame !== undefined && firstFrame.Data.player_name
        ? firstFrame.Data.player_name
        : replayPlayers.byId.get(playerIdString)?.name ?? playerIdString;
    const replayPlayerInfo =
      replayPlayers.byId.get(playerIdString) ?? replayPlayers.byName.get(name);

    players.push({
      id: playerIdString,
      name,
      isTeamZero: inferTeamSide(name, teamZeroNames, teamOneNames, firstFrame),
      cameraSettings: extractCameraSettings(replayPlayerInfo),
      frames,
    });
  }

  if (processedPlayerFrames === 0 && progressTracker.advance()) {
    await progressTracker.yieldToMainThread();
  }

  return players;
}

function buildPlayerLookup(
  players: ReplayPlayerTrack[]
): Map<string, ReplayPlayerTrack> {
  return new Map(players.map((player) => [player.id, player]));
}

function pushPad(
  pads: ReplayBoostPad[],
  x: number,
  y: number,
  z: number,
  size: ReplayBoostPadSize
): void {
  pads.push({
    index: pads.length,
    padId: null,
    size,
    position: { x, y, z },
    events: [],
  });
}

function pushMirrorX(
  pads: ReplayBoostPad[],
  x: number,
  y: number,
  z: number,
  size: ReplayBoostPadSize
): void {
  pushPad(pads, -x, y, z, size);
  pushPad(pads, x, y, z, size);
}

function pushMirrorY(
  pads: ReplayBoostPad[],
  x: number,
  y: number,
  z: number,
  size: ReplayBoostPadSize
): void {
  pushPad(pads, x, -y, z, size);
  pushPad(pads, x, y, z, size);
}

function pushMirrorXY(
  pads: ReplayBoostPad[],
  x: number,
  y: number,
  z: number,
  size: ReplayBoostPadSize
): void {
  pushMirrorX(pads, x, -y, z, size);
  pushMirrorX(pads, x, y, z, size);
}

function buildStandardSoccarBoostPads(): ReplayBoostPad[] {
  const pads: ReplayBoostPad[] = [];

  pushMirrorY(pads, 0, BOOST_PAD_GOAL_LINE_Y, BOOST_PAD_SMALL_Z, "small");
  pushMirrorXY(
    pads,
    BOOST_PAD_BACK_LANE_X,
    BOOST_PAD_BACK_LANE_Y,
    BOOST_PAD_SMALL_Z,
    "small"
  );
  pushMirrorXY(
    pads,
    BOOST_PAD_BACK_CORNER_X,
    BOOST_PAD_BACK_CORNER_Y,
    BOOST_PAD_BIG_Z,
    "big"
  );
  pushMirrorXY(
    pads,
    BOOST_PAD_BACK_MID_X,
    BOOST_PAD_BACK_MID_Y,
    BOOST_PAD_SMALL_Z,
    "small"
  );
  pushMirrorY(pads, 0, BOOST_PAD_CENTER_BACK_Y, BOOST_PAD_SMALL_Z, "small");
  pushMirrorXY(
    pads,
    BOOST_PAD_SIDE_WALL_X,
    BOOST_PAD_SIDE_WALL_Y,
    BOOST_PAD_SMALL_Z,
    "small"
  );
  pushMirrorXY(
    pads,
    BOOST_PAD_SIDE_LANE_X,
    BOOST_PAD_SIDE_LANE_Y,
    BOOST_PAD_SMALL_Z,
    "small"
  );
  pushMirrorXY(
    pads,
    BOOST_PAD_FRONT_LANE_X,
    BOOST_PAD_FRONT_LANE_Y,
    BOOST_PAD_SMALL_Z,
    "small"
  );
  pushMirrorY(pads, 0, BOOST_PAD_CENTER_MID_Y, BOOST_PAD_SMALL_Z, "small");
  pushMirrorX(pads, BOOST_PAD_SIDE_WALL_X, 0, BOOST_PAD_BIG_Z, "big");
  pushMirrorX(pads, BOOST_PAD_CENTER_X, 0, BOOST_PAD_SMALL_Z, "small");

  return pads;
}

function buildBallFrames(
  raw: RawReplayFramesData,
  progressTracker?: NormalizeReplayProgressTracker,
): BallSample[] {
  const rawBallFrames = raw.frame_data.ball_data.frames;
  if (rawBallFrames.length === 0) {
    progressTracker?.advance();
    return [];
  }

  const ballFrames = new Array<BallSample>(rawBallFrames.length);
  for (let index = 0; index < rawBallFrames.length; index += 1) {
    ballFrames[index] = parseBallFrame(rawBallFrames[index]!);
    progressTracker?.advance();
  }

  return ballFrames;
}

async function buildBallFramesAsync(
  raw: RawReplayFramesData,
  progressTracker: AsyncNormalizeReplayProgressTracker,
): Promise<BallSample[]> {
  const rawBallFrames = raw.frame_data.ball_data.frames;
  if (rawBallFrames.length === 0) {
    if (progressTracker.advance()) {
      await progressTracker.yieldToMainThread();
    }
    return [];
  }

  const ballFrames = new Array<BallSample>(rawBallFrames.length);
  for (let index = 0; index < rawBallFrames.length; index += 1) {
    ballFrames[index] = parseBallFrame(rawBallFrames[index]!);
    if (progressTracker.advance()) {
      await progressTracker.yieldToMainThread();
    }
  }

  return ballFrames;
}

function parseBoostPadAvailability(kind: unknown): boolean | null {
  if (kind === "Available") {
    return true;
  }
  if (kind && typeof kind === "object") {
    if ("Available" in kind) {
      return true;
    }
    if ("PickedUp" in kind) {
      return false;
    }
    const taggedKind = (kind as { kind?: unknown }).kind;
    if (taggedKind === "Available") {
      return true;
    }
    if (taggedKind === "PickedUp") {
      return false;
    }
  }
  return null;
}

function parseBoostPadSize(size: unknown): ReplayBoostPadSize | null {
  if (size === "big" || size === "Big") {
    return "big";
  }
  if (size === "small" || size === "Small") {
    return "small";
  }
  return null;
}

function inferBoostPadSize(events: RawBoostPadEvent[]): ReplayBoostPadSize | null {
  let lastPickupTime: number | null = null;
  for (const event of events) {
    const available = parseBoostPadAvailability(event.kind);
    if (available === false) {
      lastPickupTime = event.time;
      continue;
    }
    if (available === true && lastPickupTime !== null) {
      return event.time - lastPickupTime >= 7 ? "big" : "small";
    }
  }

  return null;
}

function buildBoostPads(
  raw: RawReplayFramesData,
  players: ReplayPlayerTrack[],
  startTime: number,
  progressTracker?: NormalizeReplayProgressTracker,
): ReplayBoostPad[] {
  const playersById = buildPlayerLookup(players);
  const eventsByPadId = new Map<string, RawBoostPadEvent[]>();

  for (const event of raw.boost_pad_events ?? []) {
    const availability = parseBoostPadAvailability(event.kind);
    if (availability === null) {
      progressTracker?.advance();
      continue;
    }
    const bucket = eventsByPadId.get(event.pad_id);
    if (bucket) {
      bucket.push(event);
    } else {
      eventsByPadId.set(event.pad_id, [event]);
    }
    progressTracker?.advance();
  }

  const rawPads = raw.boost_pads;
  if (!rawPads || rawPads.length === 0) {
    progressTracker?.advance(STANDARD_SOCCAR_BOOST_PAD_COUNT);
    return buildStandardSoccarBoostPads();
  }

  const sortedPads = [...rawPads].sort((left, right) => left.index - right.index);
  const pads = new Array<ReplayBoostPad>(sortedPads.length);

  for (let index = 0; index < sortedPads.length; index += 1) {
    const pad = sortedPads[index]!;
    const padId = typeof pad.pad_id === "string" ? pad.pad_id : null;
    const rawEvents = padId ? [...(eventsByPadId.get(padId) ?? [])] : [];
    const size =
      parseBoostPadSize(pad.size) ??
      inferBoostPadSize(rawEvents) ??
      (pad.position.z >= 72 ? "big" : "small");

    const sortedEvents = rawEvents.sort((left, right) => left.time - right.time);
    const events = new Array<ReplayBoostPadEvent>(sortedEvents.length);

    for (let eventIndex = 0; eventIndex < sortedEvents.length; eventIndex += 1) {
      const event = sortedEvents[eventIndex]!;
      const playerId = event.player ? playerIdToString(event.player) : null;
      events[eventIndex] = {
        time: normalizeReplayTime(event.time, startTime),
        frame: event.frame,
        available: parseBoostPadAvailability(event.kind) ?? true,
        playerId,
        playerName: playerId ? playersById.get(playerId)?.name ?? playerId : null,
      };
    }

    pads[index] = {
      index: pad.index,
      padId,
      size,
      position: pad.position,
      events,
    };
    progressTracker?.advance();
  }

  return pads;
}

async function buildBoostPadsAsync(
  raw: RawReplayFramesData,
  players: ReplayPlayerTrack[],
  startTime: number,
  progressTracker: AsyncNormalizeReplayProgressTracker,
): Promise<ReplayBoostPad[]> {
  const playersById = buildPlayerLookup(players);
  const eventsByPadId = new Map<string, RawBoostPadEvent[]>();

  for (const event of raw.boost_pad_events ?? []) {
    const availability = parseBoostPadAvailability(event.kind);
    if (availability === null) {
      if (progressTracker.advance()) {
        await progressTracker.yieldToMainThread();
      }
      continue;
    }
    const bucket = eventsByPadId.get(event.pad_id);
    if (bucket) {
      bucket.push(event);
    } else {
      eventsByPadId.set(event.pad_id, [event]);
    }
    if (progressTracker.advance()) {
      await progressTracker.yieldToMainThread();
    }
  }

  const rawPads = raw.boost_pads;
  if (!rawPads || rawPads.length === 0) {
    if (progressTracker.advance(STANDARD_SOCCAR_BOOST_PAD_COUNT)) {
      await progressTracker.yieldToMainThread();
    }
    return buildStandardSoccarBoostPads();
  }

  const sortedPads = [...rawPads].sort((left, right) => left.index - right.index);
  const pads = new Array<ReplayBoostPad>(sortedPads.length);

  for (let index = 0; index < sortedPads.length; index += 1) {
    const pad = sortedPads[index]!;
    const padId = typeof pad.pad_id === "string" ? pad.pad_id : null;
    const rawEvents = padId ? [...(eventsByPadId.get(padId) ?? [])] : [];
    const size =
      parseBoostPadSize(pad.size) ??
      inferBoostPadSize(rawEvents) ??
      (pad.position.z >= 72 ? "big" : "small");

    const sortedEvents = rawEvents.sort((left, right) => left.time - right.time);
    const events = new Array<ReplayBoostPadEvent>(sortedEvents.length);

    for (let eventIndex = 0; eventIndex < sortedEvents.length; eventIndex += 1) {
      const event = sortedEvents[eventIndex]!;
      const playerId = event.player ? playerIdToString(event.player) : null;
      events[eventIndex] = {
        time: normalizeReplayTime(event.time, startTime),
        frame: event.frame,
        available: parseBoostPadAvailability(event.kind) ?? true,
        playerId,
        playerName: playerId ? playersById.get(playerId)?.name ?? playerId : null,
      };
    }

    pads[index] = {
      index: pad.index,
      padId,
      size,
      position: pad.position,
      events,
    };
    if (progressTracker.advance()) {
      await progressTracker.yieldToMainThread();
    }
  }

  return pads;
}

function createTimelineEventId(
  prefix: string,
  frame: number,
  suffix: string
): string {
  return `${prefix}:${frame}:${suffix}`;
}

function goalTimelineEvent(
  event: RawGoalEvent,
  playersById: Map<string, ReplayPlayerTrack>,
  startTime: number
): ReplayTimelineEvent {
  const playerId = event.player ? playerIdToString(event.player) : null;
  const playerName = playerId ? playersById.get(playerId)?.name ?? playerId : null;
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
  startTime: number
): ReplayTimelineEvent {
  const playerId = playerIdToString(event.player);
  const playerName = playersById.get(playerId)?.name ?? playerId;
  const kind = event.kind.toLowerCase() as ReplayTimelineEvent["kind"];
  const verb =
    event.kind === "Shot" ? "shot" : event.kind === "Save" ? "save" : "assist";
  const shortLabel =
    event.kind === "Shot" ? "SH" : event.kind === "Save" ? "SV" : "A";
  return {
    id: createTimelineEventId(kind, event.frame, playerId),
    time: normalizeReplayTime(event.time, startTime),
    frame: event.frame,
    kind,
    label: `${playerName} ${verb}`,
    shortLabel,
    playerId,
    playerName,
    isTeamZero: event.is_team_0,
  };
}

function demoTimelineEvent(
  event: RawDemolishInfo,
  playersById: Map<string, ReplayPlayerTrack>,
  startTime: number
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
    isTeamZero: attacker?.isTeamZero ?? null,
  };
}

function buildTimelineEvents(
  raw: RawReplayFramesData,
  players: ReplayPlayerTrack[],
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

  if (timelineEvents.length === 0) {
    progressTracker?.advance();
  }

  return timelineEvents.sort((left, right) => {
    if (left.time !== right.time) {
      return left.time - right.time;
    }
    return (left.frame ?? 0) - (right.frame ?? 0);
  });
}

async function buildTimelineEventsAsync(
  raw: RawReplayFramesData,
  players: ReplayPlayerTrack[],
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

  if (timelineEvents.length === 0 && progressTracker.advance()) {
    await progressTracker.yieldToMainThread();
  }

  return timelineEvents.sort((left, right) => {
    if (left.time !== right.time) {
      return left.time - right.time;
    }
    return (left.frame ?? 0) - (right.frame ?? 0);
  });
}

export function normalizeReplayData(
  raw: RawReplayFramesData,
  options: NormalizeReplayDataOptions = {},
): ReplayModel {
  const progressTracker = createNormalizationProgressTracker(raw, options.onProgress);
  const startTime = raw.frame_data.metadata_frames[0]?.time ?? 0;
  const frames = buildPlaybackFrames(raw, progressTracker);
  const players = buildPlayerTracks(raw, progressTracker);
  const ballFrames = buildBallFrames(raw, progressTracker);
  const boostPads = buildBoostPads(raw, players, startTime, progressTracker);
  const timelineEvents = buildTimelineEvents(raw, players, startTime, progressTracker);
  progressTracker.finish();

  return {
    frameCount: frames.length,
    duration: frames.at(-1)?.time ?? 0,
    frames,
    ballFrames,
    boostPads,
    players,
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
  const boostPads = await buildBoostPadsAsync(
    raw,
    players,
    startTime,
    progressTracker,
  );
  const timelineEvents = await buildTimelineEventsAsync(
    raw,
    players,
    startTime,
    progressTracker,
  );
  progressTracker.finish();

  return {
    frameCount: frames.length,
    duration: frames.at(-1)?.time ?? 0,
    frames,
    ballFrames,
    boostPads,
    players,
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
