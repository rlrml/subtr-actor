export function buildPlayerNameMap(replayMeta) {
  const nameMap = new Map();
  for (const player of [...(replayMeta.team_zero ?? []), ...(replayMeta.team_one ?? [])]) {
    nameMap.set(stablePlayerId(player.remote_id), player.name);
  }
  return nameMap;
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

function stablePlayerId(playerId) {
  return JSON.stringify(playerId);
}

function normalizeTime(time, startTime) {
  return Math.max(0, time - startTime);
}
