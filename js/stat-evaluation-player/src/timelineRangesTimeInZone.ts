import type { ReplayModel, ReplayTimelineRange } from "@rlrml/player";
import type {
  PlayerStatsSnapshot,
  StatsFrame,
  StatsTimeline,
} from "./statsTimeline.ts";
import type { PositioningEvent } from "./generated/PositioningEvent.ts";
import {
  DELTA_EPSILON,
  mergeRangeForLane,
  resolveRangeBounds,
  sortTimelineEvents,
} from "./timelineRangeMerge.ts";

interface PlayerZoneSpec {
  fieldName: string;
  aliases?: string[];
  label: string;
  relativeColor: "own" | "neutral" | "opp";
}

const PLAYER_ZONE_SPECS: PlayerZoneSpec[] = [
  {
    fieldName: "time_defensive_third",
    aliases: ["time_defensive_zone"],
    label: "Def third",
    relativeColor: "own",
  },
  {
    fieldName: "time_neutral_third",
    aliases: ["time_neutral_zone"],
    label: "Neutral third",
    relativeColor: "neutral",
  },
  {
    fieldName: "time_offensive_third",
    aliases: ["time_offensive_zone"],
    label: "Off third",
    relativeColor: "opp",
  },
];

function getPlayerZoneColor(spec: PlayerZoneSpec, isTeamZero: boolean): string {
  if (spec.relativeColor === "neutral") {
    return "rgba(209, 217, 224, 0.68)";
  }

  const isOwnTeamColor = spec.relativeColor === "own";
  const shouldUseBlue = isOwnTeamColor ? isTeamZero : !isTeamZero;
  return shouldUseBlue ? "rgba(89, 195, 255, 0.74)" : "rgba(255, 193, 92, 0.78)";
}

function playerIdToString(playerId: Record<string, unknown>): string {
  const [kind, value] = Object.entries(playerId)[0] ?? ["Unknown", "unknown"];
  const normalizedValue = typeof value === "string" ? value : JSON.stringify(value);
  return `${kind}:${normalizedValue}`;
}

function extractPlayerStatValue(player: PlayerStatsSnapshot, spec: PlayerZoneSpec): number {
  const positioning = player.positioning as Record<string, unknown> | undefined;
  if (!positioning) {
    return 0;
  }

  for (const fieldName of [spec.fieldName, ...(spec.aliases ?? [])]) {
    const value = positioning[fieldName];
    if (typeof value === "number" && Number.isFinite(value)) {
      return value;
    }
  }

  return 0;
}

function playerNameById(frame: StatsTimeline["frames"][number], playerId: string): string {
  return (
    frame.players.find((player) => playerIdToString(player.player_id) === playerId)?.name ??
    playerId
  );
}

function positioningZoneValue(event: PositioningEvent, spec: PlayerZoneSpec): number {
  for (const fieldName of [spec.fieldName, ...(spec.aliases ?? [])]) {
    const value = (event as unknown as Record<string, unknown>)[fieldName];
    if (typeof value === "number" && Number.isFinite(value)) {
      return value;
    }
  }

  return 0;
}

function buildTimeInZoneTimelineRangesFromEvents(
  timeline: StatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  const events = sortTimelineEvents(timeline.events?.positioning ?? []);
  const ranges: ReplayTimelineRange[] = [];
  const lastRangeByLane = new Map<string, ReplayTimelineRange>();
  let eventIndex = 0;

  let previousFrame: StatsTimeline["frames"][number] | null = null;
  for (const frame of timeline.frames) {
    const eventsByPlayer = new Map<
      string,
      { event: PositioningEvent; zoneDeltas: Map<string, number> }
    >();
    while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
      const event = events[eventIndex] as PositioningEvent;
      const playerId = playerIdToString(event.player as Record<string, unknown>);
      const entry = eventsByPlayer.get(playerId) ?? {
        event,
        zoneDeltas: new Map<string, number>(),
      };
      entry.event = event;
      for (const spec of PLAYER_ZONE_SPECS) {
        entry.zoneDeltas.set(
          spec.fieldName,
          (entry.zoneDeltas.get(spec.fieldName) ?? 0) + positioningZoneValue(event, spec),
        );
      }
      eventsByPlayer.set(playerId, entry);
      eventIndex += 1;
    }

    if (!Number.isFinite(frame.time) || !Number.isFinite(frame.dt) || frame.dt <= 0) {
      previousFrame = frame;
      continue;
    }

    const { startTime, endTime } = resolveRangeBounds(frame, previousFrame, replay);
    if (endTime - startTime <= DELTA_EPSILON) {
      previousFrame = frame;
      continue;
    }

    for (const [playerId, { event, zoneDeltas }] of eventsByPlayer) {
      let winningSpec: PlayerZoneSpec | null = null;
      let winningDelta = 0;

      for (const spec of PLAYER_ZONE_SPECS) {
        const delta = zoneDeltas.get(spec.fieldName) ?? 0;
        if (delta > winningDelta + DELTA_EPSILON) {
          winningDelta = delta;
          winningSpec = spec;
        }
      }

      if (!winningSpec) {
        continue;
      }

      mergeRangeForLane(ranges, lastRangeByLane, {
        id: `time-in-zone:${playerId}:${winningSpec.fieldName}:${startTime.toFixed(3)}`,
        startTime,
        endTime,
        lane: `time-in-zone:${playerId}`,
        laneLabel: playerNameById(frame, playerId),
        label: winningSpec.label,
        color: getPlayerZoneColor(winningSpec, event.is_team_0),
        isTeamZero: event.is_team_0,
      });
    }

    previousFrame = frame;
  }

  return ranges;
}

export function buildTimeInZoneTimelineRanges(
  timeline: StatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  if ((timeline.events?.positioning?.length ?? 0) > 0) {
    return buildTimeInZoneTimelineRangesFromEvents(timeline, replay);
  }

  const previousValues = new Map<string, Map<string, number>>();
  const ranges: ReplayTimelineRange[] = [];
  const lastRangeByLane = new Map<string, ReplayTimelineRange>();

  let previousFrame: StatsTimeline["frames"][number] | null = null;
  for (const frame of timeline.frames) {
    if (!Number.isFinite(frame.time) || !Number.isFinite(frame.dt) || frame.dt <= 0) {
      previousFrame = frame;
      continue;
    }

    const { startTime, endTime } = resolveRangeBounds(frame, previousFrame, replay);
    if (endTime - startTime <= DELTA_EPSILON) {
      previousFrame = frame;
      continue;
    }

    for (const player of (frame as StatsFrame).players) {
      const playerId = playerIdToString(player.player_id);
      const previous = previousValues.get(playerId) ?? new Map<string, number>();

      let winningSpec: PlayerZoneSpec | null = null;
      let winningDelta = 0;

      for (const spec of PLAYER_ZONE_SPECS) {
        const value = extractPlayerStatValue(player, spec);
        const delta = value - (previous.get(spec.fieldName) ?? 0);
        if (delta > winningDelta + DELTA_EPSILON) {
          winningDelta = delta;
          winningSpec = spec;
        }
        previous.set(spec.fieldName, value);
      }

      previousValues.set(playerId, previous);

      if (!winningSpec) {
        continue;
      }

      mergeRangeForLane(ranges, lastRangeByLane, {
        id: `time-in-zone:${playerId}:${winningSpec.fieldName}:${startTime.toFixed(3)}`,
        startTime,
        endTime,
        lane: `time-in-zone:${playerId}`,
        laneLabel: player.name,
        label: winningSpec.label,
        color: getPlayerZoneColor(winningSpec, player.is_team_0),
        isTeamZero: player.is_team_0,
      });
    }

    previousFrame = frame;
  }

  return ranges;
}
