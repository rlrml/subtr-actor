import type {
  BallSample,
  CameraSettings,
  PlaybackFrame,
  PlayerSample,
  RawBallFrame,
  RawPlayerFrame,
  RawPlayerInfo,
  RawReplayFramesData,
  ReplayPlayerTrack,
} from "./types";
import { playerIdToString } from "./replay-data-utils";
import {
  normalizeQuaternion,
  normalizeVector,
  rotateVectorByQuaternion,
} from "./replay-data-geometry";
import type {
  AsyncNormalizeReplayProgressTracker,
  NormalizeReplayProgressTracker,
} from "./replay-normalization-progress";

const DEFAULT_CAMERA_SETTINGS: CameraSettings = {
  distance: 270,
  height: 100,
  pitch: -4,
  fov: 110,
};

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

export function buildPlaybackFrames(
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

export async function buildPlaybackFramesAsync(
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

export function buildPlayerTracks(
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
      frames,
    });
  }

  if (processedPlayerFrames === 0) {
    progressTracker?.advanceFrame();
  }

  return players;
}

export async function buildPlayerTracksAsync(
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
      frames,
    });
  }

  if (processedPlayerFrames === 0 && progressTracker.advanceFrame()) {
    await progressTracker.yieldToMainThread();
  }

  return players;
}

export function buildBallFrames(
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

export async function buildBallFramesAsync(
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
