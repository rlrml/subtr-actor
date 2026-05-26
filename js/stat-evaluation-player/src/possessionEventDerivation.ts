import type { LabeledFloatSums } from "./generated/LabeledFloatSums.ts";
import type { PossessionEvent } from "./generated/PossessionEvent.ts";
import type { PossessionTeamStats } from "./generated/PossessionTeamStats.ts";
import type { StatLabel } from "./generated/StatLabel.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";

interface RawPossessionStats {
  tracked_time: number;
  team_zero_time: number;
  team_one_time: number;
  neutral_time: number;
  labeled_time: LabeledFloatSums;
}

interface PossessionState {
  active: boolean;
  possessionState: string;
  fieldThird: string | null;
}

function f32(value: number): number {
  return Math.fround(value);
}

function addF32(left: number, right: number): number {
  return f32(f32(left) + f32(right));
}

function defaultRawPossessionStats(): RawPossessionStats {
  return {
    tracked_time: 0,
    team_zero_time: 0,
    team_one_time: 0,
    neutral_time: 0,
    labeled_time: { entries: [] },
  };
}

function defaultPossessionTeamStats(): PossessionTeamStats {
  return {
    tracked_time: 0,
    possession_time: 0,
    opponent_possession_time: 0,
    neutral_time: 0,
    labeled_time: { entries: [] },
  };
}

function sortPossessionEvents(events: readonly PossessionEvent[]): PossessionEvent[] {
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

function relativePossessionLabel(label: StatLabel, isTeamZero: boolean): StatLabel {
  if (label.key === "possession_state" && label.value === "team_zero") {
    return { key: "possession_state", value: isTeamZero ? "own" : "opponent" };
  }
  if (label.key === "possession_state" && label.value === "team_one") {
    return { key: "possession_state", value: isTeamZero ? "opponent" : "own" };
  }
  if (label.key === "field_third" && label.value === "team_zero_third") {
    return { key: "field_third", value: isTeamZero ? "defensive_third" : "offensive_third" };
  }
  if (label.key === "field_third" && label.value === "team_one_third") {
    return { key: "field_third", value: isTeamZero ? "offensive_third" : "defensive_third" };
  }
  return { ...label };
}

function possessionTeamStats(raw: RawPossessionStats, isTeamZero: boolean): PossessionTeamStats {
  const labeled_time: LabeledFloatSums = { entries: [] };
  for (const entry of raw.labeled_time.entries) {
    addLabeledTime(
      labeled_time,
      entry.labels.map((label) => relativePossessionLabel(label, isTeamZero)),
      entry.value,
    );
  }
  return {
    tracked_time: raw.tracked_time,
    possession_time: isTeamZero ? raw.team_zero_time : raw.team_one_time,
    opponent_possession_time: isTeamZero ? raw.team_one_time : raw.team_zero_time,
    neutral_time: raw.neutral_time,
    labeled_time,
  };
}

function applyPossessionEvent(state: PossessionState, event: PossessionEvent): void {
  state.active = event.active;
  state.possessionState = event.possession_state;
  state.fieldThird = event.field_third ?? null;
}

function accumulatePossessionFrame(
  raw: RawPossessionStats,
  state: PossessionState,
  frame: StatsFrame,
): void {
  if (!state.active) {
    return;
  }

  const dt = f32(frame.dt);
  raw.tracked_time = addF32(raw.tracked_time, dt);
  if (state.possessionState === "team_zero") {
    raw.team_zero_time = addF32(raw.team_zero_time, dt);
  } else if (state.possessionState === "team_one") {
    raw.team_one_time = addF32(raw.team_one_time, dt);
  } else {
    raw.neutral_time = addF32(raw.neutral_time, dt);
  }

  const labels: StatLabel[] = [{ key: "possession_state", value: state.possessionState }];
  if (state.fieldThird != null) {
    labels.push({ key: "field_third", value: state.fieldThird });
  }
  addLabeledTime(raw.labeled_time, labels, dt);
}

function assignPossessionStats(
  target: PossessionTeamStats,
  source: PossessionTeamStats | undefined,
): void {
  Object.assign(target, source ?? defaultPossessionTeamStats());
}

export function applyPossessionEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createPossessionEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createPossessionEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortPossessionEvents(timeline.events.possession ?? []);

  let eventIndex = 0;
  const raw = defaultRawPossessionStats();
  const state: PossessionState = {
    active: false,
    possessionState: "neutral",
    fieldThird: null,
  };

  return {
    applyFrame(frame: StatsFrame): void {
      while (eventIndex < events.length && events[eventIndex]!.frame <= frame.frame_number) {
        applyPossessionEvent(state, events[eventIndex] as PossessionEvent);
        eventIndex += 1;
      }

      accumulatePossessionFrame(raw, state, frame);
      assignPossessionStats(frame.team_zero.possession, possessionTeamStats(raw, true));
      assignPossessionStats(frame.team_one.possession, possessionTeamStats(raw, false));
    },
  };
}
