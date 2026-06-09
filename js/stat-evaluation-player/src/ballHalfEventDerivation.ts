import type { LabeledFloatSums } from "./generated/LabeledFloatSums.ts";
import type { BallHalfEvent } from "./generated/BallHalfEvent.ts";
import type { BallHalfTeamStats } from "./generated/BallHalfTeamStats.ts";
import type { StatLabel } from "./generated/StatLabel.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";

interface RawBallHalfStats {
  tracked_time: number;
  team_zero_side_time: number;
  team_one_side_time: number;
  neutral_time: number;
  labeled_time: LabeledFloatSums;
}

interface BallHalfState {
  active: boolean;
  fieldHalf: string;
}

function f32(value: number): number {
  return Math.fround(value);
}

function addF32(left: number, right: number): number {
  return f32(f32(left) + f32(right));
}

function defaultRawBallHalfStats(): RawBallHalfStats {
  return {
    tracked_time: 0,
    team_zero_side_time: 0,
    team_one_side_time: 0,
    neutral_time: 0,
    labeled_time: { entries: [] },
  };
}

function defaultBallHalfTeamStats(): BallHalfTeamStats {
  return {
    tracked_time: 0,
    defensive_half_time: 0,
    offensive_half_time: 0,
    neutral_time: 0,
    labeled_time: { entries: [] },
  };
}

function sortBallHalfEvents(events: readonly BallHalfEvent[]): BallHalfEvent[] {
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

function relativeBallHalfLabel(label: StatLabel, isTeamZero: boolean): StatLabel {
  if (label.key === "field_half" && label.value === "team_zero_side") {
    return { key: "field_half", value: isTeamZero ? "defensive_half" : "offensive_half" };
  }
  if (label.key === "field_half" && label.value === "team_one_side") {
    return { key: "field_half", value: isTeamZero ? "offensive_half" : "defensive_half" };
  }
  return { ...label };
}

function ballHalfTeamStats(raw: RawBallHalfStats, isTeamZero: boolean): BallHalfTeamStats {
  const labeled_time: LabeledFloatSums = { entries: [] };
  for (const entry of raw.labeled_time.entries) {
    addLabeledTime(
      labeled_time,
      entry.labels.map((label) => relativeBallHalfLabel(label, isTeamZero)),
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

function applyBallHalfEvent(state: BallHalfState, event: BallHalfEvent): void {
  state.active = event.active;
  state.fieldHalf = event.field_half;
}

function accumulateBallHalfFrame(
  raw: RawBallHalfStats,
  state: BallHalfState,
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

function assignBallHalfStats(
  target: BallHalfTeamStats,
  source: BallHalfTeamStats | undefined,
): void {
  Object.assign(target, source ?? defaultBallHalfTeamStats());
}

export function applyBallHalfEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createBallHalfEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createBallHalfEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortBallHalfEvents(statsEventPayloads(timeline, "ball_half"));

  let eventIndex = 0;
  const raw = defaultRawBallHalfStats();
  const state: BallHalfState = {
    active: false,
    fieldHalf: "neutral",
  };

  return {
    applyFrame(frame: StatsFrame): void {
      while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
        applyBallHalfEvent(state, events[eventIndex] as BallHalfEvent);
        eventIndex += 1;
      }

      accumulateBallHalfFrame(raw, state, frame);
      assignBallHalfStats(frame.team_zero.ball_half, ballHalfTeamStats(raw, true));
      assignBallHalfStats(frame.team_one.ball_half, ballHalfTeamStats(raw, false));
    },
  };
}
