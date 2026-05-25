import type { PowerslideEvent } from "./generated/PowerslideEvent.ts";
import type { PowerslideStats } from "./generated/PowerslideStats.ts";
import type { StatsTimeline } from "./statsTimeline.ts";

interface PowerslideState {
  active: boolean;
  isTeamZero: boolean;
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

function defaultPowerslideStats(): PowerslideStats {
  return {
    total_duration: 0,
    press_count: 0,
  };
}

function sortPowerslideEvents(events: readonly PowerslideEvent[]): PowerslideEvent[] {
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

function frameCountsTowardPowerslide(frame: StatsTimeline["frames"][number]): boolean {
  return (
    frame.gameplay_phase === "active_play" || frame.gameplay_phase === "kickoff_waiting_for_touch"
  );
}

function assignPowerslideStats(
  target: PowerslideStats,
  source: PowerslideStats | undefined,
): void {
  Object.assign(target, source ?? defaultPowerslideStats());
}

export function applyPowerslideEventDerivedStats(timeline: StatsTimeline): StatsTimeline {
  const events = sortPowerslideEvents(timeline.events.powerslide ?? []);

  let eventIndex = 0;
  const activeStates = new Map<string, PowerslideState>();
  const players = new Map<string, PowerslideStats>();
  const teamZero = defaultPowerslideStats();
  const teamOne = defaultPowerslideStats();

  for (const frame of timeline.frames) {
    const countsTowardPowerslide = frameCountsTowardPowerslide(frame);

    while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
      const event = events[eventIndex] as PowerslideEvent;
      const playerKey = remoteIdKey(event.player);
      const previousActive = activeStates.get(playerKey)?.active ?? false;
      activeStates.set(playerKey, { active: event.active, isTeamZero: event.is_team_0 });

      if (countsTowardPowerslide && event.active && !previousActive) {
        const playerStats = players.get(playerKey) ?? defaultPowerslideStats();
        players.set(playerKey, playerStats);
        playerStats.press_count += 1;
        const teamStats = event.is_team_0 ? teamZero : teamOne;
        teamStats.press_count += 1;
      }

      eventIndex += 1;
    }

    if (countsTowardPowerslide) {
      for (const player of frame.players) {
        const playerKey = remoteIdKey(player.player_id);
        const state = activeStates.get(playerKey);
        if (!state?.active) {
          continue;
        }
        const playerStats = players.get(playerKey) ?? defaultPowerslideStats();
        players.set(playerKey, playerStats);
        playerStats.total_duration += frame.dt;
        const teamStats = player.is_team_0 ? teamZero : teamOne;
        teamStats.total_duration += frame.dt;
      }
    }

    assignPowerslideStats(frame.team_zero.powerslide, teamZero);
    assignPowerslideStats(frame.team_one.powerslide, teamOne);
    for (const player of frame.players) {
      assignPowerslideStats(player.powerslide, players.get(remoteIdKey(player.player_id)));
    }
  }

  return timeline;
}
