import type { DemolitionEvent } from "./generated/DemolitionEvent.ts";
import type { DemoPlayerStats } from "./generated/DemoPlayerStats.ts";
import type { DemoTeamStats } from "./generated/DemoTeamStats.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";

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

function defaultDemoPlayerStats(): DemoPlayerStats {
  return {
    demos_inflicted: 0,
    demos_taken: 0,
  };
}

function defaultDemoTeamStats(): DemoTeamStats {
  return {
    demos_inflicted: 0,
  };
}

function sortDemolitionEvents(events: readonly DemolitionEvent[]): DemolitionEvent[] {
  return events
    .map((event, index) => ({ event, index }))
    .sort((left, right) => {
      if (left.event.time !== right.event.time) {
        return left.event.time - right.event.time;
      }
      return left.index - right.index;
    })
    .map(({ event }) => event);
}

function assignDemoPlayerStats(target: DemoPlayerStats, source: DemoPlayerStats | undefined): void {
  Object.assign(target, source ?? defaultDemoPlayerStats());
}

function assignDemoTeamStats(target: DemoTeamStats, source: DemoTeamStats): void {
  Object.assign(target, source);
}

export function applyDemoEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createDemoEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createDemoEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortDemolitionEvents(statsEventPayloads(timeline, "demolition"));

  let eventIndex = 0;
  const players = new Map<string, DemoPlayerStats>();
  const teamZero = defaultDemoTeamStats();
  const teamOne = defaultDemoTeamStats();

  function playerStats(playerId: unknown): DemoPlayerStats {
    const playerKey = remoteIdKey(playerId);
    const stats = players.get(playerKey) ?? defaultDemoPlayerStats();
    players.set(playerKey, stats);
    return stats;
  }

  return {
    applyFrame(frame: StatsFrame): void {
      while (eventIndex < events.length && events[eventIndex]!.time <= frame.time) {
        const event = events[eventIndex] as DemolitionEvent;

        playerStats(event.attacker).demos_inflicted += 1;
        if (event.attacker_is_team_0 === true) {
          teamZero.demos_inflicted += 1;
        } else if (event.attacker_is_team_0 === false) {
          teamOne.demos_inflicted += 1;
        }

        playerStats(event.victim).demos_taken += 1;

        eventIndex += 1;
      }

      assignDemoTeamStats(frame.team_zero.demo, teamZero);
      assignDemoTeamStats(frame.team_one.demo, teamOne);
      for (const player of frame.players) {
        assignDemoPlayerStats(player.demo, players.get(remoteIdKey(player.player_id)));
      }
    },
  };
}
