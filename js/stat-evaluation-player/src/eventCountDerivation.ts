import type {
  Event,
  MaterializedStatsTimeline,
  PlayerStatsSnapshot,
  StatsFrame,
  TeamStatsSnapshot,
} from "./statsTimeline.ts";
import { statsEventEnvelopes } from "./statsTimeline.ts";
import { isVisibleMechanicKind } from "./timelinePresentation.ts";

export const STATS_EVENT_STREAM_COUNT_TYPES = [
  "timeline",
  "core_player",
  "player_possession",
  "possession",
  "ball_half",
  "ball_third",
  "territorial_pressure",
  "movement",
  "player_activity",
  "field_third",
  "field_half",
  "ball_depth",
  "depth_role",
  "ball_proximity",
  "shadow_defense",
  "rotation_role",
  "first_man_change",
  "goal_context",
  "backboard",
  "ceiling_shot",
  "wall_aerial",
  "wall_aerial_shot",
  "center",
  "flick",
  "flip_reset",
  "dodge_reset",
  "double_tap",
  "fifty_fifty",
  "kickoff",
  "one_timer",
  "pass",
  "ball_carry",
  "controlled_play",
  "rush",
  "dodge",
  "speed_flip",
  "half_flip",
  "half_volley",
  "wavedash",
  "whiff",
  "powerslide",
  "touch",
  "boost_pickups",
  "boost_respawn",
  "bump",
  "demolition",
] as const;

export const STATS_MECHANIC_EVENT_COUNT_TYPES = [
  "air_dribble",
  "ball_carry",
  "ceiling_shot",
  "center",
  "double_tap",
  "flick",
  "flip_reset",
  "half_flip",
  "half_volley",
  "one_timer",
  "pass",
  "speed_flip",
  "wall_aerial",
  "wall_aerial_shot",
  "wavedash",
] as const;

export const STATS_EVENT_COUNT_TYPES = [
  ...new Set([...STATS_EVENT_STREAM_COUNT_TYPES, ...STATS_MECHANIC_EVENT_COUNT_TYPES]),
] as const;

const STATS_MECHANIC_EVENT_COUNT_TYPE_SET = new Set<string>(STATS_MECHANIC_EVENT_COUNT_TYPES);

export type StatsEventCountType = (typeof STATS_EVENT_COUNT_TYPES)[number];
export type EventCountStats = Record<StatsEventCountType, number>;

export type EventCountedPlayerStatsSnapshot = PlayerStatsSnapshot & {
  event_counts: EventCountStats;
};

export type EventCountedTeamStatsSnapshot = TeamStatsSnapshot & {
  event_counts: EventCountStats;
};

export function createEmptyEventCountStats(): EventCountStats {
  return Object.fromEntries(
    STATS_EVENT_COUNT_TYPES.map((eventType) => [eventType, 0]),
  ) as EventCountStats;
}

function cloneEventCounts(counts: EventCountStats | undefined): EventCountStats {
  return { ...(counts ?? createEmptyEventCountStats()) };
}

function incrementEventCount(counts: EventCountStats, eventType: StatsEventCountType): void {
  counts[eventType] += 1;
}

function isStatsEventCountType(value: string): value is StatsEventCountType {
  return (STATS_EVENT_COUNT_TYPES as readonly string[]).includes(value);
}

function remoteIdKey(playerId: unknown): string | null {
  if (playerId === null || playerId === undefined) {
    return null;
  }
  if (!playerId || typeof playerId !== "object") {
    return String(playerId);
  }
  const [kind, value] = Object.entries(playerId as Record<string, unknown>)[0] ?? [
    "Unknown",
    "unknown",
  ];
  return `${kind}:${typeof value === "string" ? value : JSON.stringify(value)}`;
}

function eventPlayerKey(event: Event): string | null {
  return remoteIdKey(event.meta.primary_player);
}

function eventTeamIsTeam0(event: Event): boolean | null {
  return event.meta.team_is_team_0 ?? null;
}

function mechanicEventCountType(event: Event): StatsEventCountType | null {
  const kind = event.meta.stream;
  if (!STATS_MECHANIC_EVENT_COUNT_TYPE_SET.has(kind) || !isStatsEventCountType(kind)) {
    return null;
  }
  return kind;
}

function eventFrame(event: Event): number | null {
  const frame =
    event.meta.timing.type === "span" ? event.meta.timing.end_frame : event.meta.timing.frame;
  return typeof frame === "number" && Number.isFinite(frame) ? frame : null;
}

function eventTime(event: Event): number | null {
  const time =
    event.meta.timing.type === "span" ? event.meta.timing.end_time : event.meta.timing.time;
  return typeof time === "number" && Number.isFinite(time) ? time : null;
}

function eventOccursOnOrBeforeFrame(event: Event, frame: StatsFrame): boolean {
  const frameNumber = eventFrame(event);
  if (frameNumber !== null) {
    return frameNumber <= frame.frame_number;
  }
  const time = eventTime(event);
  return time !== null && time <= frame.time;
}

function sortEvents(events: readonly Event[]): Event[] {
  return [...events].sort((left, right) => {
    const leftFrame = eventFrame(left);
    const rightFrame = eventFrame(right);
    if (leftFrame !== rightFrame) {
      return (leftFrame ?? Number.POSITIVE_INFINITY) - (rightFrame ?? Number.POSITIVE_INFINITY);
    }

    const leftTime = eventTime(left);
    const rightTime = eventTime(right);
    if (leftTime !== rightTime) {
      return (leftTime ?? Number.POSITIVE_INFINITY) - (rightTime ?? Number.POSITIVE_INFINITY);
    }

    return (eventPlayerKey(left) ?? "").localeCompare(eventPlayerKey(right) ?? "");
  });
}

export function applyEventCountDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createEventCountDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createEventCountDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const eventGroups = STATS_EVENT_COUNT_TYPES.map((eventType) => ({
    eventType,
    events: sortEvents(
      statsEventEnvelopes(timeline).filter((event) => event.meta.stream === eventType),
    ),
    index: 0,
  }));
  const playerCounts = new Map<string, EventCountStats>();
  const teamCounts = {
    teamZero: createEmptyEventCountStats(),
    teamOne: createEmptyEventCountStats(),
  };

  return {
    applyFrame(frame: StatsFrame): void {
      for (const group of eventGroups) {
        while (
          group.index < group.events.length &&
          eventOccursOnOrBeforeFrame(group.events[group.index]!, frame)
        ) {
          const event = group.events[group.index]!;
          const playerKey = eventPlayerKey(event);
          const mechanicEventType = mechanicEventCountType(event);
          if (playerKey !== null) {
            const counts = playerCounts.get(playerKey) ?? createEmptyEventCountStats();
            playerCounts.set(playerKey, counts);
            incrementEventCount(counts, group.eventType);
            if (mechanicEventType !== null && mechanicEventType !== group.eventType) {
              incrementEventCount(counts, mechanicEventType);
            }
            if (isVisibleMechanicKind(event.meta.stream)) {
              (counts as Record<string, number>).mechanics =
                ((counts as Record<string, number>).mechanics ?? 0) + 1;
            }
          }

          const isTeamZero = eventTeamIsTeam0(event);
          if (isTeamZero !== null) {
            const counts = isTeamZero ? teamCounts.teamZero : teamCounts.teamOne;
            incrementEventCount(counts, group.eventType);
            if (mechanicEventType !== null && mechanicEventType !== group.eventType) {
              incrementEventCount(counts, mechanicEventType);
            }
            if (isVisibleMechanicKind(event.meta.stream)) {
              (counts as Record<string, number>).mechanics =
                ((counts as Record<string, number>).mechanics ?? 0) + 1;
            }
          }

          group.index += 1;
        }
      }

      for (const player of frame.players as EventCountedPlayerStatsSnapshot[]) {
        const playerKey = remoteIdKey(player.player_id);
        player.event_counts = cloneEventCounts(
          playerKey === null ? undefined : playerCounts.get(playerKey),
        );
      }

      (frame.team_zero as EventCountedTeamStatsSnapshot).event_counts = cloneEventCounts(
        teamCounts.teamZero,
      );
      (frame.team_one as EventCountedTeamStatsSnapshot).event_counts = cloneEventCounts(
        teamCounts.teamOne,
      );
    },
  };
}
