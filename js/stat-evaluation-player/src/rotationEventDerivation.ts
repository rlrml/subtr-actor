import type { RotationPlayerEvent } from "./generated/RotationPlayerEvent.ts";
import type { RotationPlayerStats } from "./generated/RotationPlayerStats.ts";
import type { RotationTeamEvent } from "./generated/RotationTeamEvent.ts";
import type { RotationTeamStats } from "./generated/RotationTeamStats.ts";
import type { StatsTimeline } from "./statsTimeline.ts";

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
  stats: RotationPlayerStats,
  event: RotationPlayerEvent,
): void {
  stats.active_game_time += event.active_game_time;
  stats.tracked_time += event.tracked_time;
  stats.time_first_man += event.time_first_man;
  stats.time_second_man += event.time_second_man;
  stats.time_third_man += event.time_third_man;
  stats.time_ambiguous_role += event.time_ambiguous_role;
  stats.time_behind_play += event.time_behind_play;
  stats.time_level_with_play += event.time_level_with_play;
  stats.time_ahead_of_play += event.time_ahead_of_play;
  stats.became_first_man_count += event.became_first_man_count;
  stats.lost_first_man_count += event.lost_first_man_count;
  stats.current_role_state = event.current_role_state;
  stats.current_depth_state = event.current_depth_state;
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

export function applyRotationEventDerivedStats(timeline: StatsTimeline): StatsTimeline {
  const playerEvents = sortRotationEvents(timeline.events.rotation_player ?? []);
  const teamEvents = sortRotationEvents(timeline.events.rotation_team ?? []);

  let playerEventIndex = 0;
  let teamEventIndex = 0;
  const players = new Map<string, RotationPlayerStats>();
  const teamZero = defaultRotationTeamStats();
  const teamOne = defaultRotationTeamStats();

  for (const frame of timeline.frames) {
    while (
      playerEventIndex < playerEvents.length &&
      playerEvents[playerEventIndex]!.frame <= frame.frame_number
    ) {
      const event = playerEvents[playerEventIndex] as RotationPlayerEvent;
      const playerKey = remoteIdKey(event.player);
      const playerStats = players.get(playerKey) ?? defaultRotationPlayerStats();
      players.set(playerKey, playerStats);
      applyRotationPlayerEvent(playerStats, event);
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
      assignRotationPlayerStats(player.rotation, players.get(remoteIdKey(player.player_id)));
    }
  }

  return timeline;
}
