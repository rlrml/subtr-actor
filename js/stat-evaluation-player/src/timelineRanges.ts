import type { ReplayModel, ReplayTimelineRange } from "../../player/src/types.ts";
import type {
  DynamicStatsTimeline,
  ExportedStat,
  PlayerStatsSnapshot,
  StatsTimeline,
} from "./statsTimeline.ts";
import {
  getExportedStatDomain,
  getExportedStatLabels,
  getExportedStatName,
  getExportedStatValue,
  getExportedStatVariant,
} from "./exportedStats.ts";

const RANGE_MERGE_EPSILON_SECONDS = 0.02;
const DELTA_EPSILON = 0.0001;
const DEFAULT_PRESSURE_NEUTRAL_ZONE_HALF_WIDTH_Y = 200;

type PressureHalfControlState = "team_zero_side" | "team_one_side" | "neutral";

function getPressureNeutralZoneHalfWidthY(
  timeline: StatsTimeline | DynamicStatsTimeline,
): number {
  const configured = timeline.config?.pressure_neutral_zone_half_width_y;
  if (typeof configured === "number" && Number.isFinite(configured)) {
    return Math.max(0, configured);
  }

  return DEFAULT_PRESSURE_NEUTRAL_ZONE_HALF_WIDTH_Y;
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
    typeof ballY === "number"
    && Number.isFinite(ballY)
    && Math.abs(ballY) <= neutralZoneHalfWidthY + DELTA_EPSILON
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

export function buildPossessionTimelineRanges(
  timeline: StatsTimeline,
  dynamicTimeline?: DynamicStatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  if (dynamicTimeline) {
    return buildPossessionTimelineRangesFromDynamic(dynamicTimeline, replay);
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

    const currentTeamZero = frame.possession?.team_zero_time ?? 0;
    const currentTeamOne = frame.possession?.team_one_time ?? 0;
    const currentNeutral = frame.possession?.neutral_time ?? 0;

    const deltaTeamZero = currentTeamZero - previousTeamZero;
    const deltaTeamOne = currentTeamOne - previousTeamOne;
    const deltaNeutral = currentNeutral - previousNeutral;

    previousTeamZero = currentTeamZero;
    previousTeamOne = currentTeamOne;
    previousNeutral = currentNeutral;

    let nextRange: ReplayTimelineRange | null = null;
    const { startTime, endTime } = resolveRangeBounds(frame, previousFrame, replay);

    if (deltaTeamZero > deltaTeamOne + DELTA_EPSILON && deltaTeamZero > deltaNeutral + DELTA_EPSILON) {
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
    } else if (deltaTeamOne > deltaTeamZero + DELTA_EPSILON && deltaTeamOne > deltaNeutral + DELTA_EPSILON) {
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

export function buildPressureTimelineRanges(
  timeline: StatsTimeline,
  dynamicTimeline?: DynamicStatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  if (dynamicTimeline) {
    return buildPressureTimelineRangesFromDynamic(dynamicTimeline, replay);
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

    const currentTeamZero = frame.pressure?.team_zero_side_time ?? 0;
    const currentTeamOne = frame.pressure?.team_one_side_time ?? 0;
    const currentNeutral = frame.pressure?.neutral_time ?? 0;
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

interface PlayerZoneSpec {
  fieldName: string;
  aliases?: string[];
  label: string;
  color: string;
}

const PLAYER_ZONE_SPECS: PlayerZoneSpec[] = [
  {
    fieldName: "time_defensive_third",
    aliases: ["time_defensive_zone"],
    label: "Def third",
    color: "rgba(89, 195, 255, 0.74)",
  },
  {
    fieldName: "time_neutral_third",
    aliases: ["time_neutral_zone"],
    label: "Neutral third",
    color: "rgba(209, 217, 224, 0.68)",
  },
  {
    fieldName: "time_offensive_third",
    aliases: ["time_offensive_zone"],
    label: "Off third",
    color: "rgba(255, 193, 92, 0.78)",
  },
];

function playerIdToString(playerId: Record<string, string>): string {
  const [kind, value] = Object.entries(playerId)[0] ?? ["Unknown", "unknown"];
  return `${kind}:${value}`;
}

function extractPlayerStatValue(
  player: PlayerStatsSnapshot,
  spec: PlayerZoneSpec,
): number {
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

export function buildTimeInZoneTimelineRanges(
  timeline: StatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  const previousValues = new Map<string, Map<string, number>>();
  const ranges: ReplayTimelineRange[] = [];

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

    for (const player of frame.players) {
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

      mergeRange(ranges, {
        id: `time-in-zone:${playerId}:${winningSpec.fieldName}:${startTime.toFixed(3)}`,
        startTime,
        endTime,
        lane: `time-in-zone:${playerId}`,
        laneLabel: player.name,
        label: winningSpec.label,
        color: winningSpec.color,
        isTeamZero: player.is_team_0,
      });
    }

    previousFrame = frame;
  }

  return ranges;
}

function buildPossessionTimelineRangesFromDynamic(
  timeline: DynamicStatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  const ranges: ReplayTimelineRange[] = [];
  let previousTeamZero = 0;
  let previousTeamOne = 0;
  let previousNeutral = 0;

  let previousFrame: DynamicStatsTimeline["frames"][number] | null = null;
  for (const frame of timeline.frames) {
    if (!Number.isFinite(frame.time) || !Number.isFinite(frame.dt) || frame.dt <= 0) {
      previousFrame = frame;
      continue;
    }

    const currentTeamZero = getLabeledOrNamedTime(
      frame.possession,
      "possession",
      "possession_state",
      "team_zero",
      "team_zero_time",
    );
    const currentTeamOne = getLabeledOrNamedTime(
      frame.possession,
      "possession",
      "possession_state",
      "team_one",
      "team_one_time",
    );
    const currentNeutral = getLabeledOrNamedTime(
      frame.possession,
      "possession",
      "possession_state",
      "neutral",
      "neutral_time",
    );

    const deltaTeamZero = currentTeamZero - previousTeamZero;
    const deltaTeamOne = currentTeamOne - previousTeamOne;
    const deltaNeutral = currentNeutral - previousNeutral;

    previousTeamZero = currentTeamZero;
    previousTeamOne = currentTeamOne;
    previousNeutral = currentNeutral;

    const { startTime, endTime } = resolveRangeBounds(frame, previousFrame, replay);
    let nextRange: ReplayTimelineRange | null = null;

    if (deltaTeamZero > deltaTeamOne + DELTA_EPSILON && deltaTeamZero > deltaNeutral + DELTA_EPSILON) {
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
    } else if (deltaTeamOne > deltaTeamZero + DELTA_EPSILON && deltaTeamOne > deltaNeutral + DELTA_EPSILON) {
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

function buildPressureTimelineRangesFromDynamic(
  timeline: DynamicStatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  const ranges: ReplayTimelineRange[] = [];
  let previousTeamZero = 0;
  let previousTeamOne = 0;
  let previousNeutral = 0;
  const neutralZoneHalfWidthY = getPressureNeutralZoneHalfWidthY(timeline);

  let previousFrame: DynamicStatsTimeline["frames"][number] | null = null;
  for (const frame of timeline.frames) {
    if (!Number.isFinite(frame.time) || !Number.isFinite(frame.dt) || frame.dt <= 0) {
      previousFrame = frame;
      continue;
    }

    const currentTeamZero = getLabeledOrNamedTime(
      frame.pressure,
      "pressure",
      "field_half",
      "team_zero_side",
      "team_zero_side_time",
    );
    const currentTeamOne = getLabeledOrNamedTime(
      frame.pressure,
      "pressure",
      "field_half",
      "team_one_side",
      "team_one_side_time",
    );
    const currentNeutral = getLabeledOrNamedTime(
      frame.pressure,
      "pressure",
      "field_half",
      "neutral",
      "neutral_time",
    );
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

function getLabeledOrNamedTime(
  stats: ExportedStat[] | undefined,
  domain: string,
  labelKey: string,
  labelValue: string,
  fallbackName: string,
): number {
  const labeledValue = getLabeledTime(stats, domain, labelKey, labelValue);
  if (labeledValue !== null) {
    return labeledValue;
  }

  return getNamedTime(stats, domain, fallbackName) ?? 0;
}

function getLabeledTime(
  stats: ExportedStat[] | undefined,
  domain: string,
  labelKey: string,
  labelValue: string,
): number | null {
  if (!stats || stats.length === 0) {
    return null;
  }

  let found = false;
  let total = 0;
  for (const stat of stats) {
    if (
      getExportedStatDomain(stat) !== domain ||
      getExportedStatName(stat) !== "time" ||
      getExportedStatVariant(stat) !== "labeled"
    ) {
      continue;
    }

    const numericValue = getExportedStatValue(stat);
    if (numericValue === undefined) {
      continue;
    }

    const value = getExportedStatLabels(stat).find((label) => label.key === labelKey)?.value;
    if (value !== labelValue) {
      continue;
    }

    found = true;
    total += numericValue;
  }

  return found ? total : null;
}

function getNamedTime(
  stats: ExportedStat[] | undefined,
  domain: string,
  name: string,
): number | null {
  if (!stats || stats.length === 0) {
    return null;
  }

  for (const stat of stats) {
    if (
      getExportedStatDomain(stat) === domain &&
      getExportedStatName(stat) === name &&
      getExportedStatVariant(stat) !== "labeled"
    ) {
      const value = getExportedStatValue(stat);
      if (value !== undefined) {
        return value;
      }
    }
  }

  return null;
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

function mergeRange(
  ranges: ReplayTimelineRange[],
  nextRange: ReplayTimelineRange | null,
): void {
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
