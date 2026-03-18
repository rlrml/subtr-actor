import type { ReplayTimelineRange } from "../../player/src/types.ts";
import type {
  DynamicStatsFrame,
  DynamicStatsTimeline,
  ExportedStat,
} from "./statsTimeline.ts";

const RANGE_MERGE_EPSILON_SECONDS = 0.02;
const DELTA_EPSILON = 0.0001;

interface RangeLabelSpec {
  value: string;
  label: string;
  color: string;
  isTeamZero?: boolean | null;
}

interface LabeledRangeOptions {
  lane: string;
  laneLabel: string;
  stats: (frame: DynamicStatsFrame) => ExportedStat[] | undefined;
  domain: string;
  labelKey: string;
  labelSpecs: RangeLabelSpec[];
}

function extractLabeledTimeValues(
  stats: ExportedStat[] | undefined,
  domain: string,
  labelKey: string,
): Map<string, number> {
  const totals = new Map<string, number>();

  for (const stat of stats ?? []) {
    if (
      stat.domain !== domain ||
      stat.name !== "time" ||
      stat.variant !== "labeled" ||
      stat.value_type !== "float" ||
      !Number.isFinite(stat.value)
    ) {
      continue;
    }

    const labelValue = stat.labels?.find((label) => label.key === labelKey)?.value;
    if (!labelValue) {
      continue;
    }

    totals.set(labelValue, (totals.get(labelValue) ?? 0) + stat.value);
  }

  return totals;
}

function buildLabeledTimelineRanges(
  timeline: DynamicStatsTimeline,
  options: LabeledRangeOptions,
): ReplayTimelineRange[] {
  const ranges: ReplayTimelineRange[] = [];
  let previousValues = new Map<string, number>();

  for (const frame of timeline.frames) {
    if (!Number.isFinite(frame.time) || !Number.isFinite(frame.dt) || frame.dt <= 0) {
      continue;
    }

    const values = extractLabeledTimeValues(
      options.stats(frame),
      options.domain,
      options.labelKey,
    );

    let winningSpec: RangeLabelSpec | null = null;
    let winningDelta = 0;

    for (const spec of options.labelSpecs) {
      const delta = (values.get(spec.value) ?? 0) - (previousValues.get(spec.value) ?? 0);
      if (delta > winningDelta + DELTA_EPSILON) {
        winningDelta = delta;
        winningSpec = spec;
      }
    }

    previousValues = values;

    if (!winningSpec) {
      continue;
    }

    const startTime = Math.max(0, frame.time - frame.dt);
    const endTime = frame.time;
    if (endTime - startTime <= DELTA_EPSILON) {
      continue;
    }

    const previousRange = ranges[ranges.length - 1];
    if (
      previousRange &&
      previousRange.lane === options.lane &&
      previousRange.label === winningSpec.label &&
      Math.abs(previousRange.endTime - startTime) <= RANGE_MERGE_EPSILON_SECONDS
    ) {
      previousRange.endTime = endTime;
      continue;
    }

    ranges.push({
      id: `${options.lane}:${winningSpec.value}:${startTime.toFixed(3)}`,
      startTime,
      endTime,
      lane: options.lane,
      laneLabel: options.laneLabel,
      label: winningSpec.label,
      color: winningSpec.color,
      isTeamZero: winningSpec.isTeamZero,
    });
  }

  return ranges;
}

export function buildPossessionTimelineRanges(
  timeline: DynamicStatsTimeline,
): ReplayTimelineRange[] {
  return buildLabeledTimelineRanges(timeline, {
    lane: "possession",
    laneLabel: "Possession",
    stats: (frame) => frame.possession,
    domain: "possession",
    labelKey: "possession_state",
    labelSpecs: [
      {
        value: "team_zero",
        label: "Blue possession",
        color: "rgba(59, 130, 246, 0.88)",
        isTeamZero: true,
      },
      {
        value: "team_one",
        label: "Orange possession",
        color: "rgba(245, 158, 11, 0.88)",
        isTeamZero: false,
      },
      {
        value: "neutral",
        label: "Neutral possession",
        color: "rgba(209, 217, 224, 0.7)",
        isTeamZero: null,
      },
    ],
  });
}

export function buildPressureTimelineRanges(
  timeline: DynamicStatsTimeline,
): ReplayTimelineRange[] {
  return buildLabeledTimelineRanges(timeline, {
    lane: "half-control",
    laneLabel: "Half Control",
    stats: (frame) => frame.pressure,
    domain: "pressure",
    labelKey: "field_half",
    labelSpecs: [
      {
        value: "team_zero_side",
        label: "Blue half control",
        color: "rgba(89, 195, 255, 0.76)",
        isTeamZero: true,
      },
      {
        value: "team_one_side",
        label: "Orange half control",
        color: "rgba(255, 193, 92, 0.76)",
        isTeamZero: false,
      },
    ],
  });
}
