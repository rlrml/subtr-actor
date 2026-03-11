import type {
  BallSample,
  PlaybackFrame,
  PlayerSample,
  RawBallFrame,
  RawPlayerFrame,
  RawReplayFramesData,
  ReplayModel,
  ReplayPlayerTrack,
  Vec3,
} from "./types";

const UNIT_SCALE = 0.01;

function scaleVector(value: Vec3): Vec3 {
  return {
    x: value.x * UNIT_SCALE,
    y: value.y * UNIT_SCALE,
    z: value.z * UNIT_SCALE,
  };
}

function playerIdToString(playerId: Record<string, string>): string {
  const [kind, value] = Object.entries(playerId)[0] ?? ["Unknown", "unknown"];
  return `${kind}:${value}`;
}

function parseBallFrame(frame: RawBallFrame): BallSample {
  if (frame === "Empty") {
    return { position: null };
  }

  return {
    position: scaleVector(frame.Data.rigid_body.location),
  };
}

function parsePlayerFrame(frame: RawPlayerFrame): PlayerSample {
  if (frame === "Empty") {
    return {
      position: null,
      velocity: null,
      boostAmount: 0,
      boostActive: false,
      jumpActive: false,
      dodgeActive: false,
    };
  }

  return {
    position: scaleVector(frame.Data.rigid_body.location),
    velocity: scaleVector(frame.Data.rigid_body.linear_velocity),
    boostAmount: frame.Data.boost_amount,
    boostActive: frame.Data.boost_active,
    jumpActive: frame.Data.jump_active,
    dodgeActive: frame.Data.dodge_active,
  };
}

function buildPlaybackFrames(raw: RawReplayFramesData): PlaybackFrame[] {
  const startTime = raw.frame_data.metadata_frames[0]?.time ?? 0;
  return raw.frame_data.metadata_frames.map((frame) => ({
    time: frame.time - startTime,
    secondsRemaining: frame.seconds_remaining,
    gameState: frame.replicated_game_state_name,
  }));
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

  if (firstFrame && firstFrame !== "Empty" && firstFrame.Data.is_team_0 !== undefined) {
    return firstFrame.Data.is_team_0;
  }

  return true;
}

function buildPlayerTracks(raw: RawReplayFramesData): ReplayPlayerTrack[] {
  const teamZeroNames = new Set(raw.meta.team_zero.map((player) => player.name));
  const teamOneNames = new Set(raw.meta.team_one.map((player) => player.name));

  return raw.frame_data.players.map(([playerId, playerData]) => {
    const firstFrame = playerData.frames.find(
      (frame): frame is Exclude<RawPlayerFrame, "Empty"> => frame !== "Empty"
    );
    const name =
      firstFrame !== undefined && firstFrame.Data.player_name
        ? firstFrame.Data.player_name
        : playerIdToString(playerId);

    return {
      id: playerIdToString(playerId),
      name,
      isTeamZero: inferTeamSide(name, teamZeroNames, teamOneNames, firstFrame),
      frames: playerData.frames.map(parsePlayerFrame),
    };
  });
}

export function normalizeReplayData(raw: RawReplayFramesData): ReplayModel {
  const frames = buildPlaybackFrames(raw);
  const players = buildPlayerTracks(raw);

  return {
    frameCount: frames.length,
    duration: frames.at(-1)?.time ?? 0,
    frames,
    ballFrames: raw.frame_data.ball_data.frames.map(parseBallFrame),
    players,
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
