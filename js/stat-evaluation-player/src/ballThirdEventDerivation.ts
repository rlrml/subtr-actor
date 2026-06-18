import type { LabeledFloatSums } from "./generated/LabeledFloatSums.ts";
import type { BallThirdEvent } from "./generated/BallThirdEvent.ts";
import type { BallThirdTeamStats } from "./generated/BallThirdTeamStats.ts";
import type { StatLabel } from "./generated/StatLabel.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";

interface RawBallThirdStats {
  tracked_time: number;
  team_zero_third_time: number;
  team_one_third_time: number;
  neutral_third_time: number;
  labeled_time: LabeledFloatSums;
}

interface BallThirdState {
  active: boolean;
  fieldThird: string;
}

function f32(value: number): number {
  return Math.fround(value);
}

function addF32(left: number, right: number): number {
  return f32(f32(left) + f32(right));
}

function defaultRawBallThirdStats(): RawBallThirdStats {
  return {
    tracked_time: 0,
    team_zero_third_time: 0,
    team_one_third_time: 0,
    neutral_third_time: 0,
    labeled_time: { entries: [] },
  };
}

function defaultBallThirdTeamStats(): BallThirdTeamStats {
  return {
    tracked_time: 0,
    defensive_third_time: 0,
    neutral_third_time: 0,
    offensive_third_time: 0,
    labeled_time: { entries: [] },
  };
}

function sortBallThirdEvents(events: readonly BallThirdEvent[]): BallThirdEvent[] {
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

function relativeBallThirdLabel(label: StatLabel, isTeamZero: boolean): StatLabel {
  if (label.key === "field_third" && label.value === "team_zero_third") {
    return { key: "field_third", value: isTeamZero ? "defensive_third" : "offensive_third" };
  }
  if (label.key === "field_third" && label.value === "team_one_third") {
    return { key: "field_third", value: isTeamZero ? "offensive_third" : "defensive_third" };
  }
  return { ...label };
}

function ballThirdTeamStats(raw: RawBallThirdStats, isTeamZero: boolean): BallThirdTeamStats {
  const labeled_time: LabeledFloatSums = { entries: [] };
  for (const entry of raw.labeled_time.entries) {
    addLabeledTime(
      labeled_time,
      entry.labels.map((label) => relativeBallThirdLabel(label, isTeamZero)),
      entry.value,
    );
  }
  return {
    tracked_time: raw.tracked_time,
    defensive_third_time: isTeamZero ? raw.team_zero_third_time : raw.team_one_third_time,
    neutral_third_time: raw.neutral_third_time,
    offensive_third_time: isTeamZero ? raw.team_one_third_time : raw.team_zero_third_time,
    labeled_time,
  };
}

function applyBallThirdEvent(state: BallThirdState, event: BallThirdEvent): void {
  state.active = event.active;
  state.fieldThird = event.field_third;
}

function accumulateBallThirdFrame(
  raw: RawBallThirdStats,
  state: BallThirdState,
  frame: StatsFrame,
): void {
  if (!state.active) {
    return;
  }

  const dt = f32(frame.dt);
  raw.tracked_time = addF32(raw.tracked_time, dt);
  if (state.fieldThird === "team_zero_third") {
    raw.team_zero_third_time = addF32(raw.team_zero_third_time, dt);
  } else if (state.fieldThird === "team_one_third") {
    raw.team_one_third_time = addF32(raw.team_one_third_time, dt);
  } else {
    raw.neutral_third_time = addF32(raw.neutral_third_time, dt);
  }
  addLabeledTime(raw.labeled_time, [{ key: "field_third", value: state.fieldThird }], dt);
}

function assignBallThirdStats(
  target: BallThirdTeamStats,
  source: BallThirdTeamStats | undefined,
): void {
  Object.assign(target, source ?? defaultBallThirdTeamStats());
}

export function applyBallThirdEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createBallThirdEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createBallThirdEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortBallThirdEvents(statsEventPayloads(timeline, "ball_third"));

  let eventIndex = 0;
  const raw = defaultRawBallThirdStats();
  const state: BallThirdState = {
    active: false,
    fieldThird: "neutral_third",
  };

  return {
    applyFrame(frame: StatsFrame): void {
      while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
        applyBallThirdEvent(state, events[eventIndex] as BallThirdEvent);
        eventIndex += 1;
      }

      accumulateBallThirdFrame(raw, state, frame);
      assignBallThirdStats(frame.team_zero.ball_third, ballThirdTeamStats(raw, true));
      assignBallThirdStats(frame.team_one.ball_third, ballThirdTeamStats(raw, false));
    },
  };
}
