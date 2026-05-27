import type {
  RawDemolishInfo,
  RawGoalEvent,
  RawPlayerStatEvent,
  RawReplayFramesData,
  ReplayPlayerTrack,
  ReplayTimelineEvent,
} from "./types";
import { normalizeReplayTime, playerIdToString } from "./replay-data-utils";

interface TimelineProgressTracker {
  advance(units?: number): boolean;
}

interface AsyncTimelineProgressTracker extends TimelineProgressTracker {
  yieldToMainThread(): Promise<void>;
}

export function buildTimelineEvents(
  raw: RawReplayFramesData,
  players: ReplayPlayerTrack[],
  startTime: number,
  progressTracker?: TimelineProgressTracker,
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

  return sortTimelineEvents(timelineEvents);
}

export async function buildTimelineEventsAsync(
  raw: RawReplayFramesData,
  players: ReplayPlayerTrack[],
  startTime: number,
  progressTracker: AsyncTimelineProgressTracker,
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

  return sortTimelineEvents(timelineEvents);
}

function buildPlayerLookup(players: ReplayPlayerTrack[]): Map<string, ReplayPlayerTrack> {
  return new Map(players.map((player) => [player.id, player]));
}

function createTimelineEventId(prefix: string, frame: number, suffix: string): string {
  return `${prefix}:${frame}:${suffix}`;
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
    location: event.shot?.ball_position ?? null,
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

function sortTimelineEvents(timelineEvents: ReplayTimelineEvent[]): ReplayTimelineEvent[] {
  return timelineEvents.sort((left, right) => {
    if (left.time !== right.time) {
      return left.time - right.time;
    }
    return (left.frame ?? 0) - (right.frame ?? 0);
  });
}
