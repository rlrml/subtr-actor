import type { ReplayModel, ReplayTimelineRange } from "@rlrml/player";
import type { StatsFrame, StatsTimeline } from "./statsTimeline.ts";
import type { PossessionEvent } from "./generated/PossessionEvent.ts";
import type { PressureEvent } from "./generated/PressureEvent.ts";
import {
  DELTA_EPSILON,
  mergeRange,
  resolveRangeBounds,
  sortTimelineEvents,
} from "./timelineRangeMerge.ts";

const DEFAULT_PRESSURE_NEUTRAL_ZONE_HALF_WIDTH_Y = 200;

type PressureHalfControlState = "team_zero_side" | "team_one_side" | "neutral";

function getPressureNeutralZoneHalfWidthY(timeline: StatsTimeline): number {
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
