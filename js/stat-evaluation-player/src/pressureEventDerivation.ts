import type { LabeledFloatSums } from "./generated/LabeledFloatSums.ts";
import type { PressureEvent } from "./generated/PressureEvent.ts";
import type { PressureTeamStats } from "./generated/PressureTeamStats.ts";
import type { StatLabel } from "./generated/StatLabel.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";

interface RawPressureStats {
  tracked_time: number;
  team_zero_side_time: number;
  team_one_side_time: number;
  neutral_time: number;
  labeled_time: LabeledFloatSums;
}

interface PressureState {
  active: boolean;
  fieldHalf: string;
}

function f32(value: number): number {
  return Math.fround(value);
}

function addF32(left: number, right: number): number {
  return f32(f32(left) + f32(right));
}

function defaultRawPressureStats(): RawPressureStats {
  return {
    tracked_time: 0,
    team_zero_side_time: 0,
    team_one_side_time: 0,
    neutral_time: 0,
    labeled_time: { entries: [] },
  };
}

function defaultPressureTeamStats(): PressureTeamStats {
  return {
    tracked_time: 0,
    defensive_half_time: 0,
    offensive_half_time: 0,
    neutral_time: 0,
    labeled_time: { entries: [] },
  };
}

function sortPressureEvents(events: readonly PressureEvent[]): PressureEvent[] {
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

function sortLabels(labels: StatLabel[]): StatLabel[] {
  return labels.sort((left, right) =>
    left.key === right.key
      ? left.value.localeCompare(right.value)
      : left.key.localeCompare(right.key),
  );
}

function addLabeledTime(sums: LabeledFloatSums, labels: StatLabel[], value: number): void {
  const sortedLabels = sortLabels(labels);
  const entry = sums.entries.find(
    (candidate) =>
      candidate.labels.length === sortedLabels.length &&
      candidate.labels.every(
        (label, index) =>
          label.key === sortedLabels[index]?.key && label.value === sortedLabels[index]?.value,
      ),
  );
  if (entry) {
    entry.value = addF32(entry.value, value);
  } else {
    sums.entries.push({ labels: sortedLabels, value: f32(value) });
    sums.entries.sort((left, right) =>
      JSON.stringify(left.labels).localeCompare(JSON.stringify(right.labels)),
    );
  }
}

function relativePressureLabel(label: StatLabel, isTeamZero: boolean): StatLabel {
  if (label.key === "field_half" && label.value === "team_zero_side") {
    return { key: "field_half", value: isTeamZero ? "defensive_half" : "offensive_half" };
  }
  if (label.key === "field_half" && label.value === "team_one_side") {
    return { key: "field_half", value: isTeamZero ? "offensive_half" : "defensive_half" };
  }
  return { ...label };
}

function pressureTeamStats(raw: RawPressureStats, isTeamZero: boolean): PressureTeamStats {
  const labeled_time: LabeledFloatSums = { entries: [] };
  for (const entry of raw.labeled_time.entries) {
    addLabeledTime(
      labeled_time,
      entry.labels.map((label) => relativePressureLabel(label, isTeamZero)),
      entry.value,
    );
  }
  return {
    tracked_time: raw.tracked_time,
    defensive_half_time: isTeamZero ? raw.team_zero_side_time : raw.team_one_side_time,
    offensive_half_time: isTeamZero ? raw.team_one_side_time : raw.team_zero_side_time,
    neutral_time: raw.neutral_time,
    labeled_time,
  };
}

function applyPressureEvent(state: PressureState, event: PressureEvent): void {
  state.active = event.active;
  state.fieldHalf = event.field_half;
}

function accumulatePressureFrame(
  raw: RawPressureStats,
  state: PressureState,
  frame: StatsFrame,
): void {
  if (!state.active) {
    return;
  }

  const dt = f32(frame.dt);
  raw.tracked_time = addF32(raw.tracked_time, dt);
  if (state.fieldHalf === "team_zero_side") {
    raw.team_zero_side_time = addF32(raw.team_zero_side_time, dt);
  } else if (state.fieldHalf === "team_one_side") {
    raw.team_one_side_time = addF32(raw.team_one_side_time, dt);
  } else {
    raw.neutral_time = addF32(raw.neutral_time, dt);
  }
  addLabeledTime(raw.labeled_time, [{ key: "field_half", value: state.fieldHalf }], dt);
}

function assignPressureStats(
  target: PressureTeamStats,
  source: PressureTeamStats | undefined,
): void {
  Object.assign(target, source ?? defaultPressureTeamStats());
}

export function applyPressureEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createPressureEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createPressureEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortPressureEvents(statsEventPayloads(timeline, "pressure"));

  let eventIndex = 0;
  const raw = defaultRawPressureStats();
  const state: PressureState = {
    active: false,
    fieldHalf: "neutral",
  };

  return {
    applyFrame(frame: StatsFrame): void {
      while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
        applyPressureEvent(state, events[eventIndex] as PressureEvent);
        eventIndex += 1;
      }

      accumulatePressureFrame(raw, state, frame);
      assignPressureStats(frame.team_zero.pressure, pressureTeamStats(raw, true));
      assignPressureStats(frame.team_one.pressure, pressureTeamStats(raw, false));
    },
  };
}
