const GAME_STATE_KICKOFF_COUNTDOWN = 55;
const POSITION_SCALE = 100;

export function adaptReplayDataForViewer(frameData, statsTimeline) {
  const metadataFrames = frameData.frame_data.metadata_frames ?? [];
  const sampleFrames = statsTimeline.frames ?? [];
  const replayMeta = statsTimeline.replay_meta ?? frameData.meta ?? {};
  const startTime = metadataFrames[0]?.time ?? sampleFrames[0]?.time ?? 0;
  const times = metadataFrames.map((frame) => normalizeTime(frame.time, startTime));
  const maxTime = times.at(-1) ?? 0;
  const playerInfoById = buildPlayerInfoById(replayMeta);

  return {
    map: inferMapCode(replayMeta) ?? "unknown",
    map_type: "soccar",
    max_time: maxTime,
    ball_type: "sphere",
    balls: [buildBallTrack(frameData.frame_data.ball_data.frames ?? [], times, maxTime)],
    players: (frameData.frame_data.players ?? []).map(([playerId, playerData], index) =>
      buildPlayerTrack(playerId, playerData, times, maxTime, playerInfoById, index),
    ),
    countdowns: buildCountdowns(sampleFrames, startTime),
    rem_seconds: buildRemainingSecondsTrack(sampleFrames, startTime),
    blue_score: buildScoreTrack(sampleFrames, "team_zero", startTime),
    orange_score: buildScoreTrack(sampleFrames, "team_one", startTime),
    boost_pads: [],
    ticks: buildTicks(statsTimeline.timeline_events ?? [], startTime),
  };
}

export function buildPlayerNameMap(replayMeta) {
  const nameMap = new Map();
  for (const player of [...(replayMeta.team_zero ?? []), ...(replayMeta.team_one ?? [])]) {
    nameMap.set(stablePlayerId(player.remote_id), player.name);
  }
  return nameMap;
}

function buildPlayerInfoById(replayMeta) {
  const playerInfo = new Map();
  for (const player of replayMeta.team_zero ?? []) {
    playerInfo.set(stablePlayerId(player.remote_id), { ...player, isTeamZero: true });
  }
  for (const player of replayMeta.team_one ?? []) {
    playerInfo.set(stablePlayerId(player.remote_id), { ...player, isTeamZero: false });
  }
  return playerInfo;
}

export function normalizeStatsTimeline(statsTimeline, startTime) {
  return {
    ...statsTimeline,
    frames: (statsTimeline.frames ?? []).map((frame) => ({
      ...frame,
      time: normalizeTime(frame.time, startTime),
    })),
    timeline_events: (statsTimeline.timeline_events ?? []).map((event) => ({
      ...event,
      time: normalizeTime(event.time, startTime),
    })),
  };
}

function buildBallTrack(ballFrames, times, maxTime) {
  const pos = [];
  const quat = [];

  for (const ballFrame of ballFrames) {
    if (ballFrame.Data?.rigid_body) {
      const { location, rotation } = ballFrame.Data.rigid_body;
      pos.push(
        scalePosition(location.x),
        scalePosition(location.y),
        scalePosition(location.z),
      );
      quat.push(rotation.x, rotation.y, rotation.z, rotation.w);
    } else {
      pos.push(0, 0, 0);
      quat.push(0, 0, 0, 1);
    }
  }

  return {
    start: 0,
    end: maxTime,
    times,
    pos,
    quat,
  };
}

function buildPlayerTrack(playerId, playerData, times, maxTime, playerInfoById, fallbackIndex) {
  const playerInfo = playerInfoById.get(stablePlayerId(playerId));
  const playerName = playerInfo?.name ?? `Player ${fallbackIndex + 1}`;
  const team = playerInfo?.isTeamZero ? "blue" : "orange";
  const pos = [];
  const quat = [];
  const boostValues = [];
  const boostStart = [];
  const boostEnd = [];
  let activeBoostStart = null;

  for (let index = 0; index < times.length; index += 1) {
    const playerFrame = playerData.frames?.[index];
    const time = times[index] ?? 0;

    if (playerFrame?.Data?.rigid_body) {
      const { location, rotation } = playerFrame.Data.rigid_body;
      pos.push(
        scalePosition(location.x),
        scalePosition(location.y),
        scalePosition(location.z),
      );
      quat.push(rotation.x, rotation.y, rotation.z, rotation.w);

      const boostAmount = playerFrame.Data.boost_amount ?? 0;
      const boostPercent = Math.max(0, Math.min(100, Math.round((boostAmount / 255) * 100)));
      boostValues.push(boostPercent);

      if (playerFrame.Data.boost_active && activeBoostStart == null) {
        activeBoostStart = time;
      } else if (!playerFrame.Data.boost_active && activeBoostStart != null) {
        boostStart.push(activeBoostStart);
        boostEnd.push(time);
        activeBoostStart = null;
      }
    } else {
      pos.push(0, 0, 0);
      quat.push(0, 0, 0, 1);
      boostValues.push(0);
      if (activeBoostStart != null) {
        boostStart.push(activeBoostStart);
        boostEnd.push(time);
        activeBoostStart = null;
      }
    }
  }

  if (activeBoostStart != null) {
    boostStart.push(activeBoostStart);
    boostEnd.push(maxTime);
  }

  return {
    player: playerName,
    team,
    color: team === "blue" ? 0x209cee : 0xff9f43,
    cars: [
      {
        start: 0,
        end: maxTime,
        times,
        pos,
        quat,
      },
    ],
    boost_amount: {
      times,
      values: boostValues,
    },
    boost_state: {
      start: boostStart,
      end: boostEnd,
    },
    tracks: {},
    events: {},
  };
}

function buildRemainingSecondsTrack(frames, startTime) {
  return compressTrack(frames, (frame) => frame.seconds_remaining, "rem_seconds", startTime);
}

function buildScoreTrack(frames, teamKey, startTime) {
  return compressTrack(frames, (frame) => frame[teamKey]?.core?.goals ?? 0, "score", startTime);
}

function buildCountdowns(frames, startTime) {
  const countdowns = [];
  let previousState = null;

  for (const frame of frames) {
    if (
      frame.game_state === GAME_STATE_KICKOFF_COUNTDOWN &&
      previousState !== GAME_STATE_KICKOFF_COUNTDOWN
    ) {
      countdowns.push(normalizeTime(frame.time, startTime));
    }
    previousState = frame.game_state;
  }

  return countdowns;
}

function buildTicks(events, startTime) {
  return events.map((event) => ({
    time: normalizeTime(event.time, startTime),
    kind: String(event.kind).toLowerCase(),
  }));
}

function compressTrack(frames, valueSelector, valueKey, startTime) {
  const times = [];
  const values = [];
  let previousValue = Symbol("unset");

  for (const frame of frames) {
    const value = valueSelector(frame);
    if (value !== previousValue) {
      times.push(normalizeTime(frame.time, startTime));
      values.push(value);
      previousValue = value;
    }
  }

  return {
    times,
    [valueKey]: values,
  };
}

function inferMapCode(replayMeta) {
  const mapHeader = (replayMeta.all_headers ?? []).find(([key]) => key === "MapName");
  const headerValue = mapHeader?.[1];

  if (typeof headerValue === "string") {
    return headerValue;
  }
  if (headerValue?.Str) {
    return headerValue.Str;
  }
  return null;
}

function stablePlayerId(playerId) {
  return JSON.stringify(playerId);
}

function normalizeTime(time, startTime) {
  return Math.max(0, time - startTime);
}

function scalePosition(value) {
  return value * POSITION_SCALE;
}
