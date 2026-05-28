import type { ReplayBoostPadSize, ReplayModel, ReplayTimelineRange } from "@rlrml/player";
import type { PlayerStatsSnapshot, StatsFrame, StatsTimeline } from "./statsTimeline.ts";
import { formatMechanicKind, isVisibleMechanicKind } from "./timelineMarkers.ts";
import type { BoostPickupActivity } from "./generated/BoostPickupActivity.ts";
import type { BoostPickupComparison } from "./generated/BoostPickupComparison.ts";
import type { BoostPickupFieldHalf } from "./generated/BoostPickupFieldHalf.ts";
import type { BoostPickupPadType } from "./generated/BoostPickupPadType.ts";
import type { PossessionEvent } from "./generated/PossessionEvent.ts";
import type { PositioningEvent } from "./generated/PositioningEvent.ts";
import type { PressureEvent } from "./generated/PressureEvent.ts";

const RANGE_MERGE_EPSILON_SECONDS = 0.02;
const DELTA_EPSILON = 0.0001;
const DEFAULT_PRESSURE_NEUTRAL_ZONE_HALF_WIDTH_Y = 200;
const BOOST_PICKUP_TICK_SECONDS = 0.08;
const BLUE_TIMELINE_COLOR = "#3b82f6";
const ORANGE_TIMELINE_COLOR = "#f59e0b";
const BOOST_PICKUP_COLORS: Record<ReplayBoostPadSize, string> = {
  big: "rgba(245, 158, 11, 0.92)",
  small: "rgba(52, 211, 153, 0.86)",
};
const BOOST_PICKUP_COMPARISON_COLORS: Record<BoostPickupComparison, string> = {
  both: "rgba(52, 211, 153, 0.86)",
  ghost: "rgba(239, 68, 68, 0.9)",
  missed: "rgba(59, 130, 246, 0.9)",
};
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

export interface BoostPickupTimelineRangeOptions {
  sizes?: Iterable<ReplayBoostPadSize>;
  padTypes?: Iterable<BoostPickupPadType>;
  comparisons?: Iterable<BoostPickupComparison>;
  activities?: Iterable<BoostPickupActivity>;
  fieldHalves?: Iterable<BoostPickupFieldHalf>;
  playerIds?: Iterable<string>;
}

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

function sortTimelineEvents<T extends { frame: number; time: number }>(events: readonly T[]): T[] {
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

function buildReplayBoostPickupTimelineRanges(
  replay: ReplayModel,
  options: BoostPickupTimelineRangeOptions = {},
): ReplayTimelineRange[] {
  const enabledPadTypes = padTypesFromOptions(options);
  const enabledComparisons = new Set<BoostPickupComparison>(options.comparisons ?? ["both"]);
  const enabledActivities = new Set<BoostPickupActivity>(
    options.activities ?? ["active", "inactive", "unknown"],
  );
  const enabledFieldHalves = new Set<BoostPickupFieldHalf>(
    options.fieldHalves ?? ["own", "opponent", "unknown"],
  );
  const enabledPlayerIds = options.playerIds ? new Set(options.playerIds) : null;
  if (
    enabledPadTypes.size === 0 ||
    !enabledComparisons.has("both") ||
    !enabledActivities.has("unknown") ||
    !enabledFieldHalves.has("unknown") ||
    enabledPlayerIds?.size === 0
  ) {
    return [];
  }

  const playerTeams = new Map(replay.players.map((player) => [player.id, player.isTeamZero]));
  const ranges: ReplayTimelineRange[] = [];

  for (const pad of replay.boostPads) {
    if (!enabledPadTypes.has(pad.size)) {
      continue;
    }

    for (let eventIndex = 0; eventIndex < pad.events.length; eventIndex += 1) {
      const event = pad.events[eventIndex]!;
      if (event.available || !Number.isFinite(event.time)) {
        continue;
      }
      if (enabledPlayerIds && !event.playerId) {
        continue;
      }
      if (event.playerId && enabledPlayerIds && !enabledPlayerIds.has(event.playerId)) {
        continue;
      }

      const startTime = Math.max(0, getReplayFrameTime(replay, event.frame, event.time));
      const sizeLabel = pad.size === "big" ? "Big" : "Small";
      const playerLabel = event.playerName ? `${event.playerName} ` : "";
      const isTeamZero = event.playerId ? (playerTeams.get(event.playerId) ?? null) : null;
      ranges.push({
        id: `boost-pickup:${pad.index}:${event.frame}:${eventIndex}`,
        startTime,
        endTime: Math.max(startTime + BOOST_PICKUP_TICK_SECONDS, startTime),
        lane: "boost-pickups",
        laneLabel: "Boost Pickups",
        label: `${playerLabel}picked up ${sizeLabel.toLowerCase()} boost pad ${pad.index}`,
        shortLabel: pad.size === "big" ? "100" : "12",
        color: teamTimelineColor(isTeamZero) ?? BOOST_PICKUP_COLORS[pad.size],
        isTeamZero,
      });
    }
  }

  return ranges.sort((left, right) => {
    if (left.startTime !== right.startTime) {
      return left.startTime - right.startTime;
    }
    return (left.id ?? "").localeCompare(right.id ?? "");
  });
}

function padTypesFromOptions(options: BoostPickupTimelineRangeOptions): Set<BoostPickupPadType> {
  if (options.padTypes) {
    return new Set(options.padTypes);
  }

  if (options.sizes) {
    const sizes = new Set(options.sizes);
    const padTypes = new Set<BoostPickupPadType>();
    if (sizes.has("big")) {
      padTypes.add("big");
    }
    if (sizes.has("small")) {
      padTypes.add("small");
    }
    if (sizes.has("big") && sizes.has("small")) {
      padTypes.add("ambiguous");
    }
    return padTypes;
  }

  return new Set(["big", "small", "ambiguous"]);
}

function remoteIdToString(playerId: Record<string, unknown>): string {
  const [kind, value] = Object.entries(playerId)[0] ?? ["Unknown", "unknown"];
  const normalizedValue = typeof value === "string" ? value : JSON.stringify(value);
  return `${kind}:${normalizedValue}`;
}

function formatBoostPickupPadType(padType: BoostPickupPadType): string {
  return {
    big: "big",
    small: "small",
    ambiguous: "ambiguous",
  }[padType];
}

function formatBoostPickupComparison(comparison: BoostPickupComparison): string {
  return {
    both: "counted",
    ghost: "ghost",
    missed: "missed",
  }[comparison];
}

function boostPickupShortLabel(
  comparison: BoostPickupComparison,
  padType: BoostPickupPadType,
): string {
  if (comparison === "ghost") {
    return "G";
  }
  if (comparison === "missed") {
    return "M";
  }
  return {
    big: "100",
    small: "12",
    ambiguous: "?",
  }[padType];
}

export function buildBoostPickupTimelineRanges(
  timeline: StatsTimeline,
  replay?: ReplayModel,
  options: BoostPickupTimelineRangeOptions = {},
): ReplayTimelineRange[] {
  const events = timeline.events?.boost_pickups ?? [];
  if (events.length === 0 && replay) {
    return buildReplayBoostPickupTimelineRanges(replay, options);
  }

  const enabledPadTypes = padTypesFromOptions(options);
  const enabledComparisons = new Set<BoostPickupComparison>(options.comparisons ?? ["both"]);
  const enabledActivities = new Set<BoostPickupActivity>(
    options.activities ?? ["active", "inactive", "unknown"],
  );
  const enabledFieldHalves = new Set<BoostPickupFieldHalf>(
    options.fieldHalves ?? ["own", "opponent", "unknown"],
  );
  const enabledPlayerIds = options.playerIds ? new Set(options.playerIds) : null;
  if (
    enabledPadTypes.size === 0 ||
    enabledComparisons.size === 0 ||
    enabledActivities.size === 0 ||
    enabledFieldHalves.size === 0 ||
    enabledPlayerIds?.size === 0
  ) {
    return [];
  }

  const playerNames = new Map((replay?.players ?? []).map((player) => [player.id, player.name]));
  return events
    .filter((event) => {
      const playerId = remoteIdToString(event.player_id as Record<string, unknown>);
      return (
        enabledPadTypes.has(event.pad_type) &&
        enabledComparisons.has(event.comparison) &&
        enabledActivities.has(event.activity) &&
        enabledFieldHalves.has(event.field_half) &&
        (!enabledPlayerIds || enabledPlayerIds.has(playerId))
      );
    })
    .map((event, index): ReplayTimelineRange => {
      const playerId = remoteIdToString(event.player_id as Record<string, unknown>);
      const playerName = playerNames.get(playerId) ?? playerId;
      const startTime = Math.max(0, getReplayFrameTime(replay, event.frame, event.time));
      const comparisonLabel = formatBoostPickupComparison(event.comparison);
      const padLabel = formatBoostPickupPadType(event.pad_type);
      return {
        id: `boost-pickup:${event.comparison}:${event.frame}:${playerId}:${index}`,
        startTime,
        endTime: Math.max(startTime + BOOST_PICKUP_TICK_SECONDS, startTime),
        lane: "boost-pickups",
        laneLabel: "Boost Pickups",
        label: `${playerName} ${comparisonLabel} ${padLabel} boost pickup`,
        shortLabel: boostPickupShortLabel(event.comparison, event.pad_type),
        color:
          teamTimelineColor(event.is_team_0) ??
          (event.comparison === "both"
            ? event.pad_type === "big"
              ? BOOST_PICKUP_COLORS.big
              : event.pad_type === "small"
                ? BOOST_PICKUP_COLORS.small
                : BOOST_PICKUP_COMPARISON_COLORS.both
            : BOOST_PICKUP_COMPARISON_COLORS[event.comparison]),
        isTeamZero: event.is_team_0,
      };
    })
    .sort((left, right) => {
      if (left.startTime !== right.startTime) {
        return left.startTime - right.startTime;
      }
      return (left.id ?? "").localeCompare(right.id ?? "");
    });
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

function resolveRangeBounds(
  frame: { frame_number: number; time: number; dt: number },
  previousFrame: { frame_number: number; time: number } | null,
  replay?: ReplayModel,
): { startTime: number; endTime: number } {
  const endTime = replay?.frames[frame.frame_number]?.time ?? frame.time;
  const startTime = previousFrame
    ? (replay?.frames[previousFrame.frame_number]?.time ?? previousFrame.time)
    : Math.max(0, endTime - frame.dt);

  return {
    startTime: Math.max(0, startTime),
    endTime: Math.max(startTime, endTime),
  };
}

function mergeRange(ranges: ReplayTimelineRange[], nextRange: ReplayTimelineRange | null): void {
  if (!nextRange) {
    return;
  }

  const previousRange = ranges[ranges.length - 1];
  if (
    previousRange &&
    previousRange.lane === nextRange.lane &&
    previousRange.label === nextRange.label &&
    Math.abs(previousRange.endTime - nextRange.startTime) <= RANGE_MERGE_EPSILON_SECONDS
  ) {
    previousRange.endTime = nextRange.endTime;
    return;
  }

  ranges.push(nextRange);
}

function mergeRangeForLane(
  ranges: ReplayTimelineRange[],
  lastRangeByLane: Map<string, ReplayTimelineRange>,
  nextRange: ReplayTimelineRange | null,
): void {
  if (!nextRange) {
    return;
  }

  const laneKey = nextRange.lane ?? "";
  const previousRange = lastRangeByLane.get(laneKey);
  if (
    previousRange &&
    previousRange.label === nextRange.label &&
    Math.abs(previousRange.endTime - nextRange.startTime) <= RANGE_MERGE_EPSILON_SECONDS
  ) {
    previousRange.endTime = nextRange.endTime;
    return;
  }

  ranges.push(nextRange);
  lastRangeByLane.set(laneKey, nextRange);
}
