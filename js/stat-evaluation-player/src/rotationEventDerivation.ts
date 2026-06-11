import type { FirstManChangeEvent } from "./generated/FirstManChangeEvent.ts";
import type { RotationPlayerStats } from "./generated/RotationPlayerStats.ts";
import type { RotationTeamStats } from "./generated/RotationTeamStats.ts";
import type { RotationRoleEvent, StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";

interface RotationPlayerState {
  currentFirstManStintTime: number;
  lastFirstManEndTime: number | null;
  stats: RotationPlayerStats;
}

function addF32(left: number, right: number): number {
  return Math.fround(Math.fround(left) + Math.fround(right));
}

function remoteIdKey(playerId: unknown): string {
  if (!playerId || typeof playerId !== "object") {
    return String(playerId);
  }
  const [kind, value] = Object.entries(playerId as Record<string, unknown>)[0] ?? [
    "Unknown",
    "unknown",
  ];
  return `${kind}:${typeof value === "string" ? value : JSON.stringify(value)}`;
}

function defaultRotationPlayerStats(): RotationPlayerStats {
  return {
    active_game_time: 0,
    time_first_man: 0,
    time_second_man: 0,
    time_third_man: 0,
    time_ambiguous_role: 0,
    longest_first_man_stint_time: 0,
    first_man_stint_count: 0,
    became_first_man_count: 0,
    lost_first_man_count: 0,
    current_role_state: "unknown",
  };
}

function defaultRotationTeamStats(): RotationTeamStats {
  return {
    first_man_changes_for_team: 0,
    rotation_count: 0,
  };
}

function sortRotationEvents<T extends { time: number; frame: number }>(events: readonly T[]): T[] {
  return events
    .map((event, index) => ({ event, index }))
    .sort((left, right) => {
      if (left.event.frame !== right.event.frame) {
        return left.event.frame - right.event.frame;
      }
      if (left.event.time !== right.event.time) {
        return left.event.time - right.event.time;
      }
      return left.index - right.index;
    })
    .map(({ event }) => event);
}

/**
 * Credit a portion of a role span to the player's stats. Mirrors the Rust
 * `RotationStatsAccumulator`: first-man stints continue while the gap between
 * consecutive first-man spans stays within the configured grace, and partial
 * credits of one open span extend the same stint.
 */
function creditRoleSpan(
  state: RotationPlayerState,
  event: RotationRoleEvent,
  delta: number,
  creditedThroughTime: number,
  firstManStintEndGraceSeconds: number,
): void {
  const stats = state.stats;
  stats.active_game_time = addF32(stats.active_game_time, delta);
  stats.current_role_state = event.state;
  switch (event.state) {
    case "first_man": {
      const continuesStint =
        state.lastFirstManEndTime !== null &&
        event.time - state.lastFirstManEndTime <= firstManStintEndGraceSeconds;
      if (continuesStint) {
        state.currentFirstManStintTime = addF32(state.currentFirstManStintTime, delta);
      } else {
        state.currentFirstManStintTime = delta;
        stats.first_man_stint_count += 1;
      }
      state.lastFirstManEndTime = creditedThroughTime;
      stats.longest_first_man_stint_time = Math.max(
        stats.longest_first_man_stint_time,
        state.currentFirstManStintTime,
      );
      stats.time_first_man = addF32(stats.time_first_man, delta);
      break;
    }
    case "second_man":
      stats.time_second_man = addF32(stats.time_second_man, delta);
      break;
    case "third_man":
      stats.time_third_man = addF32(stats.time_third_man, delta);
      break;
    case "ambiguous":
      stats.time_ambiguous_role = addF32(stats.time_ambiguous_role, delta);
      break;
    case "unknown":
      break;
  }
}

function creditedDurationThroughFrame(event: RotationRoleEvent, frame: StatsFrame): number {
  if (frame.frame_number >= event.end_frame) {
    return event.duration;
  }
  const totalTime = event.end_time - event.time;
  if (totalTime <= 0) {
    return 0;
  }
  const elapsedTime = Math.max(0, frame.time - event.time);
  return event.duration * Math.min(1, elapsedTime / totalTime);
}

function getRotationPlayerState(
  players: Map<string, RotationPlayerState>,
  playerId: unknown,
): RotationPlayerState {
  const playerKey = remoteIdKey(playerId);
  const playerState = players.get(playerKey) ?? {
    currentFirstManStintTime: 0,
    lastFirstManEndTime: null,
    stats: defaultRotationPlayerStats(),
  };
  players.set(playerKey, playerState);
  return playerState;
}

function applyFirstManChangeEvent(
  stats: RotationTeamStats,
  players: Map<string, RotationPlayerState>,
  event: FirstManChangeEvent,
): void {
  stats.first_man_changes_for_team += 1;
  stats.rotation_count += 1;
  getRotationPlayerState(players, event.previous_first_man).stats.lost_first_man_count += 1;
  getRotationPlayerState(players, event.next_first_man).stats.became_first_man_count += 1;
}

function assignRotationPlayerStats(
  target: RotationPlayerStats,
  source: RotationPlayerStats | undefined,
): void {
  Object.assign(target, source ?? defaultRotationPlayerStats());
}

function assignRotationTeamStats(target: RotationTeamStats, source: RotationTeamStats): void {
  Object.assign(target, source);
}

export function applyRotationEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createRotationEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createRotationEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const roleEvents = sortRotationEvents(statsEventPayloads(timeline, "rotation_role"));
  const changeEvents = sortRotationEvents(statsEventPayloads(timeline, "first_man_change"));
  const firstManStintEndGraceSeconds = timeline.config.rotation_first_man_stint_end_grace_seconds;

  const creditedDurations = new Array(roleEvents.length).fill(0) as number[];
  let changeEventIndex = 0;
  const players = new Map<string, RotationPlayerState>();
  const teamZero = defaultRotationTeamStats();
  const teamOne = defaultRotationTeamStats();

  return {
    applyFrame(frame: StatsFrame): void {
      for (let index = 0; index < roleEvents.length; index += 1) {
        const event = roleEvents[index]!;
        if (event.frame > frame.frame_number) {
          break;
        }
        const targetDuration = creditedDurationThroughFrame(event, frame);
        const delta = targetDuration - creditedDurations[index]!;
        if (delta > 0) {
          creditedDurations[index] = targetDuration;
          const creditedThroughTime =
            frame.frame_number >= event.end_frame
              ? event.end_time
              : Math.min(frame.time, event.end_time);
          creditRoleSpan(
            getRotationPlayerState(players, event.player),
            event,
            delta,
            creditedThroughTime,
            firstManStintEndGraceSeconds,
          );
        }
      }

      while (
        changeEventIndex < changeEvents.length &&
        changeEvents[changeEventIndex]!.frame <= frame.frame_number
      ) {
        const event = changeEvents[changeEventIndex]!;
        applyFirstManChangeEvent(event.is_team_0 ? teamZero : teamOne, players, event);
        changeEventIndex += 1;
      }

      assignRotationTeamStats(frame.team_zero.rotation, teamZero);
      assignRotationTeamStats(frame.team_one.rotation, teamOne);
      for (const player of frame.players) {
        const playerState = players.get(remoteIdKey(player.player_id));
        assignRotationPlayerStats(player.rotation, playerState?.stats);
      }
    },
  };
}
