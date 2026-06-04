import type {
  MaterializedStatsTimeline,
  PlayerStatsSnapshot,
  StatsEvents,
  StatsFrame,
  TeamStatsSnapshot,
} from "./statsTimeline.ts";

export const STATS_EVENT_STREAM_COUNT_TYPES = [
  "timeline",
  "core_player",
  "core_player_goal_context",
  "possession",
  "pressure",
  "territorial_pressure",
  "movement",
  "positioning",
  "rotation_player",
  "rotation_team",
  "mechanics",
  "goal_context",
  "backboard",
  "ceiling_shot",
  "wall_aerial",
  "wall_aerial_shot",
  "center",
  "flick",
  "musty_flick",
  "dodge_reset",
  "double_tap",
  "fifty_fifty",
  "one_timer",
  "pass",
  "pass_last_completed",
  "ball_carry",
  "rush",
  "speed_flip",
  "half_flip",
  "half_volley",
  "wavedash",
  "whiff",
  "powerslide",
  "touch",
  "touch_ball_movement",
  "touch_last_touch",
  "boost_pickups",
  "boost_ledger",
  "boost_state",
  "bump",
] as const satisfies readonly (keyof StatsEvents)[];

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
  "musty_flick",
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

const STATS_EVENT_STREAM_COUNT_TYPE_SET = new Set<string>(STATS_EVENT_STREAM_COUNT_TYPES);
const STATS_MECHANIC_EVENT_COUNT_TYPE_SET = new Set<string>(STATS_MECHANIC_EVENT_COUNT_TYPES);

export type StatsEventCountType = (typeof STATS_EVENT_COUNT_TYPES)[number];
export type EventCountStats = Record<StatsEventCountType, number>;
type TimelineEventRecord = Record<string, unknown>;

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

function eventPlayerKey(event: TimelineEventRecord): string | null {
  return remoteIdKey(event.player ?? event.player_id ?? event.scorer);
}

function eventTeamIsTeam0(event: TimelineEventRecord): boolean | null {
  const teamValue = event.is_team_0 ?? event.scoring_team_is_team_0;
  return typeof teamValue === "boolean" ? teamValue : null;
}

function mechanicEventCountType(event: TimelineEventRecord): StatsEventCountType | null {
  const kind = event.kind;
  if (
    typeof kind !== "string" ||
    !STATS_MECHANIC_EVENT_COUNT_TYPE_SET.has(kind) ||
    STATS_EVENT_STREAM_COUNT_TYPE_SET.has(kind) ||
    !isStatsEventCountType(kind)
  ) {
    return null;
  }
  return kind;
}

function eventFrame(event: TimelineEventRecord): number | null {
  const timing = event.timing;
  const frame =
    event.resolved_frame ??
    event.frame ??
    (timing && typeof timing === "object" && "frame" in timing
      ? (timing as Record<string, unknown>).frame
      : undefined) ??
    (timing && typeof timing === "object" && "end_frame" in timing
      ? (timing as Record<string, unknown>).end_frame
      : undefined);
  return typeof frame === "number" && Number.isFinite(frame) ? frame : null;
}

function eventTime(event: TimelineEventRecord): number | null {
  const timing = event.timing;
  const time =
    event.resolved_time ??
    event.time ??
    (timing && typeof timing === "object" && "time" in timing
      ? (timing as Record<string, unknown>).time
      : undefined) ??
    (timing && typeof timing === "object" && "end_time" in timing
      ? (timing as Record<string, unknown>).end_time
      : undefined);
  return typeof time === "number" && Number.isFinite(time) ? time : null;
}

function eventOccursOnOrBeforeFrame(event: TimelineEventRecord, frame: StatsFrame): boolean {
  const frameNumber = eventFrame(event);
  if (frameNumber !== null) {
    return frameNumber <= frame.frame_number;
  }
  const time = eventTime(event);
  return time !== null && time <= frame.time;
}

function sortEvents(events: readonly unknown[]): TimelineEventRecord[] {
  return [...events]
    .filter((event): event is TimelineEventRecord => !!event && typeof event === "object")
    .sort((left, right) => {
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
  const eventGroups = STATS_EVENT_STREAM_COUNT_TYPES.map((eventType) => ({
    eventType,
    events: sortEvents(timeline.events[eventType] ?? []),
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
          const mechanicEventType =
            group.eventType === "mechanics" ? mechanicEventCountType(event) : null;
          if (playerKey !== null) {
            const counts = playerCounts.get(playerKey) ?? createEmptyEventCountStats();
            playerCounts.set(playerKey, counts);
            incrementEventCount(counts, group.eventType);
            if (mechanicEventType !== null) {
              incrementEventCount(counts, mechanicEventType);
            }
          }

          const isTeamZero = eventTeamIsTeam0(event);
          if (isTeamZero !== null) {
            const counts = isTeamZero ? teamCounts.teamZero : teamCounts.teamOne;
            incrementEventCount(counts, group.eventType);
            if (mechanicEventType !== null) {
              incrementEventCount(counts, mechanicEventType);
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
