import type { RotationPlayerEvent } from "./generated/RotationPlayerEvent.ts";
import type { RotationPlayerStats } from "./generated/RotationPlayerStats.ts";
import type { RotationTeamEvent } from "./generated/RotationTeamEvent.ts";
import type { RotationTeamStats } from "./generated/RotationTeamStats.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";

interface RotationPlayerState {
  active: boolean;
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
    tracked_time: 0,
    time_first_man: 0,
    time_second_man: 0,
    time_third_man: 0,
    time_ambiguous_role: 0,
    time_behind_play: 0,
    time_level_with_play: 0,
    time_ahead_of_play: 0,
    became_first_man_count: 0,
    lost_first_man_count: 0,
    current_role_state: "unknown",
    current_depth_state: "unknown",
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

function applyRotationPlayerEvent(
  state: RotationPlayerState,
  event: RotationPlayerEvent,
): void {
  state.active = event.active;
  const stats = state.stats;
  stats.became_first_man_count += event.became_first_man_count;
  stats.lost_first_man_count += event.lost_first_man_count;
  stats.current_role_state = event.current_role_state;
  stats.current_depth_state = event.current_depth_state;
}

function accumulateActiveRotationFrame(state: RotationPlayerState, frame: StatsFrame): void {
  if (!state.active) {
    return;
  }

  const stats = state.stats;
  stats.active_game_time = addF32(stats.active_game_time, frame.dt);
  stats.tracked_time = addF32(stats.tracked_time, frame.dt);

  switch (stats.current_role_state) {
    case "first_man":
      stats.time_first_man = addF32(stats.time_first_man, frame.dt);
      break;
    case "second_man":
      stats.time_second_man = addF32(stats.time_second_man, frame.dt);
      break;
    case "third_man":
      stats.time_third_man = addF32(stats.time_third_man, frame.dt);
      break;
    case "ambiguous":
      stats.time_ambiguous_role = addF32(stats.time_ambiguous_role, frame.dt);
      break;
  }

  switch (stats.current_depth_state) {
    case "behind_play":
      stats.time_behind_play = addF32(stats.time_behind_play, frame.dt);
      break;
    case "level_with_play":
      stats.time_level_with_play = addF32(stats.time_level_with_play, frame.dt);
      break;
    case "ahead_of_play":
      stats.time_ahead_of_play = addF32(stats.time_ahead_of_play, frame.dt);
      break;
  }
}

function applyRotationTeamEvent(stats: RotationTeamStats, event: RotationTeamEvent): void {
  stats.first_man_changes_for_team += event.first_man_changes_for_team;
  stats.rotation_count += event.rotation_count;
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

export function applyRotationEventDerivedStats(timeline: MaterializedStatsTimeline): MaterializedStatsTimeline {
  const accumulator = createRotationEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createRotationEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const playerEvents = sortRotationEvents(timeline.events.rotation_player ?? []);
  const teamEvents = sortRotationEvents(timeline.events.rotation_team ?? []);

  let playerEventIndex = 0;
  let teamEventIndex = 0;
  const players = new Map<string, RotationPlayerState>();
  const teamZero = defaultRotationTeamStats();
  const teamOne = defaultRotationTeamStats();

  return {
    applyFrame(frame: StatsFrame): void {
      while (
        playerEventIndex < playerEvents.length &&
        playerEvents[playerEventIndex]!.frame <= frame.frame_number
      ) {
        const event = playerEvents[playerEventIndex] as RotationPlayerEvent;
        const playerKey = remoteIdKey(event.player);
        const playerState = players.get(playerKey) ?? {
          active: false,
          stats: defaultRotationPlayerStats(),
        };
        players.set(playerKey, playerState);
        applyRotationPlayerEvent(playerState, event);
        playerEventIndex += 1;
      }

      while (
        teamEventIndex < teamEvents.length &&
        teamEvents[teamEventIndex]!.frame <= frame.frame_number
      ) {
        const event = teamEvents[teamEventIndex] as RotationTeamEvent;
        applyRotationTeamEvent(event.is_team_0 ? teamZero : teamOne, event);
        teamEventIndex += 1;
      }

      assignRotationTeamStats(frame.team_zero.rotation, teamZero);
      assignRotationTeamStats(frame.team_one.rotation, teamOne);
      for (const player of frame.players) {
        const playerState = players.get(remoteIdKey(player.player_id));
        if (playerState) {
          accumulateActiveRotationFrame(playerState, frame);
        }
        assignRotationPlayerStats(player.rotation, playerState?.stats);
      }
    },
  };
}
