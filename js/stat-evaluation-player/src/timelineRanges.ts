import type { ReplayModel, ReplayTimelineRange } from "@rlrml/player";
import type { PlayerStatsSnapshot, StatsFrame, StatsTimeline } from "./statsTimeline.ts";
import { formatMechanicKind, isVisibleMechanicKind } from "./timelineMechanics.ts";
export {
  buildBoostPickupTimelineRanges,
  type BoostPickupTimelineRangeOptions,
} from "./timelineRangeBoostPickups.ts";
import {
  DELTA_EPSILON,
  mergeRange,
  mergeRangeForLane,
  resolveRangeBounds,
  sortTimelineEvents,
} from "./timelineRangeMerge.ts";
import type { PossessionEvent } from "./generated/PossessionEvent.ts";
import type { PositioningEvent } from "./generated/PositioningEvent.ts";
import type { PressureEvent } from "./generated/PressureEvent.ts";

const DEFAULT_PRESSURE_NEUTRAL_ZONE_HALF_WIDTH_Y = 200;
const BLUE_TIMELINE_COLOR = "#3b82f6";
const ORANGE_TIMELINE_COLOR = "#f59e0b";
const MECHANIC_SHORT_LABELS: Record<string, string> = {
  air_dribble: "AD",
  ball_carry: "BC",
  ceiling_shot: "CS",
  double_tap: "DT",
  flick: "F",
  half_flip: "HF",
  half_volley: "HV",
  musty_flick: "M",
  one_timer: "OT",
  pass: "P",
  wavedash: "WD",
};

type PressureHalfControlState = "team_zero_side" | "team_one_side" | "neutral";

function getPressureNeutralZoneHalfWidthY(timeline: StatsTimeline): number {
  const configured = timeline.config?.pressure_neutral_zone_half_width_y;
  if (typeof configured === "number" && Number.isFinite(configured)) {
    return Math.max(0, configured);
  }

  return DEFAULT_PRESSURE_NEUTRAL_ZONE_HALF_WIDTH_Y;
}

function getReplayFrameTime(
  replay: ReplayModel | undefined,
  frame: number | undefined,
  fallbackTime: number,
): number {
  return replay?.frames?.[frame ?? -1]?.time ?? fallbackTime;
}

function teamTimelineColor(isTeamZero: boolean | null | undefined): string | null {
  if (isTeamZero === true) {
    return BLUE_TIMELINE_COLOR;
  }
  if (isTeamZero === false) {
    return ORANGE_TIMELINE_COLOR;
  }

  return null;
}

function mechanicShortLabel(kind: string): string {
  return (
    MECHANIC_SHORT_LABELS[kind] ??
    (kind
      .split(/[_-]+/)
      .filter((part) => part.length > 0)
      .map((part) => part.slice(0, 1).toUpperCase())
      .join("")
      .slice(0, 3) ||
      "M")
  );
}

export function buildMechanicTimelineRanges(
  statsTimeline: StatsTimeline,
  replay: ReplayModel,
  enabledKinds?: Iterable<string>,
): ReplayTimelineRange[] {
  const enabled = enabledKinds ? new Set(enabledKinds) : null;
  const playerNames = new Map(replay.players.map((player) => [player.id, player.name]));

  return (statsTimeline.events.mechanics ?? [])
    .filter(
      (event) =>
        isVisibleMechanicKind(event.kind) &&
        event.timing.type === "span" &&
        (!enabled || enabled.has(event.kind)),
    )
    .map((event): ReplayTimelineRange => {
      if (event.timing.type !== "span") {
        throw new Error("unreachable non-span mechanic event");
      }

      const playerId = remoteIdToString(event.player_id as Record<string, unknown>);
      const playerName = playerNames.get(playerId) ?? playerId;
      const mechanicLabel = formatMechanicKind(event.kind);
      const startTime = getReplayFrameTime(
        replay,
        event.timing.start_frame,
        event.timing.start_time,
      );
      const endTime = Math.max(
        startTime,
        getReplayFrameTime(replay, event.timing.end_frame, event.timing.end_time),
      );

      return {
        id: event.id,
        startTime,
        endTime,
        lane: `mechanic:${event.kind}`,
        laneLabel: mechanicLabel,
        label: `${playerName} ${mechanicLabel.toLowerCase()}`,
        shortLabel: mechanicShortLabel(event.kind),
        isTeamZero: event.is_team_0,
        color: teamTimelineColor(event.is_team_0) ?? undefined,
      };
    })
    .sort((left, right) => {
      if (left.startTime !== right.startTime) {
        return left.startTime - right.startTime;
      }
      return (left.id ?? "").localeCompare(right.id ?? "");
    });
}

function resolvePressureHalfControlState(
  frameNumber: number,
  replay: ReplayModel | undefined,
  neutralZoneHalfWidthY: number,
  deltaTeamZero: number,
  deltaTeamOne: number,
  deltaNeutral: number,
): PressureHalfControlState | null {
  const ballY = replay?.ballFrames[frameNumber]?.position?.y;
  if (
    typeof ballY === "number" &&
    Number.isFinite(ballY) &&
    Math.abs(ballY) <= neutralZoneHalfWidthY + DELTA_EPSILON
  ) {
    return "neutral";
  }

  if (deltaNeutral > DELTA_EPSILON) {
    return "neutral";
  }
  if (deltaTeamZero > deltaTeamOne + DELTA_EPSILON) {
    return "team_zero_side";
  }
  if (deltaTeamOne > deltaTeamZero + DELTA_EPSILON) {
    return "team_one_side";
  }

  return null;
}

function createPressureRange(
  halfControlState: PressureHalfControlState,
  startTime: number,
  endTime: number,
): ReplayTimelineRange {
  if (halfControlState === "neutral") {
    return {
      id: `half-control:neutral:${startTime.toFixed(3)}`,
      startTime,
      endTime,
      lane: "half-control",
      laneLabel: "Half Control",
      label: "Neutral half control",
      color: "rgba(209, 217, 224, 0.7)",
      isTeamZero: null,
    };
  }

  const isTeamZero = halfControlState === "team_zero_side";
  return {
    id: `half-control:${halfControlState}:${startTime.toFixed(3)}`,
    startTime,
    endTime,
    lane: "half-control",
    laneLabel: "Half Control",
    label: isTeamZero ? "Blue half control" : "Orange half control",
    color: isTeamZero ? "rgba(89, 195, 255, 0.76)" : "rgba(255, 193, 92, 0.76)",
    isTeamZero,
  };
}

function buildPossessionTimelineRangesFromEvents(
  timeline: StatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  const events = sortTimelineEvents(timeline.events?.possession ?? []);
  const ranges: ReplayTimelineRange[] = [];
  let eventIndex = 0;
  let active = false;
  let possessionState = "neutral";

  let previousFrame: StatsTimeline["frames"][number] | null = null;
  for (const frame of timeline.frames) {
    while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
      const event = events[eventIndex] as PossessionEvent;
      active = event.active;
      possessionState = event.possession_state;
      eventIndex += 1;
    }

    if (!Number.isFinite(frame.time) || !Number.isFinite(frame.dt) || frame.dt <= 0) {
      previousFrame = frame;
      continue;
    }

    const { startTime, endTime } = resolveRangeBounds(frame, previousFrame, replay);
    let nextRange: ReplayTimelineRange | null = null;
    if (active && possessionState === "team_zero") {
      nextRange = {
        id: `possession:team_zero:${startTime.toFixed(3)}`,
        startTime,
        endTime,
        lane: "possession",
        laneLabel: "Possession",
        label: "Blue possession",
        color: "rgba(59, 130, 246, 0.88)",
        isTeamZero: true,
      };
    } else if (active && possessionState === "team_one") {
      nextRange = {
        id: `possession:team_one:${startTime.toFixed(3)}`,
        startTime,
        endTime,
        lane: "possession",
        laneLabel: "Possession",
        label: "Orange possession",
        color: "rgba(245, 158, 11, 0.88)",
        isTeamZero: false,
      };
    } else if (active && possessionState === "neutral") {
      nextRange = {
        id: `possession:neutral:${startTime.toFixed(3)}`,
        startTime,
        endTime,
        lane: "possession",
        laneLabel: "Possession",
        label: "Neutral possession",
        color: "rgba(209, 217, 224, 0.7)",
        isTeamZero: null,
      };
    }

    mergeRange(ranges, nextRange);
    previousFrame = frame;
  }

  return ranges;
}

export function buildPossessionTimelineRanges(
  timeline: StatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  if ((timeline.events?.possession?.length ?? 0) > 0) {
    return buildPossessionTimelineRangesFromEvents(timeline, replay);
  }

  const ranges: ReplayTimelineRange[] = [];

  let previousTeamZero = 0;
  let previousTeamOne = 0;
  let previousNeutral = 0;

  let previousFrame: StatsTimeline["frames"][number] | null = null;
  for (const frame of timeline.frames) {
    if (!Number.isFinite(frame.time) || !Number.isFinite(frame.dt) || frame.dt <= 0) {
      previousFrame = frame;
      continue;
    }

    const statsFrame = frame as StatsFrame;
    const currentTeamZero = statsFrame.team_zero?.possession?.possession_time ?? 0;
    const currentTeamOne = statsFrame.team_one?.possession?.possession_time ?? 0;
    const currentNeutral = statsFrame.team_zero?.possession?.neutral_time ?? 0;

    const deltaTeamZero = currentTeamZero - previousTeamZero;
    const deltaTeamOne = currentTeamOne - previousTeamOne;
    const deltaNeutral = currentNeutral - previousNeutral;

    previousTeamZero = currentTeamZero;
    previousTeamOne = currentTeamOne;
    previousNeutral = currentNeutral;

    let nextRange: ReplayTimelineRange | null = null;
    const { startTime, endTime } = resolveRangeBounds(frame, previousFrame, replay);

    if (
      deltaTeamZero > deltaTeamOne + DELTA_EPSILON &&
      deltaTeamZero > deltaNeutral + DELTA_EPSILON
    ) {
      nextRange = {
        id: `possession:team_zero:${startTime.toFixed(3)}`,
        startTime,
        endTime,
        lane: "possession",
        laneLabel: "Possession",
        label: "Blue possession",
        color: "rgba(59, 130, 246, 0.88)",
        isTeamZero: true,
      };
    } else if (
      deltaTeamOne > deltaTeamZero + DELTA_EPSILON &&
      deltaTeamOne > deltaNeutral + DELTA_EPSILON
    ) {
      nextRange = {
        id: `possession:team_one:${startTime.toFixed(3)}`,
        startTime,
        endTime,
        lane: "possession",
        laneLabel: "Possession",
        label: "Orange possession",
        color: "rgba(245, 158, 11, 0.88)",
        isTeamZero: false,
      };
    } else if (deltaNeutral > DELTA_EPSILON) {
      nextRange = {
        id: `possession:neutral:${startTime.toFixed(3)}`,
        startTime,
        endTime,
        lane: "possession",
        laneLabel: "Possession",
        label: "Neutral possession",
        color: "rgba(209, 217, 224, 0.7)",
        isTeamZero: null,
      };
    }

    mergeRange(ranges, nextRange);
    previousFrame = frame;
  }

  return ranges;
}

function buildPressureTimelineRangesFromEvents(
  timeline: StatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  const events = sortTimelineEvents(timeline.events?.pressure ?? []);
  const ranges: ReplayTimelineRange[] = [];
  let eventIndex = 0;
  let active = false;
  let fieldHalf: PressureHalfControlState = "neutral";

  let previousFrame: StatsTimeline["frames"][number] | null = null;
  for (const frame of timeline.frames) {
    while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
      const event = events[eventIndex] as PressureEvent;
      active = event.active;
      fieldHalf =
        event.field_half === "team_zero_side" || event.field_half === "team_one_side"
          ? event.field_half
          : "neutral";
      eventIndex += 1;
    }

    if (!Number.isFinite(frame.time) || !Number.isFinite(frame.dt) || frame.dt <= 0) {
      previousFrame = frame;
      continue;
    }

    const { startTime, endTime } = resolveRangeBounds(frame, previousFrame, replay);
    mergeRange(ranges, active ? createPressureRange(fieldHalf, startTime, endTime) : null);
    previousFrame = frame;
  }

  return ranges;
}

export function buildPressureTimelineRanges(
  timeline: StatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  if ((timeline.events?.pressure?.length ?? 0) > 0) {
    return buildPressureTimelineRangesFromEvents(timeline, replay);
  }

  const ranges: ReplayTimelineRange[] = [];

  let previousTeamZero = 0;
  let previousTeamOne = 0;
  let previousNeutral = 0;
  const neutralZoneHalfWidthY = getPressureNeutralZoneHalfWidthY(timeline);

  let previousFrame: StatsTimeline["frames"][number] | null = null;
  for (const frame of timeline.frames) {
    if (!Number.isFinite(frame.time) || !Number.isFinite(frame.dt) || frame.dt <= 0) {
      previousFrame = frame;
      continue;
    }

    const statsFrame = frame as StatsFrame;
    const currentTeamZero = statsFrame.team_zero?.pressure?.defensive_half_time ?? 0;
    const currentTeamOne = statsFrame.team_one?.pressure?.defensive_half_time ?? 0;
    const currentNeutral = statsFrame.team_zero?.pressure?.neutral_time ?? 0;
    const deltaTeamZero = currentTeamZero - previousTeamZero;
    const deltaTeamOne = currentTeamOne - previousTeamOne;
    const deltaNeutral = currentNeutral - previousNeutral;

    previousTeamZero = currentTeamZero;
    previousTeamOne = currentTeamOne;
    previousNeutral = currentNeutral;

    const { startTime, endTime } = resolveRangeBounds(frame, previousFrame, replay);
    const halfControlState = resolvePressureHalfControlState(
      frame.frame_number,
      replay,
      neutralZoneHalfWidthY,
      deltaTeamZero,
      deltaTeamOne,
      deltaNeutral,
    );
    const nextRange = halfControlState
      ? createPressureRange(halfControlState, startTime, endTime)
      : null;

    mergeRange(ranges, nextRange);
    previousFrame = frame;
  }

  return ranges;
}

export function buildRushTimelineRanges(
  timeline: StatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  return timeline.events.rush.map((event, index) => {
    const startTime = replay?.frames[event.start_frame]?.time ?? event.start_time;
    const endTime = replay?.frames[event.end_frame]?.time ?? event.end_time;
    const matchupLabel = `${event.attackers}v${event.defenders}`;
    const isTeamZero = event.is_team_0;

    return {
      id: `rush-range:${event.start_frame}:${event.end_frame}:${index}`,
      startTime,
      endTime: Math.max(startTime, endTime),
      lane: "rush",
      laneLabel: "Rush",
      label: `${isTeamZero ? "Blue" : "Orange"} rush ${matchupLabel}`,
      color: isTeamZero ? "rgba(59, 130, 246, 0.4)" : "rgba(245, 158, 11, 0.4)",
      isTeamZero,
    };
  });
}

function remoteIdToString(playerId: Record<string, unknown>): string {
  const [kind, value] = Object.entries(playerId)[0] ?? ["Unknown", "unknown"];
  const normalizedValue = typeof value === "string" ? value : JSON.stringify(value);
  return `${kind}:${normalizedValue}`;
}

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
